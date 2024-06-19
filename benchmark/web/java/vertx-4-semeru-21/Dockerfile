ARG JAVA_VERSION=21

FROM maven:3-ibm-semeru-${JAVA_VERSION}-jammy

WORKDIR /app

COPY pom.xml .

RUN mvn dependency:go-offline -B

COPY src ./src

EXPOSE 3000
CMD ["mvn", "compile", "exec:java"]
