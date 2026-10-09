import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/api/fake_api.dart';
import 'package:jutyar/app_scope.dart';
import 'package:jutyar/l10n/strings.dart';
import 'package:jutyar/screens/home/home_screen.dart';
import 'package:path_provider_platform_interface/path_provider_platform_interface.dart';
import 'package:plugin_platform_interface/plugin_platform_interface.dart';

/// The phone's private folder, as a temp folder (same idea as local_store_test).
class _TempPaths extends PathProviderPlatform with MockPlatformInterfaceMixin {
  _TempPaths(this.path);
  final String path;
  @override
  Future<String?> getApplicationSupportPath() async => path;
}

// Picture taken, wheat not growing yet: the crop list keeps "not sown yet".
// Own file: LocalStore's per-file queue is static, so it must not share an isolate
// with home_no_reading_test.dart (same farm, same cache file).
void main() {
  setUpAll(() {
    final dir = Directory.systemTemp.createTempSync('jutyar_home_');
    PathProviderPlatform.instance = _TempPaths(dir.path);
  });

  Future<String> cropsText(WidgetTester t, FakeApi api) async {
    t.view.physicalSize = const Size(1080, 2400);
    t.view.devicePixelRatio = 2.625;
    addTearDown(t.view.reset);
    api.assumeOnline = true;
    final farms = (await t.runAsync(() async {
      await api.verifyOtp(phone: '+9647501234567', code: '123456');
      return api.getFarms();
    }))!;
    final upper = farms.firstWhere((f) => f.id == 'f_01HXUPPERFIELD');
    await t.pumpWidget(
      AppScope(
        ku: false,
        s: const S(false),
        api: api,
        setKu: (_) {},
        child: MaterialApp(home: HomeScreen(farm: upper)),
      ),
    );
    for (var i = 0; i < 8; i++) {
      await t.pump(const Duration(seconds: 1));
      await t.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 50)),
      );
    }
    await t.tap(find.text('Crops'));
    await t.pump();
    return t
        .widgetList<Text>(find.byType(Text))
        .map((w) => w.data ?? '')
        .where((d) => d.contains(' m² · '))
        .join('\n');
  }

  testWidgets('picture taken, wheat not growing: still "not sown yet"', (
    t,
  ) async {
    final text = await cropsText(t, FakeApi());
    expect(text, contains('not sown yet'));
    expect(text, isNot(contains('no reading yet')));
  });
}
