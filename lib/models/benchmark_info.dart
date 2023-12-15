import 'dart:io';

import 'package:yaml/yaml.dart';

class BenchmarkInfo {
  final String language;
  final String mode;
  final List<String> languageVersion;
  final String? framework;
  final List<String>? frameworkVersion;

  BenchmarkInfo({
    required this.language,
    required this.mode,
    required this.languageVersion,
    required this.framework,
    required this.frameworkVersion,
  });

  void printInfo() {
    print(' - Language: $language');
    print(' - Mode: $mode');
    print(' - Language version: ${languageVersion.length == 1 ? languageVersion.first : languageVersion}');
    if (framework != null) {
      print(' - Framework: $framework');
    }
    if (frameworkVersion != null) {
      print(' - Framework version: ${frameworkVersion!.length == 1 ? frameworkVersion!.first : frameworkVersion}');
    }
    print('');
  }

  /// Reads the benchmark info from a directory.
  /// Expects a _benchmark.yaml file in the directory.
  static BenchmarkInfo readFromDirectory(String dir) {
    final benchmarkInfo = loadYaml(
      File('$dir/_benchmark.yaml').readAsStringSync(),
    );

    return BenchmarkInfo(
      language: benchmarkInfo['language'].toString(),
      mode: benchmarkInfo['mode'].toString(),
      languageVersion: (benchmarkInfo['version'] as List).cast<String>(),
      framework: benchmarkInfo['framework']?.toString(),
      frameworkVersion: (benchmarkInfo['frameworkVersion'] as List?)?.cast<String>(),
    );
  }
}
