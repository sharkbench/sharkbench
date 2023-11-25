import 'dart:io';

import 'package:yaml/yaml.dart';

class BenchmarkInfo {
  final String language;
  final String mode;
  final List<String> languageVersion;
  final String? framework;
  final String? frameworkVersion;

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
      print(' - Framework version: $frameworkVersion');
    }
    print('');
  }

  /// Reads the benchmark info from a directory.
  /// Expects a _benchmark.yaml file in the directory.
  static BenchmarkInfo readFromDirectory(String dir) {
    final benchmarkInfo = loadYaml(
      File('$dir/_benchmark.yaml').readAsStringSync(),
    );
    final benchmarkLanguage = benchmarkInfo['language'].toString();
    final benchmarkMode = benchmarkInfo['mode'].toString();
    final benchmarkVersion = benchmarkInfo['version'];
    final benchmarkFramework = benchmarkInfo['framework']?.toString();
    final benchmarkFrameworkVersion = benchmarkInfo['frameworkVersion'];

    return BenchmarkInfo(
      language: benchmarkLanguage,
      mode: benchmarkMode,
      languageVersion: benchmarkVersion is List
          ? benchmarkVersion.cast<String>()
          : [benchmarkVersion as String],
      framework: benchmarkFramework,
      frameworkVersion: benchmarkFrameworkVersion,
    );
  }
}
