import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/api/fake_api.dart';
import 'package:jutyar/app_scope.dart';
import 'package:jutyar/l10n/strings.dart';
import 'package:jutyar/widgets/header.dart';

/// The header must fit on small and large phones with large system text,
/// in both languages, without cutting off the language switch.
void main() {
  for (final width in [320.0, 360.0, 411.0]) {
    for (final scale in [1.0, 1.3, 1.6, 2.0]) {
      for (final ku in [true, false]) {
        testWidgets(
          'header fits: ${width.toInt()} wide, text x$scale, ${ku ? 'Sorani' : 'English'}',
          (tester) async {
            tester.view.physicalSize = Size(width * 2.625, 900 * 2.625);
            tester.view.devicePixelRatio = 2.625;
            addTearDown(tester.view.reset);
            await tester.pumpWidget(
              AppScope(
                ku: ku,
                s: S(ku),
                api: FakeApi()..assumeOnline = true,
                setKu: (_) {},
                child: MaterialApp(
                  home: MediaQuery(
                    data: MediaQueryData(
                      size: Size(width, 900),
                      textScaler: TextScaler.linear(scale),
                    ),
                    child: const Scaffold(
                      body: SafeArea(child: JutyarHeader()),
                    ),
                  ),
                ),
              ),
            );
            expect(tester.takeException(), isNull);
            final toggle = tester.getRect(find.byType(LanguageToggle));
            expect(
              toggle.right,
              lessThanOrEqualTo(width - 19.9),
              reason: 'switch inside the screen',
            );
            expect(find.text('کوردی'), findsOneWidget);
          },
        );
      }
    }
  }
}
