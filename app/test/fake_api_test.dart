import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/api/api.dart';
import 'package:jutyar/api/fake_api.dart';
import 'package:jutyar/geo.dart';

void main() {
  test('a new farm is saved, counted right, and listed in My farms', () async {
    final api = FakeApi()..assumeOnline = true;
    await api.verifyOtp(phone: '+9647700000000', code: '123456');
    expect(await api.getFarms(), isEmpty);

    final t = DateTime.utc(2026, 10, 8, 18, 30);
    final square = [
      for (final (e, n) in [
        (539600.0, 3935300.0),
        (539700.0, 3935300.0),
        (539700.0, 3935400.0),
        (539600.0, 3935400.0),
      ])
        Utm.toLatLng(e, n),
    ];
    final points = [
      for (final p in square)
        GeoPoint(lat: p.latitude, lon: p.longitude, accM: 5, t: t),
    ];
    final cells = cellsInside(square);
    final painted = [
      for (final (i, c) in cells.indexed)
        CellCrop(e: c.e, n: c.n, crop: i < 75 ? 'wheat' : 'tomato'),
      const CellCrop(e: 1, n: 1, crop: 'wheat'),
    ];

    final req = NewFarmRequest(
      name: 'Test farm',
      points: points,
      cells: painted,
      createdOfflineAt: t,
    );
    final json = req.toJson();
    expect(json['created_offline_at'], '2026-10-08T18:30:00Z');
    expect((json['points'] as List).first, containsPair('acc_m', 5));

    final res = await api.createFarm(req);
    expect(res.droppedCells, 1);
    expect(res.farm.summary.areaDunam, 4);
    expect(res.farm.cells.length, 100);
    expect(res.farm.summary.crops.map((c) => (c.crop, c.dunam)), [
      ('wheat', 3.0),
      ('tomato', 1.0),
    ]);

    final list = await api.getFarms();
    expect(list.map((f) => f.name), ['Test farm']);
    expect(list.single.lastPicture, isNull);
  });

  test('a bow-tie outline is refused with bad_polygon', () async {
    final api = FakeApi()..assumeOnline = true;
    await api.verifyOtp(phone: '+9647701234567', code: '123456');
    final t = DateTime.utc(2026, 10, 8);
    final bow = [
      for (final (e, n) in [
        (539600.0, 3935300.0),
        (539700.0, 3935400.0),
        (539700.0, 3935300.0),
        (539600.0, 3935400.0),
      ])
        Utm.toLatLng(e, n),
    ];
    final req = NewFarmRequest(
      name: 'Bad',
      points: [
        for (final p in bow)
          GeoPoint(lat: p.latitude, lon: p.longitude, accM: 5, t: t),
      ],
      cells: const [],
      createdOfflineAt: t,
    );
    expect(
      () => api.createFarm(req),
      throwsA(isA<ApiException>().having((e) => e.code, 'code', 'bad_polygon')),
    );
  });

  test(
    'a tilted field: area is the exact outline area and crops add up to it',
    () async {
      final api = FakeApi()..assumeOnline = true;
      await api.verifyOtp(phone: '+9647709999999', code: '123456');
      final t = DateTime.utc(2026, 10, 8);
      final field = [
        Utm.toLatLng(539603.3, 3935301.7),
        Utm.toLatLng(539688.9, 3935322.4),
        Utm.toLatLng(539671.2, 3935397.8),
        Utm.toLatLng(539611.6, 3935381.1),
      ];
      final touching = cellsTouching(field).keys.toList();
      final res = await api.createFarm(
        NewFarmRequest(
          name: 'Tilted',
          points: [
            for (final p in field)
              GeoPoint(lat: p.latitude, lon: p.longitude, accM: 4, t: t),
          ],
          cells: [
            for (final (i, c) in touching.indexed)
              CellCrop(e: c.e, n: c.n, crop: i.isEven ? 'wheat' : 'barley'),
          ],
          createdOfflineAt: t,
        ),
      );
      final exact = polygonAreaM2(field) / 2500;
      expect(res.farm.summary.areaDunam, closeTo(exact, 0.005));
      final cropSum = res.farm.summary.crops.fold(0.0, (a, c) => a + c.dunam);
      expect(cropSum, closeTo(exact, 0.02));
      expect(res.droppedCells, 0);
    },
  );
}
