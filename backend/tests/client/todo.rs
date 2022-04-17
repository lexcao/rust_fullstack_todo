use std::ops::Deref;
use backend::domains::todo_domain::TodoStatus;
use backend::handlers::IdsRequest;
use backend::handlers::todo_handler::{CreateTodoRequest, TodoResponse, UpdateTodoRequest};
use crate::client::ScopeClient;

pub struct TodoClient(ScopeClient);

impl From<ScopeClient> for TodoClient {
    fn from(c: ScopeClient) -> Self {
        Self(c)
    }
}

impl Deref for TodoClient {
    type Target = ScopeClient;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TodoClient {
    pub async fn get_todos(&self, status: Option<TodoStatus>) -> anyhow::Result<Vec<TodoResponse>> {
        let mut request = self.inner.get(&format!("{}/todos", self.endpoint));

        if let Some(status) = status {
            request = request.query(&["status", &status.to_string()]);
        }

        let response = request.send().await?;

        let data = response.json::<Vec<TodoResponse>>().await?;

        Ok(data)
    }

    pub async fn get_todo_by_id(&self, id: i32) -> anyhow::Result<Option<TodoResponse>> {
        let response = self.inner.get(&format!("{}/todos/{}", self.endpoint, id))
            .send().await?;

        if response.status().as_u16() == 404 {
            return Ok(None);
        }

        let data = response.json::<TodoResponse>().await?;

        Ok(Some(data))
    }

    pub async fn create_todo(&self, body: CreateTodoRequest) -> anyhow::Result<TodoResponse> {
        let response = self.inner.post(&format!("{}/todos", self.endpoint))
            .json(&body)
            .send().await?;

        let data = response.json::<TodoResponse>().await?;

        Ok(data)
    }

    pub async fn update_todo(&self, id: i32, body: UpdateTodoRequest) -> anyhow::Result<TodoResponse> {
        let response = self.inner.patch(&format!("{}/todos/{}", self.endpoint, id))
            .json(&body)
            .send().await?;

        let data = response.json::<TodoResponse>().await?;

        Ok(data)
    }

    pub async fn clear_todos(&self, ids: Vec<i32>) -> anyhow::Result<()> {
        let _ = self.inner.delete(&format!("{}/todos", self.endpoint))
            .json(&IdsRequest { ids })
            .send().await?;
        Ok(())
    }

    pub async fn assert_eq(&self, id: i32, actual: &TodoResponse) -> anyhow::Result<()> {
        let created = self.get_todo_by_id(id).await?;
        assert!(created.is_some());
        let created = created.unwrap();

        assert_eq!(&created, actual);

        Ok(())
    }
}
