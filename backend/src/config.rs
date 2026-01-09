use serde::Deserialize;
use rocket::figment::{Figment, providers::{Env, Format, Toml}};

#[derive(Deserialize, Clone)]
pub struct AppConfig {
    #[serde(alias = "DATABASE_URL")]
    pub database_url: String,
    #[serde(alias = "PRESENTER_PASSWORD_HASH")]
    pub presenter_password_hash: String,
    #[serde(default = "default_rocket_port", alias = "ROCKET_PORT")]
    pub rocket_port: u16,
}

fn default_rocket_port() -> u16 {
    8000
}

impl AppConfig {
    pub fn load() -> Self {
        Figment::new()
            .merge(Toml::file("Config.toml"))
            .merge(Toml::file("../Config.toml"))
            .merge(Env::raw().only(&["DATABASE_URL", "PRESENTER_PASSWORD_HASH", "ROCKET_PORT"]))
            .extract()
            .expect("Failed to load configuration. Ensure Config.toml exists or environment variables are set (DATABASE_URL, PRESENTER_PASSWORD_HASH).")
    }
}
