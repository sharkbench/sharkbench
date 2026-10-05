FROM gcc:10

WORKDIR /app

COPY . .

RUN gcc -o server main.c -O3 -march=native -flto

CMD ["./server"]
