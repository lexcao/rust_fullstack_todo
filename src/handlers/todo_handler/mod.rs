mod create_todo;
mod get_todos;
mod update_todo;
mod update_todo_status;
mod delete_todo;

use std::fmt::{Debug, Display, Formatter};
use std::time::SystemTime;
pub use get_todos::*;
pub use create_todo::*;
pub use update_todo::*;
pub use update_todo_status::*;
pub use delete_todo::*;

use actix_web::web;
use serde::{Deserialize, Serialize};
use crate::domains::todo_domain::{Todo, TodoStatus};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_todos));
    cfg.route("", web::post().to(create_todo));
    cfg.route("/{id}", web::patch().to(update_todo));
    cfg.route("/{id}", web::delete().to(delete_todo));
    cfg.route("/{id}/{status}", web::patch().to(update_todo_status));
}

#[derive(thiserror::Error, Debug)]
pub struct WrappedAnyhowError {
    e: anyhow::Error,
}

impl Display for WrappedAnyhowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.e)
    }
}

impl actix_web::error::ResponseError for WrappedAnyhowError {}

impl From<anyhow::Error> for WrappedAnyhowError {
    fn from(err: anyhow::Error) -> Self {
        Self { e: err }
    }
}

#[derive(Deserialize, Serialize)]
pub struct CreateTodoRequest {
    content: String,
}

#[derive(Serialize, Debug, PartialEq)]
struct TodoResponse {
    id: i32,
    content: String,
    status: TodoStatus,
    created_at: SystemTime,
}

impl From<Todo> for TodoResponse {
    fn from(todo: Todo) -> Self {
        Self {
            id: todo.id,
            content: todo.content,
            status: todo.status,
            created_at: todo.created_at,
        }
    }
}
