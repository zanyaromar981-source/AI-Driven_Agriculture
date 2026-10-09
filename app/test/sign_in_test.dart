import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/api/api.dart';
import 'package:jutyar/api/fake_api.dart';
import 'package:jutyar/app_scope.dart';
import 'package:jutyar/l10n/strings.dart';
import 'package:jutyar/screens/phone_screen.dart';

void main() {
  Future<void> pumpPhone(WidgetTester t, Api api, {bool ku = false}) async {
    t.view.physicalSize = const Size(1080, 2400);
    t.view.devicePixelRatio = 2.625;
    addTearDown(t.view.reset);
    await t.pumpWidget(
      AppScope(
        ku: ku,
        s: S(ku),
        api: api,
        setKu: (_) {},
        child: const MaterialApp(home: PhoneScreen()),
      ),
    );
  }

  VoidCallback? send(WidgetTester t) =>
      t.widget<FilledButton>(find.byType(FilledButton)).onPressed;

  testWidgets('the phone number is checked, also in test mode', (t) async {
    await pumpPhone(t, FakeApi()..assumeOnline = true);
    expect(send(t), isNull, reason: 'empty number');
    await t.enterText(find.byType(TextField), '123');
    await t.pump();
    expect(send(t), isNull, reason: 'too short');
    await t.enterText(find.byType(TextField), '0650 123 4567');
    await t.pump();
    expect(send(t), isNull, reason: 'not a mobile number');
    await t.enterText(find.byType(TextField), '0750 123 4567');
    await t.pump();
    expect(send(t), isNotNull);
    await t.enterText(find.byType(TextField), '750 123 4567');
    await t.pump();
    expect(send(t), isNotNull, reason: '+964 is already shown');
  });
}
