import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/api/api.dart';
import 'package:jutyar/api/fake_api.dart';
import 'package:jutyar/geo.dart';
import 'package:jutyar/screens/home/farm_drawing.dart';
import 'package:latlong2/latlong.dart';

void main() {
  Future<FakeApi> signedIn(String phone) async {
    final api = FakeApi()..assumeOnline = true;
    await api.verifyOtp(phone: phone, code: '123456');
    return api;
  }

  test('demo farm opens with an outline that matches its card', () async {
    final api = await signedIn('+9647501234567');
    final farm = await api.getFarm('f_01HXUPPERFIELD');
    final outline = [for (final p in farm.outline) LatLng(p.lat, p.lon)];
    expect(polygonAreaM2(outline) / 2500, closeTo(120, 0.5));

    final byCrop = <String, double>{};
    for (final c in farm.cells) {
      byCrop[c.crop] = (byCrop[c.crop] ?? 0) + c.insidePct / 2500;
    }
    expect(byCrop['wheat'], closeTo(96, 1));
    expect(byCrop['tomato'], closeTo(16, 1));
    expect(byCrop['empty'], closeTo(8, 1));
  });

  test(
    'status: unsown wheat has no reading, the weak patch is north-east',
    () async {
      final api = await signedIn('+9647501234567');
      final st = await api.getFarmStatus('f_01HXUPPERFIELD');
      final farm = await api.getFarm('f_01HXUPPERFIELD');
      final cropAt = {for (final c in farm.cells) (c.e, c.n): c.crop};
      for (final r in st.cells) {
        final crop = cropAt[(r.e, r.n)];
        if (crop == 'wheat' || crop == 'empty') expect(r.greennessPct, isNull);
        if (crop == 'tomato') expect(r.greennessPct, isNotNull);
      }
      expect(st.weakWhere, 'north-east');
      expect(st.pictureDate, '2026-10-05');
      expect(
        st.crops.firstWhere((c) => c.crop == 'wheat').level,
        FarmStatus.none,
      );
    },
  );

  test('cell count matches the area: 25 cells = 1 dunam', () async {
    final api = await signedIn('+9647501234567');
    final plot = FarmShape(
      await api.getFarm('f_01HXTOMATOPLOT'),
      await api.getFarmStatus('f_01HXTOMATOPLOT'),
    );
    expect(plot.crop.length, greaterThan(200)); // edge cells touch the outline
    expect(plot.cellCount(plot.crop.keys), 200); // but count by area: 8 dunam

    final upper = FarmShape(
      await api.getFarm('f_01HXUPPERFIELD'),
      await api.getFarmStatus('f_01HXUPPERFIELD'),
    );
    expect(upper.cellCount(upper.crop.keys), 3000); // 120 dunam
    final tomato = upper.crop.keys.where((k) => upper.crop[k] == 'tomato');
    expect(upper.cellCount(tomato), closeTo(400, 3)); // 16 dunam
  });

  test('another number cannot open these farms', () async {
    final api = await signedIn('+9647700000000');
    expect(
      () => api.getFarm('f_01HXUPPERFIELD'),
      throwsA(isA<ApiException>().having((e) => e.status, 'status', 404)),
    );
  });

  test('a brand-new farm opens with no readings yet', () async {
    final api = await signedIn('+9647700000000');
    final square = [
      for (final (e, n) in [
        (539600.0, 3935300.0),
        (539700.0, 3935300.0),
        (539700.0, 3935400.0),
        (539600.0, 3935400.0),
      ])
        Utm.toLatLng(e, n),
    ];
    final t = DateTime.utc(2026, 10, 8, 18, 30);
    final made = await api.createFarm(
      NewFarmRequest(
        name: 'New',
        points: [
          for (final p in square)
            GeoPoint(lat: p.latitude, lon: p.longitude, accM: 5, t: t),
        ],
        cells: [
          for (final c in cellsInside(square))
            CellCrop(e: c.e, n: c.n, crop: 'wheat'),
        ],
        createdOfflineAt: t,
      ),
    );
    final st = await api.getFarmStatus(made.farm.summary.id);
    expect(st.pictureDate, isNull);
    expect(st.greennessPctOfNormal, isNull);
    expect(st.cells.every((c) => c.level == FarmStatus.none), isTrue);
    expect(st.nextPictureExpected, isNotNull);
  });

  test('weather rules match weather_planner.py', () {
    final days = [
      for (var i = 0; i < 10; i++)
        '2026-10-${(13 + i).toString().padLeft(2, '0')}',
    ];
    final hours = [
      for (final d in days)
        for (var h = 0; h < 24; h++) '${d}T${h.toString().padLeft(2, '0')}:00',
    ];
    // Day 3-5: 6 + 9 + 7 = 22 mm (sowing rain). Day 6: -3 C (hard frost).
    // Day 1: 10 cool wet hours (rust "some"). Day 0: dry, mild, calm daytime (spray).
    final temp = <double>[],
        rh = <double>[],
        wind = <double>[],
        prec = <double>[];
    for (var i = 0; i < hours.length; i++) {
      final day = i ~/ 24, hour = i % 24;
      final wet = day == 1 && hour < 10;
      temp.add(wet ? 10 : (hour >= 7 && hour <= 18 ? 20 : 12));
      rh.add(wet ? 95 : 50);
      wind.add(day == 0 ? 5 : 20);
      prec.add(0);
    }
    final plan = FakeApi.planFromWeather(
      forecast: {
        'daily': {
          'time': days,
          'precipitation_sum': [0, 0, 0, 6, 9, 7, 0, 0, 0, 0],
          'temperature_2m_min': [8, 8, 8, 8, 8, 8, -3, 8, 8, 8],
          'temperature_2m_max': [24, 24, 24, 24, 24, 24, 24, 24, 24, 24],
        },
        'hourly': {
          'time': hours,
          'temperature_2m': temp,
          'relative_humidity_2m': rh,
          'precipitation': prec,
          'wind_speed_10m': wind,
        },
      },
      air: {
        'hourly': {
          'time': hours.take(120).toList(),
          'pm10': [for (var i = 0; i < 120; i++) i == 50 ? 190 : 40],
        },
      },
      crops: {'wheat'},
      issued: DateTime.utc(2026, 10, 13, 6),
    );
    final p = FarmPlan.fromJson(plan);
    final codes = p.decisions.map((d) => d.code).toList();
    expect(codes, [
      'sow_go',
      'spray_ok',
      'frost_check',
      'check_rust',
      'dust_delay',
    ]);
    expect(p.decisions.first.en, contains('Fri')); // 16 Oct 2026 is a Friday
    final frost = p.alerts.firstWhere((a) => a.type == 'frost');
    expect(frost.level, FarmStatus.alarm);
    expect(frost.day, '2026-10-19');
    expect(p.alerts.firstWhere((a) => a.type == 'dust').day, '2026-10-15');
    expect(p.rainMm.length, 10);
  });
}
