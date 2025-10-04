import 'package:app/init.dart';
import 'package:arcade/arcade.dart';

Future<void> main(List<String> arguments) async {
  await runServer(port: 3000, init: init);
}
