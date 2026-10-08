import 'dart:math' as math;

import '../../api/api.dart';

/// How a topic's numbers were obtained, shown as a small label on its card
/// (instead of sure / likely / unsure, after the Codex review of 9 Oct 2026).
enum Evidence { measured, area, notTested, derived, ruleOfThumb }

/// One card of the Field history screen, worded for a farmer.
class TopicView {
  const TopicView({
    required this.key,
    required this.title,
    required this.evidence,
    required this.summary,
    required this.chips,
    required this.source,
    required this.all,
  });

  /// Backend topic name: rain, weather, greenness, soil, dryness.
  final String key;
  final String title;
  final Evidence evidence;
  final String summary;

  /// Up to three (value, label) pairs.
  final List<(String, String)> chips;
  final String source;

  /// Every number of the topic for "All numbers": (label, value with unit).
  final List<(String, String)> all;
}

String _num(double v, {int digits = 0}) {
  final s = v.toStringAsFixed(digits);
  if (!s.contains('.')) return s;
  final t = s.replaceAll(RegExp(r'0+$'), '');
  return t.endsWith('.') ? t.substring(0, t.length - 1) : t;
}

String _season(String? label) =>
    RegExp(r'(\d{4}/\d{2})').firstMatch(label ?? '')?.group(1) ?? 'last season';

String? _firstYear(String? label) =>
    RegExp(r'(\d{4})').firstMatch(label ?? '')?.group(1);

const _months = [
  'January',
  'February',
  'March',
  'April',
  'May',
  'June',
  'July',
  'August',
  'September',
  'October',
  'November',
  'December',
];
const _short = [
  'Jan',
  'Feb',
  'Mar',
  'Apr',
  'May',
  'Jun',
  'Jul',
  'Aug',
  'Sep',
  'Oct',
  'Nov',
  'Dec',
];

List<(String, String)> _allNumbers(InsightTopic t) => [
  for (final m in t.measures)
    if (m.value != null)
      (
        m.labelEn,
        '${_num(m.value!, digits: m.value!.abs() < 10 ? 2 : 0)} ${m.unit}'
            .trim(),
      ),
];

/// Seasons named as dry in the rain summary, e.g. "latest: 2020/21, 2024/25".
List<String> latestDroughts(InsightTopic? rain) {
  final m = RegExp(r'latest: ([^)]*)\)').firstMatch(rain?.summaryEn ?? '');
  if (m == null) return const [];
  return RegExp(
    r'\d{4}/\d{2}',
  ).allMatches(m.group(1)!).map((x) => x.group(0)!).toList();
}

TopicView? rainView(InsightTopic? t) {
  if (t == null) return null;
  final normal = t.m('normal_mm_oct_may');
  final last = t.m('last_season_mm');
  final pct = t.m('last_season_pct_of_normal');
  final droughts = t.m('drought_seasons');
  final season = _season(t.label('last_season_mm'));
  final since = _firstYear(t.label('drought_seasons')) ?? '1981';
  final parts = <String>[];
  if (last != null && pct != null && normal != null) {
    final diff = (pct - 100).round();
    final how = diff == 0
        ? 'about the same as'
        : diff > 0
        ? '$diff% more than'
        : '${-diff}% less than';
    parts.add(
      'In $season this area got ${_num(last)} mm of rain, $how the 1991-2020 average of ${_num(normal)} mm.',
    );
  }
  if (droughts != null) {
    final latest = latestDroughts(t);
    parts.add(
      'Since $since there were ${_num(droughts)} dry seasons (under 80% of average)'
      '${latest.isEmpty ? '' : ', the latest in ${latest.last}'}.',
    );
  }
  return TopicView(
    key: 'rain',
    title: 'Rain',
    evidence: Evidence.area,
    summary: parts.isEmpty ? t.summaryEn : parts.join(' '),
    chips: [
      if (normal != null) ('${_num(normal)} mm', 'average season'),
      if (pct != null)
        ('${pct >= 100 ? '+' : ''}${_num(pct - 100)}%', 'in $season'),
      if (droughts != null) (_num(droughts), 'dry seasons since $since'),
    ],
    source:
        'ERA5 rain for the area around the field (about 25 km wide), daily since $since',
    all: _allNumbers(t),
  );
}

/// "The last spring frost usually falls around 02-19" -> (2, 19).
(int, int)? lastFrost(InsightTopic? weather) {
  final m = RegExp(
    r'last spring frost usually falls around (\d\d)-(\d\d)',
  ).firstMatch(weather?.summaryEn ?? '');
  if (m == null) return null;
  return (int.parse(m.group(1)!), int.parse(m.group(2)!));
}

/// Seasons with hard spring frost named in the weather summary.
List<String> hardFrostSeasons(InsightTopic? weather) {
  final m = RegExp(r'came in: ([^.]*)\.').firstMatch(weather?.summaryEn ?? '');
  if (m == null) return const [];
  return RegExp(
    r'\d{4}/\d{2}',
  ).allMatches(m.group(1)!).map((x) => x.group(0)!).toList();
}

TopicView? frostView(InsightTopic? t) {
  if (t == null) return null;
  final frost = t.m('frost_days_normal');
  final heat = t.m('spring_heat_days_normal');
  final heatTrend = t.m('spring_heat_days_trend_per_decade') ?? 0;
  final lf = lastFrost(t);
  final hard = hardFrostSeasons(t);
  final parts = <String>[];
  if (frost != null) parts.add('About ${_num(frost)} frost nights a season.');
  if (lf != null) {
    final when = '${lf.$2} ${_months[lf.$1 - 1]}';
    parts.add(
      hard.isEmpty
          ? 'The last spring frost is usually around $when.'
          : 'The last spring frost is usually around $when, but hard spring frost came in ${hard.length} seasons since 1981, the latest ${hard.last}.',
    );
  }
  if (heat != null) {
    final trend = heatTrend > 0.5
        ? ', slowly rising'
        : heatTrend < -0.5
        ? ', slowly falling'
        : '';
    parts.add(
      'About ${_num(heat)} days of 31 °C or more in April and May$trend.',
    );
  }
  return TopicView(
    key: 'weather',
    title: 'Frost and heat',
    evidence: Evidence.area,
    summary: parts.isEmpty ? t.summaryEn : parts.join(' '),
    chips: [
      if (frost != null) (_num(frost), 'frost nights a season'),
      if (lf != null) ('${lf.$2} ${_short[lf.$1 - 1]}', 'usual last frost'),
      if (heat != null) (_num(heat), 'hot days in Apr-May'),
    ],
    source:
        'ERA5-Land for the area around the field (about 9 km wide), daily since 1981',
    all: _allNumbers(t),
  );
}

/// Best seasons named in the greenness summary, e.g. "Best seasons: 2024/25, 2015/16".
List<String> bestSeasons(InsightTopic? green) {
  final m = RegExp(
    r'Best seasons: ([^.]*)\.',
  ).firstMatch(green?.summaryEn ?? '');
  if (m == null) return const [];
  return RegExp(
    r'\d{4}/\d{2}',
  ).allMatches(m.group(1)!).map((x) => x.group(0)!).toList();
}

TopicView? greennessView(InsightTopic? t) {
  if (t == null) return null;
  final seasons = t.m('seasons_measured');
  final pct = t.m('last_season_pct_of_normal');
  final weak = t.m('weak_share_pct');
  final season = _season(t.label('last_season_pct_of_normal'));
  final tenM = RegExp(
    r'10 m pixels over (\d+) seasons',
  ).firstMatch(t.summaryEn)?.group(1);
  final parts = <String>[];
  if (seasons != null) {
    parts.add('Seen from space in ${_num(seasons)} seasons since 1984.');
  }
  if (pct != null) {
    final how = pct >= 110
        ? 'greener than in its usual spring'
        : pct <= 90
        ? 'less green than in its usual spring'
        : 'about as green as in its usual spring';
    parts.add('In $season the field was $how. This is greenness, not yield.');
  }
  if (weak != null) {
    parts.add(
      weak < 1
          ? 'No weaker spot was found${tenM == null ? '' : ' in $tenM seasons of 10 m pictures'}.'
          : 'About ${_num(weak)}% of the field is weaker almost every season.',
    );
  }
  return TopicView(
    key: 'greenness',
    title: 'Greenness',
    evidence: Evidence.measured,
    summary: parts.isEmpty ? t.summaryEn : parts.join(' '),
    chips: [
      if (seasons != null) (_num(seasons), 'seasons seen'),
      if (pct != null) ('${_num(pct)}%', 'of usual, in $season'),
      if (weak != null)
        (
          weak < 1 ? '0' : '${_num(weak)}%',
          weak < 1 ? 'weak spots found' : 'weak almost every season',
        ),
    ],
    source:
        'Landsat 30 m since 1984 and Sentinel-2 10 m since 2017. In 30 m pictures a small field mixes with its edges.',
    all: _allNumbers(t),
  );
}

String phWords(double ph) => ph < 6.5
    ? 'slightly acid'
    : ph < 7.3
    ? 'neutral'
    : ph <= 8.0
    ? 'slightly alkaline'
    : 'alkaline';

/// heavy / medium / light, from the topsoil clay and sand shares.
String soilClass(double? clay, double? sand) {
  if (clay == null) return 'unknown';
  if (clay >= 35) return 'heavy';
  if ((sand ?? 0) >= 50 && clay < 20) return 'light';
  return 'medium';
}

TopicView? soilView(InsightTopic? t) {
  if (t == null) return null;
  final clay = t.m('clay_pct_topsoil');
  final sand = t.m('sand_pct_topsoil');
  final ph = t.m('ph_topsoil');
  final slope = t.m('slope_deg');
  final height = t.m('elevation_m');
  final cls = soilClass(clay, sand);
  final parts = <String>[];
  if (clay != null) {
    parts.add(
      'Probably $cls soil: about ${_num(clay)}% clay${sand == null ? '' : ' and ${_num(sand)}% sand'}'
      '${ph == null ? '' : ', ${phWords(ph)} (pH ${_num(ph, digits: 1)})'}.',
    );
  }
  if (slope != null || height != null) {
    parts.add(
      '${slope != null && slope < 2
          ? 'Flat'
          : slope != null
          ? 'Sloping (${_num(slope)}°)'
          : 'Height'}'
      '${height == null ? '' : ', at about ${_num(height)} m'}.',
    );
  }
  parts.add('This comes from a 250 m soil map, not a test of your field.');
  return TopicView(
    key: 'soil',
    title: 'Soil and land',
    evidence: Evidence.notTested,
    summary: parts.join(' '),
    chips: [
      if (clay != null) ('${_num(clay)}%', 'clay'),
      if (ph != null) ('pH ${_num(ph, digits: 1)}', phWords(ph)),
      if (slope != null)
        (
          slope < 2 ? 'Flat' : '${_num(slope)}°',
          '${_num(slope, digits: 1)}° slope',
        ),
    ],
    source:
        'SoilGrids map (250 m, modelled, not sampled) · Copernicus height 30 m',
    all: _allNumbers(t),
  );
}

TopicView? drynessView(
  InsightTopic? t, {
  InsightTopic? rain,
  InsightTopic? green,
}) {
  if (t == null) return null;
  final dry = t.m('peak_ndvi_in_droughts');
  final wet = t.m('peak_ndvi_in_wet_seasons');
  final summerGreen = t.m('summer_green_seasons');
  final fires = t.m('fire_detections');
  final summers = RegExp(
    r'in \d+ of (\d+) seasons',
  ).firstMatch(t.summaryEn)?.group(1);
  final fireYears = RegExp(
    r'\(years ([^)]*)\)',
  ).firstMatch(t.summaryEn)?.group(1);
  final parts = <String>[];
  double? diff;
  if (dry != null && wet != null && wet > 0) {
    diff = (dry - wet) / wet * 100;
    final exceptions = [
      for (final s in latestDroughts(rain))
        if (bestSeasons(green).contains(s)) s,
    ];
    final base = dry < wet
        ? 'In dry seasons the field was usually less green in spring'
        : 'Dry seasons did not make the field less green in spring';
    parts.add(
      exceptions.isEmpty || dry >= wet
          ? '$base.'
          : '$base, but not always: ${exceptions.first} was dry and still one of the greenest.',
    );
  }
  if (fires != null) {
    parts.add(
      fires < 1
          ? 'No fire was seen within about 1 km.'
          : 'Satellites saw heat from fire ${_num(fires)} ${fires == 1 ? 'time' : 'times'} within about 1 km'
                '${fireYears == null ? '' : ' ($fireYears)'}.',
    );
  }
  return TopicView(
    key: 'dryness',
    title: 'Dryness and fire',
    evidence: Evidence.derived,
    summary: parts.isEmpty ? t.summaryEn : parts.join(' '),
    chips: [
      if (diff != null)
        ('${diff >= 0 ? '+' : ''}${_num(diff)}%', 'green in dry seasons'),
      if (summerGreen != null)
        (
          summers == null
              ? _num(summerGreen)
              : '${_num(summerGreen)} of $summers',
          'summers green',
        ),
      if (fires != null) (_num(fires), 'fire detections'),
    ],
    source:
        'ERA5 rain compared with Landsat and Sentinel-2 greenness · NASA FIRMS fire detections',
    all: _allNumbers(t),
  );
}

/// The cards in the order farmers asked for: weather first, then the field.
List<TopicView> topicViews(FarmInsights f) => [
  ?rainView(f.topic('rain')),
  ?frostView(f.topic('weather')),
  ?greennessView(f.topic('greenness')),
  ?soilView(f.topic('soil')),
  ?drynessView(
    f.topic('dryness'),
    rain: f.topic('rain'),
    green: f.topic('greenness'),
  ),
];

/// The oldest as_of date among the topics, e.g. "3 Oct 2026".
String? dataUpTo(FarmInsights f) {
  final dates = [
    for (final t in f.topics) DateTime.tryParse(t.asOf),
  ].whereType<DateTime>().toList();
  if (dates.isEmpty) return null;
  final d = dates.reduce((a, b) => a.isBefore(b) ? a : b);
  return '${d.day} ${_short[d.month - 1]} ${d.year}';
}

/// Share of dry seasons as "1 season in 4".
String? drySeasonShare(InsightTopic? rain) {
  final n = rain?.m('drought_seasons');
  final since = int.tryParse(_firstYear(rain?.label('drought_seasons')) ?? '');
  if (n == null || n < 1 || since == null) return null;
  final seasons = DateTime.now().year - since;
  final every = math.max(2, (seasons / n).round());
  return '1 season in $every';
}
