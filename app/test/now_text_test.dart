import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/api/api.dart';
import 'package:jutyar/screens/history/now_text.dart';

FarmInsights withNow(Map<String, double> m) => FarmInsights(
  topics: [
    InsightTopic(
      topic: 'greenness',
      asOf: '2026-03-06',
      source: '',
      confidence: 'sure',
      summaryEn: '',
      measures: [
        for (final e in m.entries)
          InsightMeasure(code: e.key, value: e.value, unit: '', labelEn: e.key),
      ],
    ),
  ],
  json: const {},
);

void main() {
  test('no reading yet: waiting', () {
    expect(nowView(null, today: DateTime(2026, 10, 9)).kind, NowKind.waiting);
    expect(
      nowView(withNow({}), today: DateTime(2026, 10, 9)).kind,
      NowKind.waiting,
    );
  });

  test('bare in October is normal before sowing', () {
    final v = nowView(
      withNow({
        'now_picture_year': 2026,
        'now_picture_month': 10,
        'now_picture_day': 6,
        'now_stage': 0,
        'now_normal_ndvi': 0.15,
      }),
      today: DateTime(2026, 10, 9),
    );
    expect(v.kind, NowKind.bare);
    expect(v.title, 'Bare soil');
    expect(v.date, '6 Oct');
    expect(v.line, contains('normal before sowing'));
  });

  test('bare in March when the field is usually green is flagged', () {
    final v = nowView(
      withNow({
        'now_picture_year': 2026,
        'now_picture_month': 3,
        'now_picture_day': 6,
        'now_stage': 0,
        'now_normal_ndvi': 0.45,
      }),
      today: DateTime(2026, 3, 9),
    );
    expect(v.kind, NowKind.behind);
    expect(
      v.line,
      contains('in most years this field is green by early March'),
    );
  });

  test('growing well with a small part behind in the north-east', () {
    final v = nowView(
      withNow({
        'now_picture_year': 2026,
        'now_picture_month': 3,
        'now_picture_day': 6,
        'now_stage': 2,
        'now_pct_of_normal': 108,
        'now_behind_share_pct': 8,
        'now_behind_dx_m': 40,
        'now_behind_dy_m': 35,
      }),
      today: DateTime(2026, 3, 9),
    );
    expect(v.kind, NowKind.growing);
    expect(v.title, 'Growing well');
    expect(
      v.line,
      'About as green as this field usually is in early March. A small part is behind, in the north-east.',
    );
    expect(v.chips, [
      ('108%', 'of usual for early March'),
      ('8%', 'of the field behind'),
      ('NE', 'where it is behind'),
    ]);
  });

  test('most of the field behind is not one spot', () {
    final v = nowView(
      withNow({
        'now_picture_year': 2026,
        'now_picture_month': 3,
        'now_picture_day': 25,
        'now_stage': 2,
        'now_pct_of_normal': 74,
        'now_behind_share_pct': 61,
        'now_behind_dx_m': 2,
        'now_behind_dy_m': -3,
      }),
      today: DateTime(2026, 3, 27),
    );
    expect(v.kind, NowKind.behind);
    expect(v.title, 'Behind usual');
    expect(
      v.line,
      startsWith(
        'Less green than this field usually is in late March. Most of the field is behind',
      ),
    );
    expect(v.chips.last, ('All', 'across the field'));
  });

  test('an old picture means clouds since then', () {
    final v = nowView(
      withNow({
        'now_picture_year': 2026,
        'now_picture_month': 2,
        'now_picture_day': 18,
        'now_stage': 2,
        'now_pct_of_normal': 101,
        'now_behind_share_pct': 0,
      }),
      today: DateTime(2026, 3, 6),
    );
    expect(v.kind, NowKind.cloudy);
    expect(v.line, contains('16 days ago: growing well then'));
  });

  test('directions', () {
    expect(direction(0, 10), 'north');
    expect(direction(10, 10), 'north-east');
    expect(direction(-10, -10), 'south-west');
  });
}
