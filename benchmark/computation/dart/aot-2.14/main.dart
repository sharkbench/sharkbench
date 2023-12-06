import 'dart:io';

const port = 3000;

void main() async {
  final server = await HttpServer.bind('0.0.0.0', port);
  server.listen((HttpRequest request) {
    final params = request.uri.queryParameters;
    final i = int.parse(params['iterations']!);
    request.response.write(pi(i).toString());
    request.response.close();
  });

  print('Running on port $port');
}

double pi(int iterations) {
  var pi = 0.0;
  var denominator = 1.0;
  for (var x = 0; x < iterations; x++) {
    if (x % 2 == 0) {
      pi = pi + (1 / denominator);
    } else {
      pi = pi - (1 / denominator);
    }
    denominator = denominator + 2;
  }
  pi = pi * 4;
  return pi;
}
