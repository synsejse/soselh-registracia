use serde::Deserialize;
use rocket::figment::{Figment, providers::{Env, Format, Toml}};

#[derive(Deserialize, Clone)]
pub struct AppConfig {
    #[serde(alias = "DATABASE_URL")]
    pub database_url: String,
    #[serde(alias = "ADMIN_PASSWORD_HASH")]
    pub admin_password_hash: String,
    #[serde(default = "default_rocket_port", alias = "ROCKET_PORT")]
    pub rocket_port: u16,
    #[serde(default = "default_rocket_address", alias = "ROCKET_ADDRESS")]
    pub rocket_address: String,
    #[serde(default = "default_static_dir", alias = "STATIC_DIR")]
    pub static_dir: String,
}

fn default_rocket_port() -> u16 {
    8000
}

fn default_rocket_address() -> String {
    "0.0.0.0".to_string()
}

fn default_static_dir() -> String {
    "static".to_string()
}

impl AppConfig {
    pub fn load() -> Self {
        Figment::new()
            .merge(Toml::file("Config.toml"))
            .merge(Toml::file("../Config.toml"))
            .merge(Env::raw().only(&["DATABASE_URL", "ADMIN_PASSWORD_HASH", "ROCKET_PORT", "ROCKET_ADDRESS"]))
            .extract()
            .expect("Failed to load configuration. Ensure Config.toml exists or environment variables are set (DATABASE_URL, ADMIN_PASSWORD_HASH).")
    }
}
