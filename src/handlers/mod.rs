use actix_web::web;

pub mod todo_handler;
pub mod hello_handler;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(hello_handler::configure);

    cfg.service(
        web::scope("/todos")
            .configure(todo_handler::configure)
    );
}
