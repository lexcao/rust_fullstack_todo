mod create_todo;
mod get_todos;
mod update_todo;
mod update_todo_status;
mod delete_todo;

pub use get_todos::*;
pub use create_todo::*;
pub use update_todo::*;
pub use update_todo_status::*;
pub use delete_todo::*;

use actix_web::web;
use serde::{Deserialize, Serialize};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(get_todos));
    cfg.route("/", web::post().to(create_todo));
    cfg.route("/{id}", web::patch().to(update_todo));
    cfg.route("/{id}", web::delete().to(delete_todo));
    cfg.route("/{id}/{status}", web::patch().to(update_todo_status));
}

#[derive(Deserialize, Serialize)]
pub struct CreateTodoRequest {
    content: String,
}

#[derive(Serialize, Debug, PartialEq)]
struct TodoResponse {
    id: u32,
    content: String,
    status: TodoStatus,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all="lowercase")]
pub enum TodoStatus {
    Todo,
    Done,
    Archive,
    Deleted,
}
