use std::time::Duration;

use serde::Deserialize;
use tokio::fs;

// #[derive(Default)]
#[derive(Deserialize, Debug)]
pub struct Config {
    pub discord_token: String,
    pub database_url: String,
    check_interval: f32,
    role_duration: f32,
    pub guild_id: u64,
    pub role_id: u64
}

impl Config {
    pub async fn init() -> Config {
        let contents: String = fs::read_to_string("./config.toml")
            .await
            .expect("Unable to read config file");

        return toml::from_str::<Config>(&contents)
            .expect("Unable to parse config file");
    }

    pub fn getInterval(&self) -> Duration {
        return Duration::from_secs_f32(self.check_interval * 3600.00);
    }

    pub fn getDuration(&self) -> Duration {
        // time::Duration::seconds(seconds)
        return Duration::from_secs_f32(self.role_duration * 3600.00);
    }
}