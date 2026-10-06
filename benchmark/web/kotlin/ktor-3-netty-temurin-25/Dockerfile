FROM gradle:9.8.0-jdk25 AS build

WORKDIR /app

COPY . .

RUN gradle installDist --no-daemon

FROM eclipse-temurin:25-alpine

WORKDIR /app

COPY --from=build /app/build/install/app .

CMD ["bin/app"]
