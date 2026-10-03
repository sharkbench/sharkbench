FROM dart:3.0

WORKDIR /app

COPY pubspec.yaml .
RUN dart pub get
COPY . .

RUN dart compile exe main.dart -o main

CMD ["./main"]
