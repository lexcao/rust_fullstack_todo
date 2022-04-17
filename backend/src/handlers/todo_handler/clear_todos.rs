use actix_web::{HttpResponse, web};
use crate::{Namespace, TodoDomainTrait};
use crate::handlers::IdsRequest;
use crate::todo_handler::WrappedAnyhowError;

pub async fn clear_todos(
    domain: web::Data<dyn TodoDomainTrait>,
    ids: web::Json<IdsRequest<i32>>,
    namespace: web::ReqData<Namespace>,
) -> Result<HttpResponse, WrappedAnyhowError> {
    let namespace = namespace.get();
    let ids = ids.into_inner().ids;

    if !ids.is_empty() {
        let _ = domain.clear_todos(namespace, ids).await;
    }

    Ok(HttpResponse::Ok().finish())
}

#[cfg(test)]
mod tests {
    use actix_web::{http, test};
    use crate::handlers::todo_handler::configure;
    use crate::tests::test_request;

    #[actix_web::test]
    async fn test_update_todo_status() {
        let request = test::TestRequest::patch()
            .uri("/3");

        test_request(configure, request, http::StatusCode::OK).await;
    }
}