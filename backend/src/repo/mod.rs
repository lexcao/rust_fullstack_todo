//! 仓储。`TodoRepo` 是 handler 测试用的边界；`PgTodoRepo` 是现有的 sqlx 0.9 查询。

mod pg;

pub use pg::PgTodoRepo;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::model::TodoStatus;

pub type TodoID = (String, i32);

#[derive(Debug, Clone)]
pub struct Todo {
    pub id: TodoID,
    pub content: String,
    pub status: TodoStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Todo {
    pub fn create(namespace: &str, content: &str) -> Self {
        Self {
            id: (namespace.to_owned(), 0),
            content: content.to_owned(),
            ..Default::default()
        }
    }
}

impl Default for Todo {
    fn default() -> Self {
        Self {
            id: ("default".to_owned(), 0),
            content: String::new(),
            status: TodoStatus::Todo,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[async_trait]
pub trait TodoRepo: Send + Sync {
    async fn query_by_id(&self, id: TodoID) -> Result<Todo>;
    async fn query_todos(&self, namespace: String, status: Option<TodoStatus>)
    -> Result<Vec<Todo>>;
    async fn insert_todo(&self, todo: Todo) -> Result<Todo>;
    async fn update_todo(&self, todo: Todo) -> Result<Todo>;
    async fn clear_todos(&self, namespace: String, ids: Vec<i32>) -> Result<bool>;
    /// `SELECT 1`。失败表示库不可达（`/healthz` 用）。
    async fn check_db(&self) -> Result<()>;
}
