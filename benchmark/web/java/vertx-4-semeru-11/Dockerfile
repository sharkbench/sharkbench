ARG JAVA_VERSION=11

FROM maven:3-ibm-semeru-${JAVA_VERSION}-focal

WORKDIR /app

COPY pom.xml .

RUN mvn dependency:go-offline -B

COPY src ./src

EXPOSE 3000
CMD ["mvn", "compile", "exec:java"]
