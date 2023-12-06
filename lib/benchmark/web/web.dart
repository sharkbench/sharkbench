import 'package:sharkbench/models/benchmark_info.dart';
import 'package:sharkbench/utils/docker_stats.dart';

/// Runs a web framework benchmark.
Future<void> benchmarkWeb({
  required String dir,
  required DockerStatsReader statsHandler,
}) async {
  print('Benchmarking "$dir"');

  final benchmarkInfo = BenchmarkInfo.readFromDirectory(dir);
  benchmarkInfo.printInfo();

  // TODO

  print(' -> Done');
}
