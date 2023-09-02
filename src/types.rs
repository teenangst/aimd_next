use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerInfo {
  #[serde(rename = "Map")]
  pub map: String,

  #[serde(rename = "Players")]
  pub players: isize,

  #[serde(rename = "MaxPlayers")]
  pub max_players: isize,
}

impl Default for ServerInfo {
  fn default() -> Self {
    ServerInfo {
      map: "".to_string(),
      players: 0,
      max_players: 0,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Server {
  pub address: String,
  pub port: u16,
  pub name: String,
  pub nextmap: Option<String>,
  pub info: Option<ServerInfo>,
}