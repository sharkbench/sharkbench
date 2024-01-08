import 'dart:convert';
import 'dart:io';

import 'package:dart_frog/dart_frog.dart';

final httpClient = HttpClient();

final _elementUrl = Uri.http('web-data-source', '/element.json');

Future<Response> onRequest(RequestContext context) async {
  final request = context.request;
  if (request.method != HttpMethod.get) {
    return Response(statusCode: 405);
  }

  final symbol = request.uri.queryParameters['symbol'];

  final tmpReq = await httpClient.getUrl(_elementUrl);
  final tmpRes = await tmpReq.close();
  final json = jsonDecode(await tmpRes.transform(utf8.decoder).join()) as Map<String, dynamic>;
  final entry = json[symbol] as Map<String, dynamic>;

  return Response(body: jsonEncode({
    'name': entry['name'],
    'number': entry['number'],
    'group': entry['group'],
  }));
}
