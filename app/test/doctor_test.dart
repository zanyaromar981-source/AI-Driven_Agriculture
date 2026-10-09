
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/api/api.dart';
import 'package:jutyar/api/fake_api.dart';
import 'package:jutyar/app_scope.dart';
import 'package:jutyar/l10n/strings.dart';
import 'package:jutyar/screens/doctor/ask_doctor_screen.dart';
import 'package:jutyar/screens/doctor/doctor_answer_screen.dart';

const _farm = FarmSummary(
  id: '5',
  name: 'Koya wheat',
  areaDunam: 7.8,
  crops: [],
  status: FarmStatus.none,
);

const _answerJson = {
  'likely': 'Too dry to sow this week',
  'confidence': 'likely',
  'why': [
    'weather_planner -> 4 mm of rain in the next 10 days',
    'field_history -> 1 season in 4 was dry here',
    'a plain line with no input',
  ],
  'actions_this_week': ['Wait for 20 mm', 'Check the seed', 'Fix the drill'],
  'cannot_tell': ['When the big rain comes'],
  'refer_to_officer': false,
  'ku': 'ئەم هەفتەیە زۆر وشکە بۆ چاندن.',
  'en': 'Too dry to sow this week.',
  'inputs_used': ['weather_planner', 'field_history'],
};

/// The demo server with a scripted Doctor: answers, or fails with [fail].
class _Doctor extends FakeApi {
  _Doctor({this.fail});
  final ApiException? fail;
  final asked = <DoctorQuestion>[];

  @override
  Future<DoctorAnswer> askDoctor(String farmId, DoctorQuestion q) async {
    asked.add(q);
    if (fail != null) throw fail!;
    return DoctorAnswer.fromJson(_answerJson);
  }
}

Widget _app(Api api, Widget home) => AppScope(
  ku: true,
  s: const S(true),
  api: api,
  setKu: (_) {},
  child: MaterialApp(home: home),
);

void main() {
  group('DoctorAnswer', () {
    test('keeps only what the contract allows', () {
      final a = DoctorAnswer.fromJson({
        'likely': 'x',
        'confidence': 'certain',
        'why': ['a', '', '  '],
        'actions_this_week': ['1', '2', '3', '4'],
        'refer_to_officer': 'yes',
      });
      expect(a.confidence, 'unsure', reason: 'unknown confidence is unsure');
      expect(a.why, ['a']);
      expect(a.actions, ['1', '2', '3']);
      expect(a.cannotTell, isEmpty);
      expect(a.referToOfficer, isFalse, reason: 'only a real true counts');
    });

    test('names the input behind each reason', () {
      expect(
        whyParts('weather_planner -> rust weather').$2,
        'Weather: rust weather',
      );
      expect(
        whyParts('field_eye -> 12 cells weak').$2,
        'Field eye: 12 cells weak',
      );
      expect(whyParts('Your photo -> stripes').$2, 'Your photo: stripes');
      expect(whyParts('field_history -> dry').$2, 'Field history: dry');
      expect(whyParts('no arrow here').$2, 'no arrow here');
    });
  });

  testWidgets('Ask: send waits for words or a photo, then shows the answer', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(1080, 2400);
    tester.view.devicePixelRatio = 2.625;
    addTearDown(tester.view.reset);
    final api = _Doctor();
    await tester.pumpWidget(_app(api, const AskDoctorScreen(farm: _farm)));

    final send = find.widgetWithText(InkWell, 'Send to the Doctor');
    await tester.tap(find.text('Send to the Doctor'));
    await tester.pump();
    expect(api.asked, isEmpty, reason: 'nothing to send yet');
    expect(send, findsWidgets);

    await tester.enterText(find.byType(TextField), 'Is it time to sow?');
    await tester.pump();
    await tester.tap(find.text('Send to the Doctor'));
    await tester.pumpAndSettle();

    expect(api.asked.single.text, 'Is it time to sow?');
    expect(api.asked.single.lang, 'ku');
    expect(find.text('The Doctor says'), findsOneWidget);
    expect(find.text('Too dry to sow this week'), findsOneWidget);
    expect(find.text('How sure: likely'), findsOneWidget);
    expect(find.text('ئەم هەفتەیە زۆر وشکە بۆ چاندن.'), findsOneWidget);
    // .last: the first is the header's language switch.
    await tester.tap(find.text('English').last);
    await tester.pump();
    expect(find.text('Too dry to sow this week.'), findsOneWidget);
    final list = find.byType(Scrollable).first;
    await tester.scrollUntilVisible(
      find.text('Weather: 4 mm of rain in the next 10 days'),
      200,
      scrollable: list,
    );
    await tester.scrollUntilVisible(
      find.text('Wait for 20 mm'),
      200,
      scrollable: list,
    );
    await tester.scrollUntilVisible(
      find.text('When the big rain comes'),
      200,
      scrollable: list,
    );
    expect(find.text('When the big rain comes'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets(
    'Ask: a Doctor that is not switched on says so and keeps the question',
    (tester) async {
      tester.view.physicalSize = const Size(1080, 2400);
      tester.view.devicePixelRatio = 2.625;
      addTearDown(tester.view.reset);
      final api = _Doctor(fail: ApiException(503, 'doctor_not_ready'));
      await tester.pumpWidget(_app(api, const AskDoctorScreen(farm: _farm)));
      await tester.enterText(find.byType(TextField), 'Yellow leaves');
      await tester.pump();
      await tester.tap(find.text('Send to the Doctor'));
      await tester.pumpAndSettle();

      expect(find.textContaining('not switched on yet'), findsOneWidget);
      expect(find.text('Yellow leaves'), findsOneWidget);
      expect(find.text('The Doctor says'), findsNothing);
    },
  );

  testWidgets(
    'Ask about one square sends that square, and can switch to the whole farm',
    (tester) async {
      tester.view.physicalSize = const Size(1080, 2400);
      tester.view.devicePixelRatio = 2.625;
      addTearDown(tester.view.reset);
      final api = _Doctor();
      await tester.pumpWidget(
        _app(
          api,
          const AskDoctorScreen(
            farm: _farm,
            cell: (e: 1, n: 2),
            cellLabel: 'B7',
          ),
        ),
      );
      expect(find.text('About square B7'), findsOneWidget);
      await tester.enterText(find.byType(TextField), 'Here?');
      await tester.pump();
      await tester.tap(find.text('Send to the Doctor'));
      await tester.pumpAndSettle();
      expect((api.asked.single.cellE, api.asked.single.cellN), (1, 2));

      await tester.tap(find.text('Ask again'));
      await tester.pumpAndSettle();
      await tester.tap(find.byIcon(Icons.close_rounded));
      await tester.pump();
      expect(find.text('About Koya wheat'), findsOneWidget);
      await tester.tap(find.text('Send to the Doctor'));
      await tester.pumpAndSettle();
      expect(api.asked.last.cellE, isNull);
    },
  );

  testWidgets('Answer: an unsure answer sends the farmer to the office', (
    tester,
  ) async {
    await tester.pumpWidget(
      _app(
        FakeApi(),
        DoctorAnswerScreen(
          farm: _farm,
          answer: DoctorAnswer.fromJson({
            ..._answerJson,
            'confidence': 'unsure',
            'refer_to_officer': true,
          }),
        ),
      ),
    );
    final list = find.byType(Scrollable).first;
    await tester.scrollUntilVisible(
      find.textContaining('plant-protection office'),
      200,
      scrollable: list,
    );
    expect(find.textContaining('plant-protection office'), findsOneWidget);
  });
}
