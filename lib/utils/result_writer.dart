import 'dart:io';

/// Writes the benchmark result to a file.
void writeResultToFile({
  required String filePath,
  required String language,
  required String mode,
  required String version,
  required String framework,
  required String frameworkVersion,
  required bool includeFramework,
  required int timeMedian,
  required int memoryMedian,
}) {
  // Read existing file
  final file = File(filePath);
  final csv = file.existsSync() ? file.readAsLinesSync().skip(1).toList() : <String>[];
  final existingIndex = csv.indexWhere((s) => s.startsWith('$language,$mode,$version'));
  if (existingIndex != -1) {
    csv.removeAt(existingIndex);
  }

  // Write new file
  csv.add('$language,$mode,$version,$timeMedian,$memoryMedian');
  csv.sort();

  final buffer = StringBuffer();
  if (includeFramework) {
    buffer.write('language,mode,version,framework,frameworkVersion,time_median,memory_median');
  } else {
    buffer.write('language,mode,version,time_median,memory_median');
  }
  buffer.write('\n');
  for (final line in csv) {
    buffer.write(line);
    buffer.write('\n');
  }
  file.writeAsStringSync(buffer.toString());
}
