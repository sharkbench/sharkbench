FROM dart:3.13 AS build

WORKDIR /app

COPY pubspec.yaml .
RUN dart pub get
COPY . .

# Dox has a native HTTP server, so the app is built as a bundle
# (the executable plus the native library) instead of a single exe.
RUN dart build cli -t bin/server.dart && mv build/cli/*/bundle /bundle

FROM debian:13-slim

COPY --from=build /bundle /app

CMD ["/app/bin/server"]
