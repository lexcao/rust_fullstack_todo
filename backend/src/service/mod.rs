//! 待办规则。现在真正在请求路径上跑的只有「按字段覆盖」；
//! 状态流转（toggle / archive / delete）还没有接到 PATCH 上，P3.4 再接，而且不能拦掉前端已经在发的操作。

use anyhow::Result;
use common::model::TodoStatus;

use crate::repo::{Todo, TodoID, TodoRepo};

pub struct UpdateTodo {
    pub content: Option<String>,
    pub status: Option<TodoStatus>,
}

pub async fn get_todo(repo: &dyn TodoRepo, id: TodoID) -> Result<Todo> {
    repo.query_by_id(id).await
}

pub async fn list_todos(
    repo: &dyn TodoRepo,
    namespace: String,
    status: Option<TodoStatus>,
) -> Result<Vec<Todo>> {
    repo.query_todos(namespace, status).await
}

pub async fn create_todo(repo: &dyn TodoRepo, namespace: &str, content: &str) -> Result<Todo> {
    repo.insert_todo(Todo::create(namespace, content)).await
}

pub async fn update_todo(repo: &dyn TodoRepo, id: TodoID, to_update: UpdateTodo) -> Result<Todo> {
    let mut found = repo.query_by_id(id).await?;

    found.content = to_update.content.unwrap_or(found.content);
    let to_status = to_update.status.unwrap_or(found.status);
    // TODO check status（P3.4：只允许前端现在实际会发的流转）
    found.status = to_status;

    repo.update_todo(found).await
}

pub async fn clear_todos(repo: &dyn TodoRepo, namespace: String, ids: Vec<i32>) -> Result<()> {
    repo.clear_todos(namespace, ids).await?;
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum TodoError {
    #[error("invalid status {0} -> {1}")]
    InvalidStatusTransition(TodoStatus, TodoStatus),
    #[error("invalid status from str {0}")]
    InvalidStatusFromStr(String),
}

pub async fn toggle_todo(repo: &dyn TodoRepo, todo: Todo) -> Result<Todo> {
    let mut todo = repo.query_by_id(todo.id).await?;

    let new_status = match todo.status {
        TodoStatus::Todo => TodoStatus::Done,
        TodoStatus::Done => TodoStatus::Todo,
        current => {
            return Err(TodoError::InvalidStatusTransition(current, TodoStatus::Todo).into());
        }
    };

    todo.status = new_status;
    repo.update_todo(todo).await
}

pub async fn archive_todo(repo: &dyn TodoRepo, todo: Todo) -> Result<Todo> {
    let mut found = repo.query_by_id(todo.id).await?;

    let new_status = match found.status {
        TodoStatus::Done => TodoStatus::Archived,
        current => {
            return Err(TodoError::InvalidStatusTransition(current, TodoStatus::Archived).into());
        }
    };

    found.status = new_status;
    repo.update_todo(found).await
}

pub async fn delete_todo(repo: &dyn TodoRepo, todo: Todo) -> Result<Todo> {
    let mut found = repo.query_by_id(todo.id).await?;

    let new_status = match found.status {
        TodoStatus::Archived => TodoStatus::Deleted,
        current => {
            return Err(TodoError::InvalidStatusTransition(current, TodoStatus::Deleted).into());
        }
    };

    found.status = new_status;
    repo.update_todo(found).await
}
