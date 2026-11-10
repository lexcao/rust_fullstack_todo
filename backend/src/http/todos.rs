//! `/todos` 的五个 handler。规则在 `service`，SQL 在 `repo`。
//!
//! 两处现状刻意保留，契约测试锁着，P3.4 再改：
//! - PATCH 一个不存在的 id：`RecordNotFound` 没有单独映射，走 `AppError::Internal`，返回 500。
//! - DELETE 的仓储错误被吞掉，仍然 200。

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use serde::Deserialize;

use common::model::{CreateTodoRequest, IdsRequest, TodoResponse, TodoStatus, UpdateTodoRequest};

use crate::http::extract::{Namespace, TodoId};
use crate::http::{AppError, AppState};
use crate::infra::db::RecordNotFound;
use crate::repo::Todo;
use crate::service::{self, UpdateTodo};

#[derive(Deserialize)]
pub(crate) struct ListQuery {
    status: Option<TodoStatus>,
}

pub async fn list_todos(
    State(state): State<AppState>,
    Namespace(namespace): Namespace,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<TodoResponse>>, AppError> {
    let todos = service::list_todos(state.repo.as_ref(), namespace, query.status).await?;
    Ok(Json(todos.into_iter().map(TodoResponse::from).collect()))
}

pub async fn create_todo(
    State(state): State<AppState>,
    Namespace(namespace): Namespace,
    body: JsonBody<CreateTodoRequest>,
) -> Result<(StatusCode, Json<TodoResponse>), AppError> {
    let todo = service::create_todo(state.repo.as_ref(), &namespace, &body.0.content).await?;
    Ok((StatusCode::CREATED, Json(TodoResponse::from(todo))))
}

pub async fn get_todo(
    State(state): State<AppState>,
    Namespace(namespace): Namespace,
    TodoId(id): TodoId,
) -> Result<Json<TodoResponse>, AppError> {
    match service::get_todo(state.repo.as_ref(), (namespace, id)).await {
        Ok(todo) => Ok(Json(TodoResponse::from(todo))),
        Err(err) if is_not_found(&err) => Err(AppError::NotFound),
        Err(err) => Err(err.into()),
    }
}

pub async fn update_todo(
    State(state): State<AppState>,
    Namespace(namespace): Namespace,
    TodoId(id): TodoId,
    body: JsonBody<UpdateTodoRequest>,
) -> Result<Json<TodoResponse>, AppError> {
    let body = body.0;
    if body.status.is_none() && body.content.is_none() {
        // 契约第 6 节第 7 条：body 一字不改。
        return Err(AppError::BadRequest(
            "expect one of `status` or `content`".to_owned(),
        ));
    }

    // 不存在的 id 不在这里转成 404，随 anyhow 变成 500。P3.4 改。
    let updated = service::update_todo(
        state.repo.as_ref(),
        (namespace, id),
        UpdateTodo {
            content: body.content,
            status: body.status,
        },
    )
    .await?;
    Ok(Json(TodoResponse::from(updated)))
}

pub async fn clear_todos(
    State(state): State<AppState>,
    Namespace(namespace): Namespace,
    body: JsonBody<IdsRequest<i32>>,
) -> StatusCode {
    let ids = body.0.ids;
    if !ids.is_empty() {
        // 现状：数据库错误被吞掉，仍然 200。P3.4 改成 500。
        let _ = service::clear_todos(state.repo.as_ref(), namespace, ids).await;
    }
    StatusCode::OK
}

fn is_not_found(err: &anyhow::Error) -> bool {
    err.chain()
        .any(|cause| cause.downcast_ref::<RecordNotFound>().is_some())
}

impl From<Todo> for TodoResponse {
    fn from(todo: Todo) -> Self {
        Self {
            namespace: todo.id.0,
            id: todo.id.1,
            content: todo.content,
            status: todo.status,
            created_at: todo.created_at,
            updated_at: todo.updated_at,
        }
    }
}

/// JSON body。解析失败统一 400，和 actix 的 `web::Json` 一样（axum 默认是 422）。
pub struct JsonBody<T>(pub T);

impl<S, T> axum::extract::FromRequest<S> for JsonBody<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req, state).await {
            Ok(axum::Json(value)) => Ok(JsonBody(value)),
            Err(_) => Err(AppError::BadRequest("invalid json body".to_owned())),
        }
    }
}
