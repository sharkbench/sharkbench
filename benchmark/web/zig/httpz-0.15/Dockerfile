FROM alpine:edge

WORKDIR /app
COPY src ./src
COPY build.zig ./build.zig

EXPOSE 3000

RUN apk add zig=0.15.2-r0
RUN zig fetch --save "git+https://github.com/karlseguin/http.zig#af1df361d599dbc3f9534b0183a8b5e3fe2af83e"
RUN zig build --release=fast

CMD ["./zig-out/bin/benchmark"]
