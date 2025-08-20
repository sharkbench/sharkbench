FROM eclipse-temurin:21-jdk-noble AS build

RUN apt update
RUN apt install -y gcc g++ make cmake

WORKDIR /app
COPY . .
RUN ./gradlew linkReleaseExecutableNative

FROM ubuntu:24.04

WORKDIR /
COPY --from=build /app/build/bin/native/releaseExecutable/ktor-3-cio-native.kexe server.kexe

EXPOSE 3000
ENTRYPOINT ["/server.kexe"]