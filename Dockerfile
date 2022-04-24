FROM ekidd/rust-musl-builder:latest AS builder

ADD --chown=rust:rust . ./

RUN cargo build --release --exclude frontend --workspace

FROM scratch

WORKDIR /home/rust

COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/backend backend
COPY application.toml application.toml

CMD ["./backend"]
