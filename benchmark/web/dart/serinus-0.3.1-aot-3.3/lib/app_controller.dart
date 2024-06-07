import 'package:serinus/serinus.dart';

import 'app_routes.dart';
import 'app_provider.dart';

class AppController extends Controller {

  AppController({super.path = '/'}){
    on(BenchRoute(path: '/api/v1/periodic-table/element'), _handleElement);
    on(BenchRoute(path: '/api/v1/periodic-table/shells'), _handleShells);
  }

  Future<Response> _handleElement(RequestContext context) async {
    final String symbol = context.queryParameters['symbol'];
    final Map<String, dynamic> element = await context.use<AppProvider>().getElement(symbol);

    return Response.json(element);
  }

  Future<Response> _handleShells(RequestContext context) async {
    final String symbol = context.queryParameters['symbol'];
    final Map<String, dynamic> shells = await context.use<AppProvider>().getShells(symbol);
    return Response.json(shells);
  }

}