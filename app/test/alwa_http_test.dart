import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/api/api.dart';
import 'package:jutyar/api/http_api.dart';

/// A tiny server shaped like the Alwa routes built today (FRONTEND.md 5):
/// no point, no seller phone, no distance, no /sold route.
class _Server {
  late HttpServer http;
  final seen =
      <
        ({
          String method,
          String path,
          String query,
          HttpHeaders headers,
          String body,
        })
      >[];

  String get url => 'http://127.0.0.1:${http.port}/v1';

  static const listing = {
    'id': '7',
    'crop': 'tomato',
    'quantity_kg': 4000,
    'asking_price_iqd_per_kg': 700,
    'best_offer_iqd_per_kg': null,
    'closes_at': '2026-10-21T08:10:00Z',
    'created_at': '2026-10-07T08:10:00Z',
    'fair_price': 'unknown',
    'grade': null,
    'market': 'sulaymaniyah',
    'offers': 0,
    'pickup': 'farm',
    'seller_name': null,
    'status': 'open',
    'zone_slug': null,
  };

  /// A list row exactly as the test server sent it on 2026-10-09: list rows
  /// have no `created_at` (only one listing by id has it).
  static const row = {
    'id': '1',
    'crop': 'tomato',
    'quantity_kg': 200,
    'asking_price_iqd_per_kg': 500,
    'grade': null,
    'pickup': 'farm',
    'market': 'sulaymaniyah',
    'zone_slug': null,
    'seller_name': null,
    'closes_at': '2026-10-23T14:47:26Z',
    'status': 'open',
    'offers': 0,
    'best_offer_iqd_per_kg': null,
    'fair_price': 'unknown',
  };

  Future<void> start() async {
    http = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
    http.listen((req) async {
      final body = await utf8.decoder.bind(req).join();
      final path = req.uri.path.replaceFirst('/v1', '');
      seen.add((
        method: req.method,
        path: path,
        query: req.uri.query,
        headers: req.headers,
        body: body,
      ));
      void send(int status, Object? json) {
        req.response.statusCode = status;
        if (json != null) {
          req.response
            ..headers.contentType = ContentType.json
            ..write(jsonEncode(json));
        }
        req.response.close();
      }

      switch ((req.method, path)) {
        case ('GET', '/alwa/listings'):
          send(200, {
            'count': 2,
            'page': 1,
            'rows_per_page': 100,
            'listings': [listing, row],
          });
        case ('GET', '/alwa/listings/7'):
          send(200, {'listing': listing});
        case ('POST', '/alwa/listings'):
          send(201, {'listing': listing});
        case ('GET', '/alwa/listings/mine'):
          send(200, {
            'listings': [listing],
          });
        case ('DELETE', '/alwa/listings/7'):
          send(204, null);
        case ('GET', '/alwa/markets'):
          send(200, {
            'markets': [
              {
                'slug': 'sulaymaniyah',
                'name_en': 'Sulaymaniyah',
                'name_ku': 'سلێمانی',
              },
            ],
          });
        case ('GET', '/alwa/markets/sulaymaniyah/prices'):
          send(200, {'day': null, 'market': 'sulaymaniyah', 'prices': []});
        default:
          send(404, {'error': 'not_found', 'detail': 'no route'});
      }
    });
  }
}

void main() {
  late _Server server;
  late HttpApi api;

  setUp(() async {
    server = _Server();
    await server.start();
    api = HttpApi(server.url)..useToken('tok123');
  });

  test(
    'the seller phone rides in seller_name until seller_phone exists',
    () async {
      await api.createAlwaListing(
        const NewAlwaListing(
          crop: 'tomato',
          quantityKg: 200,
          priceIqdPerKg: 500,
          lat: 35.56,
          lon: 45.43,
          sellerPhone: '+9647501234567',
        ),
      );
      final sent =
          jsonDecode(server.seen.lastWhere((r) => r.method == 'POST').body)
              as Map<String, dynamic>;
      expect(sent['seller_name'], '+9647501234567');
      final back = AlwaListing.fromJson({
        ..._Server.row,
        'seller_name': '+9647501234567',
      });
      expect(back.sellerPhone, '+9647501234567');
      final named = AlwaListing.fromJson({
        ..._Server.row,
        'seller_name': 'Kak Aram',
      });
      expect(named.sellerPhone, isNull, reason: 'a real name is not a phone');
    },
  );

  test('a list row without created_at is shown, not dropped', () async {
    final all = await api.alwaListings();
    final r = all.firstWhere((l) => l.id == '1');
    expect(r.crop, 'tomato');
    expect(r.quantityKg, 200);
    expect(r.isOpen, isTrue);
    expect(r.createdAt, isNull, reason: 'unknown, never guessed');
  });

  tearDown(() => server.http.close(force: true));

  test(
    'listings from today\'s server: missing fields are null, not made up',
    () async {
      final list = await api.alwaListings(lat: 35.56, lon: 45.43);
      expect(list, hasLength(2));
      final l = list.firstWhere((x) => x.id == '7');
      expect(l.crop, 'tomato');
      expect(l.quantityKg, 4000);
      expect(l.priceIqdPerKg, 700);
      expect(l.isOpen, isTrue);
      expect(l.lat, isNull);
      expect(l.sellerPhone, isNull);
      expect(l.distanceKm, isNull);
      expect(l.kmFrom(35.56, 45.43), isNull, reason: 'no point, no distance');
      final q = Uri.splitQueryString(server.seen.single.query);
      expect(q['status'], 'open');
      expect(q['lat'], '35.56');
    },
  );

  test(
    'create sends the point, the key, and what the server still needs',
    () async {
      await api.createAlwaListing(
        const NewAlwaListing(
          crop: 'tomato',
          quantityKg: 4000,
          priceIqdPerKg: 700,
          lat: 35.56,
          lon: 45.43,
          days: 14,
        ),
        idempotencyKey: 'k-1',
      );
      final post = server.seen.firstWhere((r) => r.method == 'POST');
      expect(post.headers.value('Idempotency-Key'), 'k-1');
      final body = jsonDecode(post.body) as Map<String, dynamic>;
      expect(body['crop'], 'tomato');
      expect(body['quantity_kg'], 4000);
      expect(body['asking_price_iqd_per_kg'], 700);
      // The server's fields are integers: 4000.0 would be refused.
      expect(body['quantity_kg'], isA<int>());
      expect(body['asking_price_iqd_per_kg'], isA<int>());
      expect(body['lat'], 35.56);
      expect(body['lon'], 45.43);
      expect(body['market'], 'sulaymaniyah');
      expect(body['pickup'], 'farm');
      final closes = DateTime.parse(body['closes_at'] as String);
      expect(
        closes.difference(DateTime.now()).inDays,
        13,
        reason: '14 days ahead',
      );
    },
  );

  test('mine, cancel, one listing', () async {
    expect(await api.myAlwaListings(), hasLength(1));
    expect((await api.alwaListing('7')).id, '7');
    await api.cancelAlwaListing('7');
    expect(server.seen.last.method, 'DELETE');
  });

  test('mark as sold is not on the server yet: a 404, not a crash', () async {
    await expectLater(
      api.markAlwaListingSold('7'),
      throwsA(isA<ApiException>().having((e) => e.status, 'status', 404)),
    );
  });

  test('price board with nothing typed in today', () async {
    final b = await api.alwaPriceBoard(lat: 35.56, lon: 45.43);
    expect(b, isNotNull);
    expect(b!.market.nameEn, 'Sulaymaniyah');
    expect(b.day, isNull);
    expect(b.prices, isEmpty);
    expect(b.nearest, isFalse, reason: 'markets have no point yet');
  });
}
