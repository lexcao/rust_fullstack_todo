//! 请求路径上的统一错误。可预期的问题映射成 4xx；`Internal` 只在服务端记日志。

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// 调用方能看懂的 400。`error` 字段的文案会原样进 JSON。
    #[error("{0}")]
    BadRequest(String),
    /// 空 body 的 404（`GET /todos/{id}` 不存在，以及路径 id 不是整数）。
    #[error("not found")]
    NotFound,
    /// 未预期的失败。对外只有一句固定文案，细节在日志里。
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::BadRequest(error) => {
                (StatusCode::BAD_REQUEST, Json(ErrorBody { error })).into_response()
            }
            AppError::NotFound => StatusCode::NOT_FOUND.into_response(),
            AppError::Internal(err) => {
                log::error!("internal error: {err:#}");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
            }
        }
    }
}
