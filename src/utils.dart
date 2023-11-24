import 'dart:io';

import 'models/benchmark_result.dart';

const _composeFile = '''
version: "3"
services:
  app:
    build: .
    container_name: benchmark
    ports:
      - "3000:3000"
''';

const _ignoreFile = '''
.dart_tool
node_modules
''';

Future<BenchmarkResult> benchmarkInDocker({
  required String dir,
  int rounds = 10,
  required Future<void> Function() run,
}) async {
  File('$dir/docker-compose.yml').writeAsStringSync(_composeFile);
  File('$dir/.dockerignore').writeAsStringSync(_ignoreFile);

  print(' -> Building image');
  await runShell(
    cmd: 'docker compose up --build -d',
    workingDir: dir,
  );

  await Future.delayed(const Duration(seconds: 5));

  print(' -> Running benchmark');
  final resultTimes = <int>[];
  final stopwatch = Stopwatch();
  int failCount = 0;
  bool firstRun = true; // used for warm up, we don't want to count the first run
  do {
    stopwatch.reset();
    stopwatch.start();
    try {
      await run();
    } catch (e) {
      print(' -> Retrying because of error: $e');
      failCount++;
      if (failCount > 5) {
        throw 'Too many errors';
      }
      continue;
    }

    final millis = stopwatch.elapsedMilliseconds;

    if (firstRun) {
      firstRun = false;
      print(' -> Warm up: ${millis} ms');
      continue;
    }
    print(' -> Result #${resultTimes.length + 1}: ${millis} ms');
    resultTimes.add(millis);
  } while (resultTimes.length < rounds);

  print(' -> Stopping container');
  await runShell(
    cmd: 'docker compose down --rmi all',
    workingDir: dir,
  );

  File('$dir/docker-compose.yml').deleteSync();
  File('$dir/.dockerignore').deleteSync();

  // Median
  resultTimes.sort();
  final median = resultTimes[resultTimes.length ~/ 2];
  print(' -> Median: $median ms');

  return BenchmarkResult(
    medianTime: median,
  );
}

/// Runs a shell command.
/// Shows the output in the console.
/// Throws an exception if the command fails.
Future<void> runShell({
  required String cmd,
  required String workingDir,
}) async {
  final cmdParts = cmd.split(' ');
  final result = await Process.start(
    cmdParts.first,
    cmdParts.sublist(1),
    mode: ProcessStartMode.inheritStdio,
    workingDirectory: workingDir,
  );

  if (await result.exitCode != 0) {
    throw 'Failed to run command: ${await result.exitCode}';
  }
}
