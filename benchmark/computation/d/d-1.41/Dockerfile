FROM vreitech/docker-ldc:2-v1.41.0

WORKDIR /app
COPY . .

RUN dub build -b=release-nobounds --compiler=ldc2

RUN chmod +x app

EXPOSE 5001
CMD ["./app"]
