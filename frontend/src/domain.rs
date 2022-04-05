use serde::*;

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TodoStatus {
    Todo,
    Done,
    Archived,
    Deleted,
}

#[derive(PartialEq, Clone, Deserialize, Serialize)]
pub struct Todo {
    pub id: usize,
    pub content: String,
    pub status: TodoStatus,
}

impl Todo {
    pub fn new(content: &str) -> Self {
        Self {
            id: 0,
            content: content.to_string(),
            status: TodoStatus::Todo,
        }
    }
}