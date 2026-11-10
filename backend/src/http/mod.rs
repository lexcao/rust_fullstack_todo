//! HTTP 层：路由、CORS、`Namespace` 提取器、`AppError`。
//!
//! 请求先过 CORS（对应原来的 `Cors::permissive`），再拒绝非法 `t-ns`，然后进 handler。
//! handler 只通过 [`AppState`] 里的 `Arc<dyn TodoRepo>` 访问数据，单测可以不连库。

mod error;
mod extract;
mod ping;
mod todos;

pub use error::AppError;
pub use extract::Namespace;

use std::sync::Arc;

use axum::Router;
use axum::middleware;
use axum::routing::get;
use tower_http::cors::CorsLayer;

use crate::repo::TodoRepo;

/// 每个请求共享的状态。具体类型是 `PgTodoRepo`，测试换成内存实现。
#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<dyn TodoRepo>,
}

pub fn router(repo: Arc<dyn TodoRepo>) -> Router {
    Router::new()
        .route("/ping", get(ping::ping))
        .route(
            "/todos",
            get(todos::list_todos)
                .post(todos::create_todo)
                .delete(todos::clear_todos),
        )
        .route(
            "/todos/{id}",
            get(todos::get_todo).patch(todos::update_todo),
        )
        .layer(middleware::from_fn(extract::reject_invalid_namespace))
        .layer(cors_layer())
        .with_state(AppState { repo })
}

fn cors_layer() -> CorsLayer {
    // actix `Cors::permissive()` 放行任意来源 / 方法 / 请求头，并且 `supports_credentials = true`，
    // 因此 `Access-Control-Allow-Origin` 回显请求的 Origin，而不是 `*`。
    // tower-http 的 `CorsLayer::permissive()` 会发 `*`，契约测试要求回显 Origin，
    // 对应的是 `very_permissive()`（回显 Origin、请求的方法和请求的头，并允许 credentials）。
    CorsLayer::very_permissive()
}

#[cfg(test)]
mod tests;
