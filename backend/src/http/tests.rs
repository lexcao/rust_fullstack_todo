//! handler 单测：`tower::ServiceExt::oneshot` + 内存 `TodoRepo`，不连数据库。

use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::body::Body;
use axum::http::{HeaderValue, Request, StatusCode};
use chrono::{TimeZone, Utc};
use common::model::TodoStatus;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tower::ServiceExt;

use crate::http::router;
use crate::infra::db::RecordNotFound;
use crate::repo::{Todo, TodoID, TodoRepo};

/// 这个 id 在 mock 里不存在。
const MISSING_ID: i32 = 999;

#[derive(Default)]
struct MockRepo {
    listed: Mutex<Vec<(String, Option<TodoStatus>)>>,
    inserted: Mutex<Vec<(String, String)>>,
    updated: Mutex<Vec<Todo>>,
    cleared: Mutex<Vec<(String, Vec<i32>)>>,
    fail_clear: bool,
    list_error: Option<String>,
    /// `None` 表示新建任务的 `todo`。
    current_status: Option<TodoStatus>,
    health_error: bool,
    /// 放进 `check_db` 的错误文本。日志和响应里都不能出现它。
    health_secret: String,
}

fn fixed_todo((namespace, id): TodoID) -> Todo {
    let t = Utc.with_ymd_and_hms(2026, 11, 9, 12, 0, 0).unwrap();
    Todo {
        id: (namespace, id),
        content: "fixed".into(),
        status: TodoStatus::Todo,
        created_at: t,
        updated_at: t,
    }
}

#[async_trait::async_trait]
impl TodoRepo for MockRepo {
    async fn query_by_id(&self, id: TodoID) -> anyhow::Result<Todo> {
        if id.1 == MISSING_ID {
            return Err(RecordNotFound.into());
        }
        let mut todo = fixed_todo(id);
        if let Some(status) = self.current_status {
            todo.status = status;
        }
        Ok(todo)
    }

    async fn query_todos(
        &self,
        namespace: String,
        status: Option<TodoStatus>,
    ) -> anyhow::Result<Vec<Todo>> {
        if let Some(message) = &self.list_error {
            return Err(anyhow::anyhow!("{message}"));
        }
        self.listed
            .lock()
            .unwrap()
            .push((namespace.clone(), status));
        Ok(vec![
            fixed_todo((namespace.clone(), 2)),
            fixed_todo((namespace, 1)),
        ])
    }

    async fn insert_todo(&self, todo: Todo) -> anyhow::Result<Todo> {
        self.inserted
            .lock()
            .unwrap()
            .push((todo.id.0.clone(), todo.content.clone()));
        Ok(Todo {
            id: (todo.id.0, 1),
            ..todo
        })
    }

    async fn update_todo(&self, todo: Todo) -> anyhow::Result<Todo> {
        self.updated.lock().unwrap().push(todo.clone());
        Ok(todo)
    }

    async fn clear_todos(&self, namespace: String, ids: Vec<i32>) -> anyhow::Result<bool> {
        self.cleared.lock().unwrap().push((namespace, ids));
        if self.fail_clear {
            return Err(anyhow::anyhow!("delete failed: secret-token"));
        }
        Ok(true)
    }

    async fn check_db(&self) -> anyhow::Result<()> {
        if self.health_error {
            return Err(anyhow::anyhow!(
                "database unavailable: {}",
                self.health_secret
            ));
        }
        Ok(())
    }
}

async fn call(repo: Arc<MockRepo>, req: Request<Body>) -> (StatusCode, Vec<u8>) {
    let response = router(repo).oneshot(req).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, bytes.to_vec())
}

fn body_str(body: &[u8]) -> &str {
    std::str::from_utf8(body).unwrap_or_else(|e| panic!("不是 UTF-8（{e}）：{body:?}"))
}

fn json_body(body: &[u8]) -> Value {
    serde_json::from_slice(body).unwrap_or_else(|e| panic!("不是 JSON（{e}）：{body:?}"))
}

fn get(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).body(Body::empty()).unwrap()
}

fn with_ns(mut req: Request<Body>, ns: &str) -> Request<Body> {
    req.headers_mut().insert("t-ns", ns.parse().unwrap());
    req
}

fn json_req(method: &str, uri: &str, value: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&value).unwrap()))
        .unwrap()
}

fn bad_ns(mut req: Request<Body>) -> Request<Body> {
    // 0xFF 是 HTTP obs-text：hyper 收得下，但 HeaderValue::to_str 失败。以前这里会 unwrap 崩进程。
    req.headers_mut()
        .insert("t-ns", HeaderValue::from_bytes(&[0xff]).unwrap());
    req
}

#[tokio::test]
async fn ping_is_pong_text() {
    let (status, body) = call(Arc::new(MockRepo::default()), get("/ping")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body_str(&body), "pong");
}

#[tokio::test]
async fn lists_todos_of_the_request_namespace() {
    let mock = Arc::new(MockRepo::default());
    let (status, body) = call(mock.clone(), with_ns(get("/todos"), "ns-a")).await;

    assert_eq!(status, StatusCode::OK);
    let ids: Vec<i64> = json_body(&body)
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["id"].as_i64().unwrap())
        .collect();
    assert_eq!(ids, [2, 1]);
    assert_eq!(*mock.listed.lock().unwrap(), [("ns-a".to_string(), None)]);
}

#[tokio::test]
async fn missing_namespace_header_means_default() {
    let mock = Arc::new(MockRepo::default());
    call(mock.clone(), get("/todos")).await;
    assert_eq!(mock.listed.lock().unwrap()[0].0, "default");
}

#[tokio::test]
async fn status_query_is_lowercase() {
    let mock = Arc::new(MockRepo::default());
    for (q, expect) in [
        ("todo", TodoStatus::Todo),
        ("done", TodoStatus::Done),
        ("archived", TodoStatus::Archived),
        ("deleted", TodoStatus::Deleted),
    ] {
        let (status, _) = call(mock.clone(), get(&format!("/todos?status={q}"))).await;
        assert_eq!(status, StatusCode::OK, "status={q}");
        assert_eq!(mock.listed.lock().unwrap().last().unwrap().1, Some(expect));
    }
}

#[tokio::test]
async fn unknown_status_query_is_400() {
    let mock = Arc::new(MockRepo::default());
    for q in ["Done", "bogus"] {
        let (status, _) = call(mock.clone(), get(&format!("/todos?status={q}"))).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "status={q}");
    }
    assert!(mock.listed.lock().unwrap().is_empty());
}

#[tokio::test]
async fn create_is_201_and_keeps_namespace_and_content() {
    let mock = Arc::new(MockRepo::default());
    let (status, body) = call(
        mock.clone(),
        with_ns(
            json_req("POST", "/todos", json!({"content": "buy milk"})),
            "ns-a",
        ),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let v = json_body(&body);
    assert_eq!(
        (
            v["id"].as_i64(),
            v["namespace"].as_str(),
            v["content"].as_str(),
            v["status"].as_str()
        ),
        (Some(1), Some("ns-a"), Some("buy milk"), Some("todo"))
    );
    assert_eq!(
        *mock.inserted.lock().unwrap(),
        [("ns-a".to_string(), "buy milk".to_string())]
    );
}

#[tokio::test]
async fn found_is_200_with_todo_json() {
    let (status, body) = call(
        Arc::new(MockRepo::default()),
        with_ns(get("/todos/7"), "ns-a"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let v = json_body(&body);
    assert_eq!(
        (v["id"].as_i64(), v["namespace"].as_str()),
        (Some(7), Some("ns-a"))
    );
}

#[tokio::test]
async fn missing_is_404_with_empty_body() {
    let (status, body) = call(
        Arc::new(MockRepo::default()),
        get(&format!("/todos/{MISSING_ID}")),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body.is_empty());
}

#[tokio::test]
async fn non_numeric_id_is_404() {
    let (status, _) = call(Arc::new(MockRepo::default()), get("/todos/nan")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn updates_content_and_status() {
    let mock = Arc::new(MockRepo::default());
    let req = with_ns(
        json_req(
            "PATCH",
            "/todos/3",
            json!({"content": "updated", "status": "done"}),
        ),
        "ns-a",
    );
    let (status, body) = call(mock.clone(), req).await;

    assert_eq!(status, StatusCode::OK);
    let v = json_body(&body);
    assert_eq!(
        (
            v["id"].as_i64(),
            v["content"].as_str(),
            v["status"].as_str()
        ),
        (Some(3), Some("updated"), Some("done"))
    );
    let saved = mock.updated.lock().unwrap();
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0].id, ("ns-a".to_string(), 3));
    assert_eq!(saved[0].content, "updated");
    assert_eq!(saved[0].status, TodoStatus::Done);
}

#[tokio::test]
async fn neither_field_is_400_with_exact_error_body() {
    let mock = Arc::new(MockRepo::default());
    let (status, body) = call(mock.clone(), json_req("PATCH", "/todos/3", json!({}))).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        body_str(&body),
        r#"{"error":"expect one of `status` or `content`"}"#
    );
    assert!(mock.updated.lock().unwrap().is_empty());
}

#[tokio::test]
async fn unknown_status_value_is_400() {
    let (status, _) = call(
        Arc::new(MockRepo::default()),
        json_req("PATCH", "/todos/3", json!({"status": "Done"})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn non_numeric_patch_id_is_404() {
    let (status, _) = call(
        Arc::new(MockRepo::default()),
        json_req("PATCH", "/todos/nan", json!({"content": "x"})),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// 不存在的 id：404，JSON `error` 字段，和其它 AppError 4xx 一样。GET 的 404 仍是空 body。
#[tokio::test]
async fn missing_id_is_404_with_json_error() {
    let req = json_req(
        "PATCH",
        &format!("/todos/{MISSING_ID}"),
        json!({"content": "x"}),
    );
    let (status, body) = call(Arc::new(MockRepo::default()), req).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body_str(&body), r#"{"error":"not found"}"#);
}

#[tokio::test]
async fn list_failure_is_generic_500() {
    let mock = Arc::new(MockRepo {
        list_error: Some("db password=secret-token".into()),
        ..MockRepo::default()
    });
    let (status, body) = call(mock, get("/todos")).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    let text = body_str(&body);
    assert_eq!(text, "internal server error");
    assert!(!text.contains("secret-token") && !text.contains("password"));
}

#[tokio::test]
async fn deletes_ids_in_the_request_namespace() {
    let mock = Arc::new(MockRepo::default());
    let req = with_ns(json_req("DELETE", "/todos", json!({"ids": [1, 2]})), "ns-a");
    let (status, body) = call(mock.clone(), req).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.is_empty());
    assert_eq!(
        *mock.cleared.lock().unwrap(),
        [("ns-a".to_string(), vec![1, 2])]
    );
}

#[tokio::test]
async fn empty_ids_is_200_and_touches_nothing() {
    let mock = Arc::new(MockRepo::default());
    let (status, _) = call(
        mock.clone(),
        json_req("DELETE", "/todos", json!({"ids": []})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(mock.cleared.lock().unwrap().is_empty());
}

/// DELETE 遇到仓储错误返回 500，不再当成成功。对外文案不带库错误。
#[tokio::test]
async fn delete_error_is_500() {
    let mock = Arc::new(MockRepo {
        fail_clear: true,
        ..MockRepo::default()
    });
    let (status, body) = call(
        mock.clone(),
        json_req("DELETE", "/todos", json!({"ids": [1]})),
    )
    .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body_str(&body), "internal server error");
    assert!(!body_str(&body).contains("secret-token"));
    assert_eq!(mock.cleared.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn invalid_t_ns_is_400_and_does_not_touch_the_repo() {
    let mock = Arc::new(MockRepo::default());
    let (status, body) = call(mock.clone(), bad_ns(get("/todos"))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json_body(&body)["error"], "invalid t-ns header");
    assert!(mock.listed.lock().unwrap().is_empty());

    let (status, _) = call(mock.clone(), bad_ns(get("/ping"))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn invalid_t_ns_is_400_and_server_keeps_serving() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let app = router(Arc::new(MockRepo::default()));
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let bad = raw_http(
        port,
        b"GET /todos HTTP/1.1\r\nHost: 127.0.0.1\r\nt-ns: \xff\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert!(bad.starts_with("HTTP/1.1 400"), "{bad}");

    let ok = raw_http(
        port,
        b"GET /ping HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert!(ok.starts_with("HTTP/1.1 200"), "{ok}");
    assert!(ok.contains("pong"), "{ok}");
}

async fn raw_http(port: u16, req: &[u8]) -> String {
    let mut last = String::new();
    for _ in 0..20 {
        match TcpStream::connect(("127.0.0.1", port)).await {
            Ok(mut stream) => {
                stream.write_all(req).await.unwrap();
                let _ = stream.shutdown().await;
                let mut buf = Vec::new();
                let read =
                    tokio::time::timeout(Duration::from_secs(2), stream.read_to_end(&mut buf))
                        .await;
                last = String::from_utf8_lossy(&buf).into_owned();
                if read.is_ok() && !last.is_empty() {
                    return last;
                }
            }
            Err(err) => last = err.to_string(),
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("没有拿到响应：{last}");
}

#[tokio::test]
async fn cors_preflight_mirrors_origin_methods_and_headers() {
    let origin = "https://todos.lexcao.io";
    for method in ["GET", "POST", "PATCH", "DELETE", "OPTIONS"] {
        let req = Request::builder()
            .method("OPTIONS")
            .uri("/todos")
            .header("origin", origin)
            .header("access-control-request-method", method)
            .header("access-control-request-headers", "t-ns,content-type")
            .body(Body::empty())
            .unwrap();
        let response = router(Arc::new(MockRepo::default()))
            .oneshot(req)
            .await
            .unwrap();
        assert!(
            response.status().is_success(),
            "预检 {method}：{}",
            response.status()
        );
        let headers = response.headers();
        assert_eq!(
            headers["access-control-allow-origin"], origin,
            "预检 {method}"
        );
        let allow_headers = headers["access-control-allow-headers"]
            .to_str()
            .unwrap()
            .to_ascii_lowercase();
        assert!(
            allow_headers.contains("t-ns") && allow_headers.contains("content-type"),
            "{allow_headers}"
        );
        let allow_methods = headers["access-control-allow-methods"].to_str().unwrap();
        assert!(allow_methods.contains(method), "{allow_methods}");
    }

    let req = Request::builder()
        .uri("/todos")
        .header("origin", origin)
        .header("t-ns", "ns-a")
        .body(Body::empty())
        .unwrap();
    let response = router(Arc::new(MockRepo::default()))
        .oneshot(req)
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["access-control-allow-origin"], origin);
}

#[tokio::test]
async fn fresh_todo_can_move_to_deleted_and_archived_can_move_to_deleted() {
    let (status, _) = call(
        Arc::new(MockRepo::default()),
        json_req("PATCH", "/todos/3", json!({"status": "deleted"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let mock = Arc::new(MockRepo {
        current_status: Some(TodoStatus::Archived),
        ..MockRepo::default()
    });
    let (status, body) = call(
        mock.clone(),
        json_req("PATCH", "/todos/3", json!({"status": "deleted"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{}", body_str(&body));
    assert_eq!(mock.updated.lock().unwrap()[0].status, TodoStatus::Deleted);
}

#[tokio::test]
async fn done_cannot_skip_archive_on_the_way_to_deleted() {
    let mock = Arc::new(MockRepo {
        current_status: Some(TodoStatus::Done),
        ..MockRepo::default()
    });
    let (status, body) = call(
        mock.clone(),
        json_req("PATCH", "/todos/3", json!({"status": "deleted"})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        body_str(&body),
        r#"{"error":"invalid status transition: done -> deleted"}"#
    );
    assert!(mock.updated.lock().unwrap().is_empty());
}

#[tokio::test]
async fn healthz_is_200_or_503_from_the_repo_check() {
    let (status, body) = call(Arc::new(MockRepo::default()), get("/healthz")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_empty());

    let secret = "db-secret-do-not-log";
    let (status, body) = call(
        Arc::new(MockRepo {
            health_error: true,
            health_secret: secret.to_owned(),
            ..MockRepo::default()
        }),
        get("/healthz"),
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body_str(&body), r#"{"error":"database unavailable"}"#);
    assert!(!body_str(&body).contains(secret));
}

#[tokio::test]
async fn request_timeout_is_408() {
    let app = super::with_observability(
        axum::Router::new().route(
            "/slow",
            axum::routing::get(|| async {
                tokio::time::sleep(Duration::from_secs(2)).await;
                "late"
            }),
        ),
        Duration::from_millis(200),
    );
    let response = app.oneshot(get("/slow")).await.unwrap();
    assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
}

#[tokio::test]
async fn request_id_is_echoed() {
    let request_id = "req-trace-7c2a";
    let response = router(Arc::new(MockRepo::default()))
        .oneshot(
            Request::builder()
                .uri("/ping")
                .header("x-request-id", request_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.headers()["x-request-id"], request_id);
    assert_eq!(response.status(), StatusCode::OK);

    let generated = router(Arc::new(MockRepo::default()))
        .oneshot(get("/ping"))
        .await
        .unwrap();
    let generated_id = generated.headers()["x-request-id"].to_str().unwrap();
    assert!(!generated_id.is_empty());
    assert_ne!(generated_id, request_id);
}
