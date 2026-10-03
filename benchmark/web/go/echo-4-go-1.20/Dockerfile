FROM golang:1.20

WORKDIR /app
COPY . .

RUN go get main

RUN go build -o main .

RUN chmod +x main

CMD ["./main"]
