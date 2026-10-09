import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/api/api.dart';
import 'package:jutyar/store/draft.dart';
import 'package:jutyar/store/outbox.dart';
import 'package:path_provider_platform_interface/path_provider_platform_interface.dart';
import 'package:plugin_platform_interface/plugin_platform_interface.dart';

class _TempPaths extends PathProviderPlatform with MockPlatformInterfaceMixin {
  _TempPaths(this.dir);
  final String dir;

  @override
  Future<String?> getApplicationSupportPath() async => dir;
}

/// An Api whose createFarm always fails with the given status. [extra] is
/// what the server's JSON error carried besides its code (null: no JSON).
class _Failing implements Api {
  _Failing(this.status, this.code, [this.extra]);
  final int status;
  final String code;
  final Map<String, dynamic>? extra;

  @override
  Future<CreateFarmResult> createFarm(
    NewFarmRequest request, {
    String? idempotencyKey,
  }) => Future.error(ApiException(status, code, extra));

  @override
  Future<CreateFarmResult> updateFarm(
    String id,
    NewFarmRequest request, {
    String? idempotencyKey,
  }) => Future.error(ApiException(status, code, extra));

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

/// Records which call the outbox made.
class _Recorder implements Api {
  final calls = <String>[];

  @override
  Future<CreateFarmResult> createFarm(
    NewFarmRequest request, {
    String? idempotencyKey,
  }) {
    calls.add('create ${request.name}');
    final id = 'new_${calls.length}';
    return Future.value(
      CreateFarmResult(
        farm: Farm.fromJson({
          'id': id,
          'name': request.name,
          'area_dunam': 1,
          'crops': const [],
          'status': 'none',
          'outline': const [],
          'cells': const [],
        }),
        droppedCells: 0,
      ),
    );
  }

  @override
  Future<CreateFarmResult> updateFarm(
    String id,
    NewFarmRequest request, {
    String? idempotencyKey,
  }) {
    calls.add('update $id ${request.name}');
    return Future.error(ApiException(0, 'offline'));
  }

  /// Deletes succeed, so the queue moves on to the rest.
  @override
  Future<void> deleteFarm(String id) async => calls.add('delete $id');

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

NewFarmRequest _req(String name) => NewFarmRequest(
  name: name,
  points: [
    GeoPoint(lat: 36.0, lon: 44.0, accM: 5, t: DateTime.utc(2026, 10, 8)),
    GeoPoint(lat: 36.001, lon: 44.0, accM: 5, t: DateTime.utc(2026, 10, 8)),
    GeoPoint(lat: 36.001, lon: 44.001, accM: 5, t: DateTime.utc(2026, 10, 8)),
  ],
  cells: const [],
  createdOfflineAt: DateTime.utc(2026, 10, 8),
);

const _summary = FarmSummary(
  id: 'local',
  name: 'x',
  areaDunam: 1,
  crops: [],
  status: FarmStatus.none,
);

void main() {
  // One folder for the whole file: LocalStore remembers the folder it first saw.
  late Directory dir;
  setUpAll(() async {
    dir = await Directory.systemTemp.createTemp('jutyar_outbox');
    PathProviderPlatform.instance = _TempPaths(dir.path);
  });
  tearDownAll(() => dir.delete(recursive: true));

  test(
    'a busy server (429) or a timeout (408) keeps the farm for later',
    () async {
      final box = Outbox.instance;
      await box.add(_req('Busy farm'), _summary);
      for (final (status, code) in [
        (429, 'rate_limited'),
        (405, 'http_405'),
        (408, 'timeout'),
        (503, 'upstream_down'),
        (0, 'offline'),
      ]) {
        await box.flush(_Failing(status, code));
        expect(
          box.items.map((i) => i.request.name),
          contains('Busy farm'),
          reason: '$status must keep the farm',
        );
      }
    },
  );

  test(
    'a Wi-Fi login page or a used-up data bundle never drops a farm',
    () async {
      final box = Outbox.instance;
      await box.add(_req('Portal farm'), _summary);
      // What HttpApi makes of answers that are not our server's JSON.
      for (final (status, code) in [
        (200, 'bad_response'), // a web page instead of JSON
        (302, 'http_302'), // a redirect to a login page
        (403, 'http_403'),
        (422, 'invalid'), // a 422 with no JSON error body
        (404, 'not_found'),
      ]) {
        final r = await box.flush(_Failing(status, code));
        expect(r.rejected, isEmpty, reason: '$status $code is not a refusal');
        expect(
          box.items.map((i) => i.request.name),
          contains('Portal farm'),
          reason: '$status $code must keep the farm',
        );
      }
    },
  );

  test('a farm the server can never accept (422) is reported with its reason '
      'and its walked edge goes back to Add farm', () async {
    final box = Outbox.instance;
    await box.flush(_Failing(422, 'x')); // clear what earlier tests left
    await Draft.clear();
    final before = box.items.length;
    final bad = await box.add(_req('Bad farm'), _summary);
    box.takeNews();
    final r = await box.flush(
      _Failing(422, 'bad_polygon', {'detail': 'encloses no cells'}),
    );
    final mine = r.rejected.where((x) => x.item.key == bad.key).single;
    expect(mine.code, 'bad_polygon');
    expect(mine.edgeBack, isTrue);
    expect(box.items.map((i) => i.request.name), isNot(contains('Bad farm')));
    expect(box.items.length, lessThanOrEqualTo(before));
    final draft = await Draft.load();
    expect(draft?.points.length, 3, reason: 'the walk is not lost');
    final news = box.takeNews();
    expect(news.sent, 0, reason: 'a refusal is not an upload');
    expect(news.refused.map((x) => x.item.key), contains(bad.key));
    await Draft.clear();
  });

  test('only farms that really went up count as uploaded', () async {
    final box = Outbox.instance;
    await box.flush(_Failing(422, 'x')); // clear what earlier tests left
    await box.add(_req('Good farm'), _summary);
    box.takeNews();
    final r = await box.flush(_Recorder());
    expect(r.sent, 1);
    expect(box.takeNews().sent, 1);
    expect(box.takeNews().sent, 0, reason: 'told once');
  });

  test(
    'an edit is sent as an edit, and a newer edit replaces an older one',
    () async {
      final box = Outbox.instance;
      await box.flush(_Failing(422, 'x')); // clear what earlier tests left
      await box.add(_req('Edit 1'), _summary, farmId: 'f_7');
      await box.add(_req('Edit 2'), _summary, farmId: 'f_7');
      expect(box.items.where((i) => i.farmId == 'f_7').length, 1);
      final rec = _Recorder();
      await box.flush(rec);
      // No edit call on the server yet: the farm is made again, the old one deleted.
      expect(rec.calls, ['create Edit 2', 'delete f_7']);
      expect(box.replacedBy('f_7'), 'new_1');
      expect(box.deletes, isNot(contains('f_7')));
    },
  );

  test('a delete goes first and drops any waiting edit of that farm', () async {
    final box = Outbox.instance;
    await box.flush(_Failing(422, 'x'));
    await box.add(_req('Edit of f_9'), _summary, farmId: 'f_9');
    await box.add(_req('New farm'), _summary);
    await box.delete('f_9');
    expect(box.items.any((i) => i.farmId == 'f_9'), isFalse);
    final rec = _Recorder();
    await box.flush(rec);
    expect(rec.calls, ['delete f_9', 'create New farm']);
    expect(box.deletes, isEmpty);
  });
}
