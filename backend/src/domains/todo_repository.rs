use std::str::FromStr;
use std::time::SystemTime;
use deadpool_postgres::Pool;
use super::todo_domain::TodoStatus;
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;
use postgres_types::{FromSql, ToSql};
use crate::domains::todo_domain::{Todo, TodoIdentify};
use anyhow::Result;

#[derive(PostgresMapper, Debug, FromSql, ToSql)]
#[pg_mapper(table = "todos")]
struct TodoEntity {
    namespace: String,
    id: i32,
    content: String,
    status: String,
    created_at: SystemTime,
    updated_at: SystemTime,
}

#[derive(Clone)]
pub struct TodoRepository {
    db: Pool,
}

impl TodoRepository {
    pub fn new(db: Pool) -> Self {
        Self { db }
    }

    pub async fn query_by_id(&self, (namespace, id): TodoIdentify) -> Result<Todo> {
        let client = self.db.get().await?;

        let statement = client
            .prepare_cached("SELECT * FROM todos WHERE namespace = $1 AND id = $2").await?;

        let row = client.query_one(&statement, &[&namespace, &id]).await?;

        let entity = TodoEntity::from_row(row).map(Todo::from)?;

        Ok(entity)
    }

    pub async fn query_todos(&self, namespace: String, status: Option<TodoStatus>) -> Result<Vec<Todo>> {
        let client = self.db.get().await?;

        // todo improve
        let rows = match status {
            Some(value) => {
                let statement = client
                    .prepare_cached("SELECT * FROM todos WHERE namespace = $1 AND status = $2").await?;

                client.query(&statement, &[&namespace, &value.to_string()]).await?
            }
            None => {
                let statement = client
                    .prepare_cached("SELECT * FROM todos WHERE namespace = $1").await?;

                client.query(&statement, &[&namespace]).await?
            }
        };

        let entities = rows
            .into_iter()
            .map(|r| TodoEntity::from_row(r).unwrap())
            .map(Todo::from)
            .collect();

        Ok(entities)
    }

    pub async fn insert_todo(&self, todo: Todo) -> Result<Todo> {
        let entity = TodoEntity::from(todo);

        let client = self.db.get().await?;

        let statement = client.prepare_cached(r#"
                INSERT INTO todos (namespace, content, status, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING *
                "#).await?;

        let row = client.query_one(&statement,
                                   &[
                                       &entity.namespace,
                                       &entity.content,
                                       &entity.status,
                                       &entity.created_at,
                                       &entity.updated_at
                                   ]).await?;

        let entity = TodoEntity::from_row(row).map(Todo::from)?;

        Ok(entity)
    }

    pub async fn update_todo(&self, todo: Todo) -> Result<Todo> {
        let entity = TodoEntity::from(todo);

        let client = self.db.get().await?;

        let statement = client.prepare_cached(r#"
            UPDATE todos SET content = $3, status = $4, updated_at = $5
            WHERE namespace = $1 AND id = $2
            RETURNING *
        "#).await?;

        let row = client.query_one(
            &statement,
            &[
                &entity.namespace,
                &entity.id,
                &entity.content,
                &entity.status,
                &entity.updated_at
            ]).await?;

        let entity = TodoEntity::from_row(row).map(Todo::from)?;

        Ok(entity)
    }

    #[allow(unused)]
    pub async fn delete_todo(&self, (namespace, id): TodoIdentify) -> Result<bool> {
        let client = self.db.get().await?;

        let statement = client
            .prepare_cached("DELETE FROM todos WHERE namespace = $1 AND id = $2").await?;

        let rows = client.execute(&statement, &[&namespace, &id]).await?;

        Ok(rows == 1)
    }
}

impl From<Todo> for TodoEntity {
    fn from(todo: Todo) -> Self {
        Self {
            namespace: todo.identify.0,
            id: todo.identify.1,
            content: todo.content,
            status: todo.status.to_string(),
            created_at: todo.created_at,
            updated_at: SystemTime::now(),
        }
    }
}

impl From<TodoEntity> for Todo {
    fn from(todo: TodoEntity) -> Self {
        Self {
            identify: (todo.namespace, todo.id),
            content: todo.content,
            status: TodoStatus::from_str(&todo.status).unwrap(),
            created_at: todo.created_at,
            updated_at: todo.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use deadpool_postgres::Pool;
    use tokio_postgres::NoTls;
    use crate::domains::todo_domain::{Todo, TodoStatus};
    use super::TodoRepository;

    fn test_db() -> Pool {
        let mut config = deadpool_postgres::Config::default();
        config.dbname = Some("rust_fullstack_todo".to_string());
        config.host = Some("localhost".to_string());
        config.port = Some(5432);
        config.user = Some("user".to_string());
        config.password = Some("password".to_string());

        config.create_pool(None, NoTls).unwrap()
    }

    fn repo() -> TodoRepository {
        TodoRepository::new(test_db())
    }

    static NS: &'static str = "default";

    #[actix_web::test]
    async fn query_by_id() {
        let repo = repo();
        let not_found = repo.query_by_id((NS.to_string(), 123)).await;

        assert!(not_found.is_err());

        let created = repo.insert_todo(Todo::create("new todo")).await
            .unwrap();

        let found = repo.query_by_id(created.identify).await.unwrap();
        assert_eq!(found.content, created.content);
        assert_eq!(found.status, created.status)
    }

    #[actix_web::test]
    async fn query_todos() {
        let repository = repo();
        let todos = repository.query_todos(NS.to_string(), None).await;
        println!("{:?}", todos);
    }

    #[actix_web::test]
    async fn insert_todo() {
        let todo = Todo::create("new todo");
        let created = repo().insert_todo(todo).await.unwrap();

        assert_ne!(created.identify.1, 0);
        assert_eq!(created.identify.0, "default");
        assert_eq!(created.content, "new todo".to_string());
        assert_eq!(created.status, TodoStatus::Todo);
    }

    #[actix_web::test]
    async fn update_todo() {
        let repo = repo();

        let todo = Todo::create("new todo");
        let mut created = repo.insert_todo(todo).await.unwrap();

        created.content = "updated todo".to_string();
        created.status = TodoStatus::Done;

        let id = created.identify.clone();

        let updated = repo.update_todo(created).await.unwrap();

        assert_eq!(updated.identify, id);
        assert_eq!(updated.content, "updated todo".to_string());
        assert_eq!(updated.status, TodoStatus::Done);
    }

    #[actix_web::test]
    async fn delete_todo() {
        let repo = repo();
        let todo = Todo::create("new todo");
        let created = repo.insert_todo(todo).await.unwrap();

        let deleted = repo.delete_todo(created.identify).await.unwrap();

        assert!(deleted);
    }
}
