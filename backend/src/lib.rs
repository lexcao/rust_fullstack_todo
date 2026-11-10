//! Crabnote 后端：http（路由、提取器、错误）→ service（待办规则）→ repo（SQL）。
#![cfg_attr(not(test), deny(clippy::unwrap_used))]

pub mod http;
pub mod infra;
pub mod repo;
pub mod service;

use std::net::TcpListener;
use std::sync::Arc;

use sqlx::PgPool;

use crate::repo::{PgTodoRepo, TodoRepo};

/// 在已经 bind 好的 listener 上提供 HTTP API，直到进程退出。
///
/// 调用方负责配置、连接池和迁移（`main` 以及集成测试的 `spawn_server`）。
pub async fn start_server(
    listener: TcpListener,
    db_pool: impl Into<PgPool>,
) -> std::io::Result<()> {
    let address = listener.local_addr()?;
    listener.set_nonblocking(true)?;
    let listener = tokio::net::TcpListener::from_std(listener)?;
    log::info!("starting HTTP server at {address}");

    let repo: Arc<dyn TodoRepo> = Arc::new(PgTodoRepo::new(db_pool.into()));
    axum::serve(listener, http::router(repo)).await
}
