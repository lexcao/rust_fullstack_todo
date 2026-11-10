//! `/healthz` 打真库：通的时候 200，库不在的时候 503。`/ping` 两种情况下都是 `pong`。

use std::str::FromStr;
use std::time::Duration;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

use crate::helper::{spawn_server, spawn_server_with};

mod helper;

#[tokio::test]
async fn healthz_is_200_when_the_database_is_reachable() {
    let base = spawn_server();
    let http = client();
    let health = http.get(format!("{base}/healthz")).send().await.unwrap();
    assert_eq!(health.status(), reqwest::StatusCode::OK);

    let ping = http.get(format!("{base}/ping")).send().await.unwrap();
    assert_eq!(ping.status(), reqwest::StatusCode::OK);
    assert_eq!(ping.text().await.unwrap(), "pong");
}

#[tokio::test]
async fn healthz_is_503_when_the_database_is_down() {
    // 本机 1 端口没有 Postgres。连接立刻失败，不把共享测试库停掉。
    let opts = PgConnectOptions::from_str("postgres://user:password@127.0.0.1:1/none").unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy_with(opts);
    let base = spawn_server_with(pool);
    let http = client();

    let health = http.get(format!("{base}/healthz")).send().await.unwrap();
    assert_eq!(health.status(), reqwest::StatusCode::SERVICE_UNAVAILABLE);
    let body = health.text().await.unwrap();
    assert_eq!(body, r#"{"error":"database unavailable"}"#);
    assert!(!body.contains("password"));

    let ping = http.get(format!("{base}/ping")).send().await.unwrap();
    assert_eq!(ping.status(), reqwest::StatusCode::OK);
    assert_eq!(ping.text().await.unwrap(), "pong");
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap()
}
