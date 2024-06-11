FROM dart:3.0

WORKDIR /app

COPY pubspec.yaml .
RUN dart pub get
COPY . .

EXPOSE 3000
RUN dart compile exe main.dart -o main

CMD ["./main"]
