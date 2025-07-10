// GENERATED CODE - DO NOT MODIFY BY HAND
import 'package:sarus/sarus.dart';
import 'package:app/api/endpoints.dart';

/// Generated router configuration for [PeriodicEndpoints].
/// This function creates and configures all routes defined in the endpoint class.
RouterConfig $periodicEndpointsRouterConfig(PeriodicEndpoints endpoints) {
  final routerConfig = RouterConfig();

  routerConfig.get('/periodic-table/element', (Request req) async {
    try {
      final symbolRaw = req.url.queryParameters['symbol'];
      if (symbolRaw == null) {
        throw ArgumentError('Missing required query parameter: symbol');
      }
      final symbol = symbolRaw;

      return endpoints.element(req, symbol);
    } catch (e) {
      if (e is ArgumentError) {
        return Response.badRequest(body: e.message);
      }
      if (e is FormatException) {
        return Response.badRequest(body: 'Invalid JSON format: ${e.message}');
      }
      rethrow;
    }
  });

  routerConfig.get('/periodic-table/shells', (Request req) async {
    try {
      final symbolRaw = req.url.queryParameters['symbol'];
      if (symbolRaw == null) {
        throw ArgumentError('Missing required query parameter: symbol');
      }
      final symbol = symbolRaw;

      return endpoints.shells(req, symbol);
    } catch (e) {
      if (e is ArgumentError) {
        return Response.badRequest(body: e.message);
      }
      if (e is FormatException) {
        return Response.badRequest(body: 'Invalid JSON format: ${e.message}');
      }
      rethrow;
    }
  });

  return routerConfig;
}

