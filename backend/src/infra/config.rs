use std::net::TcpListener;

use config::{ConfigError, Environment, File, FileFormat};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Server {
    pub port: u16,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub server: Server,
    pub db: deadpool_postgres::Config,
}

impl Config {
    pub fn address(&self) -> TcpListener {
        TcpListener::bind(format!("0.0.0.0:{}", self.server.port))
            .expect("Failed to bind port")
    }
}

pub fn get() -> Result<Config, ConfigError> {
    config::Config::builder()
        .add_source(File::new("application", FileFormat::Toml))
        .add_source(Environment::with_prefix("APP").separator("_"))
        .build()?
        .try_deserialize()
}

pub fn must_get() -> Config {
    get().expect("Failed to get config")
}
