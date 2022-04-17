mod create_todo;
mod get_todos;
mod update_todo;
mod clear_todos;
mod get_todo_by_id;

use std::fmt::{Debug, Display, Formatter};
pub use get_todos::*;
pub use create_todo::*;
pub use update_todo::*;
pub use clear_todos::*;
pub use get_todo_by_id::*;

use actix_web::web;
use common::model::TodoResponse;
use crate::domains::todo_domain::Todo;

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
