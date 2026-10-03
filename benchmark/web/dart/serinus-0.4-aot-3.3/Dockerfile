FROM dart:3.3

WORKDIR /app

COPY pubspec.yaml .
RUN dart pub get
COPY . .

RUN dart compile exe bin/main.dart -o main

CMD ["./main"]
