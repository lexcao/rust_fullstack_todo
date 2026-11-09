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
        TcpListener::bind(format!("0.0.0.0:{}", self.server.port)).expect("Failed to bind port")
    }
}

pub fn get() -> Result<Config, ConfigError> {
    let mut builder = config::Config::builder()
        .add_source(File::new("application", FileFormat::Toml))
        .add_source(Environment::with_prefix("APP").separator("_"));
    if let Ok(url) = std::env::var("DATABASE_URL") {
        builder = builder.set_override("db.url", url)?;
    }
    if let Ok(port) = std::env::var("PORT") {
        builder = builder.set_override("server.port", port)?;
    }
    let mut config: Config = builder.build()?.try_deserialize()?;
    // A URL is a complete connection configuration. Do not let file defaults
    // overwrite its credentials or append localhost as an additional host.
    prefer_database_url(&mut config.db);
    Ok(config)
}

fn prefer_database_url(db: &mut deadpool_postgres::Config) {
    if db.url.is_some() {
        *db = deadpool_postgres::Config {
            url: db.url.take(),
            pool: db.pool,
            manager: db.manager.clone(),
            ..Default::default()
        };
    }
}

pub fn must_get() -> Config {
    get().expect("Failed to get config")
}
