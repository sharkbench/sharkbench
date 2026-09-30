FROM baredevcontainer/zig:0.16.0

WORKDIR /app
COPY src ./src
COPY build.zig ./build.zig

EXPOSE 5001

RUN zig build -Doptimize=ReleaseFast

CMD ["./zig-out/bin/benchmark"]
