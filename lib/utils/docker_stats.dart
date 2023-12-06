import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:sharkbench/utils/shell.dart';

// The start of a stats line is marked by the following bytes.
// We ignore them because they are not valid JSON.
final _statsPrefix = String.fromCharCodes([27, 91, 50, 74, 27, 91, 72]);

class DockerStatsReader {
  final String containerName;
  bool _isTracking = false;
  Process? _process;
  final _ramUsage = <int>[];

  DockerStatsReader({
    required this.containerName,
  });

  Future<void> run() async {
    final completer = Completer<void>();
    runShell(
      cmd: ['docker', 'stats', '--format', 'json'],
      onProcess: (p) => _process = p,
      onOutput: (output) {
        if (!completer.isCompleted) {
          completer.complete();
        }
        if (!_isTracking) {
          return;
        }

        output.split('\n');
        for (final line in output.split('\n')) {
          final trimmed = line.trim().replaceFirst(_statsPrefix, '');
          if (trimmed.isEmpty) {
            continue;
          }

          final json = jsonDecode(trimmed);
          final name = json['Name'];
          if (name != containerName) {
            continue;
          }
          final memUsage = getBytesOfRam(json['MemUsage']);
          _ramUsage.add(memUsage);
        }
      },
    );

    return completer.future;
  }

  void stop() {
    _isTracking = false;
  }

  void start() {
    _ramUsage.clear();
    _isTracking = true;
  }

  void dispose() {
    _process?.kill();
  }

  int get medianMemory {
    _ramUsage.sort();
    return _ramUsage[_ramUsage.length ~/ 2];
  }

  static RegExp _memUsageRegExp = RegExp(r'(\d*\.?\d+)(\w+)');

  static int getBytesOfRam(String memUsage) {
    final memUsageParsed = memUsage.split(' / ').first;
    final memUsageMatch = _memUsageRegExp.firstMatch(memUsageParsed)!;
    final memUsageParsedValue = double.parse(memUsageMatch.group(1)!);
    final memUsageParsedUnit = memUsageMatch.group(2)!;
    switch (memUsageParsedUnit) {
      case 'KiB':
        return (memUsageParsedValue * 1024).round();
      case 'MiB':
        return (memUsageParsedValue * 1024 * 1024).round();
      case 'GiB':
        return (memUsageParsedValue * 1024 * 1024 * 1024).round();
      default:
        throw 'Unknown unit: $memUsageParsedUnit from $memUsageParsed';
    }
  }
}
