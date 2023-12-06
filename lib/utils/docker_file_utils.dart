/// Replaces every FROM line in a Dockerfile with a new version.
String updateDockerFileWithVersion({
  required String dockerFileContent,
  required String currentVersion,
  required String newVersion,
}) {
  return dockerFileContent.split('\n').map((line) {
    if (line.startsWith('FROM ')) {
      return line.replaceFirst(currentVersion, newVersion);
    } else {
      return line;
    }
  }).join('\n');
}
