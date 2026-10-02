ARG JAVA_VERSION=17
FROM gradle:jdk${JAVA_VERSION} AS build

WORKDIR /app

COPY . .

RUN gradle build -x test
RUN unzip build/distributions/vertx-5-1.0.0.zip -d /app/extracted
RUN ls -la /app/extracted/vertx-5-1.0.0

FROM ibm-semeru-runtimes:open-${JAVA_VERSION}-jre

WORKDIR /app

COPY --from=build /app/extracted/vertx-5-1.0.0 /app

RUN ls -la .
RUN ls -la bin

EXPOSE 3000

CMD ["./bin/vertx-5"]