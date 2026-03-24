import 'dart:convert' show utf8;
import 'dart:io'
    show
        ContentType,
        HttpRequest,
        HttpServer,
        HttpStatus,
        InternetAddress,
        Platform;
import 'dart:isolate' show Isolate;
import 'dart:math' show min;

/// Environment declarations are evaluated at compile-time.
const _maxIsolatesfromEnvironment = int.fromEnvironment('MAX_ISOLATES');

/// The fixed TCP port used by the server.
const _defaultPort = 3000;

/// Internal token used to notify newly spawned processes that they
/// belong to a secondary "worker group".
const _workerGroupTag = '--WORKER-GROUP';

void main(List<String> arguments) async {
  /// Create a mutable copy of the fixed-length arguments list.
  final args = [...arguments];

  /// Defines local isolate quota, using MAX_ISOLATES if provided.
  /// Falls back to total available cores while respecting hardware limits.
  var maxIsolates = _maxIsolatesfromEnvironment > 0
      ? min(_maxIsolatesfromEnvironment, Platform.numberOfProcessors)
      : Platform.numberOfProcessors;

  /// Determine if this process instance was initialized as a worker group.
  if (args.contains(_workerGroupTag)) {
    /// Sanitize the argument list.
    args.remove(_workerGroupTag);
  }
  /// Ensures that only the primary process can spawn worker groups
  else {
    /// Calculate the number of secondary worker groups required.
    final workerGroups = Platform.numberOfProcessors ~/ maxIsolates - 1;

    /// Bootstraps independent worker processes via AOT snapshots.
    for (var i = 0; i < workerGroups; i++) {
      /// [Isolate.spawnUri] spawns a new process group via [main()].
      await Isolate.spawnUri(Platform.script, [...args, _workerGroupTag], null);
    }

    /// Updates local isolate limits for the primary process.
    maxIsolates = Platform.numberOfProcessors - workerGroups * maxIsolates;
  }

  /// Create an [Isolate] containing an [HttpServer] for each child process.
  for (var i = 1; i < maxIsolates; i++) {
    await Isolate.spawn(_startServer, args);
  }

  /// Create a [HttpServer] for the first processor.
  await _startServer(args);
}

/// Creates and setup a [HttpServer]
Future<void> _startServer(List<String> args) async {
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
