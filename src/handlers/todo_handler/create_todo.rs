use actix_web::{web, HttpResponse};
use crate::domains::todo_domain::Todo;
use crate::handlers::todo_handler::{CreateTodoRequest, TodoResponse};
use crate::todo_handler::WrappedAnyhowError;
use crate::TodoDomain;

pub async fn create_todo(
    domain: web::Data<TodoDomain>,
    body: web::Json<CreateTodoRequest>,
) -> Result<HttpResponse, WrappedAnyhowError> {
    let todo = body.into_inner();

    let res = domain.create_todo(Todo::create(&todo.content)).await?;

    Ok(HttpResponse::Created().json(TodoResponse::from(res)))
}

#[cfg(test)]
mod tests {
    use actix_web::body::to_bytes;
    use actix_web::{http, test};
    use serde_json::json;
    use crate::handlers::todo_handler::configure;
    use crate::test_request;

    #[actix_web::test]
    async fn test_create_todo() {
        let request = test::TestRequest::post()
            .uri("/")
            .set_json(json!({"content": "create a todo"}));

        let response = test_request(configure, request, http::StatusCode::CREATED).await;
        let body_bytes = to_bytes(response.into_body()).await.unwrap();

        assert_eq!(body_bytes, r##"{"id":2,"content":"create a todo","status":"Todo"}"##, );
    }
}
