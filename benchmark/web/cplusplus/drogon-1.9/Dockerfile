FROM gcc:14 AS builder

WORKDIR /
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        git \
        make \
        cmake \
        libjsoncpp-dev \
        zlib1g-dev \
        uuid-dev \
    && rm -rf /var/lib/apt/lists/*

# Build Drogon framework at specific version
RUN git clone --recurse-submodules --branch v1.9.11 https://github.com/drogonframework/drogon && \
    cd drogon && \
    mkdir build && \
    cd build && \
    cmake -DCMAKE_BUILD_TYPE=Release -DBUILD_EXAMPLES=OFF .. && \
    make -j$(nproc) && \
    make install

# Build app
WORKDIR /server
COPY . .
RUN mkdir build && \
    cd build && \
    cmake -DCMAKE_BUILD_TYPE=Release .. && \
    make -j$(nproc)

FROM debian:trixie-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libjsoncpp26 \
        uuid-runtime \
        zlib1g \
        openssl \
        libbrotli1 \
        libpq5 \
        libmariadb3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /
COPY --from=builder /server/build/drogon drogon

EXPOSE 3000
ENTRYPOINT ["/drogon"]
