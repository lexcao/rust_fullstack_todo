pub fn spawn_server() -> String {
    let mut config = backend::infra::config::must_get();

    config.server.port = 0;
    let listener = config.address();
    let port = listener.local_addr().unwrap().port();
    let db_pool = backend::infra::db::must_init(&config.db);

    let _ = tokio::spawn(backend::start_server(listener, db_pool));

    format!("http://127.0.0.1:{}", port)
}
