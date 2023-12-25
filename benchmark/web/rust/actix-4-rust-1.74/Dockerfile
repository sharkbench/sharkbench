FROM rust:1.74

WORKDIR /app
COPY . .

RUN cargo install --path .

EXPOSE 3000
CMD ["benchmark"]
