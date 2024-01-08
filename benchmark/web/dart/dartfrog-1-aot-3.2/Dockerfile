FROM dart:3.2

WORKDIR /app

RUN dart pub global activate dart_frog_cli

COPY pubspec.yaml .
RUN dart pub get
COPY . .

RUN dart_frog build
RUN dart compile exe build/bin/server.dart -o server

EXPOSE 3000
ENV PORT 3000
CMD ["./server"]
