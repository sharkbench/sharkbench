FROM maven:3-eclipse-temurin-27 AS build

WORKDIR /app
COPY pom.xml .
COPY src ./src
RUN mvn -B -DskipTests -Drelease.version=27 package

FROM eclipse-temurin:27-jre

COPY --from=build /app/target/micronaut-benchmark.jar /app/micronaut-benchmark.jar
ENTRYPOINT ["java", "-jar", "/app/micronaut-benchmark.jar"]
