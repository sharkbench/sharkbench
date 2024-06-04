FROM dart:3.1

WORKDIR /app

COPY pubspec.yaml .
RUN dart pub get
COPY . .

EXPOSE 3000
RUN dart compile exe bin/main.dart -o main

CMD ["./main"]
