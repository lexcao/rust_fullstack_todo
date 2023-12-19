use actix_web::{web, HttpResponse};
use common::model::{CreateTodoRequest, TodoResponse};
use crate::todo_handler::WrappedAnyhowError;
use crate::{Namespace, TodoDomain};

pub async fn create_todo(
    domain: web::Data<TodoDomain>,
    namespace: web::ReqData<Namespace>,
    body: web::Json<CreateTodoRequest>,
) -> Result<HttpResponse, WrappedAnyhowError> {
    let namespace = namespace.get();
    let todo = body.into_inner();

    let res = domain.create_todo(&namespace, &todo.content).await?;

    Ok(HttpResponse::Created().json(TodoResponse::from(res)))
}

#[cfg(test)]
mod tests {
    use actix_web::test;

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
