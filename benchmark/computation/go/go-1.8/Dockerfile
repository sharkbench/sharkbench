FROM golang:1.8

WORKDIR /app
COPY . .

RUN go build -o main .

EXPOSE 3000
CMD ["./main"]
