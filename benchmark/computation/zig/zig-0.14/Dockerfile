FROM alpine:edge

WORKDIR /app
COPY src ./src
COPY build.zig ./build.zig

EXPOSE 3000

RUN apk add zig
RUN zig build -Doptimize=ReleaseFast

CMD ["./zig-out/bin/zig_0_14"]
