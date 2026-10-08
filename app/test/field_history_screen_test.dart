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

void main() {
  testWidgets('Field history shows the five topics, crop fit and suggestions', (
    tester,
  ) async {
    final dir = Directory.systemTemp.createTempSync('jutyar_history');
    PathProviderPlatform.instance = _TempPaths(dir.path);
    tester.view.physicalSize = const Size(1080, 2400);
    tester.view.devicePixelRatio = 2.625;
    addTearDown(tester.view.reset);

    final api = FakeApi()..assumeOnline = true;
    late final FarmSummary farm;
    await tester.runAsync(() async {
      await api.verifyOtp(phone: '+9647701234567', code: '123456');
      farm = (await api.getFarms()).first;
    });
    await tester.pumpWidget(
      AppScope(
        ku: true,
        s: const S(true),
        api: api,
        setKu: (_) {},
        child: MaterialApp(home: FieldHistoryScreen(farm: farm)),
      ),
    );
    // File reads need real time; the demo server's delay needs test time.
    for (var i = 0; i < 12; i++) {
      await tester.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 100)),
      );
      await tester.pump(const Duration(milliseconds: 300));
    }

    expect(find.text('Field history'), findsOneWidget);
    expect(find.textContaining('data up to 3 Oct 2026'), findsOneWidget);
    expect(find.text('What fits this field'), findsOneWidget);
    expect(find.text('Usually fits, rain-fed'), findsOneWidget);
    expect(find.text('Chickpea ?'), findsOneWidget);
    expect(find.text('Rule of thumb'), findsOneWidget);

    final list = find.byType(Scrollable).first;
    for (final title in [
      'Rain',
      'Frost and heat',
      'Greenness',
      'Soil and land',
      'Dryness and fire',
    ]) {
      await tester.scrollUntilVisible(find.text(title), 300, scrollable: list);
      expect(find.text(title), findsOneWidget);
    }
    expect(find.text('Reading'), findsNothing);
    expect(tester.takeException(), isNull);
    dir.deleteSync(recursive: true);
  });
}
