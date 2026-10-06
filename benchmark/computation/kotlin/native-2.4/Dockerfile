FROM gradle:9.8.0-jdk25-noble AS builder

WORKDIR /app
COPY settings.gradle.kts build.gradle.kts gradle.properties ./
COPY src ./src

RUN gradle linkReleaseExecutableLinuxX64 --no-daemon

FROM debian:13-slim AS runner

WORKDIR /app
COPY --from=builder /app/build/bin/linuxX64/releaseExecutable/benchmark.kexe ./benchmark

CMD ["./benchmark"]
