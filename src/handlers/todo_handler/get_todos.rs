use actix_web::{web, HttpResponse};
use std::collections::HashMap;
use crate::handlers::todo_handler::{TodoResponse, TodoStatus};

pub async fn get_todos(query: web::Query<HashMap<String, String>>) -> HttpResponse {
    let _status = query.get("status");

    HttpResponse::Ok().json(vec![TodoResponse {
        id: 1,
        content: String::from("first thing"),
        status: TodoStatus::Todo,
    }])
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::body::to_bytes;
    use actix_web::http;
    use actix_web::test::TestRequest;
    use actix_web::web::Query;

    #[actix_web::test]
    async fn test_get_todos() {
        let request = TestRequest::default().to_http_request();
        let query = Query::from_query(&request.query_string()).unwrap();
        let resp = get_todos(query).await;

        assert_eq!(resp.status(), http::StatusCode::OK);
        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(
            body_bytes,
            r##"[{"id":1,"content":"first thing","status":"Todo"}]"##
        );
    }

    #[actix_web::test]
    async fn test_get_todos_by_status() {
        let request = TestRequest::default().uri("?status=todo").to_http_request();
        let query = Query::from_query(&request.query_string()).unwrap();
        let resp = get_todos(query).await;

        assert_eq!(resp.status(), http::StatusCode::OK);
        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(
            body_bytes,
            r##"[{"id":1,"content":"first thing","status":"Todo"}]"##
        );
    }
}
