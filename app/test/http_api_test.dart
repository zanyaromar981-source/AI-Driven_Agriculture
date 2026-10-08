import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/api/api.dart';
import 'package:jutyar/api/http_api.dart';

/// A tiny server that answers like BACKEND.md says the real one will.
class _Server {
  late HttpServer http;
  final seen =
      <({String method, String path, HttpHeaders headers, String body})>[];

  String get url => 'http://127.0.0.1:${http.port}/v1';

  static const farm = {
    'id': 'f_1',
    'name': 'Test farm',
    'area_dunam': 1.6,
    'crops': [
      {'crop': 'wheat', 'dunam': 1.6},
    ],
    'status': 'none',
    'last_picture': null,
    'centroid': {'lat': 36.03, 'lon': 44.6},
    'outline': [
      {'lat': 36.03, 'lon': 44.6},
      {'lat': 36.031, 'lon': 44.6},
      {'lat': 36.031, 'lon': 44.601},
    ],
    'cells': [
      {
        'e': 46415,
        'n': 398748,
        'crop': 'wheat',
        'greenness_pct': null,
        'level': 'none',
        'inside_pct': 100,
      },
    ],
    'picture_date': null,
  };

  Future<void> start() async {
    http = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
    http.listen((req) async {
      final body = await utf8.decoder.bind(req).join();
      final path = req.uri.path.replaceFirst('/v1', '');
      seen.add((
        method: req.method,
        path: path,
        headers: req.headers,
        body: body,
      ));
      final authed = req.headers.value('authorization') == 'Bearer tok123';
      void send(int status, Object json) {
        req.response
          ..statusCode = status
          ..headers.contentType = ContentType.json
          ..write(jsonEncode(json));
        req.response.close();
      }

      switch ((req.method, path)) {
        case ('POST', '/auth/otp/send'):
          send(200, {'sent': true, 'retry_after_s': 59});
        case ('POST', '/auth/otp/verify'):
          final code = (jsonDecode(body) as Map)['code'];
          code == '000000'
              ? send(401, {'error': 'bad_code'})
              : send(200, {'token': 'tok123', 'farms_count': 1});
        case _ when !authed:
          send(401, {'error': 'unauthorized'});
        case ('GET', '/farms'):
          send(200, {
            'farms': [farm],
          });
        case ('POST', '/farms'):
          send(201, {
            'farm': farm,
            'dropped_cells': [
              {'e': 1, 'n': 1, 'crop': 'wheat'},
            ],
          });
        case ('GET', '/farms/f_1'):
          send(200, {'farm': farm});
        case ('GET', '/farms/f_1/status'):
          send(200, {
            'picture_date': '2026-10-05',
            'greenness_pct_of_normal': 96,
            'weak_where': 'north-east',
            'cells': [
              {
                'e': 46415,
                'n': 398748,
                'greenness_pct': 78,
                'level': 'watch',
                'since': '2026-09-25',
              },
            ],
            'crops': [
              {
                'crop': 'wheat',
                'dunam': 1.6,
                'greenness_pct_of_normal': 96,
                'level': 'normal',
              },
            ],
            'next_picture_expected': '2026-10-10',
          });
        case ('GET', '/farms/f_1/plan'):
          send(200, {
            'from': '2026-10-08',
            'days': 2,
            'rain_mm': [0, 14.2],
            'tmin': [9, -3],
            'tmax': [24, 20],
            'alerts': [
              {
                'type': 'frost',
                'day': '2026-10-09',
                'value': -3,
                'level': 'alarm',
                'ku': 'k',
                'en': 'Frost',
              },
            ],
            'decisions': [
              {'code': 'frost_check', 'ku': 'k', 'en': 'Check the heads'},
            ],
            'source': 'Open-Meteo',
            'issued': '2026-10-08T06:00:00Z',
          });
        case ('GET', '/farms/slow'):
          await Future<void>.delayed(const Duration(seconds: 3));
          send(200, {'farm': farm});
        case ('GET', '/farms/html'):
          req.response
            ..statusCode = 502
            ..headers.contentType = ContentType.html
            ..write('<html>Bad gateway</html>');
          await req.response.close();
        case ('GET', '/farms/busy'):
          send(429, {'error': 'rate_limited', 'retry_after_s': 30});
        default:
          send(404, {'error': 'not_found'});
      }
    });
  }
}

void main() {
  late _Server server;
  setUpAll(() => HttpOverrides.global = null);
  setUp(() async {
    server = _Server();
    await server.start();
  });
  tearDown(() => server.http.close(force: true));

  Future<HttpApi> signedIn({void Function()? onUnauthorized}) async {
    final api = HttpApi(server.url, onUnauthorized: onUnauthorized);
    await api.verifyOtp(phone: '+9647501234567', code: '123456');
    return api;
  }

  test('sign in, then every call carries the token', () async {
    final api = HttpApi(server.url);
    final sent = await api.sendOtp(phone: '+9647501234567', lang: 'ku');
    expect(sent.retryAfterS, 59);
    final ok = await api.verifyOtp(phone: '+9647501234567', code: '123456');
    expect(ok.token, 'tok123');
    final farms = await api.getFarms();
    expect(farms.single.name, 'Test farm');
    expect(server.seen.last.headers.value('authorization'), 'Bearer tok123');
    expect(server.seen.first.path, '/auth/otp/send'); // base path /v1 kept
    expect(jsonDecode(server.seen.first.body), {
      'phone': '+9647501234567',
      'lang': 'ku',
    });
  });

  test('wrong code: 401 bad_code, and no sign-out', () async {
    var signedOut = false;
    final api = HttpApi(server.url, onUnauthorized: () => signedOut = true);
    await expectLater(
      api.verifyOtp(phone: '+9647501234567', code: '000000'),
      throwsA(
        isA<ApiException>()
            .having((e) => e.code, 'code', 'bad_code')
            .having((e) => e.status, 'status', 401),
      ),
    );
    expect(signedOut, isFalse);
  });

  test('a refused saved token signs the farmer out', () async {
    var signedOut = false;
    final api = HttpApi(server.url, onUnauthorized: () => signedOut = true)
      ..useToken('old-token');
    await expectLater(
      api.getFarms(),
      throwsA(isA<ApiException>().having((e) => e.status, 'status', 401)),
    );
    expect(signedOut, isTrue);
  });

  test('new farm: body and Idempotency-Key are sent, answer is read', () async {
    final api = await signedIn();
    final t = DateTime.utc(2026, 10, 8, 18, 30);
    final res = await api.createFarm(
      NewFarmRequest(
        name: 'Test farm',
        points: [GeoPoint(lat: 36.03, lon: 44.6, accM: 5, t: t)],
        cells: const [CellCrop(e: 46415, n: 398748, crop: 'wheat')],
        createdOfflineAt: t,
      ),
      idempotencyKey: 'key-1',
    );
    final req = server.seen.last;
    expect(req.method, 'POST');
    expect(req.headers.value('idempotency-key'), 'key-1');
    expect(req.headers.contentType?.mimeType, 'application/json');
    final body = jsonDecode(req.body) as Map;
    expect(body['created_offline_at'], '2026-10-08T18:30:00Z');
    expect((body['cells'] as List).single, {
      'e': 46415,
      'n': 398748,
      'crop': 'wheat',
    });
    expect(res.farm.summary.id, 'f_1');
    expect(res.droppedCells, 1);
  });

  test('farm, status and plan are read into the app models', () async {
    final api = await signedIn();
    final farm = await api.getFarm('f_1');
    expect(farm.outline.length, 3);
    final st = await api.getFarmStatus('f_1');
    expect(st.cells.single.level, FarmStatus.watch);
    expect(st.crops.single.greennessPctOfNormal, 96);
    final plan = await api.getPlan('f_1');
    expect(plan.rainMm, [0, 14.2]);
    expect(plan.alerts.single.level, FarmStatus.alarm);
    expect(plan.decisions.single.code, 'frost_check');
  });

  test('server errors keep their status and code', () async {
    final api = await signedIn();
    await expectLater(
      api.getFarm('missing'),
      throwsA(isA<ApiException>().having((e) => e.code, 'code', 'not_found')),
    );
    await expectLater(
      api.getFarm('html'),
      throwsA(
        isA<ApiException>()
            .having((e) => e.status, 'status', 502)
            .having((e) => e.isOffline, 'offline', isFalse),
      ),
    );
    await expectLater(
      api.getFarm('busy'),
      throwsA(
        isA<ApiException>().having(
          (e) => e.extra?['retry_after_s'],
          'retry',
          30,
        ),
      ),
    );
  });

  test(
    'no server or no answer = offline, so the app keeps its saved copy',
    () async {
      final down = HttpApi('http://127.0.0.1:1')..useToken('tok123');
      await expectLater(
        down.getFarms(),
        throwsA(
          isA<ApiException>().having((e) => e.isOffline, 'offline', isTrue),
        ),
      );
      final api = HttpApi(server.url, timeout: const Duration(seconds: 1))
        ..useToken('tok123');
      await expectLater(
        api.getFarm('slow'),
        throwsA(
          isA<ApiException>().having((e) => e.isOffline, 'offline', isTrue),
        ),
      );
    },
  );
}
