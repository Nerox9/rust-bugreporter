use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
pub struct WindowConfig {
    pub default_width: i32,
    pub default_height: i32,
    pub application_id: String,
}

#[derive(Debug, Deserialize)]
pub struct GithubConfig {
    pub token: String,
    pub owner: String,
    pub repo: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub window: WindowConfig,
    pub github: GithubConfig,
}

pub static SETTINGS: OnceLock<Settings> = OnceLock::new();

pub fn load_config() -> Settings {
    let config = config::Config::builder()
        .add_source(config::File::with_name("config"))
        .build()
        .expect("Failed to load config");

    config.try_deserialize().expect("Failed to parse config")
}
