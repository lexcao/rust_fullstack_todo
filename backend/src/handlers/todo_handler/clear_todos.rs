use actix_web::{HttpResponse, web};
use common::model::IdsRequest;
use crate::{Namespace, TodoDomainTrait};
use crate::todo_handler::WrappedAnyhowError;

pub async fn clear_todos(
    domain: web::Data<dyn TodoDomainTrait>,
    ids: web::Json<IdsRequest<i32>>,
    namespace: web::ReqData<Namespace>,
) -> Result<HttpResponse, WrappedAnyhowError> {
    let namespace = namespace.get();
    let ids = ids.into_inner().ids;

    if !ids.is_empty() {
        let _ = domain.clear_todos(namespace, ids).await;
    }

    Ok(HttpResponse::Ok().finish())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::test::TestRequest;
    use serde_json::json;

    use crate::tests::{Mock, call};

    #[actix_web::test]
    async fn deletes_ids_in_the_request_namespace() {
        let mock = Arc::new(Mock::default());
        let req = TestRequest::delete().uri("/todos").insert_header(("t-ns", "ns-a")).set_json(json!({"ids": [1, 2]}));
        let (status, body) = call(mock.clone(), req).await;

        assert_eq!(status, 200);
        assert!(body.is_empty());
        assert_eq!(*mock.cleared.lock().unwrap(), [("ns-a".to_string(), vec![1, 2])]);
    }

    #[actix_web::test]
    async fn empty_ids_is_200_and_touches_nothing() {
        let mock = Arc::new(Mock::default());
        let (status, _) = call(mock.clone(), TestRequest::delete().uri("/todos").set_json(json!({"ids": []}))).await;
        assert_eq!(status, 200);
        assert!(mock.cleared.lock().unwrap().is_empty());
    }
}
