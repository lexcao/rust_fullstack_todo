//! 日志里有请求 id，没有数据库密码。
//!
//! 单独一个测试进程：tracing 会缓存 callsite 的开关，必须在第一次请求之前装上 subscriber。

use std::io::Write;
use std::sync::{Arc, Mutex};

use crate::helper::spawn_server;

mod helper;

#[tokio::test]
async fn logs_include_the_request_id_and_not_the_database_secret() {
    let secret = "db-secret-do-not-log";
    let buf = Arc::new(Mutex::new(Vec::<u8>::new()));
    let writer = buf.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_ansi(false)
        .with_writer(move || LogBuf(writer.clone()))
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("subscriber");

    let cfg = backend::infra::config::load(|key| {
        (key == "DATABASE_URL").then(|| format!("postgres://app:{secret}@db.internal:5432/app"))
    })
    .expect("config");
    backend::infra::config::log_loaded(&cfg);

    let base = spawn_server();
    let http = reqwest::Client::new();
    let request_id = "req-trace-7c2a";
    let ping = http
        .get(format!("{base}/ping"))
        .header("x-request-id", request_id)
        .send()
        .await
        .unwrap();
    assert_eq!(ping.status(), reqwest::StatusCode::OK);
    assert_eq!(ping.headers()["x-request-id"], request_id);
    assert_eq!(ping.text().await.unwrap(), "pong");

    let generated = http.get(format!("{base}/ping")).send().await.unwrap();
    let generated_id = generated.headers()["x-request-id"]
        .to_str()
        .unwrap()
        .to_owned();
    assert!(!generated_id.is_empty());
    assert_ne!(generated_id, request_id);

    let logs = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(logs.contains(request_id), "{logs}");
    assert!(logs.contains(&generated_id), "{logs}");
    assert!(!logs.contains(secret), "{logs}");
    assert!(!logs.contains("postgres://"), "{logs}");
}

#[derive(Clone)]
struct LogBuf(Arc<Mutex<Vec<u8>>>);

impl Write for LogBuf {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.0.lock().expect("log buffer").extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
