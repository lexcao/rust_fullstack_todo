use rust_fullstack_todo::*;
use rust_fullstack_todo::infra::config::Config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "rust_fullstack_todo=INFO");
    env_logger::init();

    let config = Config::init().unwrap();

    log::info!("LOAD Config: {:?}", config);

    start_server(config).await
}
