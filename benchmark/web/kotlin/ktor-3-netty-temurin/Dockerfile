ARG JAVA_VERSION=17
FROM gradle:jdk${JAVA_VERSION} AS build

WORKDIR /app

COPY . .

RUN gradle build -x test

FROM eclipse-temurin:${JAVA_VERSION}-alpine

WORKDIR /app

COPY --from=build /app/build/libs/*.jar app.jar

EXPOSE 3000

CMD ["java", "-jar", "app.jar"]
