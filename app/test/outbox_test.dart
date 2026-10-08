import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/api/api.dart';
import 'package:jutyar/store/outbox.dart';
import 'package:path_provider_platform_interface/path_provider_platform_interface.dart';
import 'package:plugin_platform_interface/plugin_platform_interface.dart';

class _TempPaths extends PathProviderPlatform with MockPlatformInterfaceMixin {
  _TempPaths(this.dir);
  final String dir;

  @override
  Future<String?> getApplicationSupportPath() async => dir;
}

/// An Api whose createFarm always fails with the given status.
class _Failing implements Api {
  _Failing(this.status, this.code);
  final int status;
  final String code;

  @override
  Future<CreateFarmResult> createFarm(
    NewFarmRequest request, {
    String? idempotencyKey,
  }) => Future.error(ApiException(status, code));

  @override
  Future<CreateFarmResult> updateFarm(
    String id,
    NewFarmRequest request, {
    String? idempotencyKey,
  }) => Future.error(ApiException(status, code));

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
    return Future.error(ApiException(0, 'offline'));
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
    'a farm the server can never accept (422) is dropped and reported',
    () async {
      final box = Outbox.instance;
      final before = box.items.length;
      await box.add(_req('Bad farm'), _summary);
      final r = await box.flush(_Failing(422, 'bad_polygon'));
      expect(r.rejected, contains('Bad farm'));
      expect(box.items.map((i) => i.request.name), isNot(contains('Bad farm')));
      expect(box.items.length, lessThanOrEqualTo(before));
    },
  );

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
      expect(rec.calls.first, 'update f_7 Edit 2');
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
