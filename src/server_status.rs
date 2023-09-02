use std::{path::{PathBuf, Path}, fs::{self, File}, io::{self, Read, Write}, thread, time::Duration};

use crate::{types::Server, files::has_map};
use bzip2::read::BzDecoder;
// use bzip2_rs::DecoderReader;
use console::{style, StyledObject};

fn fetch_servers() -> Result<Result<Vec<Server>, serde_json::Error>, reqwest::Error> {
  let api_url = "https://api.skylarkx.uk/tf2mapsservers?info&nextmap";

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

fn get_uris(directory:&str, nextmap: &str) -> (String, String) {(
  format!("https://ewr1.vultrobjects.com/tf2maps-maps/maps/{}.bsp.bz2", nextmap),
  format!("{}\\{}.bsp.bz2", directory, nextmap)
)}

fn check_nextmap(directory: &PathBuf, nextmap: &String, emojis: bool) -> Result<(), Box<dyn std::error::Error>> {
  print!("{}", style(format!("Downloading {}.bsp.bz2... ", nextmap)).cyan());

  let destination_directory = "temp";

  if !Path::new(destination_directory).exists() {
    fs::create_dir(destination_directory)?;
  }

  let (url, destination) = get_uris(destination_directory, nextmap);

  let mut response = reqwest::blocking::get(url)?;
  let mut file = fs::File::create(destination)?;

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
  let compressed_file = File::open(format!("temp\\{}.bsp.bz2", nextmap))?;
  let mut decompressor = BzDecoder::new(compressed_file);

  let mut uncompressed_data = Vec::new();
  decompressor.read_to_end(&mut uncompressed_data)?;

  let mut unzipped_file = File::create(unzip_destination)?;
  unzipped_file.write_all(&uncompressed_data)?;

  Ok(())
}

fn do_check_nextmap(directory: &PathBuf, nextmap: &String, emojis: bool) {
  if has_map(directory, nextmap.clone()) {
    if emojis {
      println!("✅");
    }
  } else {
    if emojis {
      println!("❌");
    }
    match check_nextmap(directory, nextmap, emojis) {
      Ok(_) => {
          println!("{}", style("✅ Downloaded and unzipped successfully!").cyan());
          let _ = fs::remove_file(format!("temp\\{}.bsp.bz2", nextmap));
      },
      Err(e) => {
        println!("{}", style("❌ Download failed!").red());
        eprintln!("\t{}", style(e).red());

        let _ = fs::remove_file(format!("temp\\{}.bsp.bz2", nextmap));

        let _ = fs::remove_file(directory.join(format!("{}.bsp", nextmap)));
      }
    }
  }
}

fn parse_server(directory: &PathBuf, server:&Server) {
  match &server.nextmap {
    Some(nextmap) => {
      match &server.info {
        Some(info) => {
          print!("{}:{} [{}] {} -> {} ", &server.address, server.port, colourise_players(info.players, info.max_players), info.map, nextmap);
          // let _ = check_nextmap(directory, nextmap, true);
          // let _ = check_nextmap(directory, &info.map, false);
          do_check_nextmap(directory, nextmap, true);
          do_check_nextmap(directory, &info.map, false);
        },
        None => {
          print!("{}:{} [?/?] ? -> {} ", &server.address, server.port, nextmap);
          let _ = check_nextmap(directory, nextmap, true);
        }
      }
    },
    None => {
      println!("{}:{} Offline", &server.address, server.port);
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