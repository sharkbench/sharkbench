import 'dart:convert';
import 'dart:io';

import 'package:serverpod/serverpod.dart';

import 'src/generated/protocol.dart';
import 'src/generated/endpoints.dart';

void run(List<String> args) async {
  final pod = Serverpod(
    ['--mode', 'production'],
    Protocol(),
    Endpoints(),
    config: ServerpodConfig(
      runMode: 'production',
      apiServer: ServerConfig(
        port: 8080,
        publicScheme: 'http',
        publicHost: '0.0.0.0',
        publicPort: 8080,
      ),
      webServer: ServerConfig(
        port: 5001,
        publicScheme: 'http',
        publicHost: '0.0.0.0',
        publicPort: 5001,
      ),
    ),
  );

  pod.webServer.addRoute(ElementRoute(), '/api/v1/periodic-table/element');
  pod.webServer.addRoute(ShellsRoute(), '/api/v1/periodic-table/shells');

  await pod.start();
}

final httpClient = HttpClient();

final _elementUrl = Uri.http('127.0.0.1:5002', '/element.json');

// WidgetRoute only responds to GET by default (served by Relic).
class ElementRoute extends WidgetRoute {
  @override
  Future<WebWidget> build(Session session, Request request) async {
    final symbol = request.url.queryParameters['symbol'];

    final tmpReq = await httpClient.getUrl(_elementUrl);
    final tmpRes = await tmpReq.close();
    final json = jsonDecode(await tmpRes.transform(utf8.decoder).join()) as Map<String, dynamic>;
    final entry = json[symbol] as Map<String, dynamic>;

    return JsonWidget(object: entry);
  }
}

final _shellsUrl = Uri.http('127.0.0.1:5002', '/shells.json');

class ShellsRoute extends WidgetRoute {
  @override
  Future<WebWidget> build(Session session, Request request) async {
    final symbol = request.url.queryParameters['symbol'];

    final tmpReq = await httpClient.getUrl(_shellsUrl);
    final tmpRes = await tmpReq.close();
    final json = jsonDecode(await tmpRes.transform(utf8.decoder).join()) as Map<String, dynamic>;

    return JsonWidget(object: {
      'shells': json[symbol],
    });
  }
}
