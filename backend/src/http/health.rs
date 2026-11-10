//! `GET /healthz`：对数据库做 `SELECT 1`。`GET /ping` 仍然不查库。

use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;

use super::{AppError, AppState};

/// 短于 Render 健康检查的 5 秒上限。挂住的库返回 503，而不是一直占着连接。
const HEALTH_TIMEOUT: Duration = Duration::from_secs(2);

pub async fn healthz(State(state): State<AppState>) -> Result<StatusCode, AppError> {
    match tokio::time::timeout(HEALTH_TIMEOUT, state.repo.check_db()).await {
        Ok(Ok(())) => Ok(StatusCode::OK),
        Ok(Err(_)) => unavailable(),
        Err(_) => unavailable(),
    }
}

fn unavailable() -> Result<StatusCode, AppError> {
    // 不记录 sqlx 错误原文：连接失败的 Display 可能带上 URL。
    tracing::warn!("healthz database check failed");
    Err(AppError::Unavailable)
}
