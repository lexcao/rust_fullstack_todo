//! HTTP 层：路由、CORS、请求 id、超时、`Namespace` 提取器、`AppError`。
//!
//! 请求从外到里：写入 `x-request-id` → 追踪 span → 把这个头抄到响应 → 超时 →
//! CORS（对应原来的 `Cors::permissive`）→ 拒绝非法 `t-ns` → handler。
//! handler 只通过 [`AppState`] 里的 `Arc<dyn TodoRepo>` 访问数据，单测可以不连库。

mod error;
mod extract;
mod health;
mod ping;
mod todos;

pub use error::AppError;
pub use extract::Namespace;

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::http::{HeaderName, Request};
use axum::middleware;
use axum::routing::get;
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::repo::TodoRepo;

/// 单个请求的上限。Render 在 SIGTERM 之后大约 30 秒才会 SIGKILL，超时要留在这之内。
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// 每个请求共享的状态。具体类型是 `PgTodoRepo`，测试换成内存实现。
#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<dyn TodoRepo>,
}

pub fn router(repo: Arc<dyn TodoRepo>) -> Router {
    with_observability(routes(repo), REQUEST_TIMEOUT)
}

fn routes(repo: Arc<dyn TodoRepo>) -> Router {
    Router::new()
        .route("/ping", get(ping::ping))
        .route("/healthz", get(health::healthz))
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

/// 最后加上的 layer 在最外层。已有的 `x-request-id` 会原样回显，没有就生成一个。
pub(super) fn with_observability(router: Router, timeout: Duration) -> Router {
    let header = HeaderName::from_static("x-request-id");
    router
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            timeout,
        ))
        .layer(PropagateRequestIdLayer::new(header.clone()))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<axum::body::Body>| {
                    let request_id = request_id_of(request);
                    tracing::info_span!(
                        "http",
                        method = %request.method(),
                        path = %request.uri().path(),
                        request_id = %request_id,
                    )
                })
                // 默认 on_request 是 DEBUG。INFO（RUST_LOG 的默认级别）下也要能看到请求 id。
                .on_request(
                    |request: &Request<axum::body::Body>, span: &tracing::Span| {
                        let request_id = request_id_of(request);
                        tracing::info!(parent: span, %request_id, "started processing request");
                    },
                ),
        )
        .layer(SetRequestIdLayer::new(header, MakeRequestUuid))
}

fn request_id_of(request: &Request<axum::body::Body>) -> String {
    request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("-")
        .to_owned()
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
