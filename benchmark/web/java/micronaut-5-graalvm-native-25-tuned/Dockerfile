FROM maven:3-eclipse-temurin-25 AS maven

FROM ghcr.io/graalvm/native-image-community:25-ol9 AS build

COPY --from=maven /usr/share/maven /usr/share/maven
ENV PATH="${JAVA_HOME}/bin:/usr/share/maven/bin:${PATH}"

WORKDIR /app
COPY pom.xml .
COPY src ./src
RUN mvn -B -DskipTests -Drelease.version=25 -Dpackaging=native-image package

FROM oraclelinux:9-slim

COPY --from=build /app/target/micronaut-benchmark /app/target/libmanagement_ext.so /app/
ENTRYPOINT ["/app/micronaut-benchmark"]
