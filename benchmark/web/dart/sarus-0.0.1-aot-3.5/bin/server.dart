import 'package:app/sarus_application.dart';

void main(List<String> args) async {
  // Create an instance of the SarusApplication
  final application = SarusApplication();

  // Set up the application
  await application.setup();

  // Start the server
  final server = await application.run();

  // Enable content compression
  server.autoCompress = true;

  print('Server listening on port ${server.port}');
}
