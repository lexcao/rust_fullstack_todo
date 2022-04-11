use actix_web::{web, HttpResponse};
use crate::handlers::todo_handler::{TodoResponse, TodoStatus, WrappedAnyhowError};
use crate::TodoDomain;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetTodosQuery {
    status: Option<TodoStatus>,
}

pub async fn get_todos(
    domain: web::Data<TodoDomain>,
    query: web::Query<GetTodosQuery>,
) -> Result<HttpResponse, WrappedAnyhowError> {
    let res: Vec<TodoResponse> = domain
        .list_todo("default".to_string(), query.into_inner().status).await
        .map_err(|e| WrappedAnyhowError { e })?
        .into_iter()
        .map(TodoResponse::from)
        .collect();

    Ok(HttpResponse::Ok().json(res))
}

#[cfg(test)]
mod tests {
    use actix_web::body::to_bytes;
    use actix_web::http;
    use actix_web::test::TestRequest;
    use crate::handlers::todo_handler::configure;
    use crate::tests::test_request;

    #[actix_web::test]
    async fn test_get_todos() {
        let request = TestRequest::get().uri("/");

        // TODO mock application data
        let response = test_request(configure, request, http::StatusCode::OK).await;
        let body_bytes = to_bytes(response.into_body()).await.unwrap();

        assert_eq!(body_bytes, r##"{"id":1,"content":"first thing","status":"Todo"}"##, );
    }

    #[actix_web::test]
    async fn test_get_todos_by_status() {
        // let request = TestRequest::default().uri("?status=todo").to_http_request();
        // let query = Query::from_query(&request.query_string()).unwrap();
        // let resp = get_todos(query).await;
        //
        // assert_eq!(resp.status(), http::StatusCode::OK);
        // let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        // assert_eq!(
        //     body_bytes,
        //     r##"[{"id":1,"content":"first thing","status":"Todo"}]"##
        // );
    }
}
