//! 蟹记 Crabnote 前端的浏览器端到端测试工具：WebDriver（fantoccini）+ headless Chrome。
//!
//! 环境变量：
//! - `E2E_BASE_URL`：被测站点，例如 `https://todos.lexcao.io/?v=1`。不设置时，自带的静态服务器
//!   会托管 `E2E_DIST`（默认 `frontend/dist`），测本地构建产物。
//! - `E2E_CLOUD`：设为 `1` 才真正跑云模式用例。没设置时该用例直接返回，避免对生产构建里的远端地址写数据。
//! - `WEBDRIVER_URL`：已运行的 chromedriver 地址。不设置时，自动启动 `CHROMEDRIVER`（默认 PATH 里的
//!   `chromedriver`）。
//! - `CHROME_BIN`：Chrome 可执行文件路径（可选）。

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use fantoccini::wd::{Capabilities, WebDriverCompatibleCommand};
use fantoccini::{Client, ClientBuilder, Locator};
use serde_json::{Value, json};

pub use fantoccini;

/// 本地 localStorage 里保存任务的键（与 frontend/src/states.rs 一致）。
pub const TODOS_KEY: &str = "rust_fullstack_todo.todos";

/// 页面打开后，前端探测后端的超时时间是 6 秒；等离线提示时留一点余量。
pub const OFFLINE_WAIT: Duration = Duration::from_secs(9);

#[derive(Clone, Copy, Debug)]
pub struct Viewport {
    pub name: &'static str,
    pub width: u32,
    pub height: u32,
    pub mobile: bool,
    pub dark: bool,
}

impl Viewport {
    pub const DESKTOP: Viewport = Viewport {
        name: "desktop",
        width: 1440,
        height: 900,
        mobile: false,
        dark: false,
    };
    pub const MOBILE: Viewport = Viewport {
        name: "mobile",
        width: 390,
        height: 844,
        mobile: true,
        dark: false,
    };
    pub const MOBILE_SMALL: Viewport = Viewport {
        name: "mobile-360",
        width: 360,
        height: 800,
        mobile: true,
        dark: false,
    };

    pub const fn dark(self, name: &'static str) -> Viewport {
        Viewport {
            name,
            dark: true,
            ..self
        }
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

// ---------- 静态服务器 ----------

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "wasm" => "application/wasm",
        "css" => "text/css; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "woff2" => "font/woff2",
        "xml" => "application/xml",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn serve_dist(dir: PathBuf) -> String {
    let server = tiny_http::Server::http("127.0.0.1:0").expect("启动静态服务器失败");
    let addr = server.server_addr().to_ip().expect("服务器地址");
    std::thread::spawn(move || {
        for request in server.incoming_requests() {
            let url = request
                .url()
                .split(['?', '#'])
                .next()
                .unwrap_or("/")
                .to_string();
            let rel = url.trim_start_matches('/');
            let rel = if rel.is_empty() { "index.html" } else { rel };
            let path = dir.join(rel);
            let safe = !rel.split('/').any(|seg| seg == "..");
            let response = match std::fs::read(&path) {
                Ok(body) if safe => {
                    let header =
                        tiny_http::Header::from_bytes("Content-Type", content_type(&path)).unwrap();
                    tiny_http::Response::from_data(body)
                        .with_header(header)
                        .boxed()
                }
                _ => tiny_http::Response::from_string("not found")
                    .with_status_code(404)
                    .boxed(),
            };
            let _ = request.respond(response);
        }
    });
    format!("http://{addr}/")
}

/// 被测站点地址。
pub fn base_url() -> &'static str {
    static URL: OnceLock<String> = OnceLock::new();
    URL.get_or_init(|| match std::env::var("E2E_BASE_URL") {
        Ok(url) if !url.is_empty() => url,
        _ => {
            let dist = std::env::var("E2E_DIST")
                .map(PathBuf::from)
                .unwrap_or_else(|_| workspace_root().join("frontend/dist"));
            assert!(
                dist.join("index.html").exists(),
                "找不到 {}/index.html，先运行 trunk build --release",
                dist.display()
            );
            serve_dist(dist)
        }
    })
}

// ---------- chromedriver ----------

struct Driver {
    url: String,
    _child: Option<Child>,
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn driver() -> &'static Driver {
    static DRIVER: OnceLock<Driver> = OnceLock::new();
    DRIVER.get_or_init(|| {
        if let Ok(url) = std::env::var("WEBDRIVER_URL") {
            return Driver { url, _child: None };
        }
        let bin = std::env::var("CHROMEDRIVER").unwrap_or_else(|_| "chromedriver".into());
        let port = free_port();
        let child = Command::new(&bin)
            .arg(format!("--port={port}"))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap_or_else(|e| {
                panic!("无法启动 {bin}: {e}（可用 CHROMEDRIVER 或 WEBDRIVER_URL 指定）")
            });
        let deadline = Instant::now() + Duration::from_secs(10);
        while std::net::TcpStream::connect(("127.0.0.1", port)).is_err() {
            assert!(Instant::now() < deadline, "chromedriver 10 秒内没有启动");
            std::thread::sleep(Duration::from_millis(100));
        }
        Driver {
            url: format!("http://127.0.0.1:{port}"),
            _child: Some(child),
        }
    })
}

// ---------- Chrome DevTools 命令（chromedriver 的 goog/cdp/execute 扩展） ----------

#[derive(Debug)]
struct Cdp {
    method: &'static str,
    params: Value,
}

impl WebDriverCompatibleCommand for Cdp {
    fn endpoint(
        &self,
        base_url: &url::Url,
        session_id: Option<&str>,
    ) -> Result<url::Url, url::ParseError> {
        base_url.join(&format!(
            "session/{}/goog/cdp/execute",
            session_id.unwrap_or_default()
        ))
    }

    fn method_and_body(&self, _: &url::Url) -> (http::Method, Option<String>) {
        (
            http::Method::POST,
            Some(json!({ "cmd": self.method, "params": self.params }).to_string()),
        )
    }
}

pub async fn cdp(
    client: &Client,
    method: &'static str,
    params: Value,
) -> Result<Value, fantoccini::error::CmdError> {
    client.issue_cmd(Cdp { method, params }).await
}

/// 新开一个独立的浏览器会话（全新 profile，localStorage 为空），并按视口模拟设备。
pub async fn session(viewport: Viewport) -> Client {
    let mut args = vec![
        "--headless=new".to_string(),
        "--no-sandbox".into(),
        "--disable-gpu".into(),
        "--disable-dev-shm-usage".into(),
        "--hide-scrollbars".into(),
        format!("--window-size={},{}", viewport.width, viewport.height),
    ];
    if let Ok(extra) = std::env::var("CHROME_ARGS") {
        args.extend(extra.split_whitespace().map(String::from));
    }
    let mut chrome = json!({ "args": args });
    if let Ok(bin) = std::env::var("CHROME_BIN") {
        chrome["binary"] = json!(bin);
    }
    let mut caps = Capabilities::new();
    caps.insert("goog:chromeOptions".into(), chrome);
    caps.insert("goog:loggingPrefs".into(), json!({ "browser": "ALL" }));

    let client = ClientBuilder::rustls()
        .expect("rustls")
        .capabilities(caps)
        .connect(&driver().url)
        .await
        .expect("连接 chromedriver 失败");

    cdp(
        &client,
        "Emulation.setDeviceMetricsOverride",
        json!({
            "width": viewport.width,
            "height": viewport.height,
            "deviceScaleFactor": if viewport.mobile { 2 } else { 1 },
            "mobile": viewport.mobile,
        }),
    )
    .await
    .expect("设置视口");
    cdp(&client, "Emulation.setEmulatedMedia", json!({
        "features": [{ "name": "prefers-color-scheme", "value": if viewport.dark { "dark" } else { "light" } }]
    }))
    .await
    .expect("设置配色");
    if viewport.mobile {
        cdp(
            &client,
            "Emulation.setTouchEmulationEnabled",
            json!({ "enabled": true, "maxTouchPoints": 5 }),
        )
        .await
        .expect("触控模拟");
    }
    client
}

/// 打开首页并等 Yew 应用挂载完成。
pub async fn open_app(client: &Client) {
    client.goto(base_url()).await.expect("打开首页");
    wait_for(client, "#new-todo").await;
}

pub async fn wait_for(client: &Client, css: &str) -> fantoccini::elements::Element {
    client
        .wait()
        .at_most(Duration::from_secs(20))
        .for_element(Locator::Css(css))
        .await
        .unwrap_or_else(|e| panic!("等待 {css} 超时: {e}"))
}

/// 在输入框里输入并回车。
pub async fn add_todo(client: &Client, text: &str) {
    let input = client.find(Locator::Css("#new-todo")).await.unwrap();
    input
        .send_keys(&format!(
            "{text}{}",
            char::from(fantoccini::key::Key::Enter)
        ))
        .await
        .unwrap();
}

pub async fn count(client: &Client, css: &str) -> usize {
    client
        .find_all(Locator::Css(css))
        .await
        .map(|v| v.len())
        .unwrap_or(0)
}

/// 页面是否出现横向滚动。
pub async fn has_horizontal_overflow(client: &Client) -> bool {
    let html = client.find(Locator::Css("html")).await.unwrap();
    num_prop(client, &html, "scrollWidth").await
        > num_prop(client, &html, "clientWidth").await + 0.5
}

/// 浏览器控制台里的 SEVERE 级别日志（页面异常会出现在这里）。
pub async fn severe_logs(client: &Client) -> Vec<String> {
    let log = client.issue_cmd(BrowserLog).await.unwrap_or(Value::Null);
    log.as_array()
        .into_iter()
        .flatten()
        .filter(|entry| entry["level"] == "SEVERE")
        .filter_map(|entry| entry["message"].as_str().map(String::from))
        .collect()
}

#[derive(Debug)]
struct BrowserLog;

impl WebDriverCompatibleCommand for BrowserLog {
    fn endpoint(
        &self,
        base_url: &url::Url,
        session_id: Option<&str>,
    ) -> Result<url::Url, url::ParseError> {
        base_url.join(&format!(
            "session/{}/se/log",
            session_id.unwrap_or_default()
        ))
    }

    fn method_and_body(&self, _: &url::Url) -> (http::Method, Option<String>) {
        (
            http::Method::POST,
            Some(json!({ "type": "browser" }).to_string()),
        )
    }
}

/// 后端离线导致的请求失败是预期内的（ping 只发一次），其余 SEVERE 日志算真错误。
pub fn unexpected_errors(logs: &[String]) -> Vec<String> {
    logs.iter()
        .filter(|m| !(m.contains("/ping") || m.contains("net::ERR_FAILED") || m.contains("CORS")))
        .cloned()
        .collect()
}

/// 整页截图：把视口高度临时拉到文档高度。
pub async fn full_page_png(client: &Client, viewport: Viewport) -> Vec<u8> {
    expand_to_page(client, viewport).await;
    let png = client.screenshot().await.unwrap();
    set_metrics(client, viewport, viewport.height).await;
    png
}

/// 把视口拉高到整页高度（截整页或截超出视口的元素前用），之后用 `set_metrics` 恢复。
pub async fn expand_to_page(client: &Client, viewport: Viewport) {
    let body = client.find(Locator::Css("html")).await.unwrap();
    let height = num_prop(client, &body, "scrollHeight").await as u32;
    set_metrics(client, viewport, height).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
}

pub async fn set_metrics(client: &Client, viewport: Viewport, height: u32) {
    cdp(
        client,
        "Emulation.setDeviceMetricsOverride",
        json!({
            "width": viewport.width, "height": height,
            "deviceScaleFactor": if viewport.mobile { 2 } else { 1 }, "mobile": viewport.mobile,
        }),
    )
    .await
    .unwrap();
}

/// 读元素的数值型 DOM 属性（如 scrollWidth）。WebDriver 的 Get Element Property 对数字返回非字符串，
/// fantoccini 的 `prop` 不接受，所以这里把元素作为参数传给一行读取脚本。
pub async fn num_prop(client: &Client, el: &fantoccini::elements::Element, name: &str) -> f64 {
    let el = serde_json::to_value(el).expect("element json");
    client
        .execute(
            "return Number(arguments[0][arguments[1]])",
            vec![el, json!(name)],
        )
        .await
        .expect("读属性")
        .as_f64()
        .unwrap_or(f64::NAN)
}
