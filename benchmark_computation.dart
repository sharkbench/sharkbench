import 'dart:io';

import 'package:http/http.dart' as http;

import 'src/models/benchmark_info.dart';
import 'src/utils.dart';
import 'src/utils/docker_stats.dart';

const query = {
  'iterations': '1000000000',
};

const expectedResult = '3.1415926525880504';

/// Runs a computation benchmark.
Future<void> benchmarkComputation({
  required String dir,
  required DockerStatsHandler statsHandler,
}) async {
  print('Benchmarking "$dir"');

  final benchmarkInfo = BenchmarkInfo.readFromDirectory(dir);
  benchmarkInfo.printInfo();

  for (final languageVersion in benchmarkInfo.languageVersion) {
    final String? originalDockerFile;
    if (benchmarkInfo.languageVersion.length > 1) {
      print(' - Testing for language version: $languageVersion');

      originalDockerFile = File('$dir/Dockerfile').readAsStringSync();
      final newDockerFile = updateDockerFileWithVersion(
        dockerFileContent: originalDockerFile,
        currentVersion: benchmarkInfo.languageVersion.first,
        newVersion: languageVersion,
      );
      File('$dir/Dockerfile').writeAsStringSync(newDockerFile);
    } else {
      originalDockerFile = null;
    }

    final result = await benchmarkInDocker(
      dir: dir,
      statsHandler: statsHandler,
      rounds: 10,
      run: () async {
        final result = await http.get(Uri.http('localhost:3000', '/', query));
        if (result.body != expectedResult) {
          throw 'Unexpected result: "${result.body}"';
        }
      },
    );

    _writeComputationBenchmark(
      language: benchmarkInfo.language,
      mode: benchmarkInfo.mode,
      version: languageVersion,
      timeMedian: result.timeMedian,
      memoryMedian: result.memoryMedian,
    );

    if (originalDockerFile != null) {
      // Reset Dockerfile
      File('$dir/Dockerfile').writeAsStringSync(originalDockerFile);
    }
  }

  print(' -> Done');
}

/// Writes the computation benchmark result to a file.
void _writeComputationBenchmark(
    {required String language, required String mode, required String version, required int timeMedian, required int memoryMedian}) {
  // Read existing file
  final file = File('result/computation_result.csv');
  final csv = file.existsSync() ? file.readAsLinesSync().skip(1).toList() : <String>[];
  final existingIndex = csv.indexWhere((s) => s.startsWith('$language,$mode,$version'));
  if (existingIndex != -1) {
    csv.removeAt(existingIndex);
  }

  // Write new file
  csv.add('$language,$mode,$version,$timeMedian,$memoryMedian');
  csv.sort();

  file.writeAsStringSync('''
language,mode,version,time_median,memory_median
${csv.join('\n')}
''');
}
