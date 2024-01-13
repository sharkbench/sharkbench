FROM debian:bookworm

WORKDIR /usr/src/app

RUN apt-get update && \
    apt-get install -y \
    build-essential \
    libssl-dev \
    git

RUN git clone https://github.com/jorisvink/kore
RUN cd kore && git checkout 4.3.0-rc2 && make && make install

COPY src/ ./src/
COPY conf/ ./conf/

RUN kodev build

EXPOSE 3000
CMD ["kodev", "run"]
