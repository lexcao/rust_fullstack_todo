#![deny(clippy::unwrap_used)]

use backend::infra::{config, db};
use backend::start_server;
use tracing_subscriber::EnvFilter;

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_tracing();

    let config = config::must_get();
    config::log_loaded(&config);

    let db_pool: sqlx::PgPool = db::must_init(&config.db).into();

    // 空库自动建表；已迁移的库是空操作（重启幂等）
    db::migrate(&db_pool)
        .await
        .map_err(|e| std::io::Error::other(format!("{e:#}")))?;
    tracing::info!("database migrations are up to date");

    start_server(config.address(), db_pool).await
}
