ARG JAVA_VERSION=17

FROM maven:3-ibm-semeru-${JAVA_VERSION}-focal AS build

WORKDIR /app

COPY pom.xml .

RUN mvn dependency:go-offline -B

COPY src ./src

RUN mvn package -DskipTests

FROM ibm-semeru-runtimes:open-${JAVA_VERSION}-jre-focal AS runtime

WORKDIR /app

COPY --from=build /app/target/*.jar app.jar

EXPOSE 3000

CMD ["java", "-jar", "app.jar"]
