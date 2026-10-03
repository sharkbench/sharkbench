FROM dart:3.9 AS build

WORKDIR /app

COPY pubspec.yaml .
RUN dart pub get
COPY . .

# AOT compile the app
RUN dart compile exe bin/server.dart -o bin/server

# Build minimal serving image from AOT-compiled `/server`
# and the pre-built AOT-runtime in the `/runtime/` directory of the base image.
FROM scratch

COPY --from=build /runtime/ /
COPY --from=build /app/bin/server /bin/server
COPY --from=build /app/.env /

EXPOSE 5001

CMD ["/bin/server"]
