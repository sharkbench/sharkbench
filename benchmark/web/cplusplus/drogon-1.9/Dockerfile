FROM alpine:3.22 AS builder

WORKDIR /
RUN apk update
RUN apk add git gcc g++ make cmake jsoncpp-dev zlib-dev ossp-uuid-dev

RUN git clone --recurse-submodules https://github.com/drogonframework/drogon
WORKDIR /drogon
RUN mkdir build
WORKDIR /drogon/build
RUN cmake -DCMAKE_BUILD_TYPE=Release -DBUILD_EXAMPLES=OFF ..
RUN make
RUN make install

WORKDIR /server
COPY . .
RUN ls -la
RUN mkdir build
WORKDIR /server/build
RUN cmake -DCMAKE_BUILD_TYPE=Release ..
RUN make

FROM alpine:3.22

RUN apk add jsoncpp ossp-uuid-dev zlib --no-cache

WORKDIR /
COPY --from=builder /server/build/drogon drogon

EXPOSE 3000
ENTRYPOINT ["/drogon"]