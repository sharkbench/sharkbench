FROM swift:5.10

WORKDIR /app
COPY . .

RUN swift build -c release

EXPOSE 3000
CMD ["./.build/release/benchmark"]

