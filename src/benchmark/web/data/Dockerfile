FROM rust:1.74 AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY static ./static

RUN cargo build --release

FROM debian:bookworm AS runtime
COPY --from=builder /app/target/release/app /app

EXPOSE 3000
ENTRYPOINT ["/app"]
