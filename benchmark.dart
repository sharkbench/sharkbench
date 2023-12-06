import 'package:sharkbench/benchmark/computation/computation.dart';
import 'package:sharkbench/utils/docker_stats.dart';

enum BenchmarkType {
  computation,
  web,
}

void main() async {
  final statsReader = DockerStatsReader(
    containerName: 'benchmark',
  );

  await statsReader.run();

  await benchmarkComputation(
    dir: 'computation/dart/aot-2.14',
    statsReader: statsReader,
  );

  print('Finishing...');

  statsReader.dispose();
}
