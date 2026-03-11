FROM alpine:edge

WORKDIR /app
COPY src ./src
COPY build.zig ./build.zig

EXPOSE 3000

RUN apk add zig=0.15.2-r0
RUN zig build -Doptimize=ReleaseFast

CMD ["./zig-out/bin/benchmark"]
