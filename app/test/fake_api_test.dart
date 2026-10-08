import 'package:flutter_test/flutter_test.dart';
import 'package:latlong2/latlong.dart';
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

  test('editing a farm changes its border, area and crops, same id', () async {
    final api = FakeApi()..assumeOnline = true;
    await api.verifyOtp(phone: '+9647700000000', code: '123456');
    final t = DateTime.utc(2026, 10, 8);
    List<GeoPoint> square(double size) => [
      for (final (e, n) in [(0.0, 0.0), (size, 0.0), (size, size), (0.0, size)])
        () {
          final p = Utm.toLatLng(539600 + e, 3935300 + n);
          return GeoPoint(lat: p.latitude, lon: p.longitude, accM: 5, t: t);
        }(),
    ];
    final made = await api.createFarm(
      NewFarmRequest(
        name: 'Field',
        points: square(100),
        cells: const [],
        createdOfflineAt: t,
      ),
    );
    final id = made.farm.summary.id;
    expect(made.farm.summary.areaDunam * 2500, closeTo(10000, 1));

    final big = square(150);
    final cells = [
      for (final c in cellsTouching([
        for (final p in big) LatLng(p.lat, p.lon),
      ]).keys)
        CellCrop(e: c.e, n: c.n, crop: 'wheat'),
    ];
    final edited = await api.updateFarm(
      id,
      NewFarmRequest(
        name: 'Field, bigger',
        points: big,
        cells: cells,
        createdOfflineAt: t,
      ),
    );
    expect(edited.farm.summary.id, id);
    expect(edited.farm.summary.areaDunam * 2500, closeTo(22500, 1));
    expect(edited.farm.summary.crops.single.crop, 'wheat');
    final list = await api.getFarms();
    expect(list.where((f) => f.id == id).single.name, 'Field, bigger');
  });

  test(
    'editing a demo farm replaces it in the list, not a copy beside it',
    () async {
      final api = FakeApi()..assumeOnline = true;
      await api.verifyOtp(phone: '+9647701234567', code: '123456');
      final before = await api.getFarms();
      final demo = before.first;
      final farm = await api.getFarm(demo.id);
      final t = DateTime.utc(2026, 10, 8);
      await api.updateFarm(
        demo.id,
        NewFarmRequest(
          name: 'Renamed demo',
          points: [
            for (final p in farm.outline)
              GeoPoint(lat: p.lat, lon: p.lon, accM: 0, t: t),
          ],
          cells: [
            for (final c in farm.cells) CellCrop(e: c.e, n: c.n, crop: c.crop),
          ],
          createdOfflineAt: t,
        ),
      );
      final after = await api.getFarms();
      expect(after.length, before.length);
      expect(after.where((f) => f.id == demo.id).single.name, 'Renamed demo');
    },
  );

  test('editing another phone\'s or a missing farm is not_found', () async {
    final api = FakeApi()..assumeOnline = true;
    await api.verifyOtp(phone: '+9647700000000', code: '123456');
    final t = DateTime.utc(2026, 10, 8);
    expect(
      () => api.updateFarm(
        'f_nope',
        NewFarmRequest(
          name: 'x',
          points: const [],
          cells: const [],
          createdOfflineAt: t,
        ),
      ),
      throwsA(isA<ApiException>().having((e) => e.code, 'code', 'not_found')),
    );
  });

  test('a deleted farm, demo or made here, is gone from the list', () async {
    final api = FakeApi()..assumeOnline = true;
    await api.verifyOtp(phone: '+9647709876543', code: '123456');
    final before = await api.getFarms();
    await api.deleteFarm(before.first.id);
    final after = await api.getFarms();
    expect(after.length, before.length - 1);
    expect(after.any((f) => f.id == before.first.id), isFalse);
    expect(
      () => api.deleteFarm(before.first.id),
      throwsA(isA<ApiException>().having((e) => e.code, 'code', 'not_found')),
    );
  });
}
