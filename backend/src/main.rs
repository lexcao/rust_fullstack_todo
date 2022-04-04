use backend::*;
use backend::infra::config::Config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "backend=INFO");
    env_logger::init();

    let config = Config::init()
        .expect("load config failed");

    log::info!("LOAD Config: {:?}", config);

    start_server(config).await
}
