ARG JAVA_VERSION=17
FROM gradle:jdk${JAVA_VERSION} AS build

WORKDIR /app

COPY . .

RUN gradle build -x test
RUN unzip build/distributions/vertx-5-1.0.0.zip -d /app/extracted

FROM eclipse-temurin:${JAVA_VERSION}-alpine

WORKDIR /app

COPY --from=build /app/extracted/vertx-5-1.0.0 /app

EXPOSE 3000

CMD ["./bin/vertx-5"]