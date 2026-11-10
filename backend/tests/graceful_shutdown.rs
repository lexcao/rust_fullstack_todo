//! 给真进程发 SIGTERM：进行中的请求要跑完，进程在 30 秒内以 0 退出。
//!
//! 子进程调用 `backend::serve(..., shutdown_signal())`，这就是 `main` → `start_server` 的停机路径。
//! `/slow` 只存在于这个测试里，用来让请求在 SIGTERM 到达时还没结束。生产路由没有它。

use std::fs;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use axum::Router;
use axum::routing::get;

#[tokio::test]
async fn sigterm_lets_the_in_flight_request_finish_and_the_process_exits_0() {
    if std::env::var_os("BACKEND_SHUTDOWN_CHILD").is_some() {
        shutdown_child().await;
        return;
    }

    let dir = std::env::temp_dir().join(format!(
        "p34-shutdown-{}-{}",
        std::process::id(),
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let port_file = dir.join("port");
    let flag_file = dir.join("inflight");
    let stdout = fs::File::create(dir.join("stdout")).unwrap();
    let stderr = fs::File::create(dir.join("stderr")).unwrap();

    let child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "sigterm_lets_the_in_flight_request_finish_and_the_process_exits_0",
            "--test-threads=1",
        ])
        .env("BACKEND_SHUTDOWN_CHILD", "1")
        .env("PORT_FILE", &port_file)
        .env("SLOW_FLAG", &flag_file)
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .unwrap();
    let mut child = KillOnDrop(Some(child));

    let port = wait_for_file(&port_file, Duration::from_secs(10))
        .await
        .trim()
        .to_owned();
    let url = format!("http://127.0.0.1:{port}/slow");
    let request = tokio::spawn(async move {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .unwrap();
        let mut last = String::new();
        for _ in 0..100 {
            match http.get(&url).send().await {
                Ok(response) => {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    return (status, body);
                }
                Err(err) => last = err.to_string(),
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        panic!("in-flight request never connected: {last}");
    });

    wait_for_file(&flag_file, Duration::from_secs(10)).await;
    let sigterm_at = Instant::now();
    let killed = Command::new("kill")
        .args(["-s", "TERM", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(killed.success(), "kill -TERM failed");

    let (status, body) = request.await.unwrap();
    assert_eq!(status, reqwest::StatusCode::OK, "{body}");
    assert_eq!(body, "done");

    let exit = child.wait_timeout(Duration::from_secs(30));
    let logs = format!(
        "stdout:\n{}\nstderr:\n{}",
        fs::read_to_string(dir.join("stdout")).unwrap_or_default(),
        fs::read_to_string(dir.join("stderr")).unwrap_or_default()
    );
    assert!(
        exit.success(),
        "process exited {exit} after {:?}\n{logs}",
        sigterm_at.elapsed()
    );
    assert!(
        sigterm_at.elapsed() < Duration::from_secs(30),
        "exit took {:?}\n{logs}",
        sigterm_at.elapsed()
    );
    let _ = fs::remove_dir_all(&dir);
}

async fn shutdown_child() {
    let port_file = std::env::var("PORT_FILE").unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    fs::write(&port_file, port.to_string()).unwrap();
    let app = Router::new().route("/slow", get(slow));
    backend::serve(listener, app, backend::shutdown_signal())
        .await
        .unwrap();
}

async fn slow() -> &'static str {
    if let Ok(path) = std::env::var("SLOW_FLAG") {
        let _ = fs::write(path, "1");
    }
    tokio::time::sleep(Duration::from_secs(1)).await;
    "done"
}

async fn wait_for_file(path: &Path, timeout: Duration) -> String {
    let start = Instant::now();
    loop {
        if let Ok(text) = fs::read_to_string(path)
            && !text.is_empty()
        {
            return text;
        }
        if start.elapsed() > timeout {
            panic!("timed out waiting for {}", path.display());
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

struct KillOnDrop(Option<Child>);

impl KillOnDrop {
    fn id(&self) -> u32 {
        self.0.as_ref().unwrap().id()
    }

    fn wait_timeout(&mut self, limit: Duration) -> std::process::ExitStatus {
        let start = Instant::now();
        loop {
            let child = self.0.as_mut().unwrap();
            if let Some(status) = child.try_wait().unwrap() {
                self.0.take();
                return status;
            }
            if start.elapsed() > limit {
                let _ = child.kill();
                let status = child.wait().unwrap();
                self.0.take();
                panic!("process did not exit within {limit:?}: {status}");
            }
            std::thread::sleep(Duration::from_millis(30));
        }
    }
}

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        if let Some(child) = self.0.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
