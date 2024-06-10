import 'dart:convert';
import 'dart:io';

import 'package:pharaoh/pharaoh.dart';

const port = 3000;

void main() async {
  final elementUrl = Uri.http('web-data-source', '/element.json');
  final shellsUrl = Uri.http('web-data-source', '/shells.json');
  final httpClient = HttpClient();

  final app = Pharaoh();

  app.get('/api/v1/periodic-table/element', (
    Request request,
    Response response,
  ) async {
    final symbol = request.query['symbol'] as String;

    final tmpReq = await httpClient.getUrl(elementUrl);
    final tmpRes = await tmpReq.close();
    final json = jsonDecode(await tmpRes.transform(utf8.decoder).join())
        as Map<String, dynamic>;
    final entry = json[symbol] as Map<String, dynamic>;

    return response.json({
      'name': entry['name'],
      'number': entry['number'],
      'group': entry['group'],
    });
  });

  app.get('/api/v1/periodic-table/shells', (
    Request request,
    Response response,
  ) async {
    final symbol = request.query['symbol'] as String;

    final tmpReq = await httpClient.getUrl(shellsUrl);
    final tmpRes = await tmpReq.close();
    final json = jsonDecode(await tmpRes.transform(utf8.decoder).join())
        as Map<String, dynamic>;

    return response.json({'shells': json[symbol]});
  });

  await app.listen(port: port);

  print('Running on port $port');
}
