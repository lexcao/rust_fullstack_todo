use actix_web::{HttpResponse, web};
use serde_json::json;
use common::model::UpdateTodoRequest;
use crate::domains::todo_domain::UpdateTodo;
use crate::handlers::todo_handler::TodoResponse;
use crate::todo_handler::WrappedAnyhowError;
use crate::{Namespace, TodoDomainTrait};

pub async fn update_todo(
    domain: web::Data<dyn TodoDomainTrait>,
    namespace: web::ReqData<Namespace>,
    path: web::Path<i32>,
    body: web::Json<UpdateTodoRequest>,
) -> Result<HttpResponse, WrappedAnyhowError> {
    let id = path.into_inner();
    let namespace = namespace.get();
    let body = body.into_inner();

    if body.status.is_none() && body.content.is_none() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "expect one of `status` or `content`"
        })));
    }

    let res = domain.update_todo((namespace, id), UpdateTodo::from(body)).await?;

    Ok(HttpResponse::Ok().json(TodoResponse::from(res)))
}

impl From<UpdateTodoRequest> for UpdateTodo {
    fn from(req: UpdateTodoRequest) -> Self {
        Self {
            content: req.content,
            status: req.status,
        }
    }
}

#[cfg(test)]
mod tests {
    use actix_web::test;

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
