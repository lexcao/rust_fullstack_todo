use actix_web::{web, HttpResponse};
use crate::handlers::todo_handler::{CreateTodoRequest, TodoResponse, TodoStatus};

pub async fn create_todo(body: web::Json<CreateTodoRequest>) -> HttpResponse {
    let todo = body.into_inner();

    HttpResponse::Created().json(
        TodoResponse {
            id: 2,
            content: todo.content,
            status: TodoStatus::Todo,
        }
    )
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
