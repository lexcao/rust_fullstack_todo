use actix_web::{HttpResponse, web};

pub async fn delete_todo(_path: web::Path<i32>) -> HttpResponse {
    HttpResponse::Ok().finish()
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