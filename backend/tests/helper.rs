#![allow(dead_code)] // 每个测试文件只用到其中一部分

use std::net::TcpListener;

use sqlx::PgPool;

/// 用 `application.toml`/`APP_DB_*` 配置的共享库起服务（API 契约测试和 ping 用）。
/// 先跑迁移再开始处理请求：端口已经绑定，迁移期间到达的请求在内核队列里等着。
pub fn spawn_server() -> String {
    let config = backend::infra::config::must_get();
    let pool: PgPool = backend::infra::db::must_init(&config.db).into();
    spawn(pool, true)
}

/// 用 `#[sqlx::test]` 给的独立测试库起服务（库已迁移好）。
pub fn spawn_server_with(pool: PgPool) -> String {
    spawn(pool, false)
}

fn spawn(pool: PgPool, migrate: bool) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind port");
    let port = listener.local_addr().unwrap().port();

    // 服务在后台跑到测试进程结束，不等待 JoinHandle
    drop(tokio::spawn(async move {
        if migrate {
            backend::infra::db::migrate(&pool)
                .await
                .expect("migration failed");
        }
        // 测试进程里不注册 SIGTERM/Ctrl-C，免得和专门的停机测试抢信号。
        let repo = std::sync::Arc::new(backend::repo::PgTodoRepo::new(pool));
        backend::serve(
            listener,
            backend::http::router(repo),
            std::future::pending(),
        )
        .await
    }));
    format!("http://127.0.0.1:{}", port)
}
