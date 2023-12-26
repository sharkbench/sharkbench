import 'dart:convert';
import 'dart:io';

const port = 3000;

void main() async {
  final url = Uri.http('web-data-source', '/data.json');
  final httpClient = HttpClient();

  final server = await HttpServer.bind('0.0.0.0', port);
  server.listen((HttpRequest request) async {
    final symbol = request.uri.queryParameters['symbol'] as String;

    final tmpReq = await httpClient.getUrl(url);
    final tmpRes = await tmpReq.close();
    final json = jsonDecode(await tmpRes.transform(utf8.decoder).join()) as Map<String, dynamic>;

    switch (request.uri.path) {
      case '/api/v1/periodic-table/element':
        final entry = json[symbol] as Map<String, dynamic>;
        request.response.write(jsonEncode({
          'name': entry['name'],
          'number': entry['number'],
          'group': entry['group'],
        }));
        break;
      case '/api/v1/periodic-table/shells':
        request.response.write(jsonEncode({
          'shells': json[symbol]['shells'],
        }));
        break;
    }
    request.response.close();
  });

  print('Running on port $port');
}
