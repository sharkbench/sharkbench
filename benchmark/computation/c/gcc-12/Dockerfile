FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y gcc

COPY . .

RUN gcc -o server main.c -O3 -march=native -flto

CMD ["./server"]
