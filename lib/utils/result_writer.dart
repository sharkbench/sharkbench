import 'dart:io';

/// Writes the benchmark result to a file.
void writeResultToFile({
  required String filePath,
  required Map<String, String> keys,
  required Map<String, Object> data,
}) {
  print(' -> Writing result:');
  for (final entry in keys.entries) {
    print('    - ${entry.key}: ${entry.value}');
  }
  for (final entry in data.entries) {
    print('    - ${entry.key}: ${entry.value}');
  }

  // Read existing file
  final file = File(filePath);
  final csv = file.existsSync() ? file.readAsLinesSync().skip(1).toList() : <String>[];
  final existingIndex = csv.indexWhere((s) => s.startsWith(keys.values.join(',')));
  if (existingIndex != -1) {
    csv.removeAt(existingIndex);
  }

  // Write new file
  csv.add(keys.values.join(',') + ',' + data.values.join(','));
  csv.sort();

  final buffer = StringBuffer();
  buffer.write(keys.keys.join(','));
  buffer.write(',');
  buffer.write(data.keys.join(','));
  buffer.write('\n');
  for (final line in csv) {
    buffer.write(line);
    buffer.write('\n');
  }
  file.writeAsStringSync(buffer.toString());
}
