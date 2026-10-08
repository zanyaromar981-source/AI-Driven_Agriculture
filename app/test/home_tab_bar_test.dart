import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/api/api.dart';
import 'package:jutyar/api/fake_api.dart';
import 'package:jutyar/app_scope.dart';
import 'package:jutyar/l10n/strings.dart';
import 'package:jutyar/screens/home/home_screen.dart';

void main() {
  const farm = FarmSummary(
    id: 'f_tabbar',
    name: 'Test farm',
    areaDunam: 1,
    crops: [],
    status: FarmStatus.none,
  );

  // The emulator that showed "BOTTOM OVERFLOWED BY 5.0 PIXELS" under
  // "Ask the Doctor": 1080 x 2400 at 420 dpi. Also checked with large text.
  for (final scale in [1.0, 1.3, 1.6]) {
    testWidgets('Home tab bar fits at text size $scale', (t) async {
      t.view.physicalSize = const Size(1080, 2400);
      t.view.devicePixelRatio = 2.625;
      addTearDown(t.view.reset);
      final errors = <String>[];
      final old = FlutterError.onError;
      FlutterError.onError = (d) => errors.add(d.toString());
      await t.pumpWidget(
        AppScope(
          ku: true,
          s: const S(true),
          api: FakeApi()..assumeOnline = true,
          setKu: (_) {},
          child: MaterialApp(
            builder: (c, child) => MediaQuery(
              data: MediaQuery.of(
                c,
              ).copyWith(textScaler: TextScaler.linear(scale)),
              child: child!,
            ),
            home: const HomeScreen(farm: farm),
          ),
        ),
      );
      await t.pump();
      // Let the farm's load finish (the fake API answers after 600 ms).
      await t.pump(const Duration(seconds: 1));
      FlutterError.onError = old;
      // Only Home's own layout is checked here (the shared header has its own owner).
      expect(errors.where((e) => e.contains('home_screen.dart')), isEmpty);
      expect(find.text('Ask the Doctor'), findsOneWidget);
    });
  }
}
