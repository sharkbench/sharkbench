ARG JAVA_VERSION=25

FROM maven:3-ibm-semeru-${JAVA_VERSION}-noble AS build

WORKDIR /app

COPY pom.xml .

RUN mvn dependency:go-offline -B

COPY src ./src

RUN mvn package -DskipTests

FROM ibm-semeru-runtimes:open-${JAVA_VERSION}-jre-noble AS runtime

WORKDIR /app

COPY --from=build /app/target/*.jar app.jar

CMD ["java", "-jar", "app.jar"]
