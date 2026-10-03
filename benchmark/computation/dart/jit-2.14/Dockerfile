FROM dart:2.14

WORKDIR /app

COPY pubspec.yaml .
RUN dart pub get
COPY . .

CMD ["dart", "main.dart"]
