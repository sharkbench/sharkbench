FROM rust:1.40-slim AS builder

WORKDIR /app
COPY src ./src
COPY Cargo.toml ./Cargo.toml

RUN cargo build --release

FROM debian:bookworm-slim AS runner

WORKDIR /app
COPY --from=builder /app/target/release/benchmark ./benchmark

CMD ["./benchmark"]
