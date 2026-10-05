import 'dart:io';

Future<void> main() async {
  while (true) {
    final line = stdin.readLineSync();
    if (line == null) {
      break;
    }
    final trimmed = line.trim();
    if (trimmed.isEmpty) {
      continue;
    }
    final i = int.parse(trimmed);
    final result = pi(i);
    stdout.writeln('${result[0]};${result[1]};${result[2]}');
    await stdout.flush();
  }
}

List<double> pi(int iterations) {
  double pi = 0.0;
  double denominator = 1.0;
  double sum = 0.0;
  double customNumber = 0.0;
  for (var x = 0; x < iterations; x++) {
    if (x % 2 == 0) {
      pi = pi + (1 / denominator);
    } else {
      pi = pi - (1 / denominator);
    }
    denominator = denominator + 2;

    // custom
    sum += pi;
    switch (x % 3) {
      case 0:
        customNumber += pi;
        break;
      case 1:
        customNumber -= pi;
        break;
      case 2:
        customNumber /= 2;
        break;
    }
  }
  pi = pi * 4;
  return [pi, sum, customNumber];
}
