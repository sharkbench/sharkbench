import 'package:shelf/shelf.dart';
import 'package:shelf/shelf_io.dart';

void main() async {
  await serve((Request request) {
    final params = request.url.queryParameters;
    final i = int.parse(params['iterations']!);
    return Response.ok(pi(i).toString());
  }, '0.0.0.0', 3000);
  print('Running!');
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
