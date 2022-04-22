use common::client::{TodoClient, ScopeClient};
use common::model::{CreateTodoRequest, TodoResponse, TodoStatus, UpdateTodoRequest};
use crate::helper::spawn_server;

mod helper;

pub const NS: &str = "testing/integration";

#[tokio::test]
async fn get_todos() -> anyhow::Result<()> {
    let base_url = spawn_server();

    let client = ScopeClient::default()
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

    let client = ScopeClient::default()
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
    let expect = vec![&todo_3, &todo_2, &todo_1];
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
async fn get_todos_query_status() -> anyhow::Result<()> {
    let base_url = spawn_server();

    let client = ScopeClient::default()
        .endpoint(&base_url)
        .namespace(&format!("{}{}", NS, "/query_status"))
        .todo_client();

    let deleted_todo = create_todo_with_status(&client, Some(TodoStatus::Deleted)).await?;
    let archive_todo = create_todo_with_status(&client, Some(TodoStatus::Archived)).await?;
    let done_todo = create_todo_with_status(&client, Some(TodoStatus::Done)).await?;
    let todo_todo = create_todo_with_status(&client, Some(TodoStatus::Todo)).await?;

    let expect = vec![&todo_todo, &done_todo, &archive_todo, &deleted_todo];
    assert_todos_with_status(&client, None, expect).await?;
    assert_todos_with_status(&client, Some(TodoStatus::Todo), vec![&todo_todo]).await?;
    assert_todos_with_status(&client, Some(TodoStatus::Done), vec![&done_todo]).await?;
    assert_todos_with_status(&client, Some(TodoStatus::Archived), vec![&archive_todo]).await?;
    assert_todos_with_status(&client, Some(TodoStatus::Deleted), vec![&deleted_todo]).await?;

    // clean data
    client.clear_todos(vec![
        todo_todo.id,
        done_todo.id,
        archive_todo.id,
        deleted_todo.id,
    ]).await?;

    Ok(())
}

async fn assert_todos_with_status(client: &TodoClient, status: Option<TodoStatus>, expect: Vec<&TodoResponse>) -> anyhow::Result<()> {
    let todos = client.get_todos(status).await?;
    let actual = todos.iter().collect::<Vec<&TodoResponse>>();
    assert_eq!(expect, actual);

    Ok(())
}

async fn create_todo_with_status(client: &TodoClient, status: Option<TodoStatus>) -> anyhow::Result<TodoResponse> {
    let created = client.create_todo(CreateTodoRequest {
        content: format!("create todo with status {:?}", status),
    }).await?;

    let response = client.update_todo(created.id.clone(), UpdateTodoRequest {
        content: None,
        status,
    }).await?;

    Ok(response)
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
        status: Some(TodoStatus::Done),
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

fn client() -> TodoClient {
    let base_url = spawn_server();

    ScopeClient::default()
        .endpoint(&base_url)
        .namespace(NS)
        .todo_client()
}
