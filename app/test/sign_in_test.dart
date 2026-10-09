import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/api/api.dart';
import 'package:jutyar/api/fake_api.dart';
import 'package:jutyar/app_scope.dart';
import 'package:jutyar/l10n/strings.dart';
import 'package:jutyar/phone.dart';
import 'package:jutyar/screens/phone_screen.dart';

/// Answers every send with the given server error.
class _Refuses extends FakeApi {
  _Refuses(this.error);
  final ApiException error;

  @override
  Future<OtpSendResult> sendOtp({
    required String phone,
    required String lang,
  }) async => throw error;
}

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

  Future<void> sendWith(
    WidgetTester t,
    ApiException e, {
    bool ku = false,
  }) async {
    await pumpPhone(t, _Refuses(e), ku: ku);
    await t.enterText(find.byType(TextField), '0750 123 4567');
    await t.pump();
    await t.tap(find.byType(FilledButton));
    await t.pump();
    await t.pump(const Duration(milliseconds: 300));
  }

  testWidgets('429: a countdown from retry_after_s, then Send again', (
    t,
  ) async {
    await sendWith(t, ApiException(429, 'rate_limited', {'retry_after_s': 42}));
    expect(find.text('You can ask for a new code in 0:42'), findsOneWidget);
    expect(send(t), isNull);
    await t.pump(const Duration(seconds: 2));
    expect(find.text('You can ask for a new code in 0:40'), findsOneWidget);
    await t.pump(const Duration(seconds: 40));
    expect(find.textContaining('ask for a new code'), findsNothing);
    expect(send(t), isNotNull);
  });

  testWidgets('503 upstream_down: try again at once', (t) async {
    await sendWith(t, ApiException(503, 'upstream_down'));
    expect(find.text('Could not send the code. Try again.'), findsOneWidget);
    expect(find.textContaining('upstream_down'), findsNothing);
    expect(send(t), isNotNull);
  });

  testWidgets('422 invalid: check the number, in Sorani too', (t) async {
    await sendWith(t, ApiException(422, 'invalid', {'field': 'phone'}));
    expect(find.text('Check the number'), findsOneWidget);
    expect(find.textContaining('invalid'), findsNothing);
    await t.pump(const Duration(seconds: 3));
    await sendWith(t, ApiException(422, 'invalid'), ku: true);
    expect(find.text('ژمارەکە بپشکنە'), findsOneWidget);
  });

  test('countdown text', () {
    expect(mmss(42), '0:42');
    expect(mmss(60), '1:00');
    expect(mmss(5), '0:05');
  });
}
