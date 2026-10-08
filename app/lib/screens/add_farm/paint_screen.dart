import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_map/flutter_map.dart';
import 'package:latlong2/latlong.dart' show LatLng;

import '../../api/api.dart';
import '../../app_scope.dart';
import '../../crops.dart';
import '../../geo.dart';
import '../../theme.dart';
import '../../widgets/common.dart';
import '../../widgets/farm_map.dart';
import 'farm_ready_screen.dart';

/// Add farm, step 2 of 3: swipe over the 10 m cells, then pick what grows there.
class PaintScreen extends StatefulWidget {
  const PaintScreen({super.key, required this.points});
  final List<GeoPoint> points;

  @override
  State<PaintScreen> createState() => _PaintScreenState();
}

class _PaintScreenState extends State<PaintScreen> {
  static const _brushPx = 12.0;

  late final List<LatLng> _outline = [
    for (final p in widget.points) LatLng(p.lat, p.lon),
  ];

  /// m² of each cell inside the border (edge cells are cut along it).
  late final Map<CellKey, double> _inside = cellsTouching(_outline);
  late final List<CellKey> _cells = _inside.keys.toList();
  late final double _totalM2 = polygonAreaM2(_outline);
  late final Set<CellKey> _cellSet = _cells.toSet();
  late final Map<CellKey, List<LatLng>> _corners = {
    for (final c in _cells) c: cellCorners(c),
  };

  final Map<CellKey, String> _crops = {};
  final Set<CellKey> _selected = {};
  String? _lastCrop;
  Offset? _lastDrag;

  void _brush(Offset o, MapCamera cam) {
    for (var dx = -_brushPx; dx <= _brushPx; dx += 4) {
      for (var dy = -_brushPx; dy <= _brushPx; dy += 4) {
        if (dx * dx + dy * dy > _brushPx * _brushPx) continue;
        final cell = Utm.cellOf(cam.screenOffsetToLatLng(o + Offset(dx, dy)));
        if (_cellSet.contains(cell)) _selected.add(cell);
      }
    }
  }

  void _dragTo(Offset o, MapCamera cam) {
    final from = _lastDrag ?? o;
    final steps = math.max(1, ((o - from).distance / 4).ceil());
    for (var i = 1; i <= steps; i++) {
      _brush(Offset.lerp(from, o, i / steps)!, cam);
    }
    _lastDrag = o;
    setState(() {});
  }

  void _tap(Offset o, MapCamera cam) {
    final cell = Utm.cellOf(cam.screenOffsetToLatLng(o));
    if (!_cellSet.contains(cell)) return;
    // A tap picks exactly the square under the finger; dragging paints wider.
    setState(
      () => _selected.contains(cell)
          ? _selected.remove(cell)
          : _selected.add(cell),
    );
  }

  void _assign(String crop) {
    if (_selected.isEmpty) {
      return showToast(context, AppScope.read(context).s.selectFirst);
    }
    setState(() {
      for (final c in _selected) {
        if (crop == 'empty') {
          _crops.remove(c);
        } else {
          _crops[c] = crop;
        }
      }
      _selected.clear();
      _lastCrop = crop;
    });
  }

  void _selectAll() => setState(
    () => _selected
      ..clear()
      ..addAll(_cells),
  );

  void _next() {
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => FarmReadyScreen(
          points: widget.points,
          outline: _outline,
          cells: _cells,
          inside: _inside,
          corners: _corners,
          crops: Map.of(_crops),
        ),
      ),
    );
  }

  Color? _style(CellKey c) {
    if (_selected.contains(c)) return Colors.white.withValues(alpha: 0.9);
    final crop = _crops[c];
    if (crop != null) return cropOf(crop).color.withValues(alpha: 0.85);
    return Colors.white.withValues(alpha: 0.12);
  }

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    final s = scope.s;
    final ku = scope.ku;
    final m2 = _selected.isEmpty
        ? _totalM2
        : _selected.fold(0.0, (a, c) => a + (_inside[c] ?? 0));
    // One square: show its size (10 x 10 m); an edge square also says how
    // much of it is inside the farm. More squares: their area in m².
    final String summary;
    if (_selected.length == 1) {
      final inside = _inside[_selected.first] ?? 100;
      final side = '${s.oneSquare}: \u206610 × 10\u2069 ${s.metres}';
      summary = inside >= 99.5
          ? side
          : '$side · \u2066${fmtM2(inside)}\u2069 ${s.m2} ${s.insideFarm}';
    } else {
      summary =
          '${_selected.isEmpty ? s.wholeFarm : s.selectedArea}: '
          '\u2066${fmtM2(m2)}\u2069 ${s.m2}';
    }

    return MapPage(
      step: 2,
      heading: MapHeading(
        ku: ku,
        title: s.paintTitle,
        sub: Text(s.paintSub, style: jText(ku, size: 15, color: JColors.muted)),
      ),
      map: Stack(
        children: [
          StyledMap(
            builder: (context, style) => FlutterMap(
              options: MapOptions(
                initialCameraFit: CameraFit.bounds(
                  bounds: LatLngBounds.fromPoints(_outline),
                  padding: const EdgeInsets.all(24),
                ),
                maxZoom: 21,
                backgroundColor: const Color(0xFF717A50),
                interactionOptions: const InteractionOptions(
                  flags: InteractiveFlag.none,
                ),
              ),
              children: [
                ...baseLayers(style, ku),
                ColoredBox(
                  color: JColors.ink.withValues(alpha: 0.16),
                  child: const SizedBox.expand(),
                ),
                CellLayer(corners: _corners, style: _style, clipTo: _outline),
                outlineLayer(_outline),
                Builder(
                  builder: (context) {
                    final cam = MapCamera.of(context);
                    return GestureDetector(
                      behavior: HitTestBehavior.opaque,
                      onPanStart: (d) {
                        _lastDrag = d.localPosition;
                        _dragTo(d.localPosition, cam);
                      },
                      onPanUpdate: (d) => _dragTo(d.localPosition, cam),
                      onPanEnd: (_) => _lastDrag = null,
                      onTapUp: (d) => _tap(d.localPosition, cam),
                      child: const SizedBox.expand(),
                    );
                  },
                ),
                mapAttribution(style),
              ],
            ),
          ),
          const Positioned(top: 10, right: 10, child: MapStyleButton()),
        ],
      ),
      sheet: Container(
        margin: const EdgeInsets.only(top: 14),
        padding: EdgeInsets.fromLTRB(
          16,
          8,
          16,
          16 + MediaQuery.paddingOf(context).bottom,
        ),
        decoration: const BoxDecoration(
          color: JColors.card,
          borderRadius: BorderRadius.vertical(top: Radius.circular(24)),
          boxShadow: [
            BoxShadow(
              color: Color(0x1A000000),
              blurRadius: 12,
              offset: Offset(0, -2),
            ),
          ],
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              width: 40,
              height: 5,
              decoration: BoxDecoration(
                color: JColors.line,
                borderRadius: BorderRadius.circular(3),
              ),
            ),
            const SizedBox(height: 8),
            Row(
              children: [
                Expanded(
                  child: Text(
                    s.sheetTitle,
                    style: jText(ku, size: 18, weight: FontWeight.w700),
                  ),
                ),
                TextButton(
                  onPressed: _selectAll,
                  child: Text(
                    s.selectAll,
                    style: jText(
                      ku,
                      size: 14,
                      weight: FontWeight.w700,
                      color: JColors.accent,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 4),
            for (final row in [kCrops.sublist(0, 6), kCrops.sublist(6)])
              Padding(
                padding: const EdgeInsets.only(bottom: 6),
                child: Row(
                  spacing: 4,
                  children: [
                    for (final c in row)
                      Expanded(
                        child: _CropChip(
                          crop: c,
                          label: s.crop(c.code),
                          selected: c.code == _lastCrop,
                          onTap: () => _assign(c.code),
                        ),
                      ),
                  ],
                ),
              ),
            const SizedBox(height: 4),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 5),
              decoration: BoxDecoration(
                color: JColors.accentSoft,
                borderRadius: BorderRadius.circular(999),
              ),
              child: Text(
                summary,
                style: jText(
                  ku,
                  size: 13,
                  weight: FontWeight.w700,
                  color: JColors.accent,
                ),
              ),
            ),
            const SizedBox(height: 12),
            PrimaryButton(label: s.next, onPressed: _next),
          ],
        ),
      ),
    );
  }
}

class _CropChip extends StatelessWidget {
  const _CropChip({
    required this.crop,
    required this.label,
    required this.selected,
    required this.onTap,
  });
  final Crop crop;
  final String label;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final ku = AppScope.of(context).ku;
    return GestureDetector(
      onTap: onTap,
      behavior: HitTestBehavior.opaque,
      child: Column(
        spacing: 4,
        children: [
          ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 56),
            child: AspectRatio(
              aspectRatio: 1,
              child: Container(
                alignment: Alignment.center,
                decoration: BoxDecoration(
                  color: selected ? JColors.accentSoft : JColors.bg,
                  borderRadius: BorderRadius.circular(14),
                  border: Border.all(
                    color: selected ? JColors.accent : JColors.line,
                    width: selected ? 1.5 : 1,
                  ),
                ),
                child: Stack(
                  alignment: Alignment.center,
                  children: [
                    Text(
                      crop.code == 'empty' ? '' : crop.emoji,
                      style: const TextStyle(fontSize: 24),
                    ),
                    if (crop.code == 'empty')
                      Container(
                        width: 22,
                        height: 22,
                        decoration: BoxDecoration(
                          border: Border.all(color: JColors.muted, width: 1.5),
                          borderRadius: BorderRadius.circular(5),
                        ),
                      ),
                  ],
                ),
              ),
            ),
          ),
          FittedBox(
            fit: BoxFit.scaleDown,
            child: Text(
              label,
              maxLines: 1,
              style: jText(
                ku,
                size: 11,
                weight: selected ? FontWeight.w700 : FontWeight.w500,
                color: selected ? JColors.accent : JColors.ink,
              ),
            ),
          ),
        ],
      ),
    );
  }
}
