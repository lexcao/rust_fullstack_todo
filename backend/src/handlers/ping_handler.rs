use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/ping", web::get().to(ping));
}

async fn ping() -> String {
    "pong".to_string()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::test::TestRequest;

    use crate::tests::{Mock, call};

    #[actix_web::test]
    async fn ping_is_pong_text() {
        let (status, body) = call(Arc::new(Mock::default()), TestRequest::get().uri("/ping")).await;
        assert_eq!(status, 200);
        assert_eq!(body, "pong");
    }
}
