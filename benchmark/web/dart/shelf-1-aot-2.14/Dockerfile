FROM dart:2.14

WORKDIR /app

COPY pubspec.yaml .
RUN dart pub get
COPY . .

RUN dart compile exe main.dart -o main

CMD ["./main"]
