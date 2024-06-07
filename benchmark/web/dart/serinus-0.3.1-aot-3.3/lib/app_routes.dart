import 'package:serinus/serinus.dart';

class BenchRoute extends Route {
  BenchRoute({required super.path}) : super(method: HttpMethod.get, queryParameters: {
    'symbol': String
  });
}