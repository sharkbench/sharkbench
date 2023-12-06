import 'dart:convert';
import 'dart:io';

/// Runs a shell command.
/// Shows the output in the console.
/// Throws an exception if the command fails.
Future<void> runShell({
  required List<String> cmd,
  String? workingDir,
  void Function(Process)? onProcess,
  void Function(String)? onOutput,
}) async {
  final process = await Process.start(
    cmd.first,
    cmd.sublist(1),
    mode: onOutput != null ? ProcessStartMode.normal : ProcessStartMode.inheritStdio,
    workingDirectory: workingDir,
  );

  if (onOutput != null) {
    process.stdout.transform(utf8.decoder).listen(onOutput);
  }

  if (onProcess != null) {
    onProcess(process);
  }

  final exitCode = await process.exitCode;
  if (exitCode != 0 && exitCode != 255) {
    throw 'Failed to run command: $exitCode';
  }
}
