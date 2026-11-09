use actix_web::{HttpResponse, web};
use crate::{Namespace, TodoDomainTrait};
use crate::infra::db::RecordNotFound;
use crate::todo_handler::{TodoResponse, WrappedAnyhowError};

pub async fn get_todo_by_id(
    domain: web::Data<dyn TodoDomainTrait>,
    namespace: web::ReqData<Namespace>,
    path: web::Path<i32>,
) -> Result<HttpResponse, WrappedAnyhowError> {
    let id = path.into_inner();

    match domain.get_todo_by_id((namespace.get(), id)).await {
        Ok(res) => Ok(HttpResponse::Ok().json(TodoResponse::from(res))),
        Err(e) => {
            if e.is::<RecordNotFound>() {
                Ok(HttpResponse::NotFound().finish())
            } else {
                Err(e.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::test::TestRequest;

    use crate::tests::{MISSING_ID, Mock, call, json};

    #[actix_web::test]
    async fn found_is_200_with_todo_json() {
        let (status, body) = call(Arc::new(Mock::default()), TestRequest::get().uri("/todos/7").insert_header(("t-ns", "ns-a"))).await;
        assert_eq!(status, 200);
        let v = json(&body);
        assert_eq!((v["id"].as_i64(), v["namespace"].as_str()), (Some(7), Some("ns-a")));
    }

    #[actix_web::test]
    async fn missing_is_404_with_empty_body() {
        let (status, body) = call(Arc::new(Mock::default()), TestRequest::get().uri(&format!("/todos/{MISSING_ID}"))).await;
        assert_eq!(status, 404);
        assert!(body.is_empty());
    }

    #[actix_web::test]
    async fn non_numeric_id_is_404() {
        let (status, _) = call(Arc::new(Mock::default()), TestRequest::get().uri("/todos/nan")).await;
        assert_eq!(status, 404);
    }
}
