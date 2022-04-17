mod create_todo;
mod get_todos;
mod update_todo;
mod clear_todos;
mod get_todo_by_id;

use std::fmt::{Debug, Display, Formatter};
use std::time::SystemTime;
pub use get_todos::*;
pub use create_todo::*;
pub use update_todo::*;
pub use clear_todos::*;
pub use get_todo_by_id::*;

use actix_web::web;
use serde::{Deserialize, Serialize};
use crate::domains::todo_domain::{Todo, TodoStatus};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_todos));
    cfg.route("", web::post().to(create_todo));
    cfg.route("", web::delete().to(clear_todos));

    cfg.route("/{id}", web::get().to(get_todo_by_id));
    cfg.route("/{id}", web::patch().to(update_todo));
}

#[derive(thiserror::Error, Debug)]
pub struct WrappedAnyhowError {
    err: anyhow::Error,
}

impl Display for WrappedAnyhowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.err)
    }
}

impl actix_web::error::ResponseError for WrappedAnyhowError {}

impl From<anyhow::Error> for WrappedAnyhowError {
    fn from(err: anyhow::Error) -> Self {
        Self { err }
    }
}

#[derive(Deserialize, Serialize)]
pub struct CreateTodoRequest {
    pub content: String,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateTodoRequest {
    pub content: Option<String>,
    pub status: Option<TodoStatus>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct TodoResponse {
    pub namespace: String,
    pub id: i32,
    pub content: String,
    pub status: TodoStatus,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
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
