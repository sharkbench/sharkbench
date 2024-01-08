import 'dart:convert';

import 'package:dart_frog/dart_frog.dart';

import 'element.dart';

final _shellsUrl = Uri.http('web-data-source', '/shells.json');

Future<Response> onRequest(RequestContext context) async {
  final request = context.request;
  if (request.method != HttpMethod.get) {
    return Response(statusCode: 405);
  }

  final symbol = request.uri.queryParameters['symbol'];

  final tmpReq = await httpClient.getUrl(_shellsUrl);
  final tmpRes = await tmpReq.close();
  final json = jsonDecode(await tmpRes.transform(utf8.decoder).join()) as Map<String, dynamic>;

  return Response(body: jsonEncode({
    'shells': json[symbol],
  }));
}
