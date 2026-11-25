ARG JAVA_VERSION=25

FROM registry.access.redhat.com/ubi9/openjdk-${JAVA_VERSION} AS build

WORKDIR /app

# Single step: dependency:go-offline cannot resolve the os-detected classifier of Netty's native transport POMs
COPY pom.xml .
COPY src ./src
RUN mvn package -DskipTests

FROM registry.access.redhat.com/ubi9/openjdk-${JAVA_VERSION}

COPY --from=build /app/target/quarkus-app/lib/ /deployments/lib/
COPY --from=build /app/target/quarkus-app/*.jar /deployments/
COPY --from=build /app/target/quarkus-app/app/ /deployments/app/
COPY --from=build /app/target/quarkus-app/quarkus/ /deployments/quarkus/

# Serial GC on 1 CPU
ENV GC_CONTAINER_OPTIONS="-XX:+UseSerialGC"
ENV JAVA_OPTS_APPEND="-Dquarkus.http.host=0.0.0.0 -Djava.util.logging.manager=org.jboss.logmanager.LogManager"
ENV JAVA_APP_JAR="/deployments/quarkus-run.jar"

ENTRYPOINT [ "/opt/jboss/container/java/run/run-java.sh" ]
