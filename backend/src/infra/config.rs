//! 配置。文件只提供本地默认值；环境变量覆盖它。
//!
//! 优先级（高的赢）：
//! 1. `DATABASE_URL` 替换整个 `[db]`（`APP_DB_*` 和 toml 里的 host/password 都不再使用）
//! 2. `PORT` 替换端口（高于 `APP_SERVER_PORT`）
//! 3. 旧变量 `APP_SERVER_PORT`、`APP_DB_URL`、`APP_DB_HOST`、`APP_DB_PORT`、`APP_DB_USER`、
//!    `APP_DB_PASSWORD`、`APP_DB_DBNAME`、`APP_DB_SSLMODE`
//! 4. `application.toml`
//!
//! 读环境的入口是 [`load`]，参数是一个函数，测试不用 `std::env::set_var`（edition 2024 里那是 unsafe）。

use std::fmt;
use std::net::TcpListener;

use config::{ConfigError, File, FileFormat};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Server {
    pub port: u16,
}

/// 数据库连接。`url` 和 `password` 是 [`SecretString`]，Debug 不会打出明文。
///
/// - 设置了 `url`（`DATABASE_URL` 或 `APP_DB_URL`）时整串交给 sqlx，其它字段不用。
///   连 Neon 用 `postgresql://…/db?sslmode=verify-full`（不开通道绑定）。
/// - 否则用 host/port/user/password/dbname 拼，`sslmode` 缺省 `prefer`（本地无 TLS 的库也能连）。
#[derive(Clone, Default)]
pub struct DbConfig {
    pub url: Option<SecretString>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub password: Option<SecretString>,
    pub dbname: Option<String>,
    pub sslmode: Option<String>,
}

/// 不打印密码和整串 URL（URL 里可能带密码）。
impl fmt::Debug for DbConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redact = |v: &Option<SecretString>| v.as_ref().map(|_| "<redacted>");
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

#[derive(Debug)]
pub struct Config {
    pub server: Server,
    pub db: DbConfig,
}

impl Config {
    pub fn address(&self) -> TcpListener {
        TcpListener::bind(format!("0.0.0.0:{}", self.server.port)).expect("Failed to bind port")
    }
}

#[derive(Deserialize)]
struct RawConfig {
    server: Server,
    db: RawDb,
}

#[derive(Deserialize, Default)]
struct RawDb {
    url: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    user: Option<String>,
    password: Option<String>,
    dbname: Option<String>,
    sslmode: Option<String>,
}

impl RawDb {
    fn into_db(self) -> DbConfig {
        DbConfig {
            url: self.url.map(SecretString::from),
            host: self.host,
            port: self.port,
            user: self.user,
            password: self.password.map(SecretString::from),
            dbname: self.dbname,
            sslmode: self.sslmode,
        }
    }
}

/// 用进程环境加载配置。启动路径走这里。
pub fn get() -> Result<Config, ConfigError> {
    load(|key| std::env::var(key).ok())
}

/// `env` 是唯一的环境变量来源。空字符串当作没设置。
pub fn load(mut env: impl FnMut(&str) -> Option<String>) -> Result<Config, ConfigError> {
    let raw: RawConfig = config::Config::builder()
        .add_source(File::new("application", FileFormat::Toml))
        .build()?
        .try_deserialize()?;

    let mut cfg = Config {
        server: raw.server,
        db: raw.db.into_db(),
    };

    if let Some(value) = nonempty(env("APP_SERVER_PORT")) {
        cfg.server.port = parse_port(&value, "APP_SERVER_PORT")?;
    }
    apply_app_db(&mut cfg.db, &mut env)?;

    // 平台变量最后盖上去：PORT 高于 APP_SERVER_PORT，DATABASE_URL 丢掉文件和 APP_DB_* 的字段。
    if let Some(value) = nonempty(env("PORT")) {
        cfg.server.port = parse_port(&value, "PORT")?;
    }
    if let Some(url) = nonempty(env("DATABASE_URL")) {
        cfg.db = DbConfig {
            url: Some(SecretString::from(url)),
            ..DbConfig::default()
        };
    }

    Ok(cfg)
}

fn apply_app_db(
    db: &mut DbConfig,
    env: &mut impl FnMut(&str) -> Option<String>,
) -> Result<(), ConfigError> {
    if let Some(value) = nonempty(env("APP_DB_URL")) {
        db.url = Some(SecretString::from(value));
    }
    if let Some(value) = nonempty(env("APP_DB_HOST")) {
        db.host = Some(value);
    }
    if let Some(value) = nonempty(env("APP_DB_PORT")) {
        db.port = Some(parse_port(&value, "APP_DB_PORT")?);
    }
    if let Some(value) = nonempty(env("APP_DB_USER")) {
        db.user = Some(value);
    }
    if let Some(value) = nonempty(env("APP_DB_PASSWORD")) {
        db.password = Some(SecretString::from(value));
    }
    if let Some(value) = nonempty(env("APP_DB_DBNAME")) {
        db.dbname = Some(value);
    }
    if let Some(value) = nonempty(env("APP_DB_SSLMODE")) {
        db.sslmode = Some(value);
    }
    Ok(())
}

fn nonempty(value: Option<String>) -> Option<String> {
    value.filter(|text| !text.is_empty())
}

fn parse_port(value: &str, name: &str) -> Result<u16, ConfigError> {
    value
        .parse()
        .map_err(|_| ConfigError::Message(format!("invalid {name} `{value}`")))
}

pub fn must_get() -> Config {
    get().expect("Failed to get config")
}

/// 启动时打一行配置。URL 和密码走 [`DbConfig`] 的 Debug，明文不会进日志。
pub fn log_loaded(config: &Config) {
    tracing::info!(?config, "configuration loaded");
}

/// 给连接池用的密码明文。只在建连接时调用，不要写进日志。
pub(crate) fn expose(secret: &SecretString) -> &str {
    secret.expose_secret()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::db;

    fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl FnMut(&str) -> Option<String> + 'a {
        move |key| {
            pairs
                .iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| (*value).to_owned())
        }
    }

    #[test]
    fn debug_output_redacts_secrets() {
        let cfg = DbConfig {
            url: Some(SecretString::from("postgresql://u:hunter2@h/db")),
            password: Some(SecretString::from("hunter2")),
            user: Some("u".into()),
            ..Default::default()
        };
        let printed = format!("{cfg:?}");
        assert!(!printed.contains("hunter2"), "{printed}");
        assert!(
            printed.contains("<redacted>") && printed.contains("\"u\""),
            "{printed}"
        );
        let secret_debug = format!("{:?}", SecretString::from("hunter2"));
        assert!(!secret_debug.contains("hunter2"), "{secret_debug}");
    }

    #[test]
    fn file_defaults_when_env_is_empty() {
        let cfg = load(|_| None).expect("application.toml");
        assert_eq!(cfg.server.port, 3000);
        assert!(cfg.db.url.is_none());
        assert_eq!(cfg.db.host.as_deref(), Some("localhost"));
        assert_eq!(cfg.db.user.as_deref(), Some("user"));
        assert_eq!(cfg.db.dbname.as_deref(), Some("rust_fullstack_todo"));
        let printed = format!("{cfg:?}");
        assert!(!printed.contains("Some(\"password\")"), "{printed}");
    }

    #[test]
    fn app_vars_override_the_file_and_port_and_database_url_win() {
        let app_only = load(env(&[
            ("APP_SERVER_PORT", "4000"),
            ("APP_DB_HOST", "db.internal"),
            ("APP_DB_PORT", "5434"),
            ("APP_DB_USER", "appuser"),
            ("APP_DB_PASSWORD", "app-secret"),
            ("APP_DB_DBNAME", "app"),
            ("APP_DB_SSLMODE", "require"),
        ]))
        .unwrap();
        assert_eq!(app_only.server.port, 4000);
        assert_eq!(app_only.db.host.as_deref(), Some("db.internal"));
        assert_eq!(app_only.db.port, Some(5434));
        assert_eq!(app_only.db.sslmode.as_deref(), Some("require"));
        assert!(!format!("{app_only:?}").contains("app-secret"));
        let opts = db::connect_options(&app_only.db).unwrap();
        assert_eq!(opts.get_host(), "db.internal");
        assert!(matches!(
            opts.get_ssl_mode(),
            sqlx::postgres::PgSslMode::Require
        ));

        let url_over_fields = load(env(&[
            ("APP_DB_HOST", "nope.example"),
            ("APP_DB_URL", "postgres://from-app@app.example:5440/appdb"),
            ("APP_SERVER_PORT", "4000"),
            ("PORT", "5000"),
        ]))
        .unwrap();
        assert_eq!(
            url_over_fields.server.port, 5000,
            "PORT 高于 APP_SERVER_PORT"
        );
        let opts = db::connect_options(&url_over_fields.db).unwrap();
        assert_eq!(opts.get_host(), "app.example");
        assert_eq!(opts.get_port(), 5440);

        let top = load(env(&[
            ("APP_SERVER_PORT", "4000"),
            ("PORT", "8080"),
            ("APP_DB_HOST", "nope.example"),
            ("APP_DB_URL", "postgres://other@nope.example/other"),
            ("APP_DB_PASSWORD", "app-secret"),
            (
                "DATABASE_URL",
                "postgres://app:db-secret-do-not-log@db.internal:5433/appdb",
            ),
        ]))
        .unwrap();
        assert_eq!(top.server.port, 8080);
        assert!(
            top.db.host.is_none(),
            "DATABASE_URL 丢掉文件和 APP_DB_* 字段"
        );
        assert!(top.db.password.is_none());
        let opts = db::connect_options(&top.db).unwrap();
        assert_eq!(opts.get_host(), "db.internal");
        assert_eq!(opts.get_port(), 5433);
        assert_eq!(opts.get_username(), "app");
        assert_eq!(opts.get_database(), Some("appdb"));
        let printed = format!("{top:?}");
        assert!(!printed.contains("db-secret-do-not-log"), "{printed}");
        assert!(!printed.contains("app-secret"), "{printed}");
    }

    #[test]
    fn invalid_port_is_an_error() {
        let err = load(env(&[("PORT", "nope")])).unwrap_err();
        assert!(err.to_string().contains("PORT"), "{err}");
    }
}
