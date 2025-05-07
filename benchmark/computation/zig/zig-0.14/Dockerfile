FROM alpine:edge

WORKDIR /app
COPY src ./src
COPY build.zig ./build.zig

EXPOSE 3000

RUN apk add zig=0.14.0-r0
RUN zig build -Doptimize=ReleaseFast

CMD ["./zig-out/bin/benchmark"]
