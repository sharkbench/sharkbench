import 'dart:convert';
import 'dart:io';

import 'package:vania/vania.dart';

class ApiRoute implements Route {
  final elementUrl = Uri.http('web-data-source', '/element.json');
  final shellsUrl = Uri.http('web-data-source', '/shells.json');
  final httpClient = HttpClient();

  @override
  void register() {
    Router.basePrefix('api/v1');

    Router.get('/periodic-table/element', (Request request) async {
      final tmpReq = await httpClient.getUrl(elementUrl);
      final tmpRes = await tmpReq.close();
      final json = jsonDecode(await tmpRes.transform(utf8.decoder).join())
          as Map<String, dynamic>;
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
