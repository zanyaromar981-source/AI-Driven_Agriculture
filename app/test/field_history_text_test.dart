import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/api/api.dart';
import 'package:jutyar/screens/history/crop_fit.dart';
import 'package:jutyar/screens/history/history_text.dart';

/// The real answer of GET /v1/farms/2/insights (test field near Erbil, 9 Oct 2026),
/// the same text the demo server returns.
FarmInsights farm2() {
  final src = File('lib/api/fake_api.dart').readAsStringSync();
  final m = RegExp(
    r"const _demoInsights = r'''\n([\s\S]*?)\n''';",
  ).firstMatch(src)!;
  return FarmInsights.fromJson(jsonDecode(m.group(1)!) as Map<String, dynamic>);
}

void main() {
  final f = farm2();

  test('all five topics are read', () {
    expect(f.ready, 5);
    expect(dataUpTo(f), '3 Oct 2026');
  });

  test('rain is worded as a plain comparison', () {
    final v = rainView(f.topic('rain'))!;
    expect(
      v.summary,
      contains(
        'In 2025/26 this area got 638 mm of rain, 60% more than the 1991-2020 average of 399 mm.',
      ),
    );
    expect(
      v.summary,
      contains('12 dry seasons (under 80% of average), the latest in 2024/25'),
    );
    expect(v.chips, [
      ('399 mm', 'average season'),
      ('+60%', 'in 2025/26'),
      ('12', 'dry seasons since 1981'),
    ]);
    expect(v.evidence, Evidence.area);
  });

  test('frost uses the usual last frost and the hard frost seasons', () {
    final v = frostView(f.topic('weather'))!;
    expect(v.summary, contains('usually around 19 February'));
    expect(
      v.summary,
      contains(
        'hard spring frost came in 4 seasons since 1981, the latest 2011/12',
      ),
    );
    expect(v.chips.map((c) => c.$1), ['17', '19 Feb', '21']);
  });

  test('greenness says it is not yield and finds no weak spot', () {
    final v = greennessView(f.topic('greenness'))!;
    expect(v.summary, contains('This is greenness, not yield.'));
    expect(
      v.summary,
      contains('No weaker spot was found in 10 seasons of 10 m pictures.'),
    );
    expect(v.chips, [
      ('42', 'seasons seen'),
      ('147%', 'of usual, in 2025/26'),
      ('0', 'weak spots found'),
    ]);
    expect(v.evidence, Evidence.measured);
  });

  test('soil is an estimate in plain words', () {
    final v = soilView(f.topic('soil'))!;
    expect(
      v.summary,
      startsWith(
        'Probably heavy soil: about 39% clay and 22% sand, slightly alkaline (pH 7.5).',
      ),
    );
    expect(v.summary, contains('not a test of your field'));
    expect(v.evidence, Evidence.notTested);
  });

  test('dryness names the dry year that stayed green, and the fires', () {
    final v = drynessView(
      f.topic('dryness'),
      rain: f.topic('rain'),
      green: f.topic('greenness'),
    )!;
    expect(
      v.summary,
      contains('but not always: 2024/25 was dry and still one of the greenest'),
    );
    expect(v.summary, contains('3 times within about 1 km (2007, 2013, 2024)'));
    expect(v.chips, [
      ('-23%', 'green in dry seasons'),
      ('4 of 41', 'summers green'),
      ('3', 'fire detections'),
    ]);
  });

  test('crop fit for heavy clay with about 400 mm', () {
    final g = cropFit(f);
    expect(g.map((x) => x.kind), ['fits', 'irrigation', 'poor']);
    expect(g[0].crops, ['wheat', 'barley', 'chickpea?']);
    expect(g[1].crops, ['tomato', 'cucumber']);
    expect(g[2].crops, ['potato', 'onion', 'watermelon']);
  });

  test('suggestions come from the field history, no doses', () {
    final s = suggestions(f);
    expect(s.length, 3);
    expect(s[0], contains('barley copes better with a dry year'));
    expect(s[1], contains('Hard spring frost came 4 times'));
    expect(s.join(' '), isNot(contains('kg')));
  });

  test('a farm with no topics yet gives no cards and no crop fit', () {
    final empty = FarmInsights.fromJson(const {'topics': []});
    expect(topicViews(empty), isEmpty);
    expect(cropFit(empty), isEmpty);
    expect(empty.ready, 0);
  });
}
