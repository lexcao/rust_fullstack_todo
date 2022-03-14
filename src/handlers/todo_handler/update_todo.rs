use actix_web::{HttpResponse, web};
use crate::handlers::todo_handler::{CreateTodoRequest, TodoResponse, TodoStatus};

pub async fn update_todo(
    path: web::Path<u32>,
    body: web::Json<CreateTodoRequest>,
) -> HttpResponse {
    let id = path.into_inner();
    let todo = body.into_inner();
    HttpResponse::Ok().json(
        TodoResponse {
            id,
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
    async fn test_update_todo() {
        let request = test::TestRequest::patch()
            .uri("/3")
            .set_json(json!({"content": "updated a todo"}));

        let response = test_request(configure, request, http::StatusCode::OK).await;
        let body_bytes = to_bytes(response.into_body()).await.unwrap();

        assert_eq!(body_bytes, r##"{"id":3,"content":"updated a todo","status":"Todo"}"##, );
    }

    #[actix_web::test]
    async fn test_update_todo_not_found() {
        let request = test::TestRequest::patch()
            .uri("/999")
            .set_json(json!({"content": "updated a todo"}));

        let response = test_request(configure, request, http::StatusCode::NOT_FOUND).await;
        let body_bytes = to_bytes(response.into_body()).await.unwrap();

        assert_eq!(body_bytes, r##"{"code":"TodoNotFound","message":"Todo [id: 999] not found.}"##, );
    }

    #[actix_web::test]
    async fn test_update_todo_invalid_id_not_number() {
        let request = test::TestRequest::patch()
            .uri("/nan")
            .set_json(json!({"content": "updated a todo"}));

        let response = test_request(configure, request, http::StatusCode::NOT_FOUND).await;
        let body_bytes = to_bytes(response.into_body()).await.unwrap();

        assert_eq!(body_bytes, r##"{"id":3,"content":"updated a todo","status":"Todo"}"##, );
    }
}
