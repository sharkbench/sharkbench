import 'dart:io';

import 'package:sharkbench/utils/shell.dart';

/// Starts a docker container with the given [composeFile].
/// The container is stopped after the function [onContainerStarted] has finished.
/// If [composeFile] is null, the directory is expected to contain a docker-compose.yml file.
Future<void> runDockerCompose({
  required String dir,
  required String? composeFile,
  required Future<void> Function() onContainerStarted,
}) async {
  if (composeFile != null) {
    File('$dir/docker-compose.yml').writeAsStringSync(composeFile);
    File('$dir/.dockerignore').writeAsStringSync(_ignoreFile);
  }

  print(' -> Building image');
  await runShell(
    cmd: ['docker', 'compose', 'up', '--build', '-d'],
    workingDir: dir,
  );

  await Future.delayed(const Duration(seconds: 5));

  await onContainerStarted();

  print(' -> Stopping container');
  await runShell(
    cmd: ['docker', 'compose', 'down', '--rmi', 'all'],
    workingDir: dir,
  );

  if (composeFile != null) {
    File('$dir/docker-compose.yml').deleteSync();
    File('$dir/.dockerignore').deleteSync();
  }
}

const _ignoreFile = '''
.dart_tool
node_modules
''';
