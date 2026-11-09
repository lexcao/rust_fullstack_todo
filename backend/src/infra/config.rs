use std::fmt;
use std::net::TcpListener;

use config::{ConfigError, Environment, File, FileFormat};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Server {
    pub port: u16,
}

/// 数据库连接配置：`application.toml` 的 `[db]`，可被 `APP_DB_*` 环境变量覆盖。
///
/// - 设置了 `url`（`APP_DB_URL`）时整串交给 sqlx 解析，其它字段不用。连 Neon 用
///   `postgresql://…/db?sslmode=verify-full`（契约第 0 节：verify-full，不开通道绑定）。
/// - 否则用 host/port/user/password/dbname 拼，`sslmode` 缺省 `prefer`（本地无 TLS 的库也能连）。
#[derive(Deserialize, Clone, Default)]
pub struct DbConfig {
    pub url: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub dbname: Option<String>,
    pub sslmode: Option<String>,
}

/// 不打印密码和整串 URL（URL 里可能带密码）。
impl fmt::Debug for DbConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redact = |v: &Option<String>| v.as_ref().map(|_| "<redacted>");
        f.debug_struct("DbConfig")
            .field("url", &redact(&self.url))
            .field("host", &self.host)
            .field("port", &self.port)
            .field("user", &self.user)
            .field("password", &redact(&self.password))
            .field("dbname", &self.dbname)
            .field("sslmode", &self.sslmode)
            .finish()
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub server: Server,
    pub db: DbConfig,
}

impl Config {
    pub fn address(&self) -> TcpListener {
        TcpListener::bind(format!("0.0.0.0:{}", self.server.port))
            .expect("Failed to bind port")
    }
}

pub fn get() -> Result<Config, ConfigError> {
    config::Config::builder()
        .add_source(File::new("application", FileFormat::Toml))
        .add_source(Environment::with_prefix("APP").separator("_"))
        .build()?
        .try_deserialize()
}

pub fn must_get() -> Config {
    get().expect("Failed to get config")
}

#[cfg(test)]
mod tests {
    use super::DbConfig;

    #[test]
    fn debug_output_redacts_secrets() {
        let cfg = DbConfig {
            url: Some("postgresql://u:hunter2@h/db".into()),
            password: Some("hunter2".into()),
            user: Some("u".into()),
            ..Default::default()
        };
        let printed = format!("{cfg:?}");
        assert!(!printed.contains("hunter2"), "{printed}");
        assert!(printed.contains("<redacted>") && printed.contains("\"u\""), "{printed}");
    }
}
