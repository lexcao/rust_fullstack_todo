/// `GET /ping` → `pong`（text/plain）。只说明进程还在，不查库。
pub async fn ping() -> &'static str {
    "pong"
}
