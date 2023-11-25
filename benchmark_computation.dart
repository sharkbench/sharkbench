import 'dart:io';

import 'package:http/http.dart' as http;

import 'src/models/benchmark_info.dart';
import 'src/utils.dart';

const query = {
  'iterations': '1000000000',
};

const expectedResult = '3.1415926525880504';

/// Runs a computation benchmark.
Future<void> benchmarkComputation(String dir) async {
  print('Benchmarking "$dir"');

  final benchmarkInfo = BenchmarkInfo.readFromDirectory(dir);
  benchmarkInfo.printInfo();

  final result = await benchmarkInDocker(
    dir: dir,
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
    version: benchmarkInfo.languageVersion,
    medianTime: result.medianTime,
  );

  print(' -> Done');
}

/// Writes the computation benchmark result to a file.
void _writeComputationBenchmark({
  required String language,
  required String mode,
  required String version,
  required int medianTime,
}) {
  // Read existing file
  final file = File('result/computation_result.csv');
  final csv = file.existsSync() ? file.readAsLinesSync().skip(1).toList() : <String>[];
  final existingIndex = csv.indexWhere((s) => s.startsWith('$language,$mode,$version'));
  if (existingIndex != -1) {
    csv.removeAt(existingIndex);
  }

  // Write new file
  csv.add('$language,$mode,$version,$medianTime');
  csv.sort();

  file.writeAsStringSync('''
language,mode,version,median_time
${csv.join('\n')}
''');
}
