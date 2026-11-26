FROM maven:3-eclipse-temurin-27 AS build

WORKDIR /app
COPY pom.xml .
COPY src ./src
RUN mvn -B -DskipTests package

FROM eclipse-temurin:27-jre

COPY --from=build /app/target/micronaut-benchmark.jar /app/micronaut-benchmark.jar
ENTRYPOINT ["java", "-XX:+UseSerialGC", "--enable-native-access=ALL-UNNAMED", "-jar", "/app/micronaut-benchmark.jar"]
