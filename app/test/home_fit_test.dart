import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/api/api.dart';
import 'package:jutyar/api/fake_api.dart';
import 'package:jutyar/app_scope.dart';
import 'package:jutyar/l10n/strings.dart';
import 'package:jutyar/screens/history/crop_fit.dart';
import 'package:jutyar/screens/history/field_history_screen.dart';

/// The demo server's field history (same text as lib/api/fake_api.dart).
Map<String, dynamic> demoJson() {
  final src = File('lib/api/fake_api.dart').readAsStringSync();
  final m = RegExp(
    r"const _demoInsights = r'''\n([\s\S]*?)\n''';",
  ).firstMatch(src)!;
  return jsonDecode(m.group(1)!) as Map<String, dynamic>;
}

/// The demo history with some topics taken out.
FarmInsights without(Set<String> drop) {
  final j = demoJson();
  return FarmInsights.fromJson({
    ...j,
    'topics': [
      for (final t in j['topics'] as List)
        if (!drop.contains((t as Map)['topic'])) t,
    ],
  });
}

// "Which crops fit my field" on the farm's Home (user, 2026-10-10).
// Home feeds HomeFitCard from FieldHistoryEntry, whose file cache does not
// finish inside a widget test, so the card is tested directly.
void main() {
  Widget app(Widget home) => AppScope(
    ku: false,
    s: const S(false),
    api: FakeApi(),
    setKu: (_) {},
    child: MaterialApp(
      home: Scaffold(body: SingleChildScrollView(child: home)),
    ),
  );

  testWidgets('Home card leads with what is grown in the area', (t) async {
    await t.pumpWidget(app(HomeFitCard(data: without(const {}))));
    expect(find.text('What fits this field'), findsOneWidget);
    expect(find.text('Grown in your area'), findsOneWidget);
    expect(find.text('Rice'), findsOneWidget);
    expect(find.text('1300 ha, irrigated'), findsOneWidget);
    expect(find.text('5200 ha'), findsOneWidget);
    expect(find.textContaining('MapSPAM 2020'), findsOneWidget);
    expect(find.text('Suits your field'), findsOneWidget);
    expect(find.text('Usually fits, rain-fed'), findsOneWidget);
    expect(find.text('Only with irrigation'), findsOneWidget);
  });

  testWidgets('without crops_grown the card shows only the rule', (t) async {
    await t.pumpWidget(app(HomeFitCard(data: without(const {'crops_grown'}))));
    expect(find.text('Grown in your area'), findsNothing);
    expect(find.text('Suits your field'), findsNothing);
    expect(find.text('Usually fits, rain-fed'), findsOneWidget);
  });

  testWidgets('soil not read yet: rain still gives advice', (t) async {
    final f = without(const {'soil', 'crops_grown'});
    final fit = cropFit(f);
    expect(fit, isNotEmpty);
    expect(fit.first.kind, 'fits');
    expect(fit.first.crops, containsAll(['wheat', 'barley']));
    expect(fit.first.why, contains('soil is not read yet'));
    expect(fit.map((g) => g.kind), contains('irrigation'));
    await t.pumpWidget(app(HomeFitCard(data: f)));
    expect(find.text('Usually fits, rain-fed'), findsOneWidget);
    expect(find.textContaining('soil is not read yet'), findsOneWidget);
  });

  test('rain not read yet: soil alone, said honestly', () {
    final fit = cropFit(without(const {'rain'}));
    expect(fit, isNotEmpty);
    expect(fit.first.why, contains('rain here is not read yet'));
    expect(cropFit(without(const {'rain', 'soil'})), isEmpty);
  });

  testWidgets('before the history is in, the card waits calmly', (t) async {
    await t.pumpWidget(app(const HomeFitCard(data: null)));
    expect(find.text('What fits this field'), findsOneWidget);
    expect(find.textContaining('Ready when the soil and rain'), findsOneWidget);
    await t.pumpWidget(
      app(HomeFitCard(data: FarmInsights.fromJson(const {'topics': []}))),
    );
    expect(find.textContaining('Ready when the soil and rain'), findsOneWidget);
  });

  test('crops_grown is kept but not counted in the five topics', () {
    final f = without(const {});
    expect(f.topic('crops_grown'), isNotNull);
    expect(f.ready, 5);
    expect(grownInArea(f).map((c) => c.code), [
      'wheat',
      'barley',
      'rice',
      'tomato',
    ]);
  });
}
