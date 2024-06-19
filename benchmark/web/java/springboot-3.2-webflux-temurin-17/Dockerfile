ARG JAVA_VERSION=17

FROM maven:3-eclipse-temurin-${JAVA_VERSION} AS build

WORKDIR /app

COPY pom.xml .

RUN mvn dependency:go-offline -B

COPY src ./src

RUN mvn package -DskipTests

FROM eclipse-temurin:${JAVA_VERSION}

WORKDIR /app

COPY --from=build /app/target/*.jar app.jar

EXPOSE 3000

CMD ["java", "-jar", "app.jar"]
