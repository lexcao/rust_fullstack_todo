use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/ping", web::get().to(ping));
}

async fn ping() -> String {
    "pong".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn test_index() {
        let string = ping().await;
        assert_eq!(string, "pong");
    }
}

