use serde::*;
use common::model::TodoStatus;

#[derive(PartialEq, Clone, Deserialize, Serialize)]
pub struct Todo {
    pub id: i32,
    pub content: String,
    pub status: TodoStatus,
}

pub fn create_todo(id: &usize, content: &str) -> Todo {
    Todo {
        id: *id as i32,
        status: TodoStatus::Todo,
        content: content.to_string(),
    }
}
