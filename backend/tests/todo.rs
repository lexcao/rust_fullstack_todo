use std::env;
use backend::domains::todo_domain::TodoStatus;
use backend::domains::todo_domain::TodoStatus::Done;
use backend::handlers::todo_handler::{CreateTodoRequest, TodoResponse, UpdateTodoRequest};
use crate::client::{NS, TodoClient};

mod client;

#[tokio::test]
async fn get_todos() -> anyhow::Result<()> {
    let base_url = spawn_server();

    let client = client::ScopeClient::default()
        .endpoint(&base_url)
        .namespace(&format!("{}{}", NS, "/get_todos"))
        .todo_client();

    let data = client.get_todos(None).await?;

    assert!(data.is_empty());

    Ok(())
}

#[tokio::test]
async fn get_todos_not_empty() -> anyhow::Result<()> {
    let base_url = spawn_server();

    let client = client::ScopeClient::default()
        .endpoint(&base_url)
        .namespace(&format!("{}{}", NS, "/not_empty"))
        .todo_client();

    let todo_1 = client.create_todo(CreateTodoRequest {
        content: "create todo 1".to_string(),
    }).await?;
    let todo_2 = client.create_todo(CreateTodoRequest {
        content: "create todo 2".to_string(),
    }).await?;
    let todo_3 = client.create_todo(CreateTodoRequest {
        content: "create todo 3".to_string(),
    }).await?;

    let todos = client.get_todos(None).await?;

    let actual = todos.iter().collect::<Vec<&TodoResponse>>();
    let expect = vec![&todo_1, &todo_2, &todo_3];
    assert_eq!(expect, actual);

    // clean data
    client.clear_todos(vec![
        todo_1.id,
        todo_2.id,
        todo_3.id,
    ]).await?;

    Ok(())
}

#[tokio::test]
async fn get_todo_by_id() -> anyhow::Result<()> {
    let client = client();

    let res = client.get_todo_by_id(99999).await?;

    assert!(res.is_none());

    Ok(())
}

#[tokio::test]
async fn create_todo() -> anyhow::Result<()> {
    let client = client();

    let data = client.create_todo(CreateTodoRequest {
        content: "create a new todo".to_string(),
    }).await?;

    assert_eq!("create a new todo", data.content);
    assert_eq!(TodoStatus::Todo, data.status);

    // check by id
    client.assert_eq(data.id, &data).await?;

    // clear data
    client.clear_todos(vec![data.id]).await?;

    Ok(())
}

#[tokio::test]
async fn update_todo() -> anyhow::Result<()> {
    let client = client();

    // create first
    let created = client.create_todo(CreateTodoRequest {
        content: "Create todo for update".to_string(),
    }).await?;

    let id = created.id;

    // then update
    let updated = client.update_todo(id, UpdateTodoRequest {
        content: Some("Update todo".to_string()),
        status: Some(Done),
    }).await?;

    // assert
    client.assert_eq(id, &updated).await?;

    // clear data
    client.clear_todos(vec![id]).await?;

    Ok(())
}

// #[tokio::test]
// async fn update_todo_notfound() -> anyhow::Result<()> {
//     todo!()
// }

fn spawn_server() -> String {
    let mut config = backend::infra::config::must_get();

    config.server.port = 0;
    let listener = config.address();
    let port = listener.local_addr().unwrap().port();
    let db_pool = backend::infra::db::must_init(&config.db);

    let _ = tokio::spawn(backend::start_server(listener, db_pool));

    format!("http://127.0.0.1:{}", port)
}

fn client() -> TodoClient {
    let base_url = spawn_server();

    client::ScopeClient::default()
        .endpoint(&base_url)
        .namespace(NS)
        .todo_client()
}
