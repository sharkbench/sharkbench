FROM dart:2.14

WORKDIR /app

ADD pubspec.yaml /app/
RUN dart pub get
ADD . /app/

EXPOSE 3000
CMD ["dart", "main.dart"]
