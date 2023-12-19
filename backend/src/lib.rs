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
    use actix_web::{App, test};
    use actix_web::body::{BoxBody, to_bytes};
    use actix_web::dev::ServiceResponse;
    use actix_web::http::StatusCode;
    use actix_web::test::TestRequest;
    use actix_web::web::ServiceConfig;

    #[cfg(test)]
    pub async fn test_request<C>(
        configure: C,
        req: TestRequest,
        expect_status_code: StatusCode,
    ) -> ServiceResponse<BoxBody>
        where C: FnOnce(&mut ServiceConfig)
    {
        let app = test::init_service(App::new().configure(configure)).await;
        let resp = req.send_request(&app).await;
        if resp.status() != expect_status_code {
            let body_bytes = to_bytes(resp.into_body()).await.unwrap();
            panic!("response body is {:?}", body_bytes)
        }
        assert_eq!(resp.status(), expect_status_code);
        return resp;
    }
}
