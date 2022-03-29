use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(index));
}

async fn index() -> String {
    "Hello Actix Web!".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn test_index() {
        let string = index().await;
        assert_eq!(string, "Hello Actix Web!");
    }
}

