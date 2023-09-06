mod types;

mod files;
use console::style;
use files::get_teamfortress2_path;

mod server_status;
use server_status::check_servers;

use std::env;
use std::thread;
use std::time::Duration;

fn main() {

  let directory = get_teamfortress2_path();

  println!("{}",style("Automatic Imps Map Downloader:Next made by Skylark/Racc\nVersion 1.2.1\n").yellow());

  let args: Vec<String> = env::args().collect();

  let mut interval: Option<u64> = None;

  let mut i = 1;
  while i < args.len() {
    match args[i].as_str() {
      "-i" | "--interval" => {
        if i + 1 < args.len() {
          if let Ok(parsed_interval) = args[i + 1].parse::<u64>() {
            interval = Some(parsed_interval);
            i += 2;
            continue;
          }
        }
        println!("Invalid or missing value for -i/--interval flag.");
        return;
      }
      _ => {
        println!("Unknown argument: {}", args[i]);
        return;
      }
    }
  }

  let interval = match interval {
    Some(value) => value.max(10),
    None => 20,
  };

  println!("Checking servers every {} seconds. Use -i/--interval flags to change the interval in seconds (minimum 10)", interval);

  loop {
    check_servers(&directory);

    thread::sleep(Duration::from_secs(interval));
  }
}
