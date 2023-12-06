import 'package:sharkbench/utils/docker_file_utils.dart';
import 'package:test/test.dart';

void main() {
  group('updateDockerFileWithVersion', () {
    test('Should replace single line', () {
      const dockerFile = '''
FROM node:14.17.0-alpine3.13
RUN apk add --no-cache bash
''';
      const expectedDockerFile = '''
FROM node:14.17.1-alpine3.13
RUN apk add --no-cache bash
''';
      final result = updateDockerFileWithVersion(
        dockerFileContent: dockerFile,
        currentVersion: '14.17.0',
        newVersion: '14.17.1',
      );
      expect(result, expectedDockerFile);
    });

    test('Should replace multiple lines', () {
      const dockerFile = '''
FROM node:14.17.0-alpine3.13 as build1
RUN apk add --no-cache bash
FROM node:14.17.0-alpine3.13 as build2
RUN apk add --no-cache bash
''';
      const expectedDockerFile = '''
FROM node:14.17.1-alpine3.13 as build1
RUN apk add --no-cache bash
FROM node:14.17.1-alpine3.13 as build2
RUN apk add --no-cache bash
''';
      final result = updateDockerFileWithVersion(
        dockerFileContent: dockerFile,
        currentVersion: '14.17.0',
        newVersion: '14.17.1',
      );
      expect(result, expectedDockerFile);
    });
  });
}
