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
    use std::sync::Arc;

    use actix_web::test::TestRequest;
    use common::model::TodoStatus;
    use serde_json::json;

    use crate::tests::{MISSING_ID, Mock, call, json};

    #[actix_web::test]
    async fn updates_content_and_status() {
        let mock = Arc::new(Mock::default());
        let req = TestRequest::patch()
            .uri("/todos/3")
            .insert_header(("t-ns", "ns-a"))
            .set_json(json!({"content": "updated", "status": "done"}));
        let (status, body) = call(mock.clone(), req).await;

        assert_eq!(status, 200);
        let v = json(&body);
        assert_eq!((v["id"].as_i64(), v["content"].as_str(), v["status"].as_str()), (Some(3), Some("updated"), Some("done")));
        assert_eq!(
            *mock.updated.lock().unwrap(),
            [(("ns-a".to_string(), 3), Some("updated".to_string()), Some(TodoStatus::Done))]
        );
    }

    #[actix_web::test]
    async fn neither_field_is_400_with_exact_error_body() {
        let mock = Arc::new(Mock::default());
        let (status, body) = call(mock.clone(), TestRequest::patch().uri("/todos/3").set_json(json!({}))).await;

        assert_eq!(status, 400);
        assert_eq!(json(&body), json!({"error": "expect one of `status` or `content`"}));
        assert!(mock.updated.lock().unwrap().is_empty());
    }

    #[actix_web::test]
    async fn unknown_status_value_is_400() {
        let (status, _) = call(Arc::new(Mock::default()), TestRequest::patch().uri("/todos/3").set_json(json!({"status": "Done"}))).await;
        assert_eq!(status, 400);
    }

    #[actix_web::test]
    async fn non_numeric_id_is_404() {
        let (status, _) = call(Arc::new(Mock::default()), TestRequest::patch().uri("/todos/nan").set_json(json!({"content": "x"}))).await;
        assert_eq!(status, 404);
    }

    /// 现状：不存在的 id 返回 500（RecordNotFound 没映射）。P3.4 按契约改成 404。
    #[actix_web::test]
    async fn missing_id_is_currently_500() {
        let req = TestRequest::patch().uri(&format!("/todos/{MISSING_ID}")).set_json(json!({"content": "x"}));
        let (status, _) = call(Arc::new(Mock::default()), req).await;
        assert_eq!(status, 500);
    }
}
