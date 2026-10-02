ARG JAVA_VERSION=17

FROM maven:3-ibm-semeru-${JAVA_VERSION}-noble

WORKDIR /app

COPY pom.xml .

RUN mvn dependency:go-offline -B

COPY src ./src

EXPOSE 3000
CMD ["mvn", "compile", "exec:java"]
