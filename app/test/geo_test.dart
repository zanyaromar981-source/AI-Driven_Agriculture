import 'dart:math' as math;

import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/geo.dart';
import 'package:latlong2/latlong.dart';

void main() {
  // Reference values from pyproj (EPSG:4326 -> EPSG:32638).
  const refs = [
    (35.5613, 45.4374, 539639.126, 3935379.986, 53963, 393537),
    (36.0312, 44.6021, 464152.261, 3987482.219, 46415, 398748),
    (36.1911, 44.0092, 410916.136, 4005599.420, 41091, 400559),
    (36.8669, 42.9503, 317297.346, 4082068.467, 31729, 408206),
    (35.6236, 45.9433, 585420.799, 3942611.091, 58542, 394261),
  ];

  test('lat/lon to UTM 38N matches pyproj within 1 cm', () {
    for (final (lat, lon, e, n, ce, cn) in refs) {
      final (x, y) = Utm.fromLatLng(lat, lon);
      expect((x - e).abs(), lessThan(0.01), reason: '$lat,$lon easting');
      expect((y - n).abs(), lessThan(0.01), reason: '$lat,$lon northing');
      expect(Utm.cellOf(LatLng(lat, lon)), (e: ce, n: cn));
    }
  });

  test('UTM back to lat/lon round-trips', () {
    for (final (lat, lon, e, n, _, _) in refs) {
      final p = Utm.toLatLng(e, n);
      expect((p.latitude - lat).abs(), lessThan(1e-7));
      expect((p.longitude - lon).abs(), lessThan(1e-7));
    }
  });

  test('a 100 m square holds 100 cells (4 dunam)', () {
    final sq = [
      Utm.toLatLng(539600, 3935300),
      Utm.toLatLng(539700, 3935300),
      Utm.toLatLng(539700, 3935400),
      Utm.toLatLng(539600, 3935400),
    ];
    final cells = cellsInside(sq);
    expect(cells.length, 100);
    expect(cellsToDunam(cells.length), 4);
    expect(polygonAreaM2(sq), closeTo(10000, 0.5));
    expect(selfIntersects(sq), isFalse);
  });

  test('a bow tie outline is caught', () {
    final bow = [
      Utm.toLatLng(539600, 3935300),
      Utm.toLatLng(539700, 3935400),
      Utm.toLatLng(539700, 3935300),
      Utm.toLatLng(539600, 3935400),
    ];
    expect(selfIntersects(bow), isTrue);
  });

  test('dunam formatting', () {
    expect(fmtDunam(96), '96');
    expect(fmtDunam(1.64), '1.6');
    expect(fmtDunam(2.0), '2');
    expect(fmtDunam(0.24), '0.24');
    expect(fmtDunam(0.5), '0.5');
    expect(fmtM2(82500.4), '82,500');
    expect(fmtM2(7185), '7,185');
    expect(fmtM2(950), '950');
    expect(fmtM2(2500000), '2,500,000');
  });

  test('cut cells add up to the exact outline area', () {
    // A tilted, uneven 4-corner field that does not line up with the grid.
    final field = [
      Utm.toLatLng(539603.3, 3935301.7),
      Utm.toLatLng(539688.9, 3935322.4),
      Utm.toLatLng(539671.2, 3935397.8),
      Utm.toLatLng(539611.6, 3935381.1),
    ];
    final exact = polygonAreaM2(field);
    final cells = cellsTouching(field);
    final sum = cells.values.fold(0.0, (a, b) => a + b);
    expect(sum, closeTo(exact, 0.5));
    expect(cells.values.every((v) => v > 0 && v <= 100.0001), isTrue);
    // Small plot, where whole-square counting used to be far off.
    final plot = [
      Utm.toLatLng(539603.0, 3935303.0),
      Utm.toLatLng(539618.0, 3935303.0),
      Utm.toLatLng(539618.0, 3935318.0),
      Utm.toLatLng(539603.0, 3935318.0),
    ];
    final plotSum = cellsTouching(plot).values.fold(0.0, (a, b) => a + b);
    expect(plotSum, closeTo(225, 0.5));
  });

  test('a walk around a field becomes at most 50 dots with the right area', () {
    // Walk a 120 x 70 m field with a rounded corner, a GPS reading every 1.3 m, 1 m jitter.
    final pts = <(double, double)>[];
    void leg(double x0, double y0, double x1, double y1) {
      final len = math.sqrt((x1 - x0) * (x1 - x0) + (y1 - y0) * (y1 - y0));
      for (var d = 0.0; d < len; d += 1.3) {
        final t = d / len;
        pts.add((
          x0 + (x1 - x0) * t + math.sin(d) * 1.0,
          y0 + (y1 - y0) * t + math.cos(d * 1.7) * 1.0,
        ));
      }
    }

    leg(0, 0, 120, 0);
    leg(120, 0, 120, 40);
    for (var a = 0.0; a <= math.pi / 2; a += 0.04) {
      pts.add((90 + 30 * math.cos(a), 40 + 30 * math.sin(a)));
    }
    leg(90, 70, 0, 70);
    leg(0, 70, 0, 0);
    final keep = simplifyTrack(pts);
    expect(keep.length, lessThanOrEqualTo(50));
    final dots = [for (final i in keep) pts[i]];
    final trueArea = 120 * 70 - (900 - math.pi * 900 / 4);
    final ll = [
      for (final (x, y) in dots) Utm.toLatLng(539000 + x, 3935000 + y),
    ];
    expect((polygonAreaM2(ll) - trueArea).abs() / trueArea, lessThan(0.03));
  });
}
