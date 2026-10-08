import 'dart:io';

import 'package:latlong2/latlong.dart';

import '../config.dart';
import '../geo.dart';
import '../store/local_store.dart';
import 'api.dart';

/// Pretends to be the server. Takes and returns the same JSON shapes as
/// BACKEND.md so the models are exercised exactly as with the real server.
///
/// Demo rules: any phone and any 6-digit code work; a phone ending in 0 has
/// no farms (empty state); every other phone has two farms. Like a real
/// server it needs internet (it fails with "offline" when the phone has none),
/// it keeps new farms across app restarts (in a file on the phone), and an
/// Idempotency-Key that was already used returns the same farm again.
class FakeApi implements Api {
  static const _latency = Duration(milliseconds: 600);

  String? _token;
  String? _phone;
  final List<Map<String, dynamic>> _created = [];
  final Map<String, Map<String, dynamic>> _byKey = {};
  bool _loaded = false;

  /// For tests: skip the internet check and the file on the phone.
  bool assumeOnline = false;

  Future<void> _load() async {
    if (_loaded) return;
    _loaded = true;
    if (assumeOnline) return;
    final j = await LocalStore.read('fake_server');
    for (final f in (j?['created'] as List? ?? const [])) {
      _created.add((f as Map).cast<String, dynamic>());
    }
    (j?['keys'] as Map?)?.forEach((k, v) {
      final f = _created.where((c) => c['id'] == v);
      if (f.isNotEmpty) _byKey[k as String] = f.first;
    });
  }

  Future<void> _save() async {
    if (assumeOnline) return;
    await LocalStore.write('fake_server', {
      'created': _created,
      'keys': {for (final e in _byKey.entries) e.key: e.value['id']},
    });
  }

  /// A real server is only reachable with internet; fail the way it would.
  Future<void> _online() async {
    if (assumeOnline) return;
    try {
      final r = await InternetAddress.lookup('example.com')
          .timeout(const Duration(seconds: 4));
      if (r.isEmpty) throw const SocketException('no address');
    } catch (_) {
      throw ApiException(0, 'offline');
    }
  }

  @override
  void useToken(String token) {
    _token = token;
    _phone = token.replaceFirst('demo-token-', '');
  }

  @override
  Future<OtpSendResult> sendOtp({
    required String phone,
    required String lang,
  }) async {
    await _online();
    await Future<void>.delayed(_latency);
    final json = <String, dynamic>{'sent': true, 'retry_after_s': 59};
    return OtpSendResult(
      sent: json['sent'] as bool,
      retryAfterS: json['retry_after_s'] as int,
    );
  }

  @override
  Future<OtpVerifyResult> verifyOtp({
    required String phone,
    required String code,
  }) async {
    await _online();
    await _load();
    await Future<void>.delayed(_latency);
    if (!kTestMode && !RegExp(r'^\d{6}$').hasMatch(code)) {
      throw ApiException(401, 'bad_code');
    }
    _phone = phone;
    _token = 'demo-token-$phone';
    final json = <String, dynamic>{
      'token': _token,
      'farms_count': _farmsJson(phone).length,
    };
    return OtpVerifyResult(
      token: json['token'] as String,
      farmsCount: json['farms_count'] as int,
    );
  }

  @override
  Future<List<FarmSummary>> getFarms() async {
    await _online();
    await _load();
    await Future<void>.delayed(_latency);
    if (_token == null) throw ApiException(401, 'unauthorized');
    final json = <String, dynamic>{'farms': _farmsJson(_phone!)};
    return [
      for (final f in json['farms'] as List)
        FarmSummary.fromJson(f as Map<String, dynamic>),
    ];
  }

  @override
  Future<CreateFarmResult> createFarm(
    NewFarmRequest request, {
    String? idempotencyKey,
  }) async {
    await _online();
    await _load();
    await Future<void>.delayed(_latency);
    if (_token == null) throw ApiException(401, 'unauthorized');
    final seen = idempotencyKey == null ? null : _byKey[idempotencyKey];
    if (seen != null) {
      return CreateFarmResult(farm: Farm.fromJson(seen), droppedCells: 0);
    }
    final body = request.toJson();

    final points = [
      for (final p in body['points'] as List)
        LatLng(
          ((p as Map)['lat'] as num).toDouble(),
          (p['lon'] as num).toDouble(),
        ),
    ];
    if (points.length < 3 || points.length > 50) {
      throw ApiException(422, 'bad_polygon', {'field': 'points'});
    }
    if (selfIntersects(points)) {
      throw ApiException(422, 'bad_polygon', {'field': 'points'});
    }

    // Server rules: area = exact outline area; cells = every cell that overlaps
    // the outline, cut along the border; sent cells that do not overlap are
    // dropped; overlapping cells that were not sent count as "empty".
    final touching = cellsTouching(points);
    final sent = <CellKey, String>{
      for (final c in body['cells'] as List)
        (e: ((c as Map)['e'] as num).toInt(), n: (c['n'] as num).toInt()):
            c['crop'] as String,
    };
    final dropped = [
      for (final e in sent.entries)
        if (!touching.containsKey(e.key))
          {'e': e.key.e, 'n': e.key.n, 'crop': e.value},
    ];
    final cropM2 = <String, double>{};
    final cells = <Map<String, dynamic>>[];
    touching.forEach((k, m2) {
      final crop = sent[k] ?? 'empty';
      if (crop != 'empty') cropM2[crop] = (cropM2[crop] ?? 0) + m2;
      cells.add({
        'e': k.e,
        'n': k.n,
        'crop': crop,
        'inside_pct': (m2 * 10).round() / 10,
        'greenness_pct': null,
        'level': 'none',
      });
    });
    final crops = cropM2.entries.toList()
      ..sort((a, b) => b.value.compareTo(a.value));
    final lat =
        points.map((p) => p.latitude).reduce((a, b) => a + b) / points.length;
    final lon =
        points.map((p) => p.longitude).reduce((a, b) => a + b) / points.length;

    final farm = <String, dynamic>{
      'id': 'f_${DateTime.now().microsecondsSinceEpoch}',
      'name': body['name'],
      'area_dunam': _round2(polygonAreaM2(points) / 2500),
      'crops': [
        for (final c in crops)
          {'crop': c.key, 'dunam': _round2(c.value / 2500)},
      ],
      'status': 'none',
      'last_picture': null,
      'centroid': {'lat': lat, 'lon': lon},
      'outline': [
        for (final p in points) {'lat': p.latitude, 'lon': p.longitude},
      ],
      'cells': cells,
      'picture_date': null,
    };
    _created.add(farm);
    if (idempotencyKey != null) _byKey[idempotencyKey] = farm;
    await _save();
    final response = <String, dynamic>{'farm': farm, 'dropped_cells': dropped};
    return CreateFarmResult(
      farm: Farm.fromJson(response['farm'] as Map<String, dynamic>),
      droppedCells: (response['dropped_cells'] as List).length,
    );
  }

  static double _round2(double v) => (v * 100).round() / 100;

  List<Map<String, dynamic>> _farmsJson(String phone) => [
    ...(phone.endsWith('0') ? const <Map<String, dynamic>>[] : _demoFarms),
    ..._created,
  ];

  static const _demoFarms = <Map<String, dynamic>>[
    {
      'id': 'f_01HXUPPERFIELD',
      'name': 'کێڵگەی سەرەوە',
      'area_dunam': 120,
      'crops': [
        {'crop': 'wheat', 'dunam': 96},
        {'crop': 'tomato', 'dunam': 16},
      ],
      'status': 'none',
      'last_picture': '2026-10-05',
      'centroid': {'lat': 36.03, 'lon': 44.60},
    },
    {
      'id': 'f_01HXTOMATOPLOT',
      'name': 'باخچەی تەماتە',
      'area_dunam': 8,
      'crops': [
        {'crop': 'tomato', 'dunam': 8},
      ],
      'status': 'normal',
      'last_picture': '2026-10-05',
      'centroid': {'lat': 36.04, 'lon': 44.61},
    },
  ];
}
