FROM vreitech/docker-ldc:latest

WORKDIR /app
COPY . .

RUN dub build -b=release-nobounds --compiler=ldc2

RUN chmod +x app

EXPOSE 3000
CMD ["./app"]
