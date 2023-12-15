import 'dart:isolate';

import 'package:http/http.dart' as http;
import 'package:pool/pool.dart';

class HttpLoadTask {
  /// List of URLs to request.
  /// e.g. [‘/api/v1/periodic-table?element=He’, ‘/api/v1/periodic-table?element=H’]
  final List<String> requests;

  final List<Uri> requestsUri;

  /// List of expected responses.
  /// e.g. ['{“name”:”Helium”,”number”:2,”group”:18}’, ‘{“name”:”Hydrogen”,”number”:1,”group”:1}’]
  final List<String> expectedResponses;

  /// Number of times to repeat the test.
  /// Total number of requests = requests.length * repeat
  final int repeat;

  /// Number of threads
  final int threads;

  /// Number of total concurrent requests
  /// This load is distributed evenly across all threads
  final int concurrency;

  HttpLoadTask({
    required this.requests,
    required this.expectedResponses,
    required this.repeat,
    required this.threads,
    required this.concurrency,
  }) : requestsUri = requests.map((e) => Uri.parse(e)).toList();
}

class HttpLoadResult {
  final int successCount;
  final int failCount;
  final int totalTimeInMillis;

  /// totalTime / successCount
  final int theoreticalRequestsPerSecond;

  HttpLoadResult({
    required this.successCount,
    required this.failCount,
    required this.totalTimeInMillis,
    required this.theoreticalRequestsPerSecond,
  });
}

class HttpThreadResult {
  final int successCount;
  final int failCount;
  final int totalTimeInMillis;

  HttpThreadResult({
    required this.successCount,
    required this.failCount,
    required this.totalTimeInMillis,
  });
}

Future<HttpLoadResult> runHttpLoadTest(HttpLoadTask task) async {
  final requestsPerThread = task.requests.length ~/ task.threads;
  final concurrencyPerThread = task.concurrency ~/ task.threads;
  print('Running HTTP load test with $requestsPerThread requests per thread repeated ${task.repeat} times');
  final List<HttpThreadResult> results = await Future.wait(
    List.generate(
      task.threads,
      (_) => _runThread(
        task: task,
        requestsPerThread: requestsPerThread,
        concurrencyPerThread: concurrencyPerThread,
      ),
    ),
  );

  int successCount = 0;
  int failCount = 0;
  int totalTimeInMillis = 0;
  for (final result in results) {
    successCount += result.successCount;
    failCount += result.failCount;
    totalTimeInMillis += result.totalTimeInMillis;
  }

  print('HTTP load test: [total: ${successCount + failCount}, success: $successCount, fail: $failCount, total time: $totalTimeInMillis ms]');

  return HttpLoadResult(
    successCount: successCount,
    failCount: failCount,
    totalTimeInMillis: totalTimeInMillis,
    theoreticalRequestsPerSecond: successCount ~/ (totalTimeInMillis / 1000),
  );
}

Future<HttpThreadResult> _runThread({
  required HttpLoadTask task,
  required int requestsPerThread,
  required int concurrencyPerThread,
}) async {
  return Isolate.run(() async {
    final pool = Pool(concurrencyPerThread, timeout: Duration(seconds: 30));
    final results = List.filled(requestsPerThread * task.repeat, '');
    int successCount = 0;
    int failCount = 0;
    final outerStopwatch = Stopwatch()..start();

    List<Future<void>> allRequests = [];
    for (int i = 0; i < task.repeat; i++) {
      for (int j = 0; j < requestsPerThread; j++) {
        allRequests.add(pool.withResource(() async {
          final response = await http.get(task.requestsUri[j]);
          results[i * requestsPerThread + j] = response.body;
        }));
      }
    }

    // Wait for all requests to complete
    await Future.wait(allRequests);

    final totalTimeInMillis = outerStopwatch.elapsedMilliseconds;

    // Verify results
    for (int i = 0; i < results.length; i++) {
      if (results[i] == task.expectedResponses[i % requestsPerThread]) {
        successCount++;
      } else {
        failCount++;
        print(' -> Expected: ${task.expectedResponses[i % task.requests.length]}, but got: ${results[i]}');
      }
    }

    print('SUCCESS: $successCount, FAIL: $failCount, TOTAL TIME: $totalTimeInMillis ms');

    return HttpThreadResult(
      successCount: successCount,
      failCount: failCount,
      totalTimeInMillis: totalTimeInMillis,
    );
  });
}
