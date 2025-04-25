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
      apiServer: ServerConfig(
        port: 8080,
        publicScheme: 'http',
        publicHost: '0.0.0.0',
        publicPort: 8080,
      ),
      webServer: ServerConfig(
        port: 3000,
        publicScheme: 'http',
        publicHost: '0.0.0.0',
        publicPort: 3000,
      ),
    ),
  );

  pod.webServer.addRoute(ElementRoute(), '/api/v1/periodic-table/element');
  pod.webServer.addRoute(ShellsRoute(), '/api/v1/periodic-table/shells');

  await pod.start();
}

final httpClient = HttpClient();

final _elementUrl = Uri.http('web-data-source', '/element.json');

class ElementRoute extends WidgetRoute {
  @override
  Future<AbstractWidget> build(Session session, HttpRequest request) async {
    if (request.method != 'GET') {
      return Future.value(WidgetJson(object: {'message': 'Error'}));
    }

    final symbol = request.uri.queryParameters['symbol'];

    final tmpReq = await httpClient.getUrl(_elementUrl);
    final tmpRes = await tmpReq.close();
    final json = jsonDecode(await tmpRes.transform(utf8.decoder).join()) as Map<String, dynamic>;
    final entry = json[symbol] as Map<String, dynamic>;

    return WidgetJson(object: entry);
  }
}

final _shellsUrl = Uri.http('web-data-source', '/shells.json');

class ShellsRoute extends WidgetRoute {
  @override
  Future<AbstractWidget> build(Session session, HttpRequest request) async {
    if (request.method != 'GET') {
      return Future.value(WidgetJson(object: {'message': 'Error'}));
    }

    final symbol = request.uri.queryParameters['symbol'];

    final tmpReq = await httpClient.getUrl(_shellsUrl);
    final tmpRes = await tmpReq.close();
    final json = jsonDecode(await tmpRes.transform(utf8.decoder).join()) as Map<String, dynamic>;

    return WidgetJson(object: {
      'shells': json[symbol],
    });
  }
}
