#![deny(clippy::unwrap_used)]

use backend::infra::{config, db};
use backend::start_server;
use env_logger::Env;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::builder()
        .parse_env(Env::default().default_filter_or("INFO"))
        .init();

    let config = config::must_get();

    let db_pool: sqlx::PgPool = db::must_init(&config.db).into();

    // 空库自动建表；已迁移的库是空操作（重启幂等）
    db::migrate(&db_pool)
        .await
        .map_err(|e| std::io::Error::other(format!("{e:#}")))?;
    log::info!("database migrations are up to date");

    start_server(config.address(), db_pool).await
}
