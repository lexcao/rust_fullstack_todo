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
    pub fn init() -> Result<Config, ConfigError> {
        config::Config::builder()
            .add_source(File::new("application", FileFormat::Toml))
            .set_default("server.port", 8000)?
            .build()
            .and_then(|c| {
                c.try_deserialize::<Config>()
            })
    }
}
