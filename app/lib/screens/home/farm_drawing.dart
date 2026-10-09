import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_map/flutter_map.dart';
import 'package:latlong2/latlong.dart' show LatLng;

import '../../api/api.dart';
import '../../app_scope.dart';
import '../../crops.dart';
import '../../geo.dart';
import '../../theme.dart';
import '../../widgets/farm_map.dart';

/// The two ways to look at a farm (design: View Toggle, Cells | Crops;
/// the Farm view was removed by the user on 2026-10-09).
enum FarmView { cells, crops }

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

/// The farm in map coordinates: outline and cell corners, worked out once.
class _FarmGeo {
  _FarmGeo(FarmShape s)
    : outline = [for (final (x, y) in s.outline) Utm.toLatLng(x, y)],
      corners = {for (final k in s.crop.keys) k: cellCorners(k)};
  final List<LatLng> outline;
  final Map<CellKey, List<LatLng>> corners;
}

/// The farm on its real map (satellite, or the style picked with the layers
/// button), with the outline and the 10 m cells on top, and tap handling.
/// The map around each farm is kept on the phone, so it also shows offline.
/// [overlay] sits on top (cell card or the whole-farm number); it moves to
/// the top when the picked cell is low.
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
    this.pin = false,
    this.styleButton = true,
  });

  final FarmShape shape;
  final FarmView view;
  final FarmStatus farmLevel;
  final CellKey? selectedCell;
  final String? selectedCrop;
  final ValueChanged<CellKey?> onCell;
  final ValueChanged<String?> onCrop;
  final Widget? overlay;

  /// A map pin on the chosen cell (Report: Where?).
  final bool pin;

  /// The map style button in the corner (off on the small Report map).
  final bool styleButton;

  static final Expando<_FarmGeo> _geos = Expando();
  _FarmGeo get _geo => _geos[shape] ??= _FarmGeo(shape);

  void _tap(CellKey key) {
    final crop = shape.crop[key];
    switch (view) {
      case FarmView.cells:
        onCell(crop == null || key == selectedCell ? null : key);
      case FarmView.crops:
        onCrop(
          crop == null || crop == 'empty' || crop == selectedCrop ? null : crop,
        );
    }
  }

  /// Colours sit over the satellite picture, so they are partly see-through.
  Color? _cellColour(CellKey k) {
    if (view == FarmView.cells) {
      return levelColor(shape.levelAt(k)).withValues(alpha: 0.6);
    }
    final crop = shape.crop[k];
    if (crop == null) return null;
    final c = crop == 'empty' ? JColors.levelNone : cropOf(crop).color;
    final dim = selectedCrop != null && crop != selectedCrop;
    return c.withValues(alpha: dim ? 0.2 : 0.65);
  }

  @override
  Widget build(BuildContext context) {
    final ku = AppScope.of(context).ku;
    final geo = _geo;
    prefetchFarmMap(geo.outline);
    return LayoutBuilder(
      builder: (context, box) {
        final w = box.maxWidth;
        final aspect =
            (shape.maxY - shape.minY) / math.max(1.0, shape.maxX - shape.minX);
        final h = ((w - 20) * aspect + 20).clamp(200.0, 330.0);
        final sel = selectedCell;
        final cardAtTop =
            sel != null &&
            sel.n * 10.0 + 5 - shape.minY < (shape.maxY - shape.minY) / 2;
        final overlayAtTop = overlay != null && cardAtTop;
        return SizedBox(
          width: w,
          height: h,
          child: ClipRRect(
            borderRadius: BorderRadius.circular(12),
            child: Stack(
              clipBehavior: Clip.none,
              children: [
                StyledMap(
                  builder: (context, style) => FlutterMap(
                    key: ValueKey(
                      Object.hash(geo.outline.first, geo.outline.length),
                    ),
                    options: MapOptions(
                      initialCameraFit: CameraFit.bounds(
                        bounds: LatLngBounds.fromPoints(geo.outline),
                        padding: const EdgeInsets.all(14),
                      ),
                      maxZoom: 21,
                      backgroundColor: const Color(0xFF717A50),
                      interactionOptions: const InteractionOptions(
                        flags: InteractiveFlag.none,
                      ),
                    ),
                    children: [
                      ...baseLayers(style, ku),
                      CellLayer(
                        corners: geo.corners,
                        clipTo: geo.outline,
                        style: _cellColour,
                      ),
                      PolygonLayer(
                        polygons: [
                          Polygon(
                            points: geo.outline,
                            color: Colors.transparent,
                            borderColor: Colors.white.withValues(alpha: 0.95),
                            borderStrokeWidth: 2,
                          ),
                        ],
                      ),
                      if (sel != null && view == FarmView.cells)
                        PolygonLayer(
                          polygons: [
                            Polygon(
                              points: cellCorners(sel),
                              color: Colors.transparent,
                              borderColor: JColors.ink,
                              borderStrokeWidth: 2.4,
                            ),
                          ],
                        ),
                      if (pin && sel != null)
                        MarkerLayer(
                          markers: [
                            Marker(
                              point: Utm.toLatLng(
                                sel.e * 10.0 + 5,
                                sel.n * 10.0 + 5,
                              ),
                              width: 20,
                              height: 20,
                              alignment: Alignment.topCenter,
                              child: const Icon(
                                Icons.location_on_rounded,
                                size: 20,
                                color: JColors.ink,
                              ),
                            ),
                          ],
                        ),
                      if (view == FarmView.crops)
                        MarkerLayer(
                          markers: [
                            for (final e in shape.cropCentres().entries)
                              Marker(
                                point: Utm.toLatLng(e.value.$1, e.value.$2),
                                width: 30,
                                height: 30,
                                child: _badge(e.key),
                              ),
                          ],
                        ),
                      Builder(
                        builder: (context) {
                          final cam = MapCamera.of(context);
                          return GestureDetector(
                            behavior: HitTestBehavior.opaque,
                            onTapUp: (d) => _tap(
                              Utm.cellOf(
                                cam.screenOffsetToLatLng(d.localPosition),
                              ),
                            ),
                            child: const SizedBox.expand(),
                          );
                        },
                      ),
                      mapAttribution(style),
                    ],
                  ),
                ),
                if (!overlayAtTop && styleButton)
                  const Positioned(top: 8, right: 8, child: MapStyleButton()),
                if (overlay != null)
                  Positioned(
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

  Widget _badge(String crop) => IgnorePointer(
    child: Container(
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
  );
}
