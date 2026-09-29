import 'dart:convert';
import 'dart:io';

class ElementService {
  final elementUrl = Uri.http('127.0.0.1:5002', '/element.json');
  final shellsUrl = Uri.http('127.0.0.1:5002', '/shells.json');
  final httpClient = HttpClient();

  Future<Map<String, dynamic>> getElement(String symbol) async {
    final tmpReq = await httpClient.getUrl(elementUrl);
    final tmpRes = await tmpReq.close();
    final json = jsonDecode(await tmpRes.transform(utf8.decoder).join());
    return json[symbol];
  }

  Future<Map<String, dynamic>> getShells(String symbol) async {
    final tmpReq = await httpClient.getUrl(shellsUrl);
    final tmpRes = await tmpReq.close();
    final json = Map<String, dynamic>.from(
        jsonDecode(await tmpRes.transform(utf8.decoder).join()));
    return {
      'shells': json[symbol],
    };
  }
}
