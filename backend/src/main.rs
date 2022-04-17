use env_logger::Env;
use backend::start_server;
use backend::infra::{db, config};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::builder()
        .parse_env(Env::default().default_filter_or("INFO"))
        .init();

    let config = config::must_get();

    let db_pool = db::must_init(&config.db);

    start_server(config.address(), db_pool).await
}
