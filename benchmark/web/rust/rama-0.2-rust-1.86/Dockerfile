FROM rust:1.86 AS builder

RUN apt-get update && \
    apt-get install --no-install-recommends -y cmake clang

WORKDIR /app
COPY Cargo.* ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm AS runtime
RUN apt update && apt install -y libssl-dev

COPY --from=builder /app/target/release/app /app

EXPOSE 3000
ENTRYPOINT ["/app"]
