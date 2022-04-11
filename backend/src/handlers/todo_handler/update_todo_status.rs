use actix_web::{HttpResponse, web};
use crate::domains::todo_domain::{TodoStatus, UpdateTodo};
use crate::handlers::todo_handler::TodoResponse;
use crate::todo_handler::WrappedAnyhowError;
use crate::TodoDomain;

pub async fn update_todo_status(
    domain: web::Data<TodoDomain>,
    path: web::Path<(i32, TodoStatus)>,
) -> Result<HttpResponse, WrappedAnyhowError> {
    let (id, status) = path.into_inner();

    let to_update = UpdateTodo { status: Some(status), content: None };

    let res = domain.update_todo(("default".to_string(), id), to_update).await?;

    Ok(HttpResponse::Ok().json(TodoResponse::from(res)))
}

#[cfg(test)]
mod tests {
    use actix_web::{http, test};
    use actix_web::body::to_bytes;
    use crate::handlers::todo_handler::configure;
    use crate::tests::test_request;

    #[actix_web::test]
    async fn test_update_todo_status() {
        let request = test::TestRequest::patch()
            .uri("/3/done");

        let response = test_request(configure, request, http::StatusCode::OK).await;
        let body_bytes = to_bytes(response.into_body()).await.unwrap();

        assert_eq!(body_bytes, r##"{"id":3,"content":"todo content","status":"Done"}"##, );
    }

    #[actix_web::test]
    async fn test_update_todo_status_invalid_input_status() {
        let request = test::TestRequest::patch()
            .uri("/3/no-status");

        let response = test_request(configure, request, http::StatusCode::NOT_FOUND).await;
        let body_bytes = to_bytes(response.into_body()).await.unwrap();

        assert_eq!(body_bytes, "unknown variant `no-status`, expected one of `todo`, `done`, `archive`, `deleted`");
    }

    #[actix_web::test]
    async fn test_update_todo_status_invalid_tranform_status() {
        let _request = test::TestRequest::patch()
            .uri("/3/done");
    }
}
