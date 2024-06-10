import 'dart:convert';
import 'dart:io';
import 'package:vania/vania.dart';

Application? app;

void main() async {
  app = Application();
  await app?.initialize(config: {
    'providers': <ServiceProvider>[
      RouteServiceProvder(),
    ],
  });
}

class RouteServiceProvder extends ServiceProvider {
  @override
  Future<void> boot() async {}

  @override
  Future<void> register() async {
    Router.get('/api/v1/periodic-table/element', ApiController().element);
    Router.get('api/v1/periodic-table/shells', ApiController().shells);
  }
}

class ApiController extends Controller {
  static ApiController? _singleton;

  factory ApiController() {
    if (_singleton == null) {
      _singleton = ApiController._internal();
    }
    return _singleton!;
  }

  ApiController._internal();

  final elementUrl = Uri.http('web-data-source', '/element.json');
  final shellsUrl = Uri.http('web-data-source', '/shells.json');
  final httpClient = HttpClient();

  Future element(Request request) async {
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
  }

  Future shells(Request request) async {
    final tmpReq = await httpClient.getUrl(shellsUrl);
    final tmpRes = await tmpReq.close();
    final json = jsonDecode(await tmpRes.transform(utf8.decoder).join())
        as Map<String, dynamic>;

    String symbol = request.input('symbol');
    return Response.json({
      'shells': json[symbol],
    });
  }
}
