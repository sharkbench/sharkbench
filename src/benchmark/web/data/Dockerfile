FROM rust:1.86 AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim AS runtime
COPY --from=builder /app/target/release/app /app
COPY static ./static

EXPOSE 80
ENTRYPOINT ["/app"]
