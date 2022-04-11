use actix_web::{middleware, App, HttpServer, web};
use std::fmt::Debug;
use std::io;
use std::io::ErrorKind;
use tokio_postgres::NoTls;
use crate::domains::todo_domain::TodoDomain;
use crate::handlers::todo_handler;
use crate::infra::config::Config;

pub mod handlers;
pub mod applications;
pub mod domains;
pub mod infra;

pub async fn start_server(config: Config) -> std::io::Result<()> {
    let pool = config.db.create_pool(None, NoTls)
        .map_err(|e| io::Error::new(ErrorKind::NotConnected, e))?;

    log::info!("starting HTTP server at http://localhost:{}", config.server.port);

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(TodoDomain::new(pool.clone())))
            .configure(handlers::routes)
    })
        .bind(("127.0.0.1", config.server.port))?
        .run()
        .await
}

pub fn assert_vec<T: PartialEq + Debug>(a: &[T], b: &[T]) {
    assert_eq!(a.len(), b.len(), "length not equal");

    for (i, _) in a.iter().enumerate() {
        assert_eq!(a[i], b[i]);
    }
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
