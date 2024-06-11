FROM maven:3-eclipse-temurin-11

WORKDIR /app

COPY pom.xml .

RUN mvn dependency:go-offline -B

COPY src ./src

EXPOSE 3000
CMD ["mvn", "compile", "exec:java"]
