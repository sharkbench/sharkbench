import 'dart:convert';
import 'dart:io';

const port = 3000;

void main() async {
  final url = Uri.http('web-data-source', '/data.json');
  final httpClient = HttpClient();

  final server = await HttpServer.bind('0.0.0.0', port);
  server.listen((HttpRequest request) async {
    final path = request.uri.path;
    final params = request.uri.queryParameters;
    final symbol = params['symbol'] as String;

    final httpClientReq = await httpClient.getUrl(url);
    final httpClientRes = await httpClientReq.close();
    final json = jsonDecode(await httpClientRes.transform(utf8.decoder).join());
    switch (path) {
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

