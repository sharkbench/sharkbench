import 'dart:convert';
import 'dart:io';

import 'package:laska/laska.dart';

const port = 3000;
const dataSourceUrl = 'web-data-source';

void main() async {
  final elementUrl = Uri.http(dataSourceUrl, '/element.json');
  final shellsUrl = Uri.http(dataSourceUrl, '/shells.json');
  final httpClient = HttpClient();

  final laska = Laska(port: port, isolateCount: 1);

  laska.get('/api/v1/periodic-table/element', (Context context) async {
    final symbol = context.queryParam('symbol');
    if (symbol == null) return context.text('invalid element');

    final tmpReq = await httpClient.getUrl(elementUrl);
    final tmpRes = await tmpReq.close();
    final json = jsonDecode(await tmpRes.transform(utf8.decoder).join())
        as Map<String, dynamic>;

    final entry = json[symbol] as Map<String, dynamic>;
    await context.json(entry);
  });

  laska.get('/api/v1/periodic-table/shells', (Context context) async {
    final symbol = context.queryParam('symbol');
    if (symbol == null) return context.text('invalid element');

    final tmpReq = await httpClient.getUrl(shellsUrl);
    final tmpRes = await tmpReq.close();
    final json = jsonDecode(await tmpRes.transform(utf8.decoder).join())
        as Map<String, dynamic>;

    await context.json({'shells': json[symbol]});
  });

  await run(laska);

  //   final symbol = request.uri.queryParameters['symbol'] as String;

  //   switch (request.uri.path) {
  //     case '/api/v1/periodic-table/element':
  //       final tmpReq = await httpClient.getUrl(elementUrl);
  //       final tmpRes = await tmpReq.close();
  //       final json = jsonDecode(await tmpRes.transform(utf8.decoder).join())
  //           as Map<String, dynamic>;

  //       final entry = json[symbol] as Map<String, dynamic>;
  //       request.response.write(jsonEncode(entry));
  //       break;
  //     case '/api/v1/periodic-table/shells':
  //       final tmpReq = await httpClient.getUrl(shellsUrl);
  //       final tmpRes = await tmpReq.close();
  //       final json = jsonDecode(await tmpRes.transform(utf8.decoder).join())
  //           as Map<String, dynamic>;

  //       request.response.write(jsonEncode({
  //         'shells': json[symbol],
  //       }));
  //       break;
  //   }
  //   request.response.close();
  // });

  print('Running on port $port');
}
