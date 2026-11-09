//! 数据库：sqlx 0.9 连接池（rustls）+ 内嵌迁移。

use std::str::FromStr;

use anyhow::Context;
use sqlx::postgres::{PgArguments, PgConnectOptions, PgPoolOptions, PgRow, PgSslMode};
use sqlx::query::Query;
use sqlx::{PgPool, Postgres};

use super::config::DbConfig;

/// 内嵌的迁移（`backend/migrations`）。`0001_init.sql` 与 `schemas/00_init_tables.sql` 逐字节相同。
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

pub fn connect_options(config: &DbConfig) -> anyhow::Result<PgConnectOptions> {
    if let Some(url) = &config.url {
        return PgConnectOptions::from_str(url).context("invalid db.url");
    }
    let mut opts = PgConnectOptions::new_without_pgpass();
    if let Some(v) = &config.host {
        opts = opts.host(v);
    }
    if let Some(v) = config.port {
        opts = opts.port(v);
    }
    if let Some(v) = &config.user {
        opts = opts.username(v);
    }
    if let Some(v) = &config.password {
        opts = opts.password(v);
    }
    if let Some(v) = &config.dbname {
        opts = opts.database(v);
    }
    let sslmode = config.sslmode.as_deref().unwrap_or("prefer");
    let sslmode = PgSslMode::from_str(sslmode).with_context(|| format!("invalid db.sslmode `{sslmode}`"))?;
    Ok(opts.ssl_mode(sslmode))
}

/// 建连接池（惰性，不立即连库）。配置非法时 panic：只在启动时调用。
pub fn must_init(config: &DbConfig) -> Pool {
    let opts = connect_options(config).expect("Invalid database configuration");
    Pool(PgPoolOptions::new().connect_lazy_with(opts))
}

/// 启动时跑迁移；并发启动时 sqlx 用 advisory lock 串行化，已执行的迁移会校验 checksum。
pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    MIGRATOR.run(pool).await.context("database migration failed")
}

/// sqlx 连接池的薄包装。
///
/// 除了交给应用用（`into()` 成 [`PgPool`]），还提供 `get()` → [`Client::query`] 这组临时 SQL 接口：
/// P3.1 的 API 契约测试用它直接查库断言存储格式，契约测试不允许修改，所以保留这个形状。
#[derive(Clone)]
pub struct Pool(PgPool);

impl Pool {
    pub async fn get(&self) -> Result<Client, sqlx::Error> {
        Ok(Client(self.0.clone()))
    }
}

impl From<Pool> for PgPool {
    fn from(p: Pool) -> Self {
        p.0
    }
}

impl From<PgPool> for Pool {
    fn from(p: PgPool) -> Self {
        Self(p)
    }
}

pub struct Client(PgPool);

impl Client {
    /// 执行一条带位置参数（`$1`…）的查询，返回所有行。
    ///
    /// `sql` 只能是调用方写死的语句，值一律走 `params` 绑定，不要把外部输入拼进 `sql`。
    pub async fn query(&self, sql: &str, params: &[&(dyn Param + Sync)]) -> Result<Vec<Row>, sqlx::Error> {
        let mut q = sqlx::query(sqlx::AssertSqlSafe(sql.to_owned()));
        for p in params {
            q = p.bind_to(q);
        }
        Ok(q.fetch_all(&self.0).await?.into_iter().map(Row).collect())
    }
}

pub struct Row(PgRow);

impl Row {
    /// 按列序号取值；类型不匹配时 panic（只给测试和运维脚本用）。
    pub fn get<T>(&self, index: usize) -> T
    where
        T: for<'r> sqlx::Decode<'r, Postgres> + sqlx::Type<Postgres>,
    {
        sqlx::Row::get(&self.0, index)
    }
}

/// 能绑定到 [`Client::query`] 的参数类型。
pub trait Param {
    fn bind_to<'q>(&'q self, q: Query<'q, Postgres, PgArguments>) -> Query<'q, Postgres, PgArguments>;
}

macro_rules! param {
    ($($t:ty),*) => {$(
        impl Param for $t {
            fn bind_to<'q>(&'q self, q: Query<'q, Postgres, PgArguments>) -> Query<'q, Postgres, PgArguments> {
                q.bind(self)
            }
        }
    )*};
}
param!(&str, String, i32, i64, bool);

#[derive(thiserror::Error, Debug)]
#[error("record not found")]
pub struct RecordNotFound;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::config;

    #[test]
    fn migration_0001_is_the_existing_schema_byte_for_byte() {
        assert_eq!(
            include_str!("../../migrations/0001_init.sql"),
            include_str!("../../schemas/00_init_tables.sql"),
        );
        let first = MIGRATOR.iter().next().expect("至少一个迁移");
        assert_eq!((first.version, first.description.as_ref()), (1, "init"));
    }

    #[test]
    fn sslmode_defaults_to_prefer_and_accepts_verify_full() {
        let cfg = DbConfig { host: Some("db.example".into()), ..Default::default() };
        assert!(matches!(connect_options(&cfg).unwrap().get_ssl_mode(), PgSslMode::Prefer));
        let cfg = DbConfig { sslmode: Some("verify-full".into()), ..cfg };
        assert!(matches!(connect_options(&cfg).unwrap().get_ssl_mode(), PgSslMode::VerifyFull));
        let cfg = DbConfig { url: Some("postgresql://u@neon.example/db?sslmode=verify-full".into()), ..Default::default() };
        let opts = connect_options(&cfg).unwrap();
        assert!(matches!(opts.get_ssl_mode(), PgSslMode::VerifyFull));
        assert_eq!(opts.get_host(), "neon.example");
        assert!(connect_options(&DbConfig { sslmode: Some("bogus".into()), ..Default::default() }).is_err());
    }

    /// verify-full 不允许降级到明文：对不开 TLS 的本地/CI 库必须连不上。
    #[tokio::test]
    async fn verify_full_refuses_a_server_without_tls() {
        let mut cfg = config::must_get().db;
        cfg.url = std::env::var("DATABASE_URL").ok();
        let opts = connect_options(&cfg).unwrap().ssl_mode(PgSslMode::VerifyFull);
        let err = PgPoolOptions::new().max_connections(1).connect_with(opts).await.err();
        let err = err.expect("verify-full 连上了不支持 TLS 的库");
        assert!(matches!(err, sqlx::Error::Tls(_)), "应是 TLS 错误，实际：{err:?}");
    }
}
