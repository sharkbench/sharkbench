import 'dart:convert';
import 'dart:io';

import 'package:sarus/sarus.dart';

import '../sarus_application.g.dart';

@Endpoint(path: '/periodic-table')
class PeriodicEndpoints extends Endpoints {
  PeriodicEndpoints() : super();

  final _httpClient = HttpClient();

  @Get(path: '/element')
  Future<Response> element(
    Request request,
    @QueryParam('symbol') String symbol,
  ) async {
    final elements = await _fetchSource('/element.json');
    final result = elements[symbol] as Map<String, dynamic>;

    return Response.ok(jsonEncode(result));
  }

  @Get(path: '/shells')
  Future<Response> shells(
    Request request,
    @QueryParam('symbol') String symbol,
  ) async {
    final shells = await _fetchSource('/shells.json');

    return Response.ok(jsonEncode({'shells': shells[symbol]}));
  }

  Future<Map<String, dynamic>> _fetchSource(String path) async {
    try {
      final elementUrl = Uri.http('web-data-source', path);

      final request = await _httpClient.getUrl(elementUrl);
      final response = await request.close();

      return jsonDecode(await response.transform(utf8.decoder).join())
          as Map<String, dynamic>;
    } catch (error) {
      return {"message": "something went wrong $error"};
    }
  }

  @override
  RouterConfig get router => $periodicEndpointsRouterConfig(this);
}
