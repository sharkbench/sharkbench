import 'dart:convert';
import 'dart:io';

const port = 3000;

void main() async {
  final elementUrl = Uri.http('web-data-source', '/element.json');
  final shellsUrl = Uri.http('web-data-source', '/shells.json');
  final httpClient = HttpClient();

  final server = await HttpServer.bind('0.0.0.0', port);
  server.listen((HttpRequest request) async {
    if (request.method != 'GET') {
      // Also check method to make it fair
      request.response.statusCode = HttpStatus.methodNotAllowed;
      request.response.close();
      return;
    }

    final symbol = request.uri.queryParameters['symbol'] as String;

    switch (request.uri.path) {
      case '/api/v1/periodic-table/element':
        final tmpReq = await httpClient.getUrl(elementUrl);
        final tmpRes = await tmpReq.close();
        final json = jsonDecode(await tmpRes.transform(utf8.decoder).join()) as Map<String, dynamic>;

        final entry = json[symbol] as Map<String, dynamic>;
        request.response.write(jsonEncode({
          'name': entry['name'],
          'number': entry['number'],
          'group': entry['group'],
        }));
        break;
      case '/api/v1/periodic-table/shells':
        final tmpReq = await httpClient.getUrl(shellsUrl);
        final tmpRes = await tmpReq.close();
        final json = jsonDecode(await tmpRes.transform(utf8.decoder).join()) as Map<String, dynamic>;

        request.response.write(jsonEncode({
          'shells': json[symbol],
        }));
        break;
    }
    request.response.close();
  });

  print('Running on port $port');
}
