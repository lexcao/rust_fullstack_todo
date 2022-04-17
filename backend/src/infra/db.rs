use deadpool_postgres::{Config, Pool};
use tokio_postgres::NoTls;

pub fn must_init(config: &Config) -> Pool {
    config
        .builder(NoTls)
        .expect("Failed to create db config builder")
        .build()
        .expect("Failed to create db pool")
}

#[derive(thiserror::Error, Debug)]
#[error("record not found")]
pub struct RecordNotFound;
