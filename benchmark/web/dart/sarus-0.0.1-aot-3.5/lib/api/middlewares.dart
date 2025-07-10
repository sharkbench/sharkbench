import 'package:sarus/sarus.dart';

final Middleware greetingMiddleware = createMiddleware(
  requestHandler: (request) {
    print('Greeting Request: ${request.method} ${request.requestedUri}');
    return null;
  },
);
