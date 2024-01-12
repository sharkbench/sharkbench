import 'dart:io';

const port = 3000;

void main() async {
  final server = await HttpServer.bind('0.0.0.0', port);
  server.listen((HttpRequest request) {
    final params = request.uri.queryParameters;
    final i = int.parse(params['iterations']!);
    final result = pi(i);
    request.response.write('${result[0]};${result[1]};${result[2]}');
    request.response.close();
  });

  print('Running on port $port');
}

List<double> pi(int iterations) {
  double pi = 0.0;
  double denominator = 1.0;
  double sum = 0.0;
  double customNumber = 0.0;
  for (var x = 0; x < iterations; x++) {
    if (x % 2 == 0) {
      pi = pi + (1 / denominator);
    } else {
      pi = pi - (1 / denominator);
    }
    denominator = denominator + 2;

    // custom
    sum += pi;
    switch (x % 3) {
      case 0:
        customNumber += pi;
        break;
      case 1:
        customNumber -= pi;
        break;
      case 2:
        customNumber /= 2;
        break;
    }
  }
  pi = pi * 4;
  return [pi, sum, customNumber];
}
