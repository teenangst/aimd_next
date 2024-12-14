use std::{env, fs::{self, File}, io::{self, Read, Write}, path::{Path, PathBuf}, thread, time::Duration};

use crate::{files::{get_temp_folder, has_map}, types::Server};
use bzip2::read::BzDecoder;
// use bzip2_rs::DecoderReader;
use console::{style, StyledObject};

fn fetch_servers() -> Result<Result<Vec<Server>, serde_json::Error>, reqwest::Error> {
  let api_url = "https://bot.tf2maps.net/api/v3/servers/";

  let response = reqwest::blocking::get(api_url)?;

  let json = response.text()?;
  Ok(serde_json::from_str(&json))
}

pub fn combine(players:isize, max_players:isize) -> String {
  format!("{} / {}", players, max_players)
}

pub fn colourise_players(players:isize, max_players:isize) -> StyledObject<std::string::String> {
  let combined = combine(players, max_players);
  if players == 0 {
    style(combined).white()
  } else if players*3 < max_players {
    style(combined).red()
  } else if players >= max_players - 4 {
    style(combined).green()
  } else {
    style(combined).yellow()
  }
}

fn get_uris(directory:&PathBuf, fastdl_url:&str, nextmap: &str) -> (String, String, PathBuf) {(
  format!("{fastdl_url}/{nextmap}.bsp.bz2"),
  format!("https://sjc1.vultrobjects.com/tf2maps-maps/maps/{nextmap}.bsp.bz2"),
  directory.join(format!("{nextmap}.bsp.bz2"))
)}

fn check_nextmap(directory: &PathBuf, nextmap: &String, fastdl_url: &String) -> Result<(), Box<dyn std::error::Error>> {
  print!("{}", style(format!("Downloading {}.bsp.bz2... ", nextmap)).cyan());

  let destination_directory = get_temp_folder(directory);

  if !destination_directory.exists() {
    fs::create_dir_all(&destination_directory)?;
  }

  let (
    url,
    backup_url,
    destination
  ) = get_uris(&destination_directory, fastdl_url, nextmap);

  let mut response = {
    let res = reqwest::blocking::get(url)?;

    if res.status().is_client_error() || res.status().is_server_error() {
      println!("\nfastdl error, using vultr fallback.");
      reqwest::blocking::get(backup_url)?
    } else {
      res
    }
  };
  let mut file = fs::File::create(&destination)?;

  if let Some(content_length) = response.content_length() {
    let content_length_mb = content_length as f64 / (1024.0 * 1024.0);
    println!("{}", style(format!("{:.2} MB", content_length_mb)).cyan().dim());
  } else {
    println!("");
  }

  io::copy(&mut response, &mut file)?;

  println!("{}", style(format!("Unzipping {}.bsp... ", nextmap)).cyan());
  thread::sleep(Duration::from_secs(1));

  let unzip_destination = directory.join(format!("{}.bsp", nextmap));
  // let compressed_file = File::open(format!("temp\\{}.bsp.bz2", nextmap))?;
  let compressed_file = File::open(&destination)?;
  let mut decompressor = BzDecoder::new(compressed_file);

  let mut uncompressed_data = Vec::new();
  decompressor.read_to_end(&mut uncompressed_data)?;

  let mut unzipped_file = File::create(unzip_destination)?;
  unzipped_file.write_all(&uncompressed_data)?;

  Ok(())
}

fn do_check_nextmap(directory: &PathBuf, nextmap: &String, fastdl_url: &String) {
  if !has_map(directory, nextmap) {
    match check_nextmap(directory, nextmap, fastdl_url) {
      Ok(_) => {
          println!("{} {}", style("/").green(), style("Downloaded and unzipped successfully!").cyan());
          let _ = fs::remove_file(format!("temp\\{}.bsp.bz2", nextmap));
      },
      Err(e) => {
        println!("{} {}", style("X").red(), style("Download failed!").red());
        eprintln!("\t{}", style(e).red());

        let _ = fs::remove_file(format!("temp\\{}.bsp.bz2", nextmap));

        let _ = fs::remove_file(directory.join(format!("{}.bsp", nextmap)));
      }
    }
  }
}

fn map_exists_symbol(directory: &PathBuf, map: &String) -> StyledObject<std::string::String> {
  if has_map(directory, map) {
    style(map.clone()).green()
  } else {
    style(map.clone()).red()
  }
}

fn parse_server(directory: &PathBuf, server:&Server) {
  match &server.nextmap {
    Some(nextmap) => {
      println!("{} [{}] {} -> {}", &server.name, colourise_players(server.players, server.max_players), map_exists_symbol(&directory, &server.map), map_exists_symbol(&directory, &nextmap));
      if let Some(fastdl_url) = &server.fastdl {
        do_check_nextmap(directory, nextmap, fastdl_url);
        do_check_nextmap(directory, &server.map, fastdl_url);
      }
    },
    None => {
      println!("{} Offline", &server.name);
    }
  }
}

pub fn check_servers(directory: &PathBuf) {
  match fetch_servers() {
    Ok(result) => {
      match result {
        Ok(servers) => {
          println!("");
          for server in servers {
            parse_server(directory, &server);
          }
        },
        Err(e) => println!("{}", style(format!("{:?}", e)).red()),
      }
    },
    Err(e) => println!("{}", style(format!("{:?}", e)).red())
  }
}