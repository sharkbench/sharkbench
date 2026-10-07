FROM baredevcontainer/zig:0.16.0

WORKDIR /app
COPY src ./src
COPY build.zig ./build.zig

RUN zig fetch --save "git+https://github.com/farbenbuilds/uWebZockets#v1.8.0"
RUN zig build --release=fast

CMD ["./zig-out/bin/benchmark"]
