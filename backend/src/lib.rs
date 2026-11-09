extern crate core;

use actix_web::{middleware, App, HttpServer, web};
use std::net::TcpListener;
use std::sync::Arc;
use actix_cors::Cors;
use actix_web::dev::{Server, Service};
use deadpool_postgres::Pool;
use handlers::Namespace;
use crate::domains::todo_domain::{TodoDomain, TodoDomainTrait};
use crate::handlers::todo_handler;

pub mod handlers;
pub mod domains;
pub mod infra;

pub fn start_server(listener: TcpListener, db_pool: Pool) -> Server {
    let address = listener.local_addr().unwrap();
    log::info!("starting HTTP server at {}", address);

    HttpServer::new(move || {
        let todo_domain = TodoDomain::new(db_pool.clone());
        let todo_domain_trait = Arc::new(todo_domain.clone())
            as Arc<dyn TodoDomainTrait>;

        App::new()
            .wrap_fn(|req, srv| {
                Namespace::inject(&req);
                srv.call(req)
            })
            .wrap(middleware::Logger::default())
            .wrap(Cors::permissive())
            .app_data(web::Data::from(todo_domain_trait.clone()))
            .app_data(web::Data::new(todo_domain.clone()))
            .configure(handlers::routes)
    })
        .listen(listener)
        .expect("Address is already in use")
        .run()
}

#[cfg(test)]
pub(crate) mod tests {
    //! handler 单测共用：内存 mock 的 TodoDomainTrait + 和 start_server 相同的中间件和路由（不连数据库）。
    use std::sync::{Arc, Mutex};

    use actix_web::body::to_bytes;
    use actix_web::dev::Service;
    use actix_web::http::StatusCode;
    use actix_web::test::{self, TestRequest};
    use actix_web::web::Bytes;
    use actix_web::{App, web};
    use chrono::{TimeZone, Utc};
    use common::model::TodoStatus;

    use crate::domains::todo_domain::{Todo, TodoDomainTrait, TodoID, UpdateTodo};
    use crate::handlers::{self, Namespace};
    use crate::infra::db::RecordNotFound;

    /// mock 记录的一次 update 调用：(id, content, status)。
    pub type UpdateCall = (TodoID, Option<String>, Option<TodoStatus>);

    /// 这个 id 在 mock 里不存在。
    pub const MISSING_ID: i32 = 999;

    /// 记录 handler 传进来的参数，返回固定数据。
    #[derive(Default)]
    pub struct Mock {
        pub listed: Mutex<Vec<(String, Option<TodoStatus>)>>,
        pub updated: Mutex<Vec<UpdateCall>>,
        pub cleared: Mutex<Vec<(String, Vec<i32>)>>,
    }

    pub fn fixed_todo((namespace, id): TodoID) -> Todo {
        let t = Utc.with_ymd_and_hms(2026, 11, 9, 12, 0, 0).unwrap();
        Todo { id: (namespace, id), content: "fixed".into(), status: TodoStatus::Todo, created_at: t, updated_at: t }
    }

    #[async_trait::async_trait]
    impl TodoDomainTrait for Mock {
        async fn get_todo_by_id(&self, id: TodoID) -> anyhow::Result<Todo> {
            if id.1 == MISSING_ID {
                return Err(RecordNotFound.into());
            }
            Ok(fixed_todo(id))
        }

        async fn list_todo(&self, namespace: String, status: Option<TodoStatus>) -> anyhow::Result<Vec<Todo>> {
            self.listed.lock().unwrap().push((namespace.clone(), status));
            Ok(vec![fixed_todo((namespace.clone(), 2)), fixed_todo((namespace, 1))])
        }

        async fn create_todo(&self, namespace: String, content: &str) -> anyhow::Result<Todo> {
            Ok(Todo { content: content.into(), ..fixed_todo((namespace, 1)) })
        }

        async fn update_todo(&self, id: TodoID, to_update: UpdateTodo) -> anyhow::Result<Todo> {
            self.updated.lock().unwrap().push((id.clone(), to_update.content.clone(), to_update.status));
            if id.1 == MISSING_ID {
                return Err(RecordNotFound.into());
            }
            let base = fixed_todo(id);
            Ok(Todo {
                content: to_update.content.unwrap_or(base.content.clone()),
                status: to_update.status.unwrap_or(base.status),
                ..base
            })
        }

        async fn clear_todos(&self, namespace: String, ids: Vec<i32>) -> anyhow::Result<()> {
            self.cleared.lock().unwrap().push((namespace, ids));
            Ok(())
        }
    }

    /// 用 mock 跑一个请求，返回状态码和 body。
    pub async fn call(mock: Arc<Mock>, req: TestRequest) -> (StatusCode, Bytes) {
        let domain = mock as Arc<dyn TodoDomainTrait>;
        let app = test::init_service(
            App::new()
                .wrap_fn(|req, srv| {
                    Namespace::inject(&req);
                    srv.call(req)
                })
                .app_data(web::Data::from(domain))
                .configure(handlers::routes),
        )
        .await;
        let resp = req.send_request(&app).await;
        let status = resp.status();
        (status, to_bytes(resp.into_body()).await.unwrap())
    }

    pub fn json(body: &Bytes) -> serde_json::Value {
        serde_json::from_slice(body).unwrap_or_else(|e| panic!("不是 JSON（{e}）：{body:?}"))
    }
}
