FROM dart:2.14

WORKDIR /app

COPY pubspec.yaml .
RUN dart pub get
COPY . .

RUN dart compile exe main.dart -o main

EXPOSE 3000
CMD ["./main"]
