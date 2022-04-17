use std::net::TcpListener;
use config::{ConfigError, File, FileFormat};
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
        TcpListener::bind(format!("127.0.0.1:{}", self.server.port))
            .expect("Failed to bind port")
    }
}

pub fn get() -> Result<Config, ConfigError> {
    config::Config::builder()
        .add_source(File::new("application", FileFormat::Toml))
        .build()?
        .try_deserialize()
}

pub fn must_get() -> Config {
    get().expect("Failed to get config from application.toml")
}
