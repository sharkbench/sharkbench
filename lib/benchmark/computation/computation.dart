import 'package:http/http.dart' as http;
import 'package:sharkbench/benchmark/runner.dart';
import 'package:sharkbench/models/benchmark_info.dart';
import 'package:sharkbench/utils/docker_stats.dart';
import 'package:sharkbench/utils/result_writer.dart';

const query = {
  'iterations': '1000000000',
};

const expectedResult = '3.1415926525880504';

/// Runs a computation benchmark.
Future<void> benchmarkComputation({
  required String dir,
  required DockerStatsReader statsReader,
}) async {
  print('Benchmarking "$dir"');

  final benchmarkInfo = BenchmarkInfo.readFromDirectory(dir);
  benchmarkInfo.printInfo();

  for (final languageVersion in benchmarkInfo.languageVersion) {
    final result = await runBenchmark(
      dir: dir,
      statsReader: statsReader,
      dockerFileManipulation: benchmarkInfo.languageVersion.length == 1
          ? null
          : DockerFileManipulation(
              initialFromVersion: benchmarkInfo.languageVersion.first,
              newFromVersion: languageVersion,
            ),
      rounds: 10,
      onIteration: () async {
        final result = await http.get(Uri.http('localhost:3000', '/', query));
        if (result.body != expectedResult) {
          throw 'Unexpected result: "${result.body}"';
        }
      },
    );

    writeResultToFile(
      filePath: 'result/computation_result.csv',
      language: benchmarkInfo.language,
      mode: benchmarkInfo.mode,
      version: languageVersion,
      framework: '',
      frameworkVersion: '',
      includeFramework: false,
      timeMedian: result.timeMedian,
      memoryMedian: result.memoryMedian,
    );
  }

  print(' -> Done');
}
