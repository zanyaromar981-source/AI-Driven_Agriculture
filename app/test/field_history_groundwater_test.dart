import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/api/api.dart';
import 'package:jutyar/api/fake_api.dart';
import 'package:jutyar/app_scope.dart';
import 'package:jutyar/l10n/strings.dart';
import 'package:jutyar/screens/history/field_history_screen.dart';
import 'package:path_provider_platform_interface/path_provider_platform_interface.dart';
import 'package:plugin_platform_interface/plugin_platform_interface.dart';

class _TempPaths extends PathProviderPlatform with MockPlatformInterfaceMixin {
  _TempPaths(this.dir);
  final String dir;

  @override
  Future<String?> getApplicationSupportPath() async => dir;
}

/// The demo field history plus the server's daily `groundwater` topic.
class _WithGroundwater extends FakeApi {
  @override
  Future<FarmInsights> getInsights(String id) async {
    final f = await super.getInsights(id);
    return FarmInsights.fromJson({
      ...f.json,
      'topics': [
        ...f.json['topics'] as List,
        {
          'topic': 'groundwater',
          'as_of': '2026-10-06',
          'source': 'NASA GRACE-DA weekly percentiles, 25 km model cell',
          'confidence': 'unsure',
          'summary_en': 'Groundwater in the 25 km area is about usual.',
          'summary_ku': null,
          'measures': [
            {
              'code': 'groundwater_percentile',
              'value': 48.2,
              'unit': 'percentile',
              'label_en': 'Groundwater storage',
              'label_ku': null,
            },
          ],
        },
      ],
    });
  }
}

void main() {
  testWidgets('Field history shows the groundwater card when the server has it', (
    tester,
  ) async {
    final dir = Directory.systemTemp.createTempSync('jutyar_groundwater');
    PathProviderPlatform.instance = _TempPaths(dir.path);
    tester.view.physicalSize = const Size(1080, 2400);
    tester.view.devicePixelRatio = 2.625;
    addTearDown(tester.view.reset);

    final api = _WithGroundwater()..assumeOnline = true;
    late final FarmSummary farm;
    await tester.runAsync(() async {
      await api.verifyOtp(phone: '+9647701234567', code: '123456');
      farm = (await api.getFarms()).first;
    });
    await tester.pumpWidget(
      AppScope(
        ku: false,
        s: const S(false),
        api: api,
        setKu: (_) {},
        child: MaterialApp(home: FieldHistoryScreen(farm: farm)),
      ),
    );
    for (var i = 0; i < 12; i++) {
      await tester.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 100)),
      );
      await tester.pump(const Duration(milliseconds: 300));
    }

    final list = find.byType(Scrollable).first;
    await tester.scrollUntilVisible(
      find.text('Groundwater, wider area'),
      300,
      scrollable: list,
    );
    await tester.scrollUntilVisible(
      find.textContaining('as of 6 Oct 2026'),
      100,
      scrollable: list,
    );
    expect(find.textContaining('in the wider area'), findsOneWidget);
    expect(
      find.text(
        'NASA GRACE-DA weekly percentiles, 25 km model cell · as of 6 Oct 2026',
      ),
      findsOneWidget,
    );
    expect(find.textContaining('your well'), findsNothing);
    // The five topics are all in: nothing is still shown as reading.
    expect(find.text('Reading'), findsNothing);
    expect(tester.takeException(), isNull);
    dir.deleteSync(recursive: true);
  });
}
