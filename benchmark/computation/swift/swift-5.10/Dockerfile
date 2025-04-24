FROM swift:5.10 AS builder

WORKDIR /app
COPY Sources ./Sources
COPY Package.swift ./Package.swift

RUN swift build -c release

FROM swift:5.10-slim AS runner

WORKDIR /app
COPY --from=builder /app/.build/release/benchmark ./benchmark

EXPOSE 3000
CMD ["./benchmark"]
