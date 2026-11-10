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

# Local coverage and cloud-mode e2e

Coverage records a baseline and does not enforce a minimum. It needs Postgres (the same `DATABASE_URL` as the tests) and `llvm-tools-preview`.

```shell
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov --locked
cargo llvm-cov -p backend
```

CI runs that in the Backend workflow, writes the summary onto the job summary, and uploads `lcov.info`. It does not pass `--fail-under-lines` or any other threshold.

The cloud-mode browser check builds a throwaway frontend and drives it with the Rust e2e crate (add, edit, archive, empty the bin). `frontend/dist-cloud` is gitignored. Do not publish it, and do not let it replace `frontend/dist`. The production workflow still builds `frontend/dist` with the public API URL.

`password` in the URL below is the docker-compose placeholder. A separate Cargo target directory is required because changing `APP_REMOTE_ENDPOINT` alone does not rebuild the wasm.

```shell
# terminal 1: postgres, then the API
cd backend && docker-compose up -d && cd ..
DATABASE_URL=postgres://user:password@localhost:5432/rust_fullstack_todo PORT=3000 cargo run -p backend

# terminal 2: from the repo root
(cd frontend && CARGO_TARGET_DIR=/tmp/crabnote-dist-cloud APP_REMOTE_ENDPOINT=http://127.0.0.1:3000 trunk build --release --dist dist-cloud)
E2E_CLOUD=1 E2E_DIST=frontend/dist-cloud cargo test -p e2e --test app cloud_mode_add_edit_archive_clear -- --ignored --exact
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
