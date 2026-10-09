import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/api/api.dart';
import 'package:jutyar/screens/history/field_history_screen.dart';
import 'package:jutyar/store/insights_copy.dart';
import 'package:jutyar/store/local_store.dart';
import 'package:path_provider_platform_interface/path_provider_platform_interface.dart';
import 'package:plugin_platform_interface/plugin_platform_interface.dart';

class _TempPaths extends PathProviderPlatform with MockPlatformInterfaceMixin {
  _TempPaths(this.dir);
  final String dir;

  @override
  Future<String?> getApplicationSupportPath() async => dir;
}

/// A server whose analysis has only the given topics so far.
class _Partial implements Api {
  _Partial(this.topics);
  final List<String> topics;

  @override
  Future<FarmInsights> getInsights(String id) async =>
      FarmInsights.fromJson(_insights(topics, 'new'));

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

Map<String, dynamic> _insights(List<String> topics, String from) => {
  'topics': [
    for (final t in topics)
      {'topic': t, 'summary_en': '$t from $from', 'measures': const []},
  ],
};

String _summary(FarmInsights f, String topic) => f.topic(topic)!.summaryEn;

void main() {
  late Directory dir;
  setUpAll(() async {
    dir = await Directory.systemTemp.createTemp('jutyar_insights');
    PathProviderPlatform.instance = _TempPaths(dir.path);
  });
  tearDownAll(() => dir.delete(recursive: true));

  test('an edit keeps the field history on show until the new analysis '
      'has the same topics, and says which parts are old', () async {
    await LocalStore.write(InsightsCopy.name('f_old'), {
      'data': _insights(FarmInsights.all, 'old'),
    });
    await InsightsCopy.carry('f_old', 'f_new');

    // Nothing from the server yet: the old history is shown, all of it.
    final loader = InsightsLoader(_Partial(const []), 'f_new');
    final cached = await loader.cached();
    expect(cached!.ready, FarmInsights.all.length);
    expect(loader.carried, FarmInsights.all.toSet());

    // Rain and weather are in: those are new, the rest still old.
    final half = InsightsLoader(_Partial(const ['rain', 'weather']), 'f_new');
    final f = await half.fresh();
    expect(f.ready, FarmInsights.all.length);
    expect(_summary(f, 'rain'), 'rain from new');
    expect(_summary(f, 'soil'), 'soil from old');
    expect(half.carried, {'greenness', 'soil', 'dryness'});
    expect(half.keepChecking, isTrue, reason: 'old parts still to replace');

    // All five are in: nothing old is left on the phone.
    final full = InsightsLoader(_Partial(FarmInsights.all), 'f_new');
    final done = await full.fresh();
    expect(_summary(done, 'soil'), 'soil from new');
    expect(full.carried, isEmpty);
    expect(full.keepChecking, isFalse);
    final copy = await LocalStore.read(InsightsCopy.name('f_new'));
    expect(copy?.containsKey('carried'), isFalse);
  });

  test('a farm with no history yet carries nothing over', () async {
    await InsightsCopy.carry('f_none', 'f_none2');
    expect(await LocalStore.read(InsightsCopy.name('f_none2')), isNull);
  });

  test('after 20 minutes unfinished the analysis counts as stuck, '
      'and the screens stop checking every 30 s', () async {
    final start = DateTime.utc(2026, 10, 9, 12);
    final copy = {'waiting_since': start.toIso8601String()};
    expect(
      InsightsCopy.stalled(copy, now: start.add(const Duration(minutes: 19))),
      isFalse,
    );
    expect(
      InsightsCopy.stalled(copy, now: start.add(const Duration(minutes: 21))),
      isTrue,
    );
    expect(InsightsCopy.stalled(const {}), isFalse);

    final halfHourAgo = DateTime.now().toUtc().subtract(
      const Duration(minutes: 30),
    );
    await LocalStore.write(InsightsCopy.name('f_stuck'), {
      'data': _insights(const ['rain'], 'server'),
      'waiting_since': halfHourAgo.toIso8601String(),
    });
    final loader = InsightsLoader(_Partial(const ['rain']), 'f_stuck');
    await loader.fresh();
    expect(loader.stalled, isTrue, reason: 'the first wait is kept');
    expect(loader.keepChecking, isFalse);
  });
}
