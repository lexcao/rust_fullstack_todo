//! `t-ns` 租户头和路径里的 todo id。
//!
//! 非法 `t-ns`（非 ASCII / `HeaderValue::to_str` 失败）返回 400，不再 `unwrap`。
//! 没有这个头时用 `default`。路径 id 解析失败返回 404，和原来的 actix `Path<i32>` 一样。

use axum::extract::{FromRequestParts, Request};
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use super::AppError;

/// 当前请求的租户。列是 `VARCHAR(36)`，这里不截断，非法字节在进 handler 之前就被拒绝。
#[derive(Clone, Debug)]
pub struct Namespace(pub String);

pub fn namespace_from_headers(headers: &HeaderMap) -> Result<Namespace, AppError> {
    match headers.get("t-ns") {
        None => Ok(Namespace("default".to_owned())),
        Some(value) => match value.to_str() {
            Ok(text) => Ok(Namespace(text.to_owned())),
            Err(_) => Err(AppError::BadRequest("invalid t-ns header".to_owned())),
        },
    }
}

/// 每个请求都检查一次。`/ping` 不用租户，但坏头以前会在中间件里把进程打崩（release 是 `panic = abort`）。
pub async fn reject_invalid_namespace(req: Request, next: Next) -> Result<Response, AppError> {
    namespace_from_headers(req.headers())?;
    Ok(next.run(req).await)
}

impl<S> FromRequestParts<S> for Namespace
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        namespace_from_headers(&parts.headers)
    }
}

/// `/todos/{id}` 的 id。不是 i32 时 404（空 body），而不是 axum 默认的 400。
pub struct TodoId(pub i32);

impl<S> FromRequestParts<S> for TodoId
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(raw) =
            axum::extract::Path::<String>::from_request_parts(parts, state)
                .await
                .map_err(|_| StatusCode::NOT_FOUND.into_response())?;
        raw.parse::<i32>()
            .map(TodoId)
            .map_err(|_| StatusCode::NOT_FOUND.into_response())
    }
}
