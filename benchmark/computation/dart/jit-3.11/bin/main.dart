import 'dart:convert' show utf8;
import 'dart:io'
    show ContentType, HttpRequest, HttpServer, HttpStatus, InternetAddress;

/// The fixed TCP port used by the server.
const _defaultPort = 3000;

void main(List<String> args) async {
  /// Binds the [HttpServer] on `0.0.0.0:3000`.
  final server = await HttpServer.bind(
    InternetAddress.anyIPv4,
    _defaultPort,
    shared: true,
  );

  /// Listens for incoming HTTP requests and routes them to the request handler.
  await for (final request in server) {
    await _handleRequest(request);
  }
}

/// Processes incoming HTTP requests.
Future<void> _handleRequest(HttpRequest request) async {
  /// Enforce GET-only requests, rejecting all other HTTP methods.
  if (request.method != 'GET') {
    _sendResponse(request, HttpStatus.methodNotAllowed);
    return;
  }

  try {
    /// Extract the required 'iterations' query parameter from the request URI.
    final iterations = int.parse(request.uri.queryParameters['iterations']!);

    /// Executes the CPU-intensive Pi calculation based on the user-provided iteration count.
    final result = _computation(iterations);

    /// Formats the calculated results and sends them as a plain text response.
    _sendText(request, '${result[0]};${result[1]};${result[2]}');
  } catch (e) {
    /// Catch any unexpected errors during request processing.
    _sendResponse(request, HttpStatus.internalServerError);
  }
}

/// Completes the given [request] by writing the [bytes] with the given [statusCode] and [type].
void _sendResponse(
  HttpRequest request,
  int statusCode, {
  ContentType? type,
  List<int> bytes = const [],
}) => request.response
  ..statusCode = statusCode
  ..headers.contentType = type
  ..contentLength = bytes.length
  ..add(bytes)
  ..close();

/// Completes the given [request] by writing the [response] as plain text.
void _sendText(HttpRequest request, String response) => _sendResponse(
  request,
  HttpStatus.ok,
  type: ContentType.text,
  bytes: utf8.encode(response),
);

/// Calculates an approximation of Pi using the Leibniz series along with cumulative sums.
List<double> _computation(int iterations) {
  var pi = 0.0;
  var denominator = 1.0;
  var sum = 0.0;
  var customNumber = 0.0;
  for (var i = 0; i < iterations; i++) {
    if (i % 2 == 0) {
      pi = pi + (1 / denominator);
    } else {
      pi = pi - (1 / denominator);
    }
    denominator = denominator + 2;
    sum += pi;
    switch (i % 3) {
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
