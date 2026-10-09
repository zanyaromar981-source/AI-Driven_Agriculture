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

  test('groundwater: the wider area, never a well, with source and date', () {
    final f = FarmInsights.fromJson({
      'topics': [
        {
          'topic': 'groundwater',
          'as_of': '2026-10-06',
          'source':
              'NASA GRACE-DA weekly percentiles, 25 km model cell; not a well depth, not measured at the farm',
          'confidence': 'unsure',
          'summary_en':
              'Groundwater in the 25 km area around this farm is lower than usual for this time of year. This is a satellite and model estimate for the wider area, not a measurement of your well.',
          'summary_ku': null,
          'measures': [
            for (final (code, value, label) in const [
              ('groundwater_percentile', 22.4, 'Groundwater storage'),
              (
                'root_zone_moisture_percentile',
                41.0,
                'Soil moisture, root zone',
              ),
              ('surface_moisture_percentile', 63.5, 'Soil moisture, top layer'),
            ])
              {
                'code': code,
                'value': value,
                'unit': 'percentile',
                'label_en': label,
                'label_ku': null,
              },
          ],
        },
      ],
    });
    expect(
      f.ready,
      0,
      reason: 'not one of the five topics the analysis waits for',
    );
    final views = topicViews(f);
    expect(views.map((v) => v.key), ['groundwater']);
    final v = views.single;
    expect(v.title, 'Groundwater, wider area');
    expect(v.evidence, Evidence.area);
    expect(v.summary, contains('in the wider area'));
    expect(
      v.summary,
      contains('is lower than usual for this time of year (22,'),
    );
    expect(v.summary.toLowerCase(), isNot(contains('your well')));
    expect(v.chips, [
      ('22', 'groundwater, 50 is usual'),
      ('41', 'root-zone moisture'),
      ('64', 'top-layer moisture'),
    ]);
    expect(v.source, startsWith('NASA GRACE-DA'));
    expect(v.source, endsWith(' · as of 6 Oct 2026'));
    expect(v.all.first, ('Groundwater storage', '22 percentile'));
    expect(groundwaterWords(5), 'much lower than usual');
    expect(groundwaterWords(50), 'about usual');
    expect(groundwaterWords(95), 'much higher than usual');
  });

  group('new measures (live since 9 Oct 2026) win over the summary text', () {
    final v2 = FarmInsights.fromJson(
      jsonDecode(
            File(
              'test/fixtures/insights_farm2_measures.json',
            ).readAsStringSync(),
          )
          as Map<String, dynamic>,
    );

    test('rain and frost read the measures', () {
      expect(latestDroughts(v2.topic('rain')), ['2024/25']);
      expect(lastFrost(v2.topic('weather')), (2, 19));
      expect(hardFrost(v2.topic('weather')), (4, '2011/12'));
      expect(
        frostView(v2.topic('weather'))!.summary,
        contains(
          'hard spring frost came in 4 seasons since 1981, the latest 2011/12',
        ),
      );
    });

    InsightTopic topic(
      String name,
      Map<String, double> m, {
      String summary = '',
    }) => InsightTopic(
      topic: name,
      asOf: '2026-10-09',
      source: '',
      confidence: 'likely',
      summaryEn: summary,
      measures: [
        for (final e in m.entries)
          InsightMeasure(code: e.key, value: e.value, unit: '', labelEn: e.key),
      ],
    );

    test('no hard spring frost ever: no frost warning, no suggestion', () {
      final w = topic('weather', {
        'frost_days_normal': 9,
        'last_spring_frost_month': 1,
        'last_spring_frost_day': 30,
        'hard_spring_frost_seasons': 0,
      });
      expect(hardFrost(w), (0, null));
      expect(
        frostView(w)!.summary,
        'About 9 frost nights a season. The last spring frost is usually around 30 January.',
      );
      final f = FarmInsights(topics: [w], json: const {});
      expect(suggestions(f).any((x) => x.contains('frost')), isFalse);
    });

    test('best seasons from measures name the dry year that stayed green', () {
      final green = topic('greenness', {
        'best_season_1': 2025,
        'best_season_2': 2024,
        'best_season_3': 2015,
        'weak_spots_10m_seasons': 8,
      });
      final rain = topic('rain', {'latest_drought_season': 2024});
      final dry = topic('dryness', {
        'peak_ndvi_in_droughts': 0.4,
        'peak_ndvi_in_wet_seasons': 0.6,
        'summer_green_seasons': 2,
        'summer_seasons_seen': 40,
        'fire_detections': 0,
      });
      expect(bestSeasons(green), ['2025/26', '2024/25', '2015/16']);
      final v = drynessView(dry, rain: rain, green: green)!;
      expect(
        v.summary,
        contains(
          'but not always: 2024/25 was dry and still one of the greenest',
        ),
      );
      expect(v.summary, contains('No fire was seen within about 1 km.'));
      expect(v.chips[1], ('2 of 40', 'summers green'));
    });
  });
}
