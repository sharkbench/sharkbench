import 'dart:io';

import 'package:sharkbench/models/benchmark_result.dart';
import 'package:sharkbench/utils/docker_file_utils.dart';
import 'package:sharkbench/utils/docker_runner.dart';
import 'package:sharkbench/utils/docker_stats.dart';

const _composeFile = '''
version: "3"
services:
  app:
    build: .
    container_name: benchmark
    ports:
      - "3000:3000"
''';

class DockerFileManipulation {
  final String initialFromVersion;
  final String newFromVersion;

  DockerFileManipulation({
    required this.initialFromVersion,
    required this.newFromVersion,
  });
}

/// Runs a benchmark in a docker container.
Future<BenchmarkResult> runBenchmark({
  required String dir,
  required DockerStatsReader statsReader,
  required DockerFileManipulation? dockerFileManipulation,
  int rounds = 10,
  required Future<void> Function() onIteration,
}) async {
  final String? originalDockerFile;
  if (dockerFileManipulation != null) {
    originalDockerFile = File('$dir/Dockerfile').readAsStringSync();
    final newDockerFile = updateDockerFileWithVersion(
      dockerFileContent: originalDockerFile,
      currentVersion: dockerFileManipulation.initialFromVersion,
      newVersion: dockerFileManipulation.newFromVersion,
    );
    File('$dir/Dockerfile').writeAsStringSync(newDockerFile);
  } else {
    originalDockerFile = null;
  }

  final executionTimes = <int>[];
  final memoryUsages = <int>[];
  await runDockerCompose(
    dir: dir,
    composeFile: _composeFile,
    onContainerStarted: () async {
      print(' -> Running benchmark');
      final stopwatch = Stopwatch();
      int failCount = 0;
      bool firstRun = true; // used for warm up, we don't want to count the first run
      do {
        stopwatch.reset();
        stopwatch.start();
        statsReader.start();
        try {
          await onIteration();
        } catch (e) {
          print(' -> Error: $e');
          failCount++;
          if (failCount > 10) {
            throw 'Too many errors';
          }
          await Future.delayed(const Duration(seconds: 1));
          print(' -> Retrying because of error.');
          continue;
        }
        statsReader.stop();

        final millis = stopwatch.elapsedMilliseconds;
        final memory = statsReader.medianMemory;

        if (firstRun) {
          firstRun = false;
          print(' -> Warm up: ${millis} ms with ${memory.bytesToString()} RAM');
          continue;
        }
        print(' -> Result #${executionTimes.length + 1}: ${millis} ms with ${memory.bytesToString()} RAM');
        executionTimes.add(millis);
        memoryUsages.add(memory);
        await Future.delayed(const Duration(seconds: 1));
      } while (executionTimes.length < rounds);
    },
  );

  if (originalDockerFile != null) {
    // Reset Dockerfile
    File('$dir/Dockerfile').writeAsStringSync(originalDockerFile);
  }

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
