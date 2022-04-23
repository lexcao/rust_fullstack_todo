# Backend

env
* `APP_DB_PASSWORD`
* `APP_DB_HOST`
* `APP_DB_PORT`
* `APP_DB_DBNAME`

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
