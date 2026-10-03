ARG JAVA_VERSION=8

FROM maven:3-eclipse-temurin-${JAVA_VERSION} AS builder

WORKDIR /app

COPY pom.xml .

RUN mvn dependency:go-offline -B

COPY src ./src

RUN mvn package -DskipTests

FROM eclipse-temurin:${JAVA_VERSION} AS runtime

WORKDIR /app

COPY --from=builder /app/target/*.jar app.jar

CMD ["java", "-jar", "app.jar"]
