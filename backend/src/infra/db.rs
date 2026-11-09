use deadpool_postgres::{Config, Pool};
use postgres_native_tls::MakeTlsConnector;
use tokio_postgres::{config::SslMode, NoTls};

pub fn must_init(config: &Config) -> Pool {
    let pg_config = config
        .get_pg_config()
        .expect("Invalid database configuration");
    // sslmode=disable explicitly opts out; prefer also supports local PostgreSQL
    // without TLS. require (as used by Neon) never falls back to plaintext.
    let builder = if pg_config.get_ssl_mode() == SslMode::Disable {
        config.builder(NoTls)
    } else {
        let tls = native_tls::TlsConnector::new().expect("Failed to create TLS connector");
        config.builder(MakeTlsConnector::new(tls))
    };
    builder
        .expect("Failed to create db config builder")
        .build()
        .expect("Failed to create db pool")
}

pub async fn init_schema(pool: &Pool) -> anyhow::Result<()> {
    pool.get()
        .await?
        .batch_execute(include_str!("../../schemas/00_init_tables.sql"))
        .await?;
    Ok(())
}

#[derive(thiserror::Error, Debug)]
#[error("record not found")]
pub struct RecordNotFound;
