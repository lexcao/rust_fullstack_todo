# Backend

The process listens on `PORT` (Render sets this) and connects with `DATABASE_URL`.
`DATABASE_URL` is a secret: it is wrapped so debug output and request logs do not print it.
When `DATABASE_URL` is set, `APP_DB_*` and the `[db]` table in `application.toml` are not used.

Priority, highest first:

1. `DATABASE_URL` (whole connection string), `PORT` (listen port)
2. `APP_SERVER_PORT`, `APP_DB_URL`, `APP_DB_HOST`, `APP_DB_PORT`, `APP_DB_USER`, `APP_DB_PASSWORD`, `APP_DB_DBNAME`, `APP_DB_SSLMODE`
3. `application.toml` (local defaults only; the password there is the docker-compose placeholder)

For Neon, use `sslmode=verify-full` in `DATABASE_URL` and do not set `channel_binding`.

`GET /ping` returns `pong` and does not touch the database.
`GET /healthz` returns 200 when `SELECT 1` succeeds, and 503 when the database is unreachable or the check exceeds 2 seconds.

Logs follow `RUST_LOG` (default `info`) via `tracing`. Each request gets an `x-request-id` header, which is echoed on the response and included in the request span. SIGTERM and Ctrl-C stop the process after in-flight requests finish. A request that runs longer than 10 seconds is cancelled with 408.

PATCH of a missing todo returns 404 `{"error":"not found"}`.
DELETE `/todos` returns 500 when the database write fails.
PATCH rejects status changes the UI cannot make (for example `done` straight to `deleted`); a new todo can still be set to `todo`, `done`, `archived`, or `deleted`.

```shell
# build
cargo build --release --exclude frontend --workspace

# run in production
cargo run --release --bin backend
```

# Frontend

1. push to GitHub
2. GitHub Action build `dist` and push to `dist` branch
3. Vercel will redeploy automatically

# Minimize Rust Image Size

base binary: 8.4MB (profile=false)
base binary: 3.8MB (profile=true)

| FROM    | size   |
|---------|--------|
| alpine  | 18.8MB |
| scratch | 13.3MB |

https://kerkour.com/rust-small-docker-image
