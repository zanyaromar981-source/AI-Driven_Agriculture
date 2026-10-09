import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/api/api.dart';
import 'package:jutyar/api/http_api.dart';

// Its own file: files with widget tests replace HttpClient with a mock.
void main() {
  test(
    'askDoctor sends one multipart request with the words, square and photos',
    () async {
      final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
      late String path, type, auth;
      late List<int> body;
      server.listen((req) async {
        path = req.uri.path;
        type = req.headers.contentType.toString();
        auth = req.headers.value('authorization') ?? '';
        body = [for (final chunk in await req.toList()) ...chunk];
        req.response
          ..statusCode = 200
          ..headers.contentType = ContentType.json
          ..write(
            jsonEncode({
              'likely': 'Too dry to sow this week',
              'confidence': 'likely',
              'actions_this_week': ['a', 'b', 'c'],
            }),
          );
        await req.response.close();
      });
      final api = HttpApi('http://127.0.0.1:${server.port}/v1')
        ..useToken('tok');
      final a = await api.askDoctor(
        '5',
        const DoctorQuestion(
          text: '  Is it time to sow?  ',
          photos: [
            DoctorPhoto(bytes: [0xFF, 0xD8, 0xFF, 0x00, 0xD9]),
          ],
          cellE: 46415,
          cellN: 398748,
          lang: 'en',
        ),
      );
      await server.close(force: true);

      expect(path, '/v1/farms/5/ask');
      expect(auth, 'Bearer tok');
      expect(type, startsWith('multipart/form-data; boundary='));
      final text = latin1.decode(body);
      expect(text, contains('name="question"\r\n\r\nIs it time to sow?\r\n'));
      expect(text, contains('name="lang"\r\n\r\nen\r\n'));
      expect(text, contains('name="cell"\r\n\r\n{"e":46415,"n":398748}\r\n'));
      expect(
        text,
        contains(
          'name="photos"; filename="photo_1.jpg"\r\nContent-Type: image/jpeg',
        ),
      );
      const jpeg = [0xFF, 0xD8, 0xFF, 0x00, 0xD9];
      final at = [
        for (var i = 0; i + jpeg.length <= body.length; i++)
          if (List.generate(jpeg.length, (k) => body[i + k]).join() ==
              jpeg.join())
            i,
      ];
      expect(at, hasLength(1), reason: 'the photo bytes go through unchanged');
      expect(a.likely, 'Too dry to sow this week');
      expect(a.actions, hasLength(3));
    },
  );
}
