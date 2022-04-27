# A Rust Fullstack Todo Application

You can try it [here](https://todos.lexcao.io), and I also write a blog about it.

* development by TDD
* all Rust code in frontend and backend with share code

# Getting Started

### Requirement

* [cargo](https://doc.rust-lang.org/stable/cargo/)
* [trunk](https://trunkrs.dev/)

### Start backend

```shell
# 1. setup local postgres in docker
$ cd backend && docker-compose up -d

# 2. start backend
$ cd backend && cargo run 

# 3. start backend (another way)
$ cargo run --bin backend
```

### Start frontend

```shell
$ cd frontend && trunk serve
```

# How it built

Thanks for the open source.

### Backend

* rust: [book](https://doc.rust-lang.org/book/)
* web: [actix-web](https://github.com/actix/actix-web)
* async: [tokio](https://github.com/tokio-rs/tokio)
* db: [postgres](https://github.com/sfackler/rust-postgres)
* more to [see](./backend/Cargo.toml)

### Frontend

* wasm: [book](https://rustwasm.github.io/docs/book/)
* web: [yew](https://github.com/yewstack/yew)
* hooks: [yew-hooks](https://github.com/jetli/yew-hooks)
* http: [reqwest](https://github.com/seanmonstar/reqwest)
* more to [see](./frontend/Cargo.toml)

### [Deployment](Deployment.md)

* frontend: [Vercel](https://vercel.com/)
* backend: [Railway](https://railway.app/)
* postgres: [Supabase](https://supabase.com/)

# [Design](Design.md)
