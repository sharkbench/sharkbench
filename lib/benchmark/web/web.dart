import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:sharkbench/benchmark/runner.dart';
import 'package:sharkbench/models/benchmark_info.dart';
import 'package:sharkbench/utils/docker_runner.dart';
import 'package:sharkbench/utils/docker_stats.dart';
import 'package:sharkbench/utils/http_load_tester.dart';
import 'package:sharkbench/utils/result_writer.dart';

/// Runs a web framework benchmark.
Future<void> benchmarkWeb({
  required String dir,
  required DockerStatsReader statsReader,
}) async {
  print('Benchmarking "$dir"');

  final benchmarkInfo = BenchmarkInfo.readFromDirectory(dir);
  benchmarkInfo.printInfo();

  final completer = Completer<void>();

  runDockerCompose(
    dir: 'lib/benchmark/web/data',
    composeFile: null,
    onContainerStarted: () => completer.future,
  );

  final frameworkName = benchmarkInfo.framework!;

  final data = _loadData();
  final loadTestConfig = HttpLoadTask(
    requests: List.generate(
          500,
          (i) => 'http://localhost:3000/api/v1/periodic-table/element?symbol=${data.keys.elementAt(i % data.length)}',
        ) +
        List.generate(
          500,
          (i) => 'http://localhost:3000/api/v1/periodic-table/shells?symbol=${data.keys.elementAt(i % data.length)}',
        ),
    expectedResponses: List.generate(500, (i) {
          final elementInfo = data.values.elementAt(i % data.length);
          return jsonEncode({
            'name': elementInfo.name,
            'number': elementInfo.number,
            'group': elementInfo.group,
          });
        }) +
        List.generate(500, (i) {
          final elementInfo = data.values.elementAt(i % data.length);
          return jsonEncode({
            'shells': elementInfo.shells,
          });
        }),
    repeat: 5,
    threads: Platform.numberOfProcessors,
    concurrency: 100,
  );

  for (final languageVersion in benchmarkInfo.languageVersion) {
    for (final frameworkVersion in benchmarkInfo.frameworkVersion!) {
      final requestsPerSeconds = <int>[];
      final result = await runBenchmark(
        dir: dir,
        statsReader: statsReader,
        dockerFileManipulation: benchmarkInfo.languageVersion.length == 1
            ? null
            : DockerFileManipulation(
                initialFromVersion: benchmarkInfo.languageVersion.first,
                newFromVersion: languageVersion,
              ),
        rounds: 5,
        onIteration: () async {
          final result = await runHttpLoadTest(loadTestConfig);
          requestsPerSeconds.add(result.theoreticalRequestsPerSecond);
        },
      );

      requestsPerSeconds.sort();
      final requestsPerSecondMedian = requestsPerSeconds[requestsPerSeconds.length ~/ 2];

      writeResultToFile(
        filePath: 'result/web_result.csv',
        keys: {
          'language': benchmarkInfo.language,
          'mode': benchmarkInfo.mode,
          'version': languageVersion,
          'framework': frameworkName,
          'framework_version': frameworkVersion,
        },
        data: {
          'requests_per_second_median': requestsPerSecondMedian,
          'memory_median': result.memoryMedian,
        },
      );
    }
  }

  print(' -> Done');
}

class _PeriodicTableElement {
  final String name;
  final int number;
  final int group;
  final List<int> shells;

  _PeriodicTableElement({
    required this.name,
    required this.number,
    required this.group,
    required this.shells,
  });
}

Map<String, _PeriodicTableElement> _loadData() {
  final data = jsonDecode(File('lib/benchmark/web/data/data.json').readAsStringSync()) as Map<String, dynamic>;
  final result = <String, _PeriodicTableElement>{};
  for (final entry in data.entries) {
    final element = entry.value as Map<String, dynamic>;
    result[entry.key] = _PeriodicTableElement(
      name: element['name'] as String,
      number: element['number'] as int,
      group: element['group'] as int,
      shells: (element['shells'] as List<dynamic>).cast<int>(),
    );
  }
  return result;
}
