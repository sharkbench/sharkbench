import 'dart:convert';
import 'dart:io';

import 'package:shelf/shelf.dart';
import 'package:shelf/shelf_io.dart';
import 'package:shelf_router/shelf_router.dart';

const port = 3000;

void main() async {
  final elementUrl = Uri.http('web-data-source', '/element.json');
  final shellsUrl = Uri.http('web-data-source', '/shells.json');
  final httpClient = HttpClient();

  final app = Router();

  app.get('/api/v1/periodic-table/element', (Request request) async {
    final symbol = request.requestedUri.queryParameters['symbol'] as String;

    final tmpReq = await httpClient.getUrl(elementUrl);
    final tmpRes = await tmpReq.close();
    final json = jsonDecode(await tmpRes.transform(utf8.decoder).join()) as Map<String, dynamic>;
    final entry = json[symbol] as Map<String, dynamic>;

    return Response.ok(jsonEncode(entry));
  });

  app.get('/api/v1/periodic-table/shells', (Request request) async {
    final symbol = request.requestedUri.queryParameters['symbol'] as String;

    final tmpReq = await httpClient.getUrl(shellsUrl);
    final tmpRes = await tmpReq.close();
    final json = jsonDecode(await tmpRes.transform(utf8.decoder).join()) as Map<String, dynamic>;

    return Response.ok(jsonEncode({
      'shells': json[symbol],
    }));
  });

  await serve(app, '0.0.0.0', port);

  print('Running on port $port');
}
