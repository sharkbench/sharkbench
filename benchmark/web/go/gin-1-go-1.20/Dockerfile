FROM golang:1.20

WORKDIR /app
COPY . .

RUN go get main

RUN go build -o main .

RUN chmod +x main

EXPOSE 3000
CMD ["./main"]
