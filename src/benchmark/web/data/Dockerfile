FROM rust:1.79 AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY static ./static

RUN cargo build --release

FROM debian:bookworm-slim AS runtime
COPY --from=builder /app/target/release/app /app

EXPOSE 3000
ENTRYPOINT ["/app"]
