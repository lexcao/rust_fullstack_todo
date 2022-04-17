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
    use actix_web::body::to_bytes;
    use actix_web::{App, test, web};
    use actix_web::test::TestRequest;
    use crate::domains::todo_domain::{Todo, TodoDomainTrait, TodoID, UpdateTodo};
    use crate::handlers::routes;
    use crate::Namespace;
    use actix_web::dev::Service;
    use common::model::TodoStatus;

    struct Mock {}

    impl Mock {
        fn new() -> Self {
            Mock {}
        }
    }

    #[async_trait::async_trait]
    impl TodoDomainTrait for Mock {
        async fn get_todo_by_id(&self, _id: TodoID) -> anyhow::Result<Todo> {
            Ok(Todo::default())
        }

        async fn list_todo(&self, _namespace: String, _status: Option<TodoStatus>) -> anyhow::Result<Vec<Todo>> {
            Ok(vec![])
        }

        async fn create_todo(&self, _namespace: String, _content: &str) -> anyhow::Result<Todo> {
            Ok(Todo::default())
        }

        async fn update_todo(&self, _id: TodoID, _to_update: UpdateTodo) -> anyhow::Result<Todo> {
            Ok(Todo::default())
        }

        async fn clear_todos(&self, _namespace: String, _ids: Vec<i32>) -> anyhow::Result<()> {
            Ok(())
        }
    }

    #[actix_web::test]
    async fn test_get_todos() {
        let service = Arc::new(Mock::new()) as Arc<dyn TodoDomainTrait>;

        let app = App::new()
            .wrap_fn(|req, srv| {
                Namespace::inject(&req);
                srv.call(req)
            })
            .app_data(web::Data::from(service))
            .configure(routes);

        let app = test::init_service(app).await;
        let request = TestRequest::get().uri("/todos");
        let response = request.send_request(&app).await;
        assert_eq!(response.status().as_u16(), 200);

        let body_bytes = to_bytes(response.into_body()).await.unwrap();

        assert_eq!(body_bytes, r##"[]"##, );
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
