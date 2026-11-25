import 'package:dox_core/dox_core.dart';
import 'package:dox_http_client/dox_http_client.dart';

void main() async {
  Dox().initialize(
    AppConfig(
      appKey: 'benchbenchbenchbenchbenchbench12',
      host: '0.0.0.0',
      serverPort: 5001,
      routers: <Router>[ApiRouter()],
    ),
  );
  await Dox().startServer();
}

class ApiRouter extends Router {
  @override
  String get prefix => 'api/v1/periodic-table';

  @override
  void register() {
    ApiController controller = ApiController();
    Route.get('/element', controller.element);
    Route.get('/shells', controller.shells);
  }
}

class ApiController {
  static final Uri _elementUrl = Uri.http('127.0.0.1:5002', '/element.json');
  static final Uri _shellsUrl = Uri.http('127.0.0.1:5002', '/shells.json');

  final DoxHttpClient _httpClient = DoxHttpClient();

  Future<Map<String, dynamic>> _fetch(Uri url) async {
    DoxHttpClientResponse res = await _httpClient.get(url);
    return JSON.parse(res.bodyBytes) as Map<String, dynamic>;
  }

  Future<Map<String, dynamic>> element(DoxRequest req) async {
    Map<String, dynamic> json = await _fetch(_elementUrl);
    return json[req.query['symbol']] as Map<String, dynamic>;
  }

  Future<Map<String, dynamic>> shells(DoxRequest req) async {
    Map<String, dynamic> json = await _fetch(_shellsUrl);
    return <String, dynamic>{'shells': json[req.query['symbol']]};
  }
}
