
FROM dart:3.11.0 AS build
WORKDIR /app

COPY . .

RUN dart compile exe bin/server.dart -o server

FROM scratch
COPY --from=build /runtime/ /
COPY --from=build /app/server /bin/server

EXPOSE 3000
ENTRYPOINT ["server"]