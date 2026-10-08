import 'dart:convert';
import 'dart:io';
import 'dart:math' as math;

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

  /// Ids of deleted farms (also hides deleted demo farms).
  final Set<String> _deleted = {};
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
    _deleted.addAll([
      for (final d in (j?['deleted'] as List? ?? const [])) d as String,
    ]);
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
      'deleted': _deleted.toList(),
    });
  }

  /// A real server is only reachable with internet; fail the way it would.
  Future<void> _online() async {
    if (assumeOnline) return;
    try {
      final r = await InternetAddress.lookup(
        'example.com',
      ).timeout(const Duration(seconds: 4));
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
    final (farm, dropped) = _buildFarm(
      request.toJson(),
      'f_${DateTime.now().microsecondsSinceEpoch}',
    );
    _created.add(farm);
    if (idempotencyKey != null) _byKey[idempotencyKey] = farm;
    await _save();
    final response = <String, dynamic>{'farm': farm, 'dropped_cells': dropped};
    return CreateFarmResult(
      farm: Farm.fromJson(response['farm'] as Map<String, dynamic>),
      droppedCells: (response['dropped_cells'] as List).length,
    );
  }

  /// PUT /farms/{id}: replaces the outline, cells and name of one of this
  /// phone's farms (a demo farm gets an edited copy that hides the original).
  @override
  Future<CreateFarmResult> updateFarm(
    String id,
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
    if (!_farmsJson(_phone!).any((f) => f['id'] == id)) {
      throw ApiException(404, 'not_found');
    }
    final old = _farmsJson(_phone!).firstWhere((f) => f['id'] == id);
    final (farm, dropped) = _buildFarm(request.toJson(), id);
    farm['status'] = old['status'] ?? 'none';
    farm['last_picture'] = old['last_picture'];
    final i = _created.indexWhere((f) => f['id'] == id);
    if (i >= 0) {
      _created[i] = farm;
    } else {
      _created.add(farm);
    }
    if (idempotencyKey != null) _byKey[idempotencyKey] = farm;
    await _save();
    final response = <String, dynamic>{'farm': farm, 'dropped_cells': dropped};
    return CreateFarmResult(
      farm: Farm.fromJson(response['farm'] as Map<String, dynamic>),
      droppedCells: (response['dropped_cells'] as List).length,
    );
  }

  /// DELETE /farms/{id}: 404 when it is not one of this phone's farms.
  @override
  Future<void> deleteFarm(String id) async {
    await _online();
    await _load();
    await Future<void>.delayed(_latency);
    if (_token == null) throw ApiException(401, 'unauthorized');
    if (!_farmsJson(_phone!).any((f) => f['id'] == id)) {
      throw ApiException(404, 'not_found');
    }
    _created.removeWhere((f) => f['id'] == id);
    _deleted.add(id);
    await _save();
  }

  /// Server rules for a farm body: area = exact outline area; cells = every
  /// cell that overlaps the outline, cut along the border; sent cells that do
  /// not overlap are dropped; overlapping cells not sent count as "empty".
  (Map<String, dynamic>, List<Map<String, dynamic>>) _buildFarm(
    Map<String, dynamic> body,
    String id,
  ) {
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
      'id': id,
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
    return (farm, dropped);
  }

  // ---- Farm Home: GET /farms/{id}, /status, /plan ----

  /// The full farm JSON, only if it belongs to the signed-in phone.
  Map<String, dynamic> _ownFarm(String id) {
    if (_token == null) throw ApiException(401, 'unauthorized');
    if (!_farmsJson(_phone!).any((f) => f['id'] == id)) {
      throw ApiException(404, 'not_found');
    }
    final made = _created.where((f) => f['id'] == id);
    if (made.isNotEmpty) return made.first;
    return _demoDetail(id)!;
  }

  @override
  Future<Farm> getFarm(String id) async {
    await _online();
    await _load();
    await Future<void>.delayed(_latency);
    final json = <String, dynamic>{'farm': _ownFarm(id)};
    return Farm.fromJson(json['farm'] as Map<String, dynamic>);
  }

  @override
  Future<FarmStatusReport> getFarmStatus(String id) async {
    await _online();
    await _load();
    await Future<void>.delayed(_latency);
    return FarmStatusReport.fromJson(statusFromFarm(_ownFarm(id)));
  }

  @override
  Future<FarmPlan> getPlan(String id) async {
    await _online();
    await _load();
    final farm = _ownFarm(id);
    final c = farm['centroid'] as Map;
    final at = 'latitude=${c['lat']}&longitude=${c['lon']}';
    final forecast = await _getJson(
      'https://api.open-meteo.com/v1/forecast?$at'
      '&daily=precipitation_sum,temperature_2m_min,temperature_2m_max'
      '&hourly=temperature_2m,relative_humidity_2m,precipitation,wind_speed_10m'
      '&forecast_days=10&timezone=Asia%2FBaghdad',
    );
    Map<String, dynamic>? air;
    try {
      air = await _getJson(
        'https://air-quality-api.open-meteo.com/v1/air-quality?$at'
        '&hourly=pm10&forecast_days=5&timezone=Asia%2FBaghdad',
      );
    } on ApiException {
      air = null; // dust is a bonus; the plan still works without it
    }
    return FarmPlan.fromJson(
      planFromWeather(
        forecast: forecast,
        air: air,
        crops: {
          for (final x in farm['crops'] as List) (x as Map)['crop'] as String,
        },
        issued: DateTime.now(),
      ),
    );
  }

  static Future<Map<String, dynamic>> _getJson(String url) async {
    final client = HttpClient()..connectionTimeout = const Duration(seconds: 8);
    try {
      final req = await client.getUrl(Uri.parse(url));
      req.headers.set('User-Agent', 'krd.jutyar');
      final res = await req.close().timeout(const Duration(seconds: 15));
      if (res.statusCode != 200) throw const SocketException('bad status');
      final body = await res.transform(utf8.decoder).join();
      return jsonDecode(body) as Map<String, dynamic>;
    } catch (_) {
      throw ApiException(503, 'upstream_down', {'source': 'weather'});
    } finally {
      client.close();
    }
  }

  /// Demo cut-offs for a cell's greenness vs its own normal. The real levels
  /// come from the backend's Field Eye; the app only paints `level`.
  static String _levelOf(int? pct) => pct == null
      ? 'none'
      : pct >= 85
      ? 'normal'
      : pct >= 70
      ? 'watch'
      : 'alarm';

  /// BACKEND.md 2.3 answer, worked out from the farm's cells.
  static Map<String, dynamic> statusFromFarm(Map<String, dynamic> farm) {
    final cells = [for (final c in farm['cells'] as List) c as Map];
    double m2(Map c) => (c['inside_pct'] as num? ?? 100).toDouble();
    final measured = cells.where((c) => c['greenness_pct'] != null).toList();
    final weak = measured.where((c) => c['level'] != 'normal').toList();
    final dataM2 = measured.fold(0.0, (a, c) => a + m2(c));
    final allM2 = cells.fold(0.0, (a, c) => a + m2(c));
    int? meanPct(Iterable<Map> cs) {
      final w = cs.fold(0.0, (a, c) => a + m2(c));
      if (w == 0) return null;
      return (cs.fold(0.0, (a, c) => a + m2(c) * (c['greenness_pct'] as num)) /
              w)
          .round();
    }

    final whole = meanPct(measured);

    // Where the weak cells sit, worded like field_eye.py: north/middle/south + west/centre/east.
    String? where;
    if (weak.isNotEmpty) {
      final es = cells.map((c) => c['e'] as int);
      final ns = cells.map((c) => c['n'] as int);
      final e0 = es.reduce(math.min), e1 = es.reduce(math.max);
      final n0 = ns.reduce(math.min), n1 = ns.reduce(math.max);
      final col =
          weak.map((c) => (c['e'] as int) - e0).reduce((a, b) => a + b) /
          weak.length /
          math.max(1, e1 - e0);
      final row =
          weak.map((c) => n1 - (c['n'] as int)).reduce((a, b) => a + b) /
          weak.length /
          math.max(1, n1 - n0);
      where =
          '${row < 0.4
              ? 'north'
              : row > 0.6
              ? 'south'
              : 'middle'}-'
          '${col < 0.4
              ? 'west'
              : col > 0.6
              ? 'east'
              : 'centre'}';
    }

    final byCrop = <String, List<Map>>{};
    for (final c in cells) {
      if (c['crop'] != 'empty') {
        byCrop.putIfAbsent(c['crop'] as String, () => []).add(c);
      }
    }
    final picture = farm['last_picture'] as String?;
    return {
      'picture_date': picture,
      'cloud_pct': picture == null ? null : 0,
      'greenness_pct_of_normal': whole,
      'pct_of_neighbours': whole == null ? null : whole - 3,
      'surface': picture == null
          ? null
          : dataM2 < 0.2 * allM2
          ? 'bare'
          : 'growing',
      'weak_share_pct': dataM2 == 0
          ? 0
          : (100 * weak.fold(0.0, (a, c) => a + m2(c)) / dataM2).round(),
      'weak_where': where,
      'cells': [
        for (final c in cells)
          {
            'e': c['e'],
            'n': c['n'],
            'greenness_pct': c['greenness_pct'],
            'level': c['level'] ?? 'none',
            'since': c['since'],
          },
      ],
      'crops': [
        for (final e in byCrop.entries)
          {
            'crop': e.key,
            'dunam': _round2(e.value.fold(0.0, (a, c) => a + m2(c)) / 2500),
            'greenness_pct_of_normal': meanPct(
              e.value.where((c) => c['greenness_pct'] != null),
            ),
            'level': _levelOf(
              meanPct(e.value.where((c) => c['greenness_pct'] != null)),
            ),
          },
      ]..sort((a, b) => (b['dunam'] as num).compareTo(a['dunam'] as num)),
      'history': const <Map<String, dynamic>>[],
      'next_picture_expected': _nextPicture(picture),
    };
  }

  /// Sentinel-2 sees each spot about every 5 days; demo cycle anchored on 5 Oct 2026.
  static String _nextPicture(String? last) {
    var d = last == null
        ? DateTime.utc(2026, 10, 5)
        : DateTime.parse('${last}T00:00:00Z');
    final today = DateTime.now().toUtc();
    do {
      d = d.add(const Duration(days: 5));
    } while (!d.isAfter(today));
    return d.toIso8601String().substring(0, 10);
  }

  static int _hash(int e, int n) =>
      ((e * 73856093) ^ (n * 19349663) ^ 0x5bd1e995) & 0x7fffffff;

  static final Map<String, Map<String, dynamic>> _demoCache = {};

  /// Shapes of the two demo farms, in metres around the centroid (east, north).
  static const _demoShapes = <String, List<(double, double)>>{
    'f_01HXUPPERFIELD': [
      (-290, 230),
      (120, 300),
      (330, 80),
      (250, -250),
      (-60, -300),
      (-300, -120),
    ],
    'f_01HXTOMATOPLOT': [(-85, -55), (85, -62), (88, 58), (-82, 60)],
  };

  /// A demo farm with a real outline and cells. The shape is scaled so the
  /// area matches its card in My farms, and the crops are laid out (main crop
  /// west, the others in a strip on the east side) so their dunams match too.
  static Map<String, dynamic>? _demoDetail(String id) {
    final cached = _demoCache[id];
    if (cached != null) return cached;
    final summary = _demoFarms.where((f) => f['id'] == id).firstOrNull;
    final shape = _demoShapes[id];
    if (summary == null || shape == null) return null;

    final c = summary['centroid'] as Map;
    final (cx, cy) = Utm.fromLatLng(c['lat'] as double, c['lon'] as double);
    var a = 0.0;
    for (var i = 0, j = shape.length - 1; i < shape.length; j = i++) {
      a += (shape[j].$1 + shape[i].$1) * (shape[j].$2 - shape[i].$2);
    }
    final k = math.sqrt((summary['area_dunam'] as num) * 2500 / (a.abs() / 2));
    final outline = [
      for (final (x, y) in shape) Utm.toLatLng(cx + k * x, cy + k * y),
    ];
    final touching = cellsTouching(outline);

    final crops = [
      for (final x in summary['crops'] as List)
        (crop: (x as Map)['crop'] as String, m2: (x['dunam'] as num) * 2500.0),
    ];
    final used = crops.fold(0.0, (s, x) => s + x.m2);
    final total = touching.values.fold(0.0, (s, v) => s + v);
    final strip = [
      ...crops.skip(1),
      if (total - used > 100) (crop: 'empty', m2: total - used),
    ];
    final byEast = touching.keys.toList()..sort((p, q) => q.e.compareTo(p.e));
    final stripM2 = strip.fold(0.0, (s, x) => s + x.m2);
    final stripCells = <CellKey>[];
    var acc = 0.0;
    for (final key in byEast) {
      if (acc >= stripM2) break;
      stripCells.add(key);
      acc += touching[key]!;
    }
    stripCells.sort((p, q) => q.n.compareTo(p.n));
    final cropOf = <CellKey, String>{};
    var si = 0;
    var filled = 0.0;
    for (final key in stripCells) {
      while (si < strip.length - 1 && filled >= strip[si].m2) {
        filled = 0;
        si++;
      }
      cropOf[key] = strip[si].crop;
      filled += touching[key]!;
    }

    final es = touching.keys.map((q) => q.e);
    final ns = touching.keys.map((q) => q.n);
    final e0 = es.reduce(math.min), e1 = es.reduce(math.max);
    final n0 = ns.reduce(math.min), n1 = ns.reduce(math.max);
    final notSown = summary['status'] == 'none';
    final cells = <Map<String, dynamic>>[];
    touching.forEach((key, m2) {
      final crop = cropOf[key] ?? crops.first.crop;
      final h = _hash(key.e, key.n);
      final x = (key.e - e0) / math.max(1, e1 - e0); // 0 west, 1 east
      final y = (key.n - n0) / math.max(1, n1 - n0); // 0 south, 1 north
      int? pct;
      String? since;
      final winterGrain = crop == 'wheat' || crop == 'barley';
      if (crop != 'empty' && !(notSown && winterGrain)) {
        pct = 94 + h % 15;
        // Demo weak patches: upper field north-east corner, tomato plot south edge.
        if (id == 'f_01HXUPPERFIELD' && x > 0.8 && y > 0.78) {
          pct = x > 0.9 && y > 0.88 ? 52 + h % 14 : 71 + h % 12;
          since = pct < 70 ? '2026-09-30' : '2026-09-25';
        } else if (id == 'f_01HXTOMATOPLOT' && y < 0.12 && h % 3 == 0) {
          pct = 74 + h % 9;
          since = '2026-10-01';
        }
      }
      cells.add({
        'e': key.e,
        'n': key.n,
        'crop': crop,
        'inside_pct': (m2 * 10).round() / 10,
        'greenness_pct': pct,
        'level': _levelOf(pct),
        'since': since,
      });
    });

    return _demoCache[id] = {
      ...summary,
      'outline': [
        for (final p in outline) {'lat': p.latitude, 'lon': p.longitude},
      ],
      'cells': cells,
      'picture_date': summary['last_picture'],
    };
  }

  static const _wd = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];
  static String _day(String ymd) => _wd[DateTime.parse(ymd).weekday - 1];
  static String _mm(double v) => v.round().toString();

  /// The Weather Planner's rules (farm_doctor/weather_planner.py, same numbers)
  /// on an Open-Meteo answer, shaped as BACKEND.md 2.4.
  /// Sorani text is not written yet: `ku` repeats `en` until a native speaker checks it.
  static Map<String, dynamic> planFromWeather({
    required Map<String, dynamic> forecast,
    Map<String, dynamic>? air,
    required Set<String> crops,
    required DateTime issued,
  }) {
    List<double?> nums(Object? v) => [
      for (final x in (v as List? ?? const [])) (x as num?)?.toDouble(),
    ];
    final d = forecast['daily'] as Map<String, dynamic>;
    final days = (d['time'] as List).cast<String>();
    final rain = nums(d['precipitation_sum']);
    final tmin = nums(d['temperature_2m_min']);
    final tmax = nums(d['temperature_2m_max']);
    final h = forecast['hourly'] as Map<String, dynamic>;
    final ht = (h['time'] as List).cast<String>();
    final temp = nums(h['temperature_2m']);
    final rh = nums(h['relative_humidity_2m']);
    final hp = nums(h['precipitation']);
    final wind = nums(h['wind_speed_10m']);
    final month = int.parse(days.first.substring(5, 7));
    final grain = crops.contains('wheat') || crops.contains('barley');
    double r(int i) => rain[i] ?? 0;

    final alerts = <Map<String, dynamic>>[];
    final decisions = <Map<String, dynamic>>[];
    void alert(
      String type,
      String day,
      double? value,
      String level,
      String en,
    ) => alerts.add({
      'type': type,
      'day': day,
      'value': value,
      'level': level,
      'ku': en,
      'en': en,
    });
    void decide(String code, String en) =>
        decisions.add({'code': code, 'ku': en, 'en': en});

    // Sowing: 20 mm or more within 3 days, Oct to Dec.
    if (grain && month >= 10) {
      final i = [
        for (var i = 0; i + 2 < days.length; i++) i,
      ].where((i) => r(i) + r(i + 1) + r(i + 2) >= 20).firstOrNull;
      if (i != null) {
        final sum = r(i) + r(i + 1) + r(i + 2);
        alert(
          'sowing_rain',
          days[i],
          sum,
          'watch',
          'Sowing rain: ${_mm(sum)} mm in 3 days from ${_day(days[i])}',
        );
        decide(
          'sow_go',
          'Good sowing rain (20 mm or more in 3 days) starts ${_day(days[i])}.',
        );
      } else {
        decide(
          'sow_wait',
          'No sowing rain (20 mm) in the next 10 days. Wait, do not dry-sow.',
        );
      }
    }
    // Urea: spread on dry soil just before 12 mm or more, Jan to Mar.
    if (crops.contains('wheat') && month <= 3) {
      final i = [
        for (var i = 0; i < days.length; i++) i,
      ].where((i) => r(i) >= 12).firstOrNull;
      if (i != null) {
        alert(
          'urea_rain',
          days[i],
          r(i),
          'watch',
          '${_mm(r(i))} mm rain ${_day(days[i])}: urea goes on just before',
        );
        decide(
          'urea_go',
          'Spread urea on dry soil just before ${_day(days[i])} (${_mm(r(i))} mm).',
        );
      } else {
        decide(
          'urea_hold',
          'No 12 mm rain coming. Hold the urea top-dressing.',
        );
      }
    }
    // Spray windows: 6 hours in a row, dry, 15 to 24 C, wind under 15 km/h, 7:00 to 18:00.
    final spray = <String>{};
    for (var i = 0; i + 6 <= ht.length; i++) {
      final ok = [for (var j = i; j < i + 6; j++) j].every((j) {
        final t = temp[j];
        final hour = int.parse(ht[j].substring(11, 13));
        return (hp[j] ?? 0) == 0 &&
            t != null &&
            t >= 15 &&
            t <= 24 &&
            (wind[j] ?? 0) < 15 &&
            hour >= 7 &&
            hour <= 18;
      });
      if (ok) spray.add(ht[i].substring(0, 10));
    }
    if (spray.isNotEmpty) {
      decide(
        'spray_ok',
        'Safe spray days: ${(spray.toList()..sort()).take(5).map(_day).join(', ')}.',
      );
    }
    // Frost.
    final hard = <String>[];
    for (var i = 0; i < days.length; i++) {
      final t = tmin[i];
      if (t != null && t <= 0) {
        alert(
          'frost',
          days[i],
          t,
          t <= -2 ? 'alarm' : 'watch',
          'Frost ${t.toStringAsFixed(0)} °C ${_day(days[i])} night',
        );
        if (t <= -2) hard.add(_day(days[i]));
      }
    }
    if (hard.isNotEmpty) {
      decide(
        'frost_check',
        'Hard frost on ${hard.join(', ')}. Check the heads 7 to 10 days after.',
      );
    }
    // Heat during flowering, Apr and May.
    if (month == 4 || month == 5) {
      final hot = <String>[];
      for (var i = 0; i < days.length; i++) {
        final t = tmax[i];
        if (t != null && t >= 31) {
          alert(
            'heat',
            days[i],
            t,
            'watch',
            'Heat ${t.toStringAsFixed(0)} °C ${_day(days[i])}',
          );
          hot.add(_day(days[i]));
        }
      }
      if (hot.isNotEmpty) {
        decide(
          'heat_check',
          'Over 31 °C on ${hot.join(', ')} during flowering.',
        );
      }
    }
    // Rust weather: hours at 6 to 16 C with humidity 90% or more.
    if (grain) {
      final wet = [
        for (var j = 0; j < ht.length; j++)
          if (temp[j] != null &&
              temp[j]! >= 6 &&
              temp[j]! <= 16 &&
              (rh[j] ?? 0) >= 90)
            j,
      ];
      if (wet.length >= 8) {
        final high = wet.length >= 24;
        final day = ht[wet.first].substring(0, 10);
        alert(
          'rust_weather',
          day,
          wet.length.toDouble(),
          high ? 'alarm' : 'watch',
          'Rust weather: ${wet.length} cool wet hours',
        );
        decide(
          'check_rust',
          '${high ? 'High' : 'Some'} rust weather (${wet.length} cool wet hours). Check the leaves.',
        );
      }
    }
    // Dust: PM10 150 or more in the next 5 days.
    if (air != null) {
      final ah = air['hourly'] as Map<String, dynamic>;
      final at = (ah['time'] as List).cast<String>();
      final pm = nums(ah['pm10']);
      var top = -1;
      for (var j = 0; j < pm.length; j++) {
        if (pm[j] != null && (top < 0 || pm[j]! > pm[top]!)) top = j;
      }
      if (top >= 0 && pm[top]! >= 150) {
        final day = at[top].substring(0, 10);
        alert(
          'dust',
          day,
          pm[top],
          'watch',
          'Dust ${_day(day)}: PM10 up to ${_mm(pm[top]!)}',
        );
        decide(
          'dust_delay',
          'Dust (PM10 up to ${_mm(pm[top]!)}) ${_day(day)}. Delay spraying and harvest, shelter animals.',
        );
      }
    }

    alerts.sort((a, b) => (a['day'] as String).compareTo(b['day'] as String));
    return {
      'from': days.first,
      'days': days.length,
      'rain_mm': rain,
      'tmin': tmin,
      'tmax': tmax,
      'alerts': alerts,
      'decisions': decisions,
      'source': 'Open-Meteo (ECMWF/GraphCast family)',
      'issued': isoUtc(issued),
    };
  }

  static double _round2(double v) => (v * 100).round() / 100;

  /// Demo farms (unless edited, then the edited copy in [_created] is used)
  /// plus farms made on this phone.
  List<Map<String, dynamic>> _farmsJson(String phone) => [
    if (!phone.endsWith('0'))
      for (final d in _demoFarms)
        if (!_created.any((c) => c['id'] == d['id']) &&
            !_deleted.contains(d['id']))
          d,
    for (final c in _created)
      if (!_deleted.contains(c['id'])) c,
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
