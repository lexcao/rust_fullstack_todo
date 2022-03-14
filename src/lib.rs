use actix_web::{middleware, App, HttpServer, test};
use std::fmt::Debug;
use actix_web::body::BoxBody;
use actix_web::dev::ServiceResponse;
use actix_web::http::StatusCode;
use actix_web::test::TestRequest;
use actix_web::web::ServiceConfig;

pub mod handlers;

pub async fn start_server(port: u16) -> std::io::Result<()> {
    log::info!("starting HTTP server at http://localhost:{}", port);

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .configure(handlers::configure)
    })
        .bind(("127.0.0.1", port))?
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
async fn test_request<C>(
    configure: C,
    req: TestRequest,
    expect_status_code: StatusCode,
) -> ServiceResponse<BoxBody>
    where C: FnOnce(&mut ServiceConfig)
{
    let app = test::init_service(App::new().configure(configure)).await;
    let resp = req.send_request(&app).await;
    assert_eq!(resp.status(), expect_status_code, "{:?}", resp);
    return resp;
}
