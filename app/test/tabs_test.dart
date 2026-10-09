import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/api/api.dart';
import 'package:jutyar/api/fake_api.dart';
import 'package:jutyar/app_scope.dart';
import 'package:jutyar/l10n/strings.dart';
import 'package:jutyar/screens/alerts/alerts_screen.dart';
import 'package:jutyar/screens/home/farm_drawing.dart';
import 'package:jutyar/screens/home/home_screen.dart';
import 'package:jutyar/screens/report/report_screen.dart';
import 'package:jutyar/screens/settings/settings_screen.dart';

/// The demo server, but the alerts route is not built (as on the real one).
class _NoAlerts extends FakeApi {
  @override
  Future<List<FarmAlert>> getAlerts(String farmId) async =>
      throw ApiException(404, 'not_found');
}

void main() {
  late FakeApi api;
  late List<FarmSummary> farms;

  Future<void> signIn(WidgetTester t, [FakeApi? with_]) async {
    t.view.physicalSize = const Size(1080, 2400);
    t.view.devicePixelRatio = 2.625;
    addTearDown(t.view.reset);
    api = (with_ ?? FakeApi())..assumeOnline = true;
    farms = (await t.runAsync(() async {
      await api.verifyOtp(phone: '+9647501234567', code: '123456');
      return api.getFarms();
    }))!;
  }

  Widget app(Widget home) => AppScope(
    ku: false,
    s: const S(false),
    api: api,
    setKu: (_) {},
    child: MaterialApp(home: home),
  );

  Future<void> settle(WidgetTester t) async {
    for (var i = 0; i < 4; i++) {
      await t.pump(const Duration(seconds: 1));
    }
  }

  test('a report line reads like the design', () {
    final line = reportLine(
      FarmerMessage(
        id: '1',
        kind: 'report',
        text: 'Yellow stripes, square E12: since Monday',
        state: 'read',
        createdAt: DateTime(2026, 10, 5, 9),
      ),
    );
    expect(line, '5 Oct · Yellow stripes · Cell E12 · Seen by officer');
    expect(
      const NewReport(farmId: '1', type: 'Insects', square: 'B4').text,
      'Insects, square B4',
    );
  });

  testWidgets('tab bar: Alerts, Settings, then Home returns to the farm', (
    t,
  ) async {
    await signIn(t);
    await t.pumpWidget(app(HomeScreen(farm: farms.first)));
    await settle(t);
    for (final tab in [
      'Home',
      'Alerts',
      'Ask the Doctor',
      'Alwa',
      'Settings',
    ]) {
      expect(find.text(tab), findsOneWidget, reason: tab);
    }

    await t.tap(find.text('Alerts'));
    await settle(t);
    expect(find.byType(AlertsScreen), findsOneWidget);
    expect(find.text('Frost -3 tonight'), findsOneWidget);
    expect(
      find.text('Red was sent to your phone. Yellow is only here.'),
      findsOneWidget,
    );
    expect(find.textContaining('TODAY, '), findsOneWidget);
    expect(find.text('YESTERDAY'), findsOneWidget);

    await t.tap(find.text('Settings'));
    await settle(t);
    expect(find.byType(SettingsScreen), findsOneWidget);
    expect(
      find.byType(AlertsScreen),
      findsNothing,
      reason: 'one tab at a time',
    );
    expect(find.text('+964 750 123 4567'), findsOneWidget);
    expect(find.text('${farms.length} farms'), findsOneWidget);

    await t.tap(find.text('Home'));
    await settle(t);
    expect(find.byType(SettingsScreen), findsNothing);
    expect(find.byType(HomeScreen), findsOneWidget);
    expect(t.takeException(), isNull);
  });

  testWidgets('Alerts: a server without alerts says so calmly', (t) async {
    await signIn(t, _NoAlerts());
    await t.pumpWidget(app(Scaffold(body: AlertsScreen(farm: farms.first))));
    await settle(t);
    expect(find.textContaining('Alerts are coming soon'), findsOneWidget);
  });

  testWidgets('Report: needs a type, then sends and lists it', (t) async {
    await signIn(t);
    final farm = (await t.runAsync(() => api.getFarm(farms.first.id)))!;
    final shape = FarmShape(farm, null);
    final cell = shape.crop.keys.first;
    await t.pumpWidget(
      app(ReportScreen(farm: farms.first, shape: shape, cell: cell)),
    );
    await settle(t);
    expect(find.text('Report a problem'), findsOneWidget);
    expect(
      find.text('Cell ${shape.label(cell)} · tap to change'),
      findsOneWidget,
    );

    await t.tap(find.text('Insects'));
    await t.pump();
    final list = find.byType(Scrollable).first;
    await t.scrollUntilVisible(find.byType(TextField), 200, scrollable: list);
    await t.enterText(find.byType(TextField), 'small green bugs');
    await t.scrollUntilVisible(find.text('Send report'), 200, scrollable: list);
    await t.tap(find.text('Send report'));
    await settle(t);

    final mine = (await t.runAsync(api.getMyMessages))!;
    expect(
      mine.first.text,
      'Insects, square ${shape.label(cell)}: small green bugs',
    );
    await t.scrollUntilVisible(
      find.textContaining('Insects · Cell ${shape.label(cell)} · Sent'),
      200,
      scrollable: list,
    );
    // Map tiles have no file store in tests; their errors are not ours.
    t.takeException();
  });
}
