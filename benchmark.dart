import 'benchmark_computation.dart';
import 'src/utils/docker_stats.dart';

enum BenchmarkType {
  computation,
  web,
}

void main() async {
  final statsHandler = DockerStatsHandler(
    containerName: 'benchmark',
  );

  await statsHandler.run();

  await benchmarkComputation(
    dir: 'computation/dart/aot-2.14',
    statsHandler: statsHandler,
  );

  print('Finishing...');

  statsHandler.dispose();
}
