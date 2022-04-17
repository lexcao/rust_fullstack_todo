use actix_web::{HttpResponse, web};
use crate::{Namespace, TodoDomainTrait};
use crate::infra::db::RecordNotFound;
use crate::todo_handler::{TodoResponse, WrappedAnyhowError};

pub async fn get_todo_by_id(
    domain: web::Data<dyn TodoDomainTrait>,
    namespace: web::ReqData<Namespace>,
    path: web::Path<i32>,
) -> Result<HttpResponse, WrappedAnyhowError> {
    let id = path.into_inner();

    match domain.get_todo_by_id((namespace.get(), id)).await {
        Ok(res) => Ok(HttpResponse::Ok().json(TodoResponse::from(res))),
        Err(e) => {
            if e.is::<RecordNotFound>() {
                Ok(HttpResponse::NotFound().finish())
            } else {
                Err(e.into())
            }
        }
    }
}

// TODO unit tests with error handling