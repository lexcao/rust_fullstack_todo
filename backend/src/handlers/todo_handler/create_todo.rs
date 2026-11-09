use actix_web::{web, HttpResponse};
use common::model::{CreateTodoRequest, TodoResponse};
use crate::todo_handler::WrappedAnyhowError;
use crate::{Namespace, TodoDomain};

pub async fn create_todo(
    domain: web::Data<TodoDomain>,
    namespace: web::ReqData<Namespace>,
    body: web::Json<CreateTodoRequest>,
) -> Result<HttpResponse, WrappedAnyhowError> {
    let namespace = namespace.get();
    let todo = body.into_inner();

    let res = domain.create_todo(&namespace, &todo.content).await?;

    Ok(HttpResponse::Created().json(TodoResponse::from(res)))
}

// 单测：create_todo 依赖具体的 TodoDomain（要连数据库），没法用 mock；由 tests/api_contract.rs 覆盖。
