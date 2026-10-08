import 'dart:math' as math;

import 'package:latlong2/latlong.dart';

/// One 10 m cell of the Sentinel-2 grid (BACKEND.md section 1):
/// e = floor(easting / 10), n = floor(northing / 10) in UTM zone 38N.
typedef CellKey = ({int e, int n});

double _sinh(double x) => (math.exp(x) - math.exp(-x)) / 2;
double _cosh(double x) => (math.exp(x) + math.exp(-x)) / 2;
double _atanh(double x) => 0.5 * math.log((1 + x) / (1 - x));

/// WGS84 <-> UTM zone 38N (EPSG:32638), Krüger series, mm accuracy in the zone.
class Utm {
  static const _a = 6378137.0;
  static const _f = 1 / 298.257223563;
  static const _k0 = 0.9996;
  static const _falseEasting = 500000.0;
  static const _lon0 = 45.0;
  static const _deg = math.pi / 180;

  static final double _n = _f / (2 - _f);
  static final double _bigA =
      _a / (1 + _n) * (1 + _n * _n / 4 + math.pow(_n, 4) / 64);
  static final List<double> _alpha = [
    _n / 2 - 2 * _n * _n / 3 + 5 * math.pow(_n, 3) / 16,
    13 * _n * _n / 48 - 3 * math.pow(_n, 3) / 5,
    61 * math.pow(_n, 3) / 240,
  ];
  static final List<double> _beta = [
    _n / 2 - 2 * _n * _n / 3 + 37 * math.pow(_n, 3) / 96,
    _n * _n / 48 + math.pow(_n, 3) / 15,
    17 * math.pow(_n, 3) / 480,
  ];
  static final List<double> _delta = [
    2 * _n - 2 * _n * _n / 3 - 2 * math.pow(_n, 3),
    7 * _n * _n / 3 - 8 * math.pow(_n, 3) / 5,
    56 * math.pow(_n, 3) / 15,
  ];

  /// (easting, northing) in metres.
  static (double, double) fromLatLng(double lat, double lon) {
    final phi = lat * _deg;
    final dl = (lon - _lon0) * _deg;
    final c = 2 * math.sqrt(_n) / (1 + _n);
    final t = _sinh(_atanh(math.sin(phi)) - c * _atanh(c * math.sin(phi)));
    final xiP = math.atan2(t, math.cos(dl));
    final etaP = _atanh(math.sin(dl) / math.sqrt(1 + t * t));
    var e = etaP;
    var n = xiP;
    for (var j = 1; j <= 3; j++) {
      e += _alpha[j - 1] * math.cos(2 * j * xiP) * _sinh(2 * j * etaP);
      n += _alpha[j - 1] * math.sin(2 * j * xiP) * _cosh(2 * j * etaP);
    }
    return (_falseEasting + _k0 * _bigA * e, _k0 * _bigA * n);
  }

  static LatLng toLatLng(double easting, double northing) {
    final xi = northing / (_k0 * _bigA);
    final eta = (easting - _falseEasting) / (_k0 * _bigA);
    var xiP = xi;
    var etaP = eta;
    for (var j = 1; j <= 3; j++) {
      xiP -= _beta[j - 1] * math.sin(2 * j * xi) * _cosh(2 * j * eta);
      etaP -= _beta[j - 1] * math.cos(2 * j * xi) * _sinh(2 * j * eta);
    }
    final chi = math.asin(math.sin(xiP) / _cosh(etaP));
    var phi = chi;
    for (var j = 1; j <= 3; j++) {
      phi += _delta[j - 1] * math.sin(2 * j * chi);
    }
    final lon = _lon0 + math.atan2(_sinh(etaP), math.cos(xiP)) / _deg;
    return LatLng(phi / _deg, lon);
  }

  static CellKey cellOf(LatLng p) {
    final (e, n) = fromLatLng(p.latitude, p.longitude);
    return (e: (e / 10).floor(), n: (n / 10).floor());
  }
}

/// The four corners of a cell, for drawing.
List<LatLng> cellCorners(CellKey c) {
  final x = c.e * 10.0;
  final y = c.n * 10.0;
  return [
    Utm.toLatLng(x, y),
    Utm.toLatLng(x + 10, y),
    Utm.toLatLng(x + 10, y + 10),
    Utm.toLatLng(x, y + 10),
  ];
}

LatLng cellCentre(CellKey c) => Utm.toLatLng(c.e * 10.0 + 5, c.n * 10.0 + 5);

List<(double, double)> _toXY(List<LatLng> pts) => [
  for (final p in pts) Utm.fromLatLng(p.latitude, p.longitude),
];

bool _inside(double x, double y, List<(double, double)> poly) {
  var inside = false;
  for (var i = 0, j = poly.length - 1; i < poly.length; j = i++) {
    final (xi, yi) = poly[i];
    final (xj, yj) = poly[j];
    if ((yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi) {
      inside = !inside;
    }
  }
  return inside;
}

/// Every cell whose centre lies inside the outline (the outline closes itself).
List<CellKey> cellsInside(List<LatLng> outline) {
  if (outline.length < 3) return const [];
  final poly = _toXY(outline);
  final xs = poly.map((p) => p.$1);
  final ys = poly.map((p) => p.$2);
  final e0 = (xs.reduce(math.min) / 10).floor();
  final e1 = (xs.reduce(math.max) / 10).floor();
  final n0 = (ys.reduce(math.min) / 10).floor();
  final n1 = (ys.reduce(math.max) / 10).floor();
  return [
    for (var n = n0; n <= n1; n++)
      for (var e = e0; e <= e1; e++)
        if (_inside(e * 10.0 + 5, n * 10.0 + 5, poly)) (e: e, n: n),
  ];
}

/// Outline area in m² (shoelace in UTM metres).
double polygonAreaM2(List<LatLng> outline) {
  if (outline.length < 3) return 0;
  final p = _toXY(outline);
  var s = 0.0;
  for (var i = 0, j = p.length - 1; i < p.length; j = i++) {
    s += (p[j].$1 + p[i].$1) * (p[j].$2 - p[i].$2);
  }
  return s.abs() / 2;
}

/// True if any two non-neighbouring edges cross (a "bow tie" outline).
bool selfIntersects(List<LatLng> outline) {
  final p = _toXY(outline);
  final k = p.length;
  if (k < 4) return false;
  double cross((double, double) o, (double, double) a, (double, double) b) =>
      (a.$1 - o.$1) * (b.$2 - o.$2) - (a.$2 - o.$2) * (b.$1 - o.$1);
  bool crosses(
    (double, double) a,
    (double, double) b,
    (double, double) c,
    (double, double) d,
  ) {
    final d1 = cross(c, d, a),
        d2 = cross(c, d, b),
        d3 = cross(a, b, c),
        d4 = cross(a, b, d);
    return ((d1 > 0) != (d2 > 0)) &&
        ((d3 > 0) != (d4 > 0)) &&
        d1 != 0 &&
        d2 != 0 &&
        d3 != 0 &&
        d4 != 0;
  }

  for (var i = 0; i < k; i++) {
    for (var j = i + 1; j < k; j++) {
      if (j == i + 1 || (i == 0 && j == k - 1)) continue;
      if (crosses(p[i], p[(i + 1) % k], p[j], p[(j + 1) % k])) return true;
    }
  }
  return false;
}

/// 1 dunam = 2,500 m² = 25 cells.
double cellsToDunam(int cells) => cells / 25;

/// 96 -> "96", 1.64 -> "1.6", 2.0 -> "2", 0.24 -> "0.24".
String fmtDunam(double d) {
  if (d >= 10) return d.round().toString();
  var s = d.toStringAsFixed(d < 1 ? 2 : 1);
  if (s.contains('.')) {
    s = s.replaceAll(RegExp(r'0+$'), '');
    if (s.endsWith('.')) s = s.substring(0, s.length - 1);
  }
  return s;
}

double _shoelace(List<(double, double)> p) {
  if (p.length < 3) return 0;
  var s = 0.0;
  for (var i = 0, j = p.length - 1; i < p.length; j = i++) {
    s += (p[j].$1 + p[i].$1) * (p[j].$2 - p[i].$2);
  }
  return s.abs() / 2;
}

/// Sutherland-Hodgman: the part of [poly] inside the box x0..x1, y0..y1.
List<(double, double)> _clipToBox(
  List<(double, double)> poly,
  double x0,
  double y0,
  double x1,
  double y1,
) {
  var out = poly;
  for (var edge = 0; edge < 4 && out.isNotEmpty; edge++) {
    final input = out;
    out = [];
    bool inside((double, double) p) => switch (edge) {
      0 => p.$1 >= x0,
      1 => p.$1 <= x1,
      2 => p.$2 >= y0,
      _ => p.$2 <= y1,
    };
    (double, double) cut((double, double) a, (double, double) b) {
      if (edge < 2) {
        final x = edge == 0 ? x0 : x1;
        final t = (x - a.$1) / (b.$1 - a.$1);
        return (x, a.$2 + t * (b.$2 - a.$2));
      }
      final y = edge == 2 ? y0 : y1;
      final t = (y - a.$2) / (b.$2 - a.$2);
      return (a.$1 + t * (b.$1 - a.$1), y);
    }

    var prev = input.last;
    for (final cur in input) {
      final ci = inside(cur), pi = inside(prev);
      if (ci) {
        if (!pi) out.add(cut(prev, cur));
        out.add(cur);
      } else if (pi) {
        out.add(cut(prev, cur));
      }
      prev = cur;
    }
  }
  return out;
}

double _distToSegment(
  (double, double) p,
  (double, double) a,
  (double, double) b,
) {
  final dx = b.$1 - a.$1, dy = b.$2 - a.$2;
  final len2 = dx * dx + dy * dy;
  var t = len2 == 0 ? 0.0 : ((p.$1 - a.$1) * dx + (p.$2 - a.$2) * dy) / len2;
  t = t.clamp(0.0, 1.0);
  final x = a.$1 + t * dx - p.$1, y = a.$2 + t * dy - p.$2;
  return math.sqrt(x * x + y * y);
}

/// Every cell that overlaps the outline, with how many m² of it lie inside
/// (100 = whole cell). Edge cells are cut along the border, so the values add
/// up to the exact outline area.
Map<CellKey, double> cellsTouching(List<LatLng> outline) {
  if (outline.length < 3) return const {};
  final poly = _toXY(outline);
  final xs = poly.map((p) => p.$1);
  final ys = poly.map((p) => p.$2);
  final e0 = (xs.reduce(math.min) / 10).floor();
  final e1 = (xs.reduce(math.max) / 10).floor();
  final n0 = (ys.reduce(math.min) / 10).floor();
  final n1 = (ys.reduce(math.max) / 10).floor();
  const halfDiagonal = 7.0711;
  final out = <CellKey, double>{};
  for (var n = n0; n <= n1; n++) {
    for (var e = e0; e <= e1; e++) {
      final c = (e * 10.0 + 5, n * 10.0 + 5);
      var near = double.infinity;
      for (var i = 0, j = poly.length - 1; i < poly.length; j = i++) {
        near = math.min(near, _distToSegment(c, poly[j], poly[i]));
        if (near < halfDiagonal) break;
      }
      final double inside;
      if (near >= halfDiagonal) {
        inside = _inside(c.$1, c.$2, poly) ? 100 : 0;
      } else {
        inside = math.min(
          100,
          _shoelace(
            _clipToBox(poly, e * 10.0, n * 10.0, e * 10.0 + 10, n * 10.0 + 10),
          ),
        );
      }
      if (inside > 0.01) out[(e: e, n: n)] = inside;
    }
  }
  return out;
}

/// Walked GPS track (UTM metres) -> indices of the points to keep as dots.
/// Douglas-Peucker keeps the bends and drops points on straight stretches;
/// the tolerance is loosened until at most [maxPoints] are left.
List<int> simplifyTrack(
  List<(double, double)> xy, {
  double tolerance = 2.5,
  int maxPoints = 50,
}) {
  if (xy.length <= 3) return [for (var i = 0; i < xy.length; i++) i];
  var tol = tolerance;
  while (true) {
    final keep = List<bool>.filled(xy.length, false);
    keep[0] = true;
    keep[xy.length - 1] = true;
    final stack = <(int, int)>[(0, xy.length - 1)];
    while (stack.isNotEmpty) {
      final (a, b) = stack.removeLast();
      var far = -1.0;
      var at = -1;
      for (var i = a + 1; i < b; i++) {
        final d = _distToSegment(xy[i], xy[a], xy[b]);
        if (d > far) {
          far = d;
          at = i;
        }
      }
      if (at != -1 && far > tol) {
        keep[at] = true;
        stack
          ..add((a, at))
          ..add((at, b));
      }
    }
    final idx = [
      for (var i = 0; i < xy.length; i++)
        if (keep[i]) i,
    ];
    if (idx.length <= maxPoints) return idx;
    tol *= 1.4;
  }
}

/// Straight-line distance in metres between two GPS points.
double metresBetween(LatLng a, LatLng b) {
  final (ax, ay) = Utm.fromLatLng(a.latitude, a.longitude);
  final (bx, by) = Utm.fromLatLng(b.latitude, b.longitude);
  return math.sqrt((ax - bx) * (ax - bx) + (ay - by) * (ay - by));
}
