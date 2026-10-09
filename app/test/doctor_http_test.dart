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

  test(
    'askDoctor waits past the usual answer timeout (FRONTEND.md 6)',
    () async {
      final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
      server.listen((req) async {
        await req.drain<void>();
        await Future<void>.delayed(const Duration(milliseconds: 600));
        req.response
          ..statusCode = 200
          ..headers.contentType = ContentType.json
          ..write(
            jsonEncode({
              'likely': 'Rust',
              'confidence': 'unsure',
              'refer_to_officer': false,
              'ku': '',
              'en': '',
            }),
          );
        await req.response.close();
      });
      // Usual calls give up after 200 ms here; the Doctor takes 600 ms.
      final api = HttpApi(
        'http://127.0.0.1:${server.port}/v1',
        timeout: const Duration(milliseconds: 200),
      )..useToken('tok');
      await expectLater(
        api.getFarms(),
        throwsA(isA<ApiException>().having((e) => e.code, 'code', 'offline')),
      );
      final a = await api.askDoctor('5', const DoctorQuestion(text: 'Spots?'));
      await server.close(force: true);
      expect(a.likely, 'Rust');
    },
  );

  test('a photo is sent as the type its bytes are, not its file name', () {
    expect(photoMime([0xFF, 0xD8, 0xFF, 0xE0, 0x00]), 'image/jpeg');
    expect(
      photoMime([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00]),
      'image/png',
    );
    expect(photoMime([0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70]), isNull);
    expect(photoMime([0xFF]), isNull);
  });
}
