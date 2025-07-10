FROM dart:3.8 AS build

WORKDIR /app

COPY pubspec.yaml .
RUN dart pub get
COPY . .

EXPOSE 3000
RUN dart compile exe bin/server.dart -o bin/server

FROM scratch
COPY --from=build /runtime/ /
COPY --from=build /app/bin/server /app/bin/


CMD ["/app/bin/server"]
