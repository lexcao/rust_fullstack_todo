use backend::infra::{config, db};
use backend::start_server;
use env_logger::Env;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::builder()
        .parse_env(Env::default().default_filter_or("INFO"))
        .init();

    let config = config::must_get();

    let db_pool = db::must_init(&config.db);

    db::init_schema(&db_pool)
        .await
        .expect("Failed to initialize database schema");

    start_server(config.address(), db_pool).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_precedence_and_local_database_settings() {
        // This binary has one test, so environment changes cannot race other tests.
        std::env::set_var("APP_DB_HOST", "localhost");
        std::env::set_var("APP_DB_USER", "local_user");
        std::env::set_var("APP_DB_DBNAME", "local_db");
        std::env::set_var("APP_DB_PORT", "5433");
        std::env::set_var("APP_SERVER_PORT", "3001");
        std::env::set_var("PORT", "10000");
        std::env::set_var(
            "APP_DB_URL",
            "postgresql://app_user@app.example/app_db?sslmode=disable",
        );
        std::env::set_var(
            "DATABASE_URL",
            "postgresql://neon_user@neon.example/todos?sslmode=require",
        );
        let cfg = config::get().unwrap();
        assert_eq!(cfg.server.port, 10000);
        let pg = cfg.db.get_pg_config().unwrap();
        assert_eq!(pg.get_user(), Some("neon_user"));
        assert_eq!(pg.get_dbname(), Some("todos"));
        assert_eq!(pg.get_ports(), &[5432]);
        assert_eq!(
            pg.get_hosts(),
            &[tokio_postgres::config::Host::Tcp("neon.example".into())]
        );
        assert_eq!(pg.get_ssl_mode(), tokio_postgres::config::SslMode::Require);
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("PORT");
        let cfg = config::get().unwrap();
        assert_eq!(cfg.server.port, 3001);
        let pg = cfg.db.get_pg_config().unwrap();
        assert_eq!(pg.get_user(), Some("app_user"));
        assert_eq!(pg.get_ssl_mode(), tokio_postgres::config::SslMode::Disable);
        std::env::remove_var("APP_DB_URL");
        let pg = config::get().unwrap().db.get_pg_config().unwrap();
        assert_eq!(pg.get_user(), Some("local_user"));
        assert_eq!(pg.get_dbname(), Some("local_db"));
        assert_eq!(pg.get_ports(), &[5433]);
    }
}
