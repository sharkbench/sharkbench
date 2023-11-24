import 'benchmark_computation.dart';

enum BenchmarkType {
  computation,
  web,
}

void main() {
  benchmarkComputation('computation/dart/dart-aot-3-2');
}
