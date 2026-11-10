//! Crabnote 后端：http（路由、提取器、错误）→ service（待办规则）→ repo（SQL）。
#![cfg_attr(not(test), deny(clippy::unwrap_used))]

pub mod http;
pub mod infra;
pub mod repo;
pub mod service;

use std::future::Future;
use std::net::TcpListener;
use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::repo::{PgTodoRepo, TodoRepo};

/// 在已经 bind 好的 listener 上提供 HTTP API，直到收到 SIGTERM 或 Ctrl-C。
///
/// 进行中的请求会先跑完，然后关掉连接池。调用方负责配置和迁移（`main`）。
/// 集成测试走 [`serve`] 加一个永不完成的 shutdown，避免在测试进程里注册信号。
pub async fn start_server(
    listener: TcpListener,
    db_pool: impl Into<PgPool>,
) -> std::io::Result<()> {
    let pool = db_pool.into();
    let repo: Arc<dyn TodoRepo> = Arc::new(PgTodoRepo::new(pool.clone()));
    let result = serve(listener, http::router(repo), shutdown_signal()).await;
    pool.close().await;
    result
}

/// 在 `listener` 上跑 `app`，直到 `shutdown` 完成，然后把已经接进来的请求处理完。
///
/// `app` 必须已经 [`Router::with_state`]（axum 只对 `Router<()>` 实现 serve）。
pub async fn serve(
    listener: TcpListener,
    app: Router,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> std::io::Result<()> {
    let address = listener.local_addr()?;
    listener.set_nonblocking(true)?;
    let listener = tokio::net::TcpListener::from_std(listener)?;
    tracing::info!("starting HTTP server at {address}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
}

/// Ctrl-C 或 SIGTERM。装不上某个信号时只放弃那一条，另一条仍然有效。
pub async fn shutdown_signal() {
    let ctrl_c = async {
        if tokio::signal::ctrl_c().await.is_err() {
            tracing::error!("failed to install Ctrl-C handler");
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(err) => {
                tracing::error!(%err, "failed to install SIGTERM handler");
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("interrupt received, draining in-flight requests"),
        _ = terminate => tracing::info!("SIGTERM received, draining in-flight requests"),
    }
}
