import 'dart:convert';
import 'dart:io';

import 'package:vania/vania.dart';

Application? app;

final elementUrl = Uri.http('web-data-source', '/element.json');
final shellsUrl = Uri.http('web-data-source', '/shells.json');
final httpClient = HttpClient();

void main() async {
  app = Application();
  await app?.initialize(config: {
    'providers': <ServiceProvider>[RouteServiceProvder()]
  });
}

class RouteServiceProvder extends ServiceProvider {
  @override
  Future<void> boot() async {}

  @override
  Future<void> register() async {
    Router.basePrefix('api/v1');
    Router.get('/periodic-table/element', (Request request) async {
      final tmpReq = await httpClient.getUrl(elementUrl);
      final tmpRes = await tmpReq.close();
      final json = jsonDecode(await tmpRes.transform(utf8.decoder).join())
          as Map<String, dynamic>;
      print(request.all());
      String symbol = request.query('symbol');

      final entry = json[symbol] as Map<String, dynamic>;
      return Response.json({
        'name': entry['name'],
        'number': entry['number'],
        'group': entry['group'],
      });
    });

    Router.get('/periodic-table/shells', (Request request) async {
      final tmpReq = await httpClient.getUrl(shellsUrl);
      final tmpRes = await tmpReq.close();
      final json = jsonDecode(await tmpRes.transform(utf8.decoder).join())
          as Map<String, dynamic>;

      String symbol = request.input('symbol');
      return Response.json({
        'shells': json[symbol],
      });
    });
  }
}
