import 'package:dbcrypt/dbcrypt.dart';
import 'package:shelf/shelf.dart';
import 'package:shelf/shelf_io.dart';

void main() async {
  await serve((Request request) {
    final params = request.url.queryParameters;
    return Response.ok(run(params['password']!, params['salt']!));
  }, '0.0.0.0', 3000);
  print('Running!');
}

String run(String password, String salt) {
  return new DBCrypt().hashpw(password, salt);
}
