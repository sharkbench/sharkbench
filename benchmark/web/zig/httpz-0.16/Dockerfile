FROM alpine:edge

WORKDIR /app
COPY src ./src
COPY build.zig ./build.zig

EXPOSE 3000

RUN apk add zig=0.16.0-r0
RUN zig fetch --save "git+https://github.com/karlseguin/http.zig#03658c279937066201aba8c0a8f1936d821d0709"
RUN zig build --release=fast

CMD ["./zig-out/bin/benchmark"]
