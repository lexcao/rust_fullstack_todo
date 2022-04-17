use actix_web::{HttpMessage, web};
use actix_web::dev::{ServiceRequest};

pub mod todo_handler;
pub mod hello_handler;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.configure(hello_handler::configure);

    cfg.service(web::scope("/todos")
        .configure(todo_handler::configure));
}

#[derive(Clone)]
pub struct Namespace(String);

impl Namespace {
    pub fn inject(req: &ServiceRequest) {
        let namespace = req.headers().get("t-ns")
            .map(|h| h.to_str().unwrap())
            .unwrap_or_else(|| "default")
            .to_string();
        req.extensions_mut().insert(Namespace(namespace));
    }

    pub fn get(&self) -> String {
        self.0.clone()
    }
}