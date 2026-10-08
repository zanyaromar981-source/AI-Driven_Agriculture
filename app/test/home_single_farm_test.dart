import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/api/api.dart';
import 'package:jutyar/api/fake_api.dart';
import 'package:jutyar/app_scope.dart';
import 'package:jutyar/l10n/strings.dart';
import 'package:jutyar/screens/home/farm_section.dart';
import 'package:jutyar/screens/home/home_screen.dart';

// Each farm opens on its own screen (user, 2026-10-08: "every farm should be
// opened separately, not a window that has all the maps stacked").
void main() {
  late FakeApi api;
  late List<FarmSummary> farms;

  Future<void> signIn(WidgetTester t) async {
    t.view.physicalSize = const Size(1080, 2400);
    t.view.devicePixelRatio = 2.625;
    addTearDown(t.view.reset);
    api = FakeApi()..assumeOnline = true;
    farms = (await t.runAsync(() async {
      await api.verifyOtp(phone: '+9647501234567', code: '123456');
      return api.getFarms();
    }))!;
    expect(farms.length, greaterThan(1));
  }

  Widget app(Widget home) => AppScope(
    ku: false,
    s: const S(false),
    api: api,
    setKu: (_) {},
    child: MaterialApp(home: home),
  );

  /// Long enough for the push and for the farm's load to finish.
  Future<void> settle(WidgetTester t) async {
    for (var i = 0; i < 5; i++) {
      await t.pump(const Duration(seconds: 1));
    }
  }

  testWidgets('a farm opens alone: one FarmSection, no other farm', (t) async {
    await signIn(t);
    final farm = farms[1];
    await t.pumpWidget(app(HomeScreen(farm: farm)));
    await settle(t);

    expect(find.byType(FarmSection), findsOneWidget);
    expect(find.text(farm.name), findsOneWidget);
    for (final f in farms.where((f) => f.name != farm.name)) {
      expect(find.text(f.name), findsNothing);
    }

    // The end of the page is the footer, with no second farm before it.
    await t.scrollUntilVisible(
      find.text('Farming assistant · Jutyar'),
      400,
      scrollable: find.byType(Scrollable).first,
    );
    expect(find.byType(FarmSection), findsOneWidget);
  });

  Future<void> open(WidgetTester t) async {
    await t.tap(find.text('open'));
    await settle(t);
    expect(find.byType(HomeScreen), findsOneWidget);
  }

  testWidgets('back arrow, Home tab and system back return to My farms', (
    t,
  ) async {
    await signIn(t);
    final farm = farms.first;
    await t.pumpWidget(
      app(
        Builder(
          builder: (c) => Scaffold(
            body: Center(
              child: TextButton(
                onPressed: () => Navigator.of(c).push(
                  MaterialPageRoute<void>(
                    builder: (_) => HomeScreen(farm: farm),
                  ),
                ),
                child: const Text('open'),
              ),
            ),
          ),
        ),
      ),
    );

    await open(t);
    await t.tap(find.text('My farms'));
    await settle(t);
    expect(find.byType(HomeScreen), findsNothing);
    expect(find.text('open'), findsOneWidget);

    await open(t);
    await t.tap(find.text('Home'));
    await settle(t);
    expect(find.byType(HomeScreen), findsNothing);

    await open(t);
    await t.binding.handlePopRoute(); // Android back gesture
    await settle(t);
    expect(find.byType(HomeScreen), findsNothing);
  });
}
