use std::str::FromStr;
use std::time::SystemTime;
use deadpool_postgres::Pool;
use super::todo_domain::TodoStatus;
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;
use postgres_types::{FromSql, ToSql};
use crate::domains::todo_domain::Todo;
use anyhow::Result;

#[derive(PostgresMapper, Debug, FromSql, ToSql)]
#[pg_mapper(table = "todos")]
struct TodoEntity {
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

    pub async fn query_by_id(&self, id: i32) -> Result<Todo> {
        let client = self.db.get().await?;

        let statement = client
            .prepare_cached("SELECT * FROM todos WHERE id = $1").await?;

        let row = client.query_one(&statement, &[&id]).await?;

        let entity = TodoEntity::from_row(row).map(Todo::from)?;

        Ok(entity)
    }

    pub async fn query_todos(&self, status: Option<TodoStatus>) -> Result<Vec<Todo>> {
        let client = self.db.get().await?;

        let rows = match status {
            Some(value) => {
                let statement = client
                    .prepare_cached( "SELECT * FROM todos WHERE status = $1").await?;

                client.query(&statement, &[&value.to_string()]).await?
            }
            None => {
                let statement = client
                    .prepare_cached( "SELECT * FROM todos").await?;

                client.query(&statement, &[]).await?
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
                INSERT INTO todos (content, status, created_at, updated_at)
                VALUES ($1, $2, $3, $4)
                RETURNING *
                "#).await?;

        let row = client.query_one(&statement,
                                   &[
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
            UPDATE todos SET content = $2, status = $3, updated_at = $4
            WHERE id = $1
            RETURNING *
        "#).await?;

        let row = client.query_one(
            &statement,
            &[
                &entity.id,
                &entity.content,
                &entity.status,
                &entity.updated_at
            ]).await?;

        let entity = TodoEntity::from_row(row).map(Todo::from)?;

        Ok(entity)
    }

    #[allow(unused)]
    pub async fn delete_todo(&self, todo: Todo) -> Result<bool> {
        let client = self.db.get().await?;

        let statement = client
            .prepare_cached("DELETE FROM todos WHERE id = $1").await?;

        let rows = client.execute(&statement, &[&todo.id]).await?;

        Ok(rows == 1)
    }
}

impl From<Todo> for TodoEntity {
    fn from(todo: Todo) -> Self {
        Self {
            id: todo.id,
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
            id: todo.id,
            content: todo.content,
            status: TodoStatus::from_str(&todo.status).unwrap(),
            created_at: todo.created_at,
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

    #[actix_web::test]
    async fn query_by_id() {
        let repo = repo();
        let not_found = repo.query_by_id(123).await;

        assert!(not_found.is_err());

        let created = repo.insert_todo(Todo::create("new todo")).await
            .unwrap();

        let found = repo.query_by_id(created.id).await.unwrap();
        assert_eq!(found.content, created.content);
        assert_eq!(found.status, created.status)
    }

    #[actix_web::test]
    async fn query_todos() {
        let repository = repo();
        let todos = repository.query_todos(None).await;
        println!("{:?}", todos);
    }

    #[actix_web::test]
    async fn insert_todo() {
        let todo = Todo::create("new todo");
        let created = repo().insert_todo(todo).await.unwrap();

        assert_ne!(created.id, 0);
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
        let id = created.id;

        let updated = repo.update_todo(created).await.unwrap();

        assert_eq!(updated.id, id);
        assert_eq!(updated.content, "updated todo".to_string());
        assert_eq!(updated.status, TodoStatus::Done);
    }

    #[actix_web::test]
    async fn delete_todo() {
        let repo = repo();
        let todo = Todo::create("new todo");
        let created = repo.insert_todo(todo).await.unwrap();

        let deleted = repo.delete_todo(created).await.unwrap();

        assert!(deleted);
    }
}
