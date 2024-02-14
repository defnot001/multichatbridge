use std::{fs::File, io::BufReader};

use serde::Deserialize;
use serenity::all::ChannelId;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub bot_token: String,
    pub server_id: String,
    pub chatbridge_channels: Vec<ChatbridgeChannel>,
}

#[derive(Debug, Deserialize)]
pub struct ChatbridgeChannel {
    client_id: String,
    channel_id: ChannelId,
    webhook_url: String,
}

impl Config {
    pub fn load() -> Result<Self, std::io::Error> {
        let file = BufReader::new(File::open("config.json")?);
        serde_json::from_reader(file).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}
