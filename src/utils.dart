import 'dart:convert';
import 'dart:io';

import 'models/benchmark_result.dart';
import 'utils/docker_stats.dart';

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
  required DockerStatsHandler statsHandler,
  int rounds = 10,
  required Future<void> Function() run,
}) async {
  File('$dir/docker-compose.yml').writeAsStringSync(_composeFile);
  File('$dir/.dockerignore').writeAsStringSync(_ignoreFile);

  print(' -> Building image');
  await runShell(
    cmd: ['docker', 'compose', 'up', '--build', '-d'],
    workingDir: dir,
  );

  await Future.delayed(const Duration(seconds: 5));

  print(' -> Running benchmark');
  final executionTimes = <int>[];
  final memoryUsages = <int>[];
  final stopwatch = Stopwatch();
  int failCount = 0;
  bool firstRun = true; // used for warm up, we don't want to count the first run
  do {
    stopwatch.reset();
    stopwatch.start();
    statsHandler.start();
    try {
      await run();
    } catch (e) {
      print(' -> Retrying because of error: $e');
      failCount++;
      if (failCount > 10) {
        throw 'Too many errors';
      }
      await Future.delayed(const Duration(seconds: 1));
      continue;
    }
    statsHandler.stop();

    final millis = stopwatch.elapsedMilliseconds;
    final memory = statsHandler.medianMemory;

    if (firstRun) {
      firstRun = false;
      print(' -> Warm up: ${millis} ms with ${memory.bytesToString()} RAM');
      continue;
    }
    print(' -> Result #${executionTimes.length + 1}: ${millis} ms with ${memory.bytesToString()} RAM');
    executionTimes.add(millis);
    memoryUsages.add(memory);
  } while (executionTimes.length < rounds);

  print(' -> Stopping container');
  await runShell(
    cmd: ['docker', 'compose', 'down', '--rmi', 'all'],
    workingDir: dir,
  );

  File('$dir/docker-compose.yml').deleteSync();
  File('$dir/.dockerignore').deleteSync();

  // Calculate median time
  executionTimes.sort();
  final timeMedian = executionTimes[executionTimes.length ~/ 2];
  print(' -> Median: $timeMedian ms');

  // Calculate median memory
  memoryUsages.sort();
  final memoryMedian = memoryUsages[memoryUsages.length ~/ 2];
  print(' -> Median: ${memoryMedian.bytesToString()}');

  return BenchmarkResult(
    timeMedian: timeMedian,
    memoryMedian: memoryMedian,
  );
}

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

/// Replaces every FROM line in a Dockerfile with a new version.
String updateDockerFileWithVersion({
  required String dockerFileContent,
  required String currentVersion,
  required String newVersion,
}) {
  return dockerFileContent.split('\n').map((line) {
    if (line.startsWith('FROM ')) {
      return line.replaceFirst(currentVersion, newVersion);
    } else {
      return line;
    }
  }).join('\n');
}

extension SizeExt on int {
  String bytesToString() {
    final kb = this / 1024;
    if (kb < 1024) {
      return '${kb.toStringAsFixed(2)} KB';
    }
    final mb = kb / 1024;
    if (mb < 1024) {
      return '${mb.toStringAsFixed(2)} MB';
    }
    final gb = mb / 1024;
    return '${gb.toStringAsFixed(2)} GB';
  }
}