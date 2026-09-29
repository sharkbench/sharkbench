FROM golang:1.8

WORKDIR /app
COPY . .

RUN go build -o main .

EXPOSE 5001
CMD ["./main"]
