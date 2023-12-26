FROM gradle:8.4-jdk21 AS build

WORKDIR /app

COPY . .

RUN gradle build -x test

FROM eclipse-temurin:21-alpine

WORKDIR /app

COPY --from=build /app/build/libs/*.jar app.jar

EXPOSE 3000

CMD ["java", "-jar", "app.jar"]
