import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../../api/api.dart';
import '../../crops.dart';
import '../../geo.dart';
import '../../theme.dart';

/// The three ways to look at a farm (design: View Toggle).
enum FarmView { cells, crops, farm }

Color levelColor(FarmStatus l) => switch (l) {
  FarmStatus.normal => JColors.levelNormal,
  FarmStatus.watch => JColors.levelWatch,
  FarmStatus.alarm => JColors.levelAlarm,
  FarmStatus.none => JColors.levelNone,
};

Color levelSoft(FarmStatus l) => switch (l) {
  FarmStatus.normal => JColors.levelNormalSoft,
  FarmStatus.watch => JColors.levelWatchSoft,
  FarmStatus.alarm => JColors.levelAlarmSoft,
  FarmStatus.none => JColors.levelNoneSoft,
};

/// Level from a greenness % of normal, for crop plots and the whole farm
/// when the server gives only the number. Same cut-offs as the demo server.
FarmStatus levelFromPct(int? pct) => pct == null
    ? FarmStatus.none
    : pct >= 85
    ? FarmStatus.normal
    : pct >= 70
    ? FarmStatus.watch
    : FarmStatus.alarm;

/// A farm in UTM metres, ready to draw at any size.
class FarmShape {
  FarmShape(Farm farm, FarmStatusReport? status)
    : outline = [for (final p in farm.outline) Utm.fromLatLng(p.lat, p.lon)],
      crop = {for (final c in farm.cells) (e: c.e, n: c.n): c.crop},
      inside = {for (final c in farm.cells) (e: c.e, n: c.n): c.insidePct},
      reading = {
        for (final r in status?.cells ?? const <CellReading>[])
          (e: r.e, n: r.n): r,
      } {
    final xs = outline.map((p) => p.$1);
    final ys = outline.map((p) => p.$2);
    minX = xs.reduce(math.min);
    maxX = xs.reduce(math.max);
    minY = ys.reduce(math.min);
    maxY = ys.reduce(math.max);
    final keys = crop.keys;
    eMin = keys.isEmpty ? 0 : keys.map((k) => k.e).reduce(math.min);
    nMax = keys.isEmpty ? 0 : keys.map((k) => k.n).reduce(math.max);
  }

  final List<(double, double)> outline;
  final Map<CellKey, String> crop;

  /// m² of each cell inside the outline (100 = whole cell; edge cells less).
  final Map<CellKey, double> inside;

  /// m² of these cells inside the outline: an edge cell half inside adds 50,
  /// so all cells together add up to the farm's exact area.
  double areaM2(Iterable<CellKey> keys) =>
      keys.fold(0.0, (a, k) => a + (inside[k] ?? 100));
  final Map<CellKey, CellReading> reading;
  late final double minX, maxX, minY, maxY;
  late final int eMin, nMax;

  FarmStatus levelAt(CellKey k) => reading[k]?.level ?? FarmStatus.none;

  /// Spreadsheet-style name: column letters from the west, row number from the north.
  String label(CellKey k) {
    var col = k.e - eMin;
    var letters = '';
    do {
      letters = String.fromCharCode(65 + col % 26) + letters;
      col = col ~/ 26 - 1;
    } while (col >= 0);
    return '$letters${nMax - k.n + 1}';
  }

  /// Mean greenness of the up-to-8 cells around [k] that have a reading.
  int? neighbours(CellKey k) {
    final v = <int>[
      for (var de = -1; de <= 1; de++)
        for (var dn = -1; dn <= 1; dn++)
          if (de != 0 || dn != 0)
            ?reading[(e: k.e + de, n: k.n + dn)]?.greennessPct,
    ];
    return v.isEmpty ? null : (v.reduce((a, b) => a + b) / v.length).round();
  }

  /// Centre of each crop's cells, in metres (for the crop badges).
  Map<String, (double, double)> cropCentres() {
    final sum = <String, (double, double, int)>{};
    crop.forEach((k, c) {
      if (c == 'empty') return;
      final (x, y, n) = sum[c] ?? (0, 0, 0);
      sum[c] = (x + k.e * 10 + 5, y + k.n * 10 + 5, n + 1);
    });
    return {
      for (final e in sum.entries)
        e.key: (e.value.$1 / e.value.$3, e.value.$2 / e.value.$3),
    };
  }
}

/// Fits a [FarmShape] into a box, north up.
class _Fit {
  _Fit(this.s, Size size, {double pad = 10}) {
    final w = math.max(1.0, s.maxX - s.minX);
    final h = math.max(1.0, s.maxY - s.minY);
    k = math.min((size.width - 2 * pad) / w, (size.height - 2 * pad) / h);
    ox = (size.width - w * k) / 2;
    oy = (size.height - h * k) / 2;
  }
  final FarmShape s;
  late final double k, ox, oy;

  Offset to(double x, double y) =>
      Offset(ox + (x - s.minX) * k, oy + (s.maxY - y) * k);

  CellKey cellAt(Offset o) {
    final x = s.minX + (o.dx - ox) / k;
    final y = s.maxY - (o.dy - oy) / k;
    return (e: (x / 10).floor(), n: (y / 10).floor());
  }

  Rect cellRect(CellKey c) => Rect.fromPoints(
    to(c.e * 10.0, c.n * 10.0 + 10),
    to(c.e * 10.0 + 10, c.n * 10.0),
  );
}

/// The farm drawing with tap handling. [overlay] sits on top (cell card or
/// the whole-farm number); it moves to the top when the picked cell is low.
class FarmDrawing extends StatelessWidget {
  const FarmDrawing({
    super.key,
    required this.shape,
    required this.view,
    required this.farmLevel,
    this.selectedCell,
    this.selectedCrop,
    required this.onCell,
    required this.onCrop,
    this.overlay,
  });

  final FarmShape shape;
  final FarmView view;
  final FarmStatus farmLevel;
  final CellKey? selectedCell;
  final String? selectedCrop;
  final ValueChanged<CellKey?> onCell;
  final ValueChanged<String?> onCrop;
  final Widget? overlay;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, box) {
        final w = box.maxWidth;
        final aspect =
            (shape.maxY - shape.minY) / math.max(1.0, shape.maxX - shape.minX);
        final h = ((w - 20) * aspect + 20).clamp(200.0, 330.0);
        final size = Size(w, h);
        final fit = _Fit(shape, size);
        final sel = selectedCell;
        final cardAtTop = sel != null && fit.cellRect(sel).center.dy > h / 2;
        return SizedBox(
          width: w,
          height: h,
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTapUp: (d) {
              final key = fit.cellAt(d.localPosition);
              final crop = shape.crop[key];
              switch (view) {
                case FarmView.cells:
                  onCell(crop == null || key == selectedCell ? null : key);
                case FarmView.crops:
                  onCrop(
                    crop == null || crop == 'empty' || crop == selectedCrop
                        ? null
                        : crop,
                  );
                case FarmView.farm:
                  break;
              }
            },
            child: Stack(
              clipBehavior: Clip.none,
              children: [
                CustomPaint(
                  size: size,
                  painter: _FarmPainter(
                    fit: fit,
                    view: view,
                    farmLevel: farmLevel,
                    selectedCell: selectedCell,
                    selectedCrop: selectedCrop,
                  ),
                ),
                if (view == FarmView.crops)
                  for (final e in shape.cropCentres().entries)
                    _badge(fit.to(e.value.$1, e.value.$2), e.key),
                if (overlay != null)
                  view == FarmView.farm
                      ? Positioned.fill(child: Center(child: overlay))
                      : Positioned(
                          left: 8,
                          right: 8,
                          top: cardAtTop ? 8 : null,
                          bottom: cardAtTop ? null : 8,
                          child: overlay!,
                        ),
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _badge(Offset at, String crop) => Positioned(
    left: at.dx - 15,
    top: at.dy - 15,
    child: IgnorePointer(
      child: Container(
        width: 30,
        height: 30,
        alignment: Alignment.center,
        decoration: BoxDecoration(
          color: Colors.white,
          shape: BoxShape.circle,
          border: Border.all(
            color: crop == selectedCrop ? JColors.ink : Colors.white,
            width: 2,
          ),
          boxShadow: const [
            BoxShadow(
              color: Color(0x33000000),
              blurRadius: 4,
              offset: Offset(0, 1),
            ),
          ],
        ),
        child: Text(cropOf(crop).emoji, style: const TextStyle(fontSize: 15)),
      ),
    ),
  );
}

class _FarmPainter extends CustomPainter {
  _FarmPainter({
    required this.fit,
    required this.view,
    required this.farmLevel,
    this.selectedCell,
    this.selectedCrop,
  });

  final _Fit fit;
  final FarmView view;
  final FarmStatus farmLevel;
  final CellKey? selectedCell;
  final String? selectedCrop;

  @override
  void paint(Canvas canvas, Size size) {
    final s = fit.s;
    final edge = Path()
      ..addPolygon([for (final (x, y) in s.outline) fit.to(x, y)], true);
    final edgePaint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.6
      ..strokeJoin = StrokeJoin.round
      ..color = JColors.ink.withValues(alpha: 0.75);

    if (view == FarmView.farm) {
      canvas.drawPath(edge, Paint()..color = levelSoft(farmLevel));
      canvas.drawPath(edge, edgePaint);
      return;
    }

    canvas.save();
    canvas.clipPath(edge);
    final groups = <Color, Path>{};
    final grid = Path();
    s.crop.forEach((key, crop) {
      final r = fit.cellRect(key);
      var c = view == FarmView.cells
          ? levelColor(s.levelAt(key))
          : crop == 'empty'
          ? JColors.levelNone
          : cropOf(crop).color;
      if (view == FarmView.crops &&
          selectedCrop != null &&
          crop != selectedCrop) {
        c = c.withValues(alpha: 0.3);
      }
      groups.putIfAbsent(c, Path.new).addRect(r);
      if (view == FarmView.cells) grid.addRect(r);
    });
    groups.forEach((c, p) => canvas.drawPath(p, Paint()..color = c));
    if (view == FarmView.cells && fit.k * 10 >= 3.5) {
      canvas.drawPath(
        grid,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 0.5
          ..color = Colors.white.withValues(alpha: 0.55),
      );
    }
    canvas.restore();
    canvas.drawPath(edge, edgePaint);

    final sel = selectedCell;
    if (sel != null && view == FarmView.cells) {
      final r = fit.cellRect(sel).inflate(math.max(2, fit.k * 2));
      canvas.drawRect(
        r,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 2.4
          ..color = JColors.ink,
      );
    }
  }

  @override
  bool shouldRepaint(covariant _FarmPainter old) =>
      old.fit.s != fit.s ||
      old.fit.k != fit.k ||
      old.view != view ||
      old.farmLevel != farmLevel ||
      old.selectedCell != selectedCell ||
      old.selectedCrop != selectedCrop;
}
