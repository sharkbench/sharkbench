FROM dart:3.14 AS build

WORKDIR /app

COPY pubspec.yaml .
RUN dart pub get
COPY . .

# Serverpod 4 depends on packages with build hooks (sqlite3), which
# 'dart compile exe' does not support, so the app is built as a bundle
# (the executable plus native libraries) instead of a single exe.
RUN dart build cli -t bin/main.dart && mv build/cli/*/bundle /bundle

FROM debian:13-slim

COPY --from=build /bundle /app

CMD ["/app/bin/main"]
