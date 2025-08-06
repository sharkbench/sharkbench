FROM dlangdockerized/ldc:1.40.0

WORKDIR /app
COPY . .

RUN dub build -b=release

RUN chmod +x app

EXPOSE 3000
CMD ["./app"]
