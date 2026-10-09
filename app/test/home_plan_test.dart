import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/api/api.dart';
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

/// The plan route answers with [error], as the test server does today
/// (FRONTEND.md 4: `GET /v1/farms/{id}/plan` is not built and answers 404).
class _NoPlan extends FakeApi {
  _NoPlan(this.error);
  final ApiException error;

  @override
  Future<FarmPlan> getPlan(String id) async => throw error;
}

void main() {
  setUpAll(() {
    final dir = Directory.systemTemp.createTempSync('jutyar_plan_');
    PathProviderPlatform.instance = _TempPaths(dir.path);
  });

  /// [which] picks a demo farm; each test opens its own, so no test sees a
  /// copy of the farm saved on the phone by another.
  Future<void> openHome(WidgetTester t, FakeApi api, int which) async {
    t.view.physicalSize = const Size(1080, 2400);
    t.view.devicePixelRatio = 2.625;
    addTearDown(t.view.reset);
    api.assumeOnline = true;
    final farms = (await t.runAsync(() async {
      await api.verifyOtp(phone: '+9647501234567', code: '123456');
      return api.getFarms();
    }))!;
    await t.pumpWidget(
      AppScope(
        ku: false,
        s: const S(false),
        api: api,
        setKu: (_) {},
        child: MaterialApp(home: HomeScreen(farm: farms[which])),
      ),
    );
    for (var i = 0; i < 8; i++) {
      await t.pump(const Duration(seconds: 1));
      await t.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 50)),
      );
    }
  }

  testWidgets('plan not built (404): a calm "coming soon", no error', (
    t,
  ) async {
    await openHome(t, _NoPlan(ApiException(404, 'not_found')), 0);
    expect(
      find.text('10-day plan coming soon', skipOffstage: false),
      findsOneWidget,
    );
    expect(
      find.text(
        'Weather forecast not available right now',
        skipOffstage: false,
      ),
      findsNothing,
    );
    expect(find.byType(CircularProgressIndicator), findsNothing);
  });

  testWidgets('plan down (503): still says the forecast is not available', (
    t,
  ) async {
    await openHome(t, _NoPlan(ApiException(503, 'upstream_down')), 1);
    expect(
      find.text(
        'Weather forecast not available right now',
        skipOffstage: false,
      ),
      findsOneWidget,
    );
    expect(
      find.text('10-day plan coming soon', skipOffstage: false),
      findsNothing,
    );
  });
}
