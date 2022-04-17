use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use crate::domains::todo_repository::TodoRepository;
use anyhow::{Error, Result};
use thiserror::Error;
use async_trait::async_trait;

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
            _ => Err(TodoError::InvalidStatusFromStr(s.to_string()).into())
        }
    }
}

#[derive(Debug)]
pub struct Todo {
    pub id: TodoID,
    pub content: String,
    pub status: TodoStatus,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

pub type TodoID = (String, i32);

pub struct UpdateTodo {
    pub content: Option<String>,
    pub status: Option<TodoStatus>,
}

impl Todo {
    pub fn create(namespace: &str, content: &str) -> Self {
        Self {
            id: (namespace.to_string(), 0),
            content: content.to_string(),
            ..Default::default()
        }
    }
}

impl Default for Todo {
    fn default() -> Self {
        Self {
            id: ("default".to_string(), 0),
            content: "".to_string(),
            status: TodoStatus::Todo,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }
}

#[async_trait]
pub trait TodoDomainTrait {
    async fn get_todo_by_id(&self, id: TodoID) -> Result<Todo>;
    async fn list_todo(&self, namespace: String, status: Option<TodoStatus>) -> Result<Vec<Todo>>;
    async fn create_todo(&self, namespace: String, content: &str) -> Result<Todo>;
    async fn update_todo(&self, id: TodoID, to_update: UpdateTodo) -> Result<Todo>;
    async fn clear_todos(&self, namespace: String, ids: Vec<i32>) -> Result<()>;
}

#[async_trait]
impl TodoDomainTrait for TodoDomain {
    async fn get_todo_by_id(&self, id: TodoID) -> Result<Todo> {
        self.repo.query_by_id(id).await
    }

    async fn list_todo(&self, namespace: String, status: Option<TodoStatus>) -> Result<Vec<Todo>> {
        self.repo.query_todos(namespace, status).await
    }

    async fn create_todo(&self, namespace: String, content: &str) -> Result<Todo> {
        self.repo.insert_todo(Todo::create(&namespace, &content)).await
    }

    async fn update_todo(&self, id: TodoID, to_update: UpdateTodo) -> Result<Todo> {
        let mut found = self.repo.query_by_id(id).await?;

        found.content = to_update.content.unwrap_or_else(|| found.content);
        let to_status = to_update.status.unwrap_or_else(|| found.status);
        // TODO check status
        found.status = to_status;

        self.repo.update_todo(found).await
    }

    async fn clear_todos(&self, namespace: String, ids: Vec<i32>) -> Result<()> {
        self.repo.clear_todos(namespace, ids).await?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct TodoDomain {
    repo: TodoRepository,
}

impl TodoDomain {
    pub fn new(pool: deadpool_postgres::Pool) -> Self {
        Self { repo: TodoRepository::new(pool) }
    }

    pub async fn list_todo(&self, namespace: String, status: Option<TodoStatus>) -> Result<Vec<Todo>> {
        self.repo.query_todos(namespace, status).await
    }

    pub async fn create_todo(&self, namespace: &str, content: &str) -> Result<Todo> {
        self.repo.insert_todo(Todo::create(namespace, content)).await
    }

    pub async fn update_todo(&self, id: TodoID, to_update: UpdateTodo) -> Result<Todo> {
        let mut found = self.repo.query_by_id(id).await?;

        found.content = to_update.content.unwrap_or_else(|| found.content);
        let to_status = to_update.status.unwrap_or_else(|| found.status);
        // TODO check status
        found.status = to_status;

        self.repo.update_todo(found).await
    }

    pub async fn toggle_todo(&self, todo: Todo) -> Result<Todo> {
        let mut todo = self.repo.query_by_id(todo.id).await?;

        let new_status = match todo.status {
            TodoStatus::Todo => TodoStatus::Done,
            TodoStatus::Done => TodoStatus::Todo,
            current => return Err(TodoError::InvalidStatusTransition(current, TodoStatus::Todo).into()),
        };

        todo.status = new_status;

        self.repo.update_todo(todo).await
    }
    pub async fn archive_todo(&self, todo: Todo) -> Result<Todo> {
        let mut found = self.repo.query_by_id(todo.id).await?;

        let new_status = match found.status {
            TodoStatus::Done => TodoStatus::Archived,
            current => return Err(TodoError::InvalidStatusTransition(current, TodoStatus::Archived).into()),
        };

        found.status = new_status;

        self.repo.update_todo(found).await
    }
    pub async fn delete_todo(&self, todo: Todo) -> Result<Todo> {
        let mut found = self.repo.query_by_id(todo.id).await?;

        let new_status = match found.status {
            TodoStatus::Archived => TodoStatus::Deleted,
            current => return Err(TodoError::InvalidStatusTransition(current, TodoStatus::Deleted).into()),
        };

        found.status = new_status;

        self.repo.update_todo(found).await
    }
}


#[derive(Error, Debug)]
pub enum TodoError {
    #[error("invalid status {0} -> {1}")]
    InvalidStatusTransition(TodoStatus, TodoStatus),
    #[error("invalid status from str {0}")]
    InvalidStatusFromStr(String),
}

trait TodoStatusMachine {}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn check() {}
}
