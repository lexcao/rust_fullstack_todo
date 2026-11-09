# Backend: Render Free Web + Neon Free Postgres

1. Create a Neon Free Postgres project and copy its connection URL with
   `sslmode=require`. Keep the real URL in the Render environment settings only.
2. Create a Render Blueprint from this repository using `render.yaml`. It defines
   a Docker web service on the free plan with `/ping` as its health check.
3. Set the prompted `DATABASE_URL` to the Neon URL. The multi-stage Dockerfile
   builds only the backend; the Debian runtime includes OpenSSL and CA certificates
   for certificate and hostname verification. No frontend build is required.
4. Deploy and check `/ping`. Startup obtains a database connection and executes
   the embedded `backend/schemas/00_init_tables.sql` before accepting requests.
   `CREATE TABLE IF NOT EXISTS` makes restarts safe; connection or schema errors
   stop startup. The database role needs permission to create the table.

See the [Render Blueprint reference](https://render.com/docs/blueprint-spec).
The runtime contains only the server, configuration and system libraries; Rust
compilation happens in the build stage, outside the 512 MB free runtime.

## Environment configuration

Configuration loads `application.toml` from the working directory, then `APP_*`
environment variables. Never commit database credentials or connection URLs.

| Variable | Behavior |
| --- | --- |
| `DATABASE_URL` | Preferred complete Postgres URL; overrides `APP_DB_URL` and individual database fields. Use Neon's URL with `sslmode=require`. |
| `APP_DB_URL` | Alternative complete Postgres URL; overrides individual database fields. |
| `APP_DB_HOST`, `APP_DB_PORT`, `APP_DB_USER`, `APP_DB_PASSWORD`, `APP_DB_DBNAME` | Existing separate connection settings, used when no URL is set. |
| `PORT` | Render's listening port; overrides `APP_SERVER_PORT`. Server binds `0.0.0.0`. |
| `APP_SERVER_PORT` | Local/custom port; defaults to 3000 from TOML. |
| `RUST_LOG` | Logging filter; Blueprint sets `info`. |

The TLS connector honors `sslmode=require` without plaintext fallback. With no
SSL mode specified, PostgreSQL's `prefer` mode permits local docker-compose
Postgres without TLS. A local URL may explicitly use `sslmode=disable` to select
`NoTls`. TLS certificate verification remains enabled when TLS is used.

For a local foreground build/run from the repository root:

```shell
cargo build --locked -p backend
cargo run --bin backend
```

Render Free web services sleep when idle, and Neon Free compute can suspend when
inactive. Allow for cold starts on the first request; free-tier limits may change.
See [Render Free](https://render.com/docs/free) and
[Neon scale to zero](https://neon.com/docs/introduction/scale-to-zero).

# Frontend

The existing frontend deployment remains separate (Vercel). Updating its backend
endpoint and CI runners is outside this backend deployment change.
