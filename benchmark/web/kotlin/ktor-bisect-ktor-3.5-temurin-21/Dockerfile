ARG JAVA_VERSION=21

FROM gradle:9.8.0-jdk${JAVA_VERSION} AS build

WORKDIR /app

COPY . .

RUN gradle installDist --no-daemon

FROM eclipse-temurin:${JAVA_VERSION}-alpine

WORKDIR /app

COPY --from=build /app/build/install/app .

CMD ["bin/app"]
