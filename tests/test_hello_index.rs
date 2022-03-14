use actix_web::{http, test,  App, Error};
use actix_web::dev::Service;
use actix_web::body::to_bytes;
use rust_fullstack_todo::handlers::*;

#[actix_web::test]
async fn test_index() -> Result<(), Error> {
    let app = App::new().configure(hello_handler::configure);
    let app = test::init_service(app).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = app.call(req).await?;

    assert_eq!(resp.status(), http::StatusCode::OK);

    let response_body = resp.into_body();
    assert_eq!(to_bytes(response_body).await.unwrap(), r##"Hello world!"##);

    Ok(())
}