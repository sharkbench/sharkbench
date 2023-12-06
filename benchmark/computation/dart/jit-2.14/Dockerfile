FROM dart:2.14

WORKDIR /app

COPY pubspec.yaml .
RUN dart pub get
COPY . .

EXPOSE 3000
CMD ["dart", "main.dart"]
