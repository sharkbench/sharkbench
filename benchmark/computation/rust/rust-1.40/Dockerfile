FROM rust:1.40-slim

WORKDIR /app
COPY . .

RUN cargo install --path .

CMD ["benchmark"]
