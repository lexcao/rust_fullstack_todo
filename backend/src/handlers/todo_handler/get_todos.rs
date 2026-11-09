use actix_web::{web, HttpResponse};
use serde::Deserialize;
use common::model::{TodoResponse, TodoStatus};
use crate::domains::todo_domain::TodoDomainTrait;
use crate::Namespace;
use crate::todo_handler::WrappedAnyhowError;

#[derive(Deserialize)]
pub struct GetTodosQuery {
    status: Option<TodoStatus>,
}

pub async fn get_todos(
    domain: web::Data<dyn TodoDomainTrait>,
    namespace: web::ReqData<Namespace>,
    query: web::Query<GetTodosQuery>,
) -> Result<HttpResponse, WrappedAnyhowError> {
    let res: Vec<TodoResponse> = domain
        .list_todo(namespace.get(), query.into_inner().status).await
        .map_err(|err| WrappedAnyhowError { err })?
        .into_iter()
        .map(TodoResponse::from)
        .collect();

    Ok(HttpResponse::Ok().json(res))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::test::TestRequest;
    use common::model::TodoStatus;

    use crate::tests::{Mock, call, json};

    #[actix_web::test]
    async fn lists_todos_of_the_request_namespace() {
        let mock = Arc::new(Mock::default());
        let (status, body) = call(mock.clone(), TestRequest::get().uri("/todos").insert_header(("t-ns", "ns-a"))).await;

        assert_eq!(status, 200);
        let ids: Vec<i64> = json(&body).as_array().unwrap().iter().map(|t| t["id"].as_i64().unwrap()).collect();
        assert_eq!(ids, [2, 1]);
        assert_eq!(*mock.listed.lock().unwrap(), [("ns-a".to_string(), None)]);
    }

    #[actix_web::test]
    async fn missing_namespace_header_means_default() {
        let mock = Arc::new(Mock::default());
        call(mock.clone(), TestRequest::get().uri("/todos")).await;
        assert_eq!(mock.listed.lock().unwrap()[0].0, "default");
    }

    #[actix_web::test]
    async fn status_query_is_lowercase() {
        let mock = Arc::new(Mock::default());
        for (q, expect) in [
            ("todo", TodoStatus::Todo),
            ("done", TodoStatus::Done),
            ("archived", TodoStatus::Archived),
            ("deleted", TodoStatus::Deleted),
        ] {
            let (status, _) = call(mock.clone(), TestRequest::get().uri(&format!("/todos?status={q}"))).await;
            assert_eq!(status, 200, "status={q}");
            assert_eq!(mock.listed.lock().unwrap().last().unwrap().1, Some(expect));
        }
    }

    #[actix_web::test]
    async fn unknown_status_query_is_400() {
        let mock = Arc::new(Mock::default());
        for q in ["Done", "bogus"] {
            let (status, _) = call(mock.clone(), TestRequest::get().uri(&format!("/todos?status={q}"))).await;
            assert_eq!(status, 400, "status={q}");
        }
        assert!(mock.listed.lock().unwrap().is_empty());
    }
}
