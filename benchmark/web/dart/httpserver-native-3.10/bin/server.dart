import 'dart:convert' show jsonDecode, JsonUtf8Encoder, utf8;
import 'dart:io'
    show
        ContentType,
        HttpClient,
        HttpRequest,
        HttpServer,
        HttpStatus,
        InternetAddress,
        Platform;
import 'dart:isolate' show Isolate;

/// A type alias for standard JSON objects for improving readability
typedef Json = Map<String, Object?>;

/// The fixed TCP port used by the server.
const _defaultPort = 3000;

/// A reusable instance of the UTF-8 JSON encoder.
final _jsonEncoder = JsonUtf8Encoder();

/// Defines the routing paths exposed by this HTTP server.
final class _Endpoints {
  static const element = '/api/v1/periodic-table/element';
  static const shells = '/api/v1/periodic-table/shells';
}

/// Defines the URIs for the internal upstream services for data sources.
final class _DataSource {
  static final element = Uri.parse('http://web-data-source/element.json');
  static final shells = Uri.parse('http://web-data-source/shells.json');
}

void main(List<String> args) async {
  /// Create an [Isolate] containing an [HttpServer] for each child process.
  for (var i = 1; i < Platform.numberOfProcessors; i++) {
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

  /// Initializes a reusable [HttpClient] for making outbound requests.
  final client = HttpClient();

  /// Listens for incoming HTTP requests and routes them to the request handler.
  await for (final request in server) {
    await _handleRequest(request, client);
  }
}

/// Processes incoming HTTP requests.
Future<void> _handleRequest(HttpRequest request, HttpClient client) async {
  /// Enforce GET-only requests, rejecting all other HTTP methods.
  if (request.method != 'GET') {
    _sendResponse(request, HttpStatus.methodNotAllowed);
    return;
  }

  try {
    /// Extract the required 'symbol' query parameter from the request URI.
    final query = request.uri.queryParameters['symbol']!;

    /// Route the request to the correct handler based on the requested path.
    switch (request.uri.path) {
      case _Endpoints.element:
        await _handleElement(request, query, client);
        break;
      case _Endpoints.shells:
        await _handleShell(request, query, client);
        break;
      default:
        _sendResponse(request, HttpStatus.notFound);
    }
  } catch (e) {
    /// Catch any unexpected errors during request processing.
    _sendResponse(request, HttpStatus.internalServerError);
  } finally {
    /// Ensure the client connection is closed after processing.
    client.close();
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

/// Completes the given [request] by writing the [response] as JSON.
void _sendJson(
  HttpRequest request,
  Object response,
) => _sendResponse(
  request,
  HttpStatus.ok,
  type: ContentType.json,
  bytes: _jsonEncoder.convert(response),
);

/// Fetches JSON data from a designated upstream [url] using the provided [client].
Future<Json> _fetchData(
  Uri url,
  String query,
  HttpClient client,
) => client
    .getUrl(url)
    .then((request) => request.close())
    .then((response) => response.transform(utf8.decoder).join())
    .then((json) => jsonDecode(json) as Json);

/// Handles requests to the [Endpoints.element] route.
Future<void> _handleElement(
  HttpRequest request,
  String query,
  HttpClient client,
) async {
  final data = await _fetchData(_DataSource.element, query, client);
  _sendJson(request, data);
}

/// Handles requests to the [Endpoints.shells] route.
Future<void> _handleShell(
  HttpRequest request,
  String query,
  HttpClient client,
) async {
  final data = await _fetchData(_DataSource.shells, query, client);
  _sendJson(request, {'shells': data});
}
