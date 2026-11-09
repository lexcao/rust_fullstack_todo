//! API 契约测试：锁住前端云模式（`common::client`）依赖的线上格式。
//!
//! 对应 `p3-scope-contract.md` 第 6 节「API 兼容承诺」。P3 后续换框架、换数据层时，这个文件不应修改；
//! 标了「现状，P3.4 改」的断言除外（契约允许的向前兼容变化）。
//!
//! 直接发原始 HTTP（不用 `common::client`，它遇到非预期状态码会 panic），断言状态码、头和 JSON 原文。
//! 需要 Postgres：连接参数来自 `application.toml` / `APP_DB_*`，表结构见 `schemas/00_init_tables.sql`。

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU32, Ordering};

use reqwest::{Client, Method, StatusCode};
use serde_json::{Value, json};

use crate::helper::spawn_server;

mod helper;

/// 每个测试一个独立租户（`t-ns`），互不干扰，也不受库里旧数据影响。列宽 VARCHAR(36)。
fn unique_ns() -> String {
    static N: AtomicU32 = AtomicU32::new(0);
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    format!("contract-{}-{}-{}", std::process::id(), nanos % 1_000_000_000, N.fetch_add(1, Ordering::Relaxed))
}

struct Api {
    base: String,
    ns: Option<String>,
    http: Client,
}

impl Api {
    fn new() -> Self {
        Self { base: spawn_server(), ns: Some(unique_ns()), http: Client::new() }
    }

    fn without_ns() -> Self {
        Self { ns: None, ..Self::new() }
    }

    fn ns(&self) -> &str {
        self.ns.as_deref().unwrap_or("default")
    }

    async fn send(&self, method: Method, path: &str, body: Option<Value>) -> (StatusCode, reqwest::header::HeaderMap, String) {
        let mut req = self.http.request(method, format!("{}{path}", self.base));
        if let Some(ns) = &self.ns {
            req = req.header("t-ns", ns);
        }
        if let Some(body) = body {
            req = req.json(&body);
        }
        let resp = req.send().await.expect("请求失败");
        let status = resp.status();
        let headers = resp.headers().clone();
        (status, headers, resp.text().await.unwrap())
    }

    async fn create(&self, content: &str) -> Value {
        let (status, _, body) = self.send(Method::POST, "/todos", Some(json!({ "content": content }))).await;
        assert_eq!(status, StatusCode::CREATED, "POST /todos 必须是 201：{body}");
        serde_json::from_str(&body).unwrap()
    }

    async fn list(&self, query: &str) -> Vec<Value> {
        let (status, _, body) = self.send(Method::GET, &format!("/todos{query}"), None).await;
        assert_eq!(status, StatusCode::OK, "GET /todos{query}：{body}");
        serde_json::from_str(&body).unwrap()
    }

    async fn clear(&self, ids: Vec<i64>) {
        let (status, _, _) = self.send(Method::DELETE, "/todos", Some(json!({ "ids": ids }))).await;
        assert_eq!(status, StatusCode::OK);
    }
}

fn ids(todos: &[Value]) -> Vec<i64> {
    todos.iter().map(|t| t["id"].as_i64().unwrap()).collect()
}

/// TodoResponse 的字段集合、类型和格式。
fn assert_todo_shape(t: &Value, ns: &str) {
    let keys: BTreeSet<&str> = t.as_object().expect("TodoResponse 必须是对象").keys().map(String::as_str).collect();
    assert_eq!(keys, BTreeSet::from(["namespace", "id", "content", "status", "created_at", "updated_at"]), "{t}");
    assert_eq!(t["namespace"], ns);
    let id = t["id"].as_i64().expect("id 是整数");
    assert!(id > 0 && id <= i32::MAX as i64, "id 在 i32 范围：{id}");
    assert!(t["content"].is_string());
    let status = t["status"].as_str().expect("status 是字符串");
    assert!(["todo", "done", "archived", "deleted"].contains(&status), "status 小写：{status}");
    for k in ["created_at", "updated_at"] {
        let s = t[k].as_str().expect("时间是字符串");
        assert!(chrono::DateTime::parse_from_rfc3339(s).is_ok() && s.ends_with('Z'), "{k} 是 UTC RFC 3339（Z 结尾）：{s}");
    }
}

#[tokio::test]
async fn ping_is_plain_text_pong() {
    let api = Api::without_ns();
    let (status, headers, body) = api.send(Method::GET, "/ping", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "pong");
    assert!(headers["content-type"].to_str().unwrap().starts_with("text/plain"), "{headers:?}");
}

#[tokio::test]
async fn create_is_201_with_todo_json() {
    let api = Api::new();
    let t = api.create("buy milk").await;
    assert_todo_shape(&t, api.ns());
    assert_eq!(t["content"], "buy milk");
    assert_eq!(t["status"], "todo");
    api.clear(vec![t["id"].as_i64().unwrap()]).await;
}

#[tokio::test]
async fn list_is_newest_first_and_scoped_to_namespace() {
    let api = Api::new();
    let a = api.create("a").await;
    let b = api.create("b").await;
    let c = api.create("c").await;

    let todos = api.list("").await;
    for t in &todos {
        assert_todo_shape(t, api.ns());
    }
    assert_eq!(ids(&todos), ids(&[c.clone(), b.clone(), a.clone()]), "按 id 倒序");
    assert_eq!(todos[0], c, "列表里的对象和创建返回的一致");

    let other = Api { ns: Some(unique_ns()), ..Api::new() };
    assert!(other.list("").await.is_empty(), "别的 t-ns 看不到");

    api.clear(ids(&todos)).await;
    assert!(api.list("").await.is_empty());
}

#[tokio::test]
async fn status_filter_takes_lowercase_and_db_keeps_capitalized_values() {
    let api = Api::new();
    let mut by_status = Vec::new();
    for status in ["todo", "done", "archived", "deleted"] {
        let t = api.create(status).await;
        let (code, _, body) = api
            .send(Method::PATCH, &format!("/todos/{}", t["id"]), Some(json!({ "status": status })))
            .await;
        assert_eq!(code, StatusCode::OK, "{body}");
        let t: Value = serde_json::from_str(&body).unwrap();
        assert_todo_shape(&t, api.ns());
        assert_eq!(t["status"], status);
        by_status.push((status, t));
    }

    for (status, t) in &by_status {
        let listed = api.list(&format!("?status={status}")).await;
        assert_eq!(ids(&listed), vec![t["id"].as_i64().unwrap()], "?status={status}");
    }
    let (code, _, _) = api.send(Method::GET, "/todos?status=Done", None).await;
    assert_eq!(code, StatusCode::BAD_REQUEST, "查询参数只认小写");

    // 库里存的是首字母大写（Display），换数据层时不能变
    let pool = backend::infra::db::must_init(&backend::infra::config::must_get().db);
    let client = pool.get().await.unwrap();
    let rows = client
        .query("SELECT status FROM todos WHERE namespace = $1 ORDER BY id", &[&api.ns()])
        .await
        .unwrap();
    let stored: Vec<String> = rows.iter().map(|r| r.get(0)).collect();
    assert_eq!(stored, ["Todo", "Done", "Archived", "Deleted"]);

    api.clear(by_status.iter().map(|(_, t)| t["id"].as_i64().unwrap()).collect()).await;
}

#[tokio::test]
async fn get_by_id_is_200_or_404_with_empty_body() {
    let api = Api::new();
    let t = api.create("find me").await;
    let id = t["id"].as_i64().unwrap();

    let (status, _, body) = api.send(Method::GET, &format!("/todos/{id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(serde_json::from_str::<Value>(&body).unwrap(), t);

    api.clear(vec![id]).await;
    let (status, _, body) = api.send(Method::GET, &format!("/todos/{id}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body, "");

    let (status, _, _) = api.send(Method::GET, "/todos/nan", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn patch_updates_fields_and_rejects_empty_body() {
    let api = Api::new();
    let t = api.create("draft").await;
    let id = t["id"].as_i64().unwrap();

    let (status, _, body) = api.send(Method::PATCH, &format!("/todos/{id}"), Some(json!({ "content": "final" }))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let u: Value = serde_json::from_str(&body).unwrap();
    assert_todo_shape(&u, api.ns());
    assert_eq!((u["id"].clone(), u["content"].clone(), u["status"].clone()), (json!(id), json!("final"), json!("todo")));
    assert_eq!(u["created_at"], t["created_at"], "改内容不动 created_at");
    assert!(u["updated_at"].as_str() >= t["updated_at"].as_str());

    let (status, _, body) = api.send(Method::PATCH, &format!("/todos/{id}"), Some(json!({ "status": "done" }))).await;
    assert_eq!(status, StatusCode::OK);
    let d: Value = serde_json::from_str(&body).unwrap();
    assert_eq!((d["content"].clone(), d["status"].clone()), (json!("final"), json!("done")), "只改状态不动内容");

    // 两个字段都没给：400，body 一字不改
    let (status, _, body) = api.send(Method::PATCH, &format!("/todos/{id}"), Some(json!({}))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body, r#"{"error":"expect one of `status` or `content`"}"#);

    api.clear(vec![id]).await;

    // 现状，P3.4 改成 404：不存在的 id 返回 500
    let (status, _, _) = api.send(Method::PATCH, &format!("/todos/{id}"), Some(json!({ "content": "x" }))).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR, "现状是 500；P3.4 改 404 时更新这里");
}

#[tokio::test]
async fn delete_clears_given_ids_only_and_returns_empty_200() {
    let api = Api::new();
    let keep = api.create("keep").await;
    let gone = api.create("gone").await;

    let (status, _, body) = api.send(Method::DELETE, "/todos", Some(json!({ "ids": [gone["id"]] }))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "");
    assert_eq!(ids(&api.list("").await), vec![keep["id"].as_i64().unwrap()]);

    // 别的租户删不到
    let other = Api { ns: Some(unique_ns()), ..Api::new() };
    other.clear(vec![keep["id"].as_i64().unwrap()]).await;
    assert_eq!(api.list("").await.len(), 1);

    let (status, _, _) = api.send(Method::DELETE, "/todos", Some(json!({ "ids": [] }))).await;
    assert_eq!(status, StatusCode::OK, "空 ids 也是 200");

    api.clear(vec![keep["id"].as_i64().unwrap()]).await;
}

#[tokio::test]
async fn missing_namespace_header_uses_default() {
    let api = Api::without_ns();
    let t = api.create("in default namespace").await;
    assert_todo_shape(&t, "default");
    api.clear(vec![t["id"].as_i64().unwrap()]).await;
}

#[tokio::test]
async fn cors_allows_any_origin_with_t_ns_header() {
    let api = Api::new();
    let origin = "https://todos.lexcao.io";

    for method in ["GET", "POST", "PATCH", "DELETE"] {
        let resp = api
            .http
            .request(Method::OPTIONS, format!("{}/todos", api.base))
            .header("origin", origin)
            .header("access-control-request-method", method)
            .header("access-control-request-headers", "t-ns,content-type")
            .send()
            .await
            .unwrap();
        assert!(resp.status().is_success(), "预检 {method}：{}", resp.status());
        let h = resp.headers();
        assert_eq!(h["access-control-allow-origin"], origin, "预检 {method}");
        let allow_headers = h["access-control-allow-headers"].to_str().unwrap().to_lowercase();
        assert!(allow_headers.contains("t-ns") && allow_headers.contains("content-type"), "{allow_headers}");
        let allow_methods = h["access-control-allow-methods"].to_str().unwrap();
        assert!(allow_methods.contains(method), "{allow_methods}");
    }

    let resp = api.http.get(format!("{}/todos", api.base)).header("origin", origin).header("t-ns", api.ns()).send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers()["access-control-allow-origin"], origin);
}
