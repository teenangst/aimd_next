extern crate winreg;

use std::fs;
use std::io;
use std::io::BufRead;
use std::path::Path;
use std::path::PathBuf;

use console::style;
use winreg::enums::*;
use winreg::RegKey;

pub fn check_directory(directory: &PathBuf) -> bool {
  match fs::metadata(directory) {
    Ok(dir) => {
      dir.is_dir()
    },
    Err(_) => false
  }
}

pub fn get_path_from_user() -> PathBuf {
  println!("Unable to automatically locate Steam directory. Please enter the path to your Team Fortress 2 folder:");
  println!("{}", style("It will look something like this: G:\\SteamLibrary\\steamapps\\common\\Team Fortress 2").white().dim());
  let mut user_input = String::new();
  match io::stdin().read_line(&mut user_input) {
    Ok(_) => {
      println!("Checking {}", user_input);
      let download_path = Path::new(&user_input).join(Path::new("tf\\download\\maps"));
      match check_directory(&download_path) {
        true => {
          download_path
        },
        false => {
          get_path_from_user()
        }
      }
    },
    Err(_) => {
      println!("{}", style("Error reading input").red());
      get_path_from_user()
    }
  }
}

pub fn find_teamfortress2_path(steam_directory: &String) -> Result<PathBuf, io::Error> {
  let libraryfolders_vdf = Path::new(&steam_directory).join(Path::new("config\\libraryfolders.vdf"));

  let file = std::fs::File::open(&libraryfolders_vdf)?;
  let reader = std::io::BufReader::new(file);

  let lines: Vec<String> = reader.lines().collect::<Result<_, _>>().unwrap();
  let paths =
    lines
    .iter()
    .filter(|line| line.contains("\"path\""))
    .map(|line| {
      let parts: Vec<&str> = line.split('"').collect::<Vec<_>>();
      parts.get(3).copied()
    })
    .collect::<Vec<Option<_>>>();

  for path in paths {
    if let Some(path) = path {
      // println!("Found path: {}", path);
      let path = Path::new(&path).join(Path::new("steamapps\\common\\Team Fortress 2\\tf\\download\\maps"));
      if check_directory(&path) {
        return Ok(path);
      }
    }
  }

  Ok(get_path_from_user())
}

pub fn get_teamfortress2_path() -> PathBuf {
  let registry_key = "SOFTWARE\\Valve\\Steam";
  let registry_value = "SteamPath";

  let hkey_current_user = RegKey::predef(HKEY_CURRENT_USER);
  match hkey_current_user.open_subkey(registry_key) {
    Ok(regkey) => {
      match regkey.get_value::<String, _>(registry_value) {
        Ok(value) => {
          match find_teamfortress2_path(&value) {
            Ok(path) => path,
            Err(_) => get_path_from_user()
          }
        }
        Err(err) => {
          eprintln!("Error reading registry value: {:?}", err);
          get_path_from_user()
        }
      }
    },
    Err(e) => {
      eprintln!("Error opening registry key: {:?}", e);
      get_path_from_user()
    }
  }
}

fn get_files(directory: &PathBuf) -> Vec<String> {
  let mut downloads = fs::read_dir(directory).unwrap().collect::<Vec<_>>();
  let maps = fs::read_dir(directory.parent().unwrap().parent().unwrap().join("maps")).unwrap().collect::<Vec<_>>();
  downloads.extend(maps);

  downloads.iter().map(|entry| {
    if let Ok(entry) = entry {
      entry.path().file_name().unwrap().to_str().unwrap().to_string()
    } else {
      "".to_string()
    }
  }).collect::<Vec<_>>()
}

fn format_map_name(map: &str) -> String {
  if map.ends_with(".bsp") {
      map.to_string()
  } else {
      format!("{}.bsp", map)
  }
}

pub fn has_map(directory: &PathBuf, map: &String) -> bool {
  let file_name = format_map_name(&map);

  let files = get_files(directory);
  files.iter().any(|file| {
    file.ends_with(&file_name)
  })
}