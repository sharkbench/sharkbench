import 'dart:convert';
import 'dart:io';

import 'package:vania/vania.dart';
import 'package:vania/route.dart';
import 'package:vania/service_provider.dart';
import 'package:vania/http/controller.dart';
import 'package:vania/http/request.dart';
import 'package:vania/http/response.dart';

void main() async {
  await Application().initialize(config: {
    'name': env('APP_NAME', 'periodic-table'),
    'url': env('APP_URL', 'http://localhost'),
    'providers': <ServiceProvider>[
      RouteServiceProvider(),
    ],
  });
}

class RouteServiceProvider extends ServiceProvider {
  @override
  Future<void> register() async {}

  @override
  Future<void> boot() async {
    ApiRoute().register();
  }
}

class ApiRoute extends Route {
  @override
  String? get prefix => 'api/v1/periodic-table';

  @override
  void register() {
    super.register();

    Router.get('/element', apiController.element);
    Router.get('/shells', apiController.shells);
  }
}

class ApiController extends Controller {
  static final _elementUrl = Uri.http('web-data-source', '/element.json');
  static final _shellsUrl = Uri.http('web-data-source', '/shells.json');

  final HttpClient _httpClient = HttpClient();

  Future<Map<String, dynamic>> _fetch(Uri url) async {
    final req = await _httpClient.getUrl(url);
    final res = await req.close();
    if (res.statusCode != HttpStatus.ok) {
      await res.drain<void>();
      throw HttpResponseException(
        message: 'Data source returned ${res.statusCode}',
        code: HttpStatus.badGateway,
      );
    }
    final body = await res.transform(utf8.decoder).join();
    return jsonDecode(body) as Map<String, dynamic>;
  }

  String _symbol(Request request) {
    final symbol = request.query('symbol') as String?;
    if (symbol == null || symbol.isEmpty) {
      throw HttpResponseException(
        message: 'symbol is required',
        code: HttpStatus.badRequest,
      );
    }
    return symbol;
  }

  Future<Response> element(Request request) async {
    final symbol = _symbol(request);
    final element = (await _fetch(_elementUrl))[symbol];
    if (element == null) {
      throw NotFoundException(
        message: 'Unknown element symbol: $symbol',
        responseType: ResponseType.json,
      );
    }
    return Response.json(element);
  }

  Future<Response> shells(Request request) async {
    final symbol = _symbol(request);
    final shells = (await _fetch(_shellsUrl))[symbol];
    if (shells == null) {
      throw NotFoundException(
        message: 'Unknown element symbol: $symbol',
        responseType: ResponseType.json,
      );
    }
    return Response.json({'shells': shells});
  }
}

final ApiController apiController = ApiController();