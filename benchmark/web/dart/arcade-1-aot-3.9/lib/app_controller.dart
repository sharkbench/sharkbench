import 'package:app/element_service.dart';
import 'package:arcade/arcade.dart';

class ElementController {
  ElementController(this.elementService) {
    route.group<RequestContext>(
      '/api/v1/periodic-table',
      defineRoutes: (route) {
        route().get('/element').handle(_handleElement);
        route().get('/shells').handle(_handleShells);
      },
    );
  }

  final ElementService elementService;

  Future<Map<String, dynamic>> _handleElement(RequestContext context) async {
    final String symbol = context.queryParameters['symbol']!;
    final Map<String, dynamic> element =
        await elementService.getElement(symbol);

    return element;
  }

  Future<Map<String, dynamic>> _handleShells(RequestContext context) async {
    final String symbol = context.queryParameters['symbol']!;
    final Map<String, dynamic> shells = await elementService.getShells(symbol);
    return shells;
  }
}
