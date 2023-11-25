import 'benchmark_computation.dart';

enum BenchmarkType {
  computation,
  web,
}

void main() {
  benchmarkComputation('computation/java/temurin-21');
}
