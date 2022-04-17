use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::time::SystemTime;
use anyhow::{bail, Error};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct IdsRequest<T> {
    pub ids: Vec<T>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TodoStatus {
    Todo,
    Done,
    Archived,
    Deleted,
}

impl Display for TodoStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for TodoStatus {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Todo" => Ok(TodoStatus::Todo),
            "Done" => Ok(TodoStatus::Done),
            "Archived" => Ok(TodoStatus::Archived),
            "Deleted" => Ok(TodoStatus::Deleted),
            _ => bail!("invalid todo status [{}] from db", s)
        }
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