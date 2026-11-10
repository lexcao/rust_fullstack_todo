//! `todos` 表的 sqlx 实现。SQL 文本和 P3.2 提交的 `.sqlx` 一致，改查询要重新 `cargo sqlx prepare`。

use std::str::FromStr;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::PgPool;

use common::model::TodoStatus;

use super::{Todo, TodoID, TodoRepo};
use crate::infra::db::RecordNotFound;

/// `todos` 表的一行。时间列是 `TIMESTAMP`（不带时区），按 UTC 存取，和换 sqlx 之前一致。
#[derive(Debug)]
struct TodoEntity {
    namespace: String,
    id: i32,
    content: String,
    status: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Clone)]
pub struct PgTodoRepo {
    db: PgPool,
}

impl PgTodoRepo {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    async fn fetch_by_id(&self, (namespace, id): TodoID) -> Result<Todo> {
        let row = sqlx::query_as!(
            TodoEntity,
            "SELECT namespace, id, content, status, created_at, updated_at FROM todos WHERE namespace = $1 AND id = $2",
            namespace,
            id
        )
        .fetch_optional(&self.db)
        .await?;

        match row {
            None => Err(RecordNotFound.into()),
            Some(entity) => Todo::try_from(entity),
        }
    }

    async fn fetch_todos(
        &self,
        namespace: String,
        status: Option<TodoStatus>,
    ) -> Result<Vec<Todo>> {
        let rows = match status {
            Some(value) => {
                sqlx::query_as!(
                    TodoEntity,
                    "SELECT namespace, id, content, status, created_at, updated_at FROM todos WHERE namespace = $1 AND status = $2 ORDER BY id DESC",
                    namespace,
                    value.to_string()
                )
                .fetch_all(&self.db)
                .await?
            }
            None => {
                sqlx::query_as!(
                    TodoEntity,
                    "SELECT namespace, id, content, status, created_at, updated_at FROM todos WHERE namespace = $1 ORDER BY id DESC",
                    namespace
                )
                .fetch_all(&self.db)
                .await?
            }
        };

        rows.into_iter().map(Todo::try_from).collect()
    }

    async fn insert(&self, todo: Todo) -> Result<Todo> {
        let entity = TodoEntity::from(todo);

        let row = sqlx::query_as!(
            TodoEntity,
            r#"
            INSERT INTO todos (namespace, content, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING namespace, id, content, status, created_at, updated_at
            "#,
            entity.namespace,
            entity.content,
            entity.status,
            entity.created_at,
            entity.updated_at
        )
        .fetch_one(&self.db)
        .await?;

        Todo::try_from(row)
    }

    async fn update(&self, todo: Todo) -> Result<Todo> {
        let entity = TodoEntity::from(todo);

        let row = sqlx::query_as!(
            TodoEntity,
            r#"
            UPDATE todos SET content = $3, status = $4, updated_at = $5
            WHERE namespace = $1 AND id = $2
            RETURNING namespace, id, content, status, created_at, updated_at
            "#,
            entity.namespace,
            entity.id,
            entity.content,
            entity.status,
            entity.updated_at
        )
        .fetch_one(&self.db)
        .await?;

        Todo::try_from(row)
    }

    async fn clear(&self, namespace: String, ids: Vec<i32>) -> Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM todos WHERE namespace = $1 AND id = ANY($2)",
            namespace,
            &ids
        )
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected() as usize == ids.len())
    }
}

#[async_trait]
impl TodoRepo for PgTodoRepo {
    async fn query_by_id(&self, id: TodoID) -> Result<Todo> {
        self.fetch_by_id(id).await
    }

    async fn query_todos(
        &self,
        namespace: String,
        status: Option<TodoStatus>,
    ) -> Result<Vec<Todo>> {
        self.fetch_todos(namespace, status).await
    }

    async fn insert_todo(&self, todo: Todo) -> Result<Todo> {
        self.insert(todo).await
    }

    async fn update_todo(&self, todo: Todo) -> Result<Todo> {
        self.update(todo).await
    }

    async fn clear_todos(&self, namespace: String, ids: Vec<i32>) -> Result<bool> {
        self.clear(namespace, ids).await
    }
}

/// 写库：`updated_at` 总是取当前时间（插入和更新都是），和之前的行为一致。
impl From<Todo> for TodoEntity {
    fn from(todo: Todo) -> Self {
        Self {
            namespace: todo.id.0,
            id: todo.id.1,
            content: todo.content,
            status: todo.status.to_string(),
            created_at: todo.created_at.naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}

/// 读库：库里 status 是首字母大写（`Todo`/`Done`/…）；读到别的值返回错误而不是 panic。
impl TryFrom<TodoEntity> for Todo {
    type Error = anyhow::Error;

    fn try_from(todo: TodoEntity) -> Result<Self> {
        Ok(Self {
            id: (todo.namespace, todo.id),
            content: todo.content,
            status: TodoStatus::from_str(&todo.status)?,
            created_at: DateTime::from_naive_utc_and_offset(todo.created_at, Utc),
            updated_at: DateTime::from_naive_utc_and_offset(todo.updated_at, Utc),
        })
    }
}

#[cfg(test)]
mod tests {
    use common::model::TodoStatus;
    use sqlx::PgPool;

    use super::PgTodoRepo;
    use crate::infra::db::RecordNotFound;
    use crate::repo::{Todo, TodoRepo};

    // 每个 #[sqlx::test] 一个新建的空库（跑完迁移），测试之间互不影响，跑完自动删除。
    static NS: &str = "default";

    #[sqlx::test]
    async fn query_by_id(pool: PgPool) {
        let repo = PgTodoRepo::new(pool);
        let not_found = repo.query_by_id((NS.to_string(), 99999)).await;

        assert!(not_found.is_err());
        assert!(not_found.unwrap_err().is::<RecordNotFound>());

        let created = repo
            .insert_todo(Todo::create(NS, "new todo"))
            .await
            .unwrap();

        let found = repo.query_by_id(created.id.clone()).await.unwrap();
        assert_eq!(found.id, created.id);
        assert_eq!(found.content, created.content);
        assert_eq!(found.status, created.status);
        assert_eq!(found.created_at, created.created_at);
    }

    #[sqlx::test]
    async fn query_todos(pool: PgPool) {
        let repo = PgTodoRepo::new(pool);
        assert!(
            repo.query_todos(NS.to_string(), None)
                .await
                .unwrap()
                .is_empty(),
            "新库是空的"
        );

        let a = repo.insert_todo(Todo::create(NS, "a")).await.unwrap();
        let mut b = repo.insert_todo(Todo::create(NS, "b")).await.unwrap();
        repo.insert_todo(Todo::create("other", "c")).await.unwrap();
        b.status = TodoStatus::Done;
        repo.update_todo(b).await.unwrap();

        let all = repo.query_todos(NS.to_string(), None).await.unwrap();
        assert_eq!(
            all.iter().map(|t| t.content.as_str()).collect::<Vec<_>>(),
            ["b", "a"],
            "按 id 倒序，只有本租户"
        );
        let done = repo
            .query_todos(NS.to_string(), Some(TodoStatus::Done))
            .await
            .unwrap();
        assert_eq!(
            done.iter().map(|t| t.content.as_str()).collect::<Vec<_>>(),
            ["b"]
        );
        let open = repo
            .query_todos(NS.to_string(), Some(TodoStatus::Todo))
            .await
            .unwrap();
        assert_eq!(
            open.iter().map(|t| t.id.clone()).collect::<Vec<_>>(),
            [a.id]
        );
    }

    #[sqlx::test]
    async fn insert_todo(pool: PgPool) {
        let created = PgTodoRepo::new(pool)
            .insert_todo(Todo::create(NS, "new todo"))
            .await
            .unwrap();

        assert_ne!(created.id.1, 0);
        assert_eq!(created.id.0, "default");
        assert_eq!(created.content, "new todo".to_string());
        assert_eq!(created.status, TodoStatus::Todo);
    }

    #[sqlx::test]
    async fn update_todo(pool: PgPool) {
        let repo = PgTodoRepo::new(pool);

        let mut created = repo
            .insert_todo(Todo::create(NS, "new todo"))
            .await
            .unwrap();

        created.content = "updated todo".to_string();
        created.status = TodoStatus::Done;

        let id = created.id.clone();
        let created_at = created.created_at;
        let before = created.updated_at;

        let updated = repo.update_todo(created).await.unwrap();

        assert_eq!(updated.id, id);
        assert_eq!(updated.content, "updated todo".to_string());
        assert_eq!(updated.status, TodoStatus::Done);
        assert_eq!(updated.created_at, created_at, "更新不改 created_at");
        assert!(updated.updated_at >= before);
    }

    #[sqlx::test]
    async fn clear_todos(pool: PgPool) {
        let repo = PgTodoRepo::new(pool);
        let created = repo
            .insert_todo(Todo::create(NS, "new todo"))
            .await
            .unwrap();

        assert!(
            !repo
                .clear_todos("other".to_string(), vec![created.id.1])
                .await
                .unwrap(),
            "别的租户删不到"
        );
        assert!(
            repo.clear_todos(NS.to_string(), vec![created.id.1])
                .await
                .unwrap()
        );
        assert!(
            repo.query_todos(NS.to_string(), None)
                .await
                .unwrap()
                .is_empty()
        );
    }

    /// 库里出现无法识别的 status 时返回错误，不 panic（release 是 panic=abort，panic 会让进程退出）。
    #[sqlx::test]
    async fn unknown_status_in_db_is_an_error_not_a_panic(pool: PgPool) {
        sqlx::query("INSERT INTO todos (namespace, content, status, created_at, updated_at) VALUES ('default', 'x', 'Bogus', now(), now())")
            .execute(&pool)
            .await
            .unwrap();
        let repo = PgTodoRepo::new(pool);
        assert!(repo.query_todos(NS.to_string(), None).await.is_err());
    }
}
