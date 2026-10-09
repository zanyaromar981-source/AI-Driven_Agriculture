import 'dart:math' as math;

import '../../api/api.dart';

/// What the "How it's doing now" card shows.
enum NowKind { waiting, bare, comingUp, growing, behind, harvested, cloudy }

class NowView {
  const NowView({
    required this.kind,
    required this.title,
    required this.line,
    this.date,
    this.chips = const [],
    this.next,
  });
  final NowKind kind;
  final String title;
  final String line;

  /// Date of the picture, e.g. "6 Mar".
  final String? date;
  final List<(String, String)> chips;
  final String? next;
}

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

/// "early March", "mid March", "late March".
String partOfMonth(DateTime d) =>
    '${d.day <= 10
        ? 'early'
        : d.day <= 20
        ? 'mid'
        : 'late'} ${_months[d.month - 1]}';

/// Offset of the behind cells from the field centre -> "north-east".
String direction(double dx, double dy) {
  final deg = (math.atan2(dx, dy) * 180 / math.pi + 360) % 360;
  const names = [
    'north',
    'north-east',
    'east',
    'south-east',
    'south',
    'south-west',
    'west',
    'north-west',
  ];
  return names[((deg + 22.5) ~/ 45) % 8];
}

String _abbr(String dir) =>
    dir.split('-').map((w) => w[0].toUpperCase()).join();

const _next = 'Next picture in about 5 days, if it is not cloudy';

/// The farm's state from the latest clear picture (greenness topic, now_*
/// measures). [today] is passed in for tests.
NowView nowView(FarmInsights? f, {required DateTime today}) {
  final g = f?.topic('greenness');
  final y = g?.m('now_picture_year');
  final mo = g?.m('now_picture_month');
  final d = g?.m('now_picture_day');
  if (g == null || y == null || mo == null || d == null) {
    return const NowView(
      kind: NowKind.waiting,
      title: 'Reading the latest picture',
      line:
          'Finding the latest clear satellite picture of this field. This takes a few minutes after a farm is added.',
    );
  }
  final pic = DateTime(y.round(), mo.round(), d.round());
  final date = '${pic.day} ${_short[pic.month - 1]}';
  final age = today.difference(pic).inDays;
  final stage = g.m('now_stage')?.round();
  final pct = g.m('now_pct_of_normal');
  final normal = g.m('now_normal_ndvi');
  final behind = g.m('now_behind_share_pct') ?? 0;
  final dx = g.m('now_behind_dx_m');
  final dy = g.m('now_behind_dy_m');
  final when = partOfMonth(pic);

  NowView fromPicture() {
    switch (stage) {
      case 0:
        final usuallyGreen = normal != null && normal >= 0.3;
        return NowView(
          kind: usuallyGreen ? NowKind.behind : NowKind.bare,
          title: usuallyGreen ? 'Still bare' : 'Bare soil',
          date: date,
          line: usuallyGreen
              ? 'Nothing green yet, but in most years this field is green by $when. Check whether it was sown and came up.'
              : 'Nothing is growing yet, which is normal before sowing (October to December). The crop shows from space about 3 to 4 weeks after it comes up.',
          next: _next,
        );
      case 1:
        return NowView(
          kind: NowKind.comingUp,
          title: 'Coming up',
          date: date,
          line:
              'The crop is coming up: the field is turning green. It is too early to compare with other years.',
          next: _next,
        );
      case 4:
        return NowView(
          kind: NowKind.harvested,
          title: 'After the season',
          date: date,
          line:
              'The field looks harvested or dried off, as usual after the season.',
          next: _next,
        );
      default:
        if (pct == null) {
          return NowView(
            kind: NowKind.growing,
            title: 'Growing',
            date: date,
            line: 'The crop is growing.',
            next: _next,
          );
        }
        final good = pct >= 90;
        final how = pct >= 110
            ? 'Greener than this field usually is in $when.'
            : pct >= 90
            ? 'About as green as this field usually is in $when.'
            : 'Less green than this field usually is in $when.';
        final dir = dx != null && dy != null ? direction(dx, dy) : null;
        final where = behind < 1
            ? ' No part of the field is behind the rest.'
            : behind >= 40
            ? ' Most of the field is behind, not one spot: look at water and sowing date first.'
            : ' A small part is behind${dir == null ? '' : ', in the $dir'}.';
        return NowView(
          kind: good ? NowKind.growing : NowKind.behind,
          title: good
              ? 'Growing well'
              : (pct < 70 ? 'Well behind usual' : 'Behind usual'),
          date: date,
          line: '$how$where'.trim(),
          chips: [
            ('${pct.round()}%', 'of usual for $when'),
            ('${behind.round()}%', 'of the field behind'),
            if (behind >= 1)
              (
                behind >= 40 ? 'All' : (dir == null ? '-' : _abbr(dir)),
                behind >= 40 ? 'across the field' : 'where it is behind',
              ),
          ],
          next: _next,
        );
    }
  }

  final v = fromPicture();
  // Sentinel-2 passes about every 5 days: no clear picture for 12 days
  // means clouds hid the field since then.
  if (age <= 12) return v;
  return NowView(
    kind: NowKind.cloudy,
    title: 'No clear picture',
    date: date,
    line:
        'Clouds covered the field in the last pictures. This is the last clear one, $age days ago: ${v.title.toLowerCase()} then.',
    next: 'Waiting for a clear sky',
  );
}
