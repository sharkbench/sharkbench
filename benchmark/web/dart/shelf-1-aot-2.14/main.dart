import 'dart:convert';
import 'dart:io';

import 'package:shelf/shelf.dart';
import 'package:shelf/shelf_io.dart';
import 'package:shelf_router/shelf_router.dart';

const port = 3000;

void main() async {
  final url = Uri.http('web-data-source', '/data.json');
  final httpClient = HttpClient();

  final app = Router();

  app.get('/api/v1/periodic-table/element', (Request request) async {
    final symbol = request.requestedUri.queryParameters['symbol'] as String;
    final json = await fetchJson(url, httpClient);
    final entry = json[symbol] as Map<String, dynamic>;

    return Response.ok(jsonEncode({
      'name': entry['name'],
      'number': entry['number'],
      'group': entry['group'],
    }));
  });

  app.get('/api/v1/periodic-table/shells', (Request request) async {
    final symbol = request.requestedUri.queryParameters['symbol'] as String;
    final json = await fetchJson(url, httpClient);

    return Response.ok(jsonEncode({
      'shells': json[symbol]['shells'],
    }));
  });

  await serve(app, '0.0.0.0', port);

  print('Running on port $port');
}

Future<dynamic> fetchJson(Uri uri, HttpClient client) async {
  final httpClientReq = await client.getUrl(uri);
  final httpClientRes = await httpClientReq.close();
  return jsonDecode(await httpClientRes.transform(utf8.decoder).join());
}
