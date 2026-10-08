import 'package:flutter/material.dart';
import 'package:flutter_map/flutter_map.dart';
import 'package:latlong2/latlong.dart' show LatLng;

import '../app_scope.dart';
import '../geo.dart';
import '../store/tile_cache.dart';
import '../theme.dart';
import 'common.dart';
import 'header.dart';
import 'place_names.dart';

/// Map styles the farmer can switch between; the choice is shared by every map.
enum MapStyle { satellite, map, terrain }

final mapStyle = ValueNotifier<MapStyle>(MapStyle.satellite);

/// Where each style's map pictures come from.
const kSatelliteTiles = (
  url:
      'https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}',
  maxNativeZoom: 18,
);
const kStreetTiles = (
  url: 'https://tile.openstreetmap.org/{z}/{x}/{y}.png',
  maxNativeZoom: 19,
);
const kTerrainTiles = (
  url:
      'https://server.arcgisonline.com/ArcGIS/rest/services/World_Topo_Map/MapServer/tile/{z}/{y}/{x}',
  maxNativeZoom: 16,
);

({String url, int maxNativeZoom}) tilesFor(MapStyle style) => switch (style) {
  MapStyle.satellite => kSatelliteTiles,
  MapStyle.map => kStreetTiles,
  MapStyle.terrain => kTerrainTiles,
};

/// Every tile shown is kept on the phone (see TileCache), so maps work offline.
TileLayer _tiles(({String url, int maxNativeZoom}) t) => TileLayer(
  key: ValueKey(t.url),
  urlTemplate: t.url,
  userAgentPackageName: 'krd.jutyar',
  maxNativeZoom: t.maxNativeZoom,
  tileProvider: CachedTileProvider(),
);

/// Download the map around a farm for the satellite view and the chosen
/// style, so the farm shows on its map without internet.
Future<void> prefetchFarmMap(List<LatLng> outline) =>
    TileCache.prefetchArea(outline, [
      kSatelliteTiles,
      if (mapStyle.value != MapStyle.satellite) tilesFor(mapStyle.value),
    ]);

/// Background tiles for [style] plus Kurdish place names on the satellite view.
/// Put these first in every FlutterMap's children.
List<Widget> baseLayers(MapStyle style, bool ku) => switch (style) {
  MapStyle.satellite => [_tiles(kSatelliteTiles), PlaceLabelLayer(ku: ku)],
  MapStyle.map => [_tiles(kStreetTiles)],
  MapStyle.terrain => [_tiles(kTerrainTiles)],
};

/// Required credit line for the tiles and names in use.
Widget mapAttribution(MapStyle style) => Align(
  alignment: Alignment.bottomRight,
  child: Container(
    margin: const EdgeInsets.all(4),
    padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 1),
    decoration: BoxDecoration(
      color: Colors.white.withValues(alpha: 0.7),
      borderRadius: BorderRadius.circular(4),
    ),
    child: Text(
      switch (style) {
        MapStyle.satellite => 'Esri, Maxar · names © OpenStreetMap',
        MapStyle.map => '© OpenStreetMap contributors',
        MapStyle.terrain => 'Esri',
      },
      textDirection: TextDirection.ltr,
      style: const TextStyle(fontSize: 9, color: Color(0xFF333333)),
    ),
  ),
);

/// Rebuilds [builder] whenever the map style changes.
class StyledMap extends StatelessWidget {
  const StyledMap({super.key, required this.builder});
  final Widget Function(BuildContext context, MapStyle style) builder;

  @override
  Widget build(BuildContext context) => ValueListenableBuilder<MapStyle>(
    valueListenable: mapStyle,
    builder: (c, s, _) => builder(c, s),
  );
}

/// Round button that opens the style picker.
class MapStyleButton extends StatelessWidget {
  const MapStyleButton({super.key});

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    final s = scope.s;
    return ValueListenableBuilder<MapStyle>(
      valueListenable: mapStyle,
      builder: (context, current, _) => PopupMenuButton<MapStyle>(
        tooltip: s.mapStyle,
        initialValue: current,
        onSelected: (v) => mapStyle.value = v,
        color: JColors.card,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(14)),
        itemBuilder: (_) => [
          for (final st in MapStyle.values)
            PopupMenuItem<MapStyle>(
              value: st,
              child: Row(
                spacing: 10,
                children: [
                  Icon(
                    switch (st) {
                      MapStyle.satellite => Icons.satellite_alt_outlined,
                      MapStyle.map => Icons.map_outlined,
                      MapStyle.terrain => Icons.terrain_outlined,
                    },
                    size: 20,
                    color: st == current ? JColors.accent : JColors.muted,
                  ),
                  Expanded(
                    child: Text(
                      s.styleName(st.name),
                      style: jText(
                        scope.ku,
                        size: 15,
                        weight: st == current
                            ? FontWeight.w700
                            : FontWeight.w500,
                      ),
                    ),
                  ),
                  if (st == current)
                    const Icon(
                      Icons.check_rounded,
                      size: 18,
                      color: JColors.accent,
                    ),
                ],
              ),
            ),
        ],
        child: Container(
          width: 40,
          height: 40,
          decoration: BoxDecoration(
            color: Colors.white.withValues(alpha: 0.92),
            shape: BoxShape.circle,
            boxShadow: const [
              BoxShadow(
                color: Color(0x22000000),
                blurRadius: 4,
                offset: Offset(0, 1),
              ),
            ],
          ),
          child: const Icon(
            Icons.layers_outlined,
            size: 20,
            color: JColors.ink,
          ),
        ),
      ),
    );
  }
}

/// The farm outline: white border, faint fill.
PolygonLayer outlineLayer(List<LatLng> outline) => PolygonLayer(
  polygons: [
    Polygon(
      points: outline,
      color: Colors.white.withValues(alpha: 0.06),
      borderColor: Colors.white.withValues(alpha: 0.85),
      borderStrokeWidth: 2,
    ),
  ],
);

/// How one cell is drawn. Null fill = not drawn.
typedef CellStyle = Color? Function(CellKey cell);

/// Draws thousands of 10 m cells in one pass, grouped by colour.
class CellLayer extends StatelessWidget {
  const CellLayer({
    super.key,
    required this.corners,
    required this.style,
    this.clipTo,
  });

  /// Pre-computed corners per cell (see [cellCorners]).
  final Map<CellKey, List<LatLng>> corners;
  final CellStyle style;

  /// Farm outline: edge cells are cut along it so nothing shows outside the farm.
  final List<LatLng>? clipTo;

  @override
  Widget build(BuildContext context) {
    final camera = MapCamera.of(context);
    return SizedBox.expand(
      child: CustomPaint(painter: _CellPainter(camera, corners, style, clipTo)),
    );
  }
}

class _CellPainter extends CustomPainter {
  _CellPainter(this.camera, this.corners, this.style, this.clipTo);
  final MapCamera camera;
  final Map<CellKey, List<LatLng>> corners;
  final CellStyle style;
  final List<LatLng>? clipTo;

  @override
  void paint(Canvas canvas, Size size) {
    final clip = clipTo;
    if (clip != null && clip.length >= 3) {
      canvas.clipPath(
        Path()..addPolygon([
          for (final p in clip) camera.latLngToScreenOffset(p),
        ], true),
      );
    }
    final groups = <Color, Path>{};
    final grid = Path();
    double? cellPx;
    corners.forEach((cell, pts) {
      final o = [for (final p in pts) camera.latLngToScreenOffset(p)];
      cellPx ??= (o[1] - o[0]).distance;
      final path = Path()..addPolygon(o, true);
      grid.addPath(path, Offset.zero);
      final c = style(cell);
      if (c != null) groups.putIfAbsent(c, Path.new).addPath(path, Offset.zero);
    });
    groups.forEach((c, p) => canvas.drawPath(p, Paint()..color = c));
    if ((cellPx ?? 0) >= 4) {
      canvas.drawPath(
        grid,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 0.6
          ..color = Colors.black.withValues(alpha: 0.22),
      );
    }
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => true;
}

/// Page frame for the map screens: header, progress, heading, a big map,
/// then whatever goes below it.
class MapPage extends StatelessWidget {
  const MapPage({
    super.key,
    required this.step,
    required this.heading,
    required this.map,
    this.below = const [],
    this.sheet,
  });

  final int step;
  final Widget heading;
  final Widget map;
  final List<Widget> below;
  final Widget? sheet;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: JColors.bg,
      body: SafeArea(
        bottom: sheet == null,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const JutyarHeader(),
            Padding(
              padding: const EdgeInsets.fromLTRB(20, 12, 20, 0),
              child: ProgressSteps(step: step),
            ),
            Padding(
              padding: const EdgeInsets.fromLTRB(20, 16, 20, 14),
              child: heading,
            ),
            Expanded(
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: 20),
                child: ClipRRect(
                  borderRadius: BorderRadius.circular(14),
                  child: ColoredBox(color: const Color(0xFF717A50), child: map),
                ),
              ),
            ),
            if (below.isNotEmpty)
              Padding(
                padding: const EdgeInsets.fromLTRB(20, 14, 20, 20),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  spacing: 10,
                  children: below,
                ),
              ),
            ?sheet,
          ],
        ),
      ),
    );
  }
}

/// Title + one muted line under it.
class MapHeading extends StatelessWidget {
  const MapHeading({
    super.key,
    required this.ku,
    required this.title,
    required this.sub,
    this.trailing,
  });
  final bool ku;
  final String title;
  final Widget sub;
  final Widget? trailing;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      spacing: 4,
      children: [
        Row(
          spacing: 8,
          children: [
            Flexible(
              child: Text(
                title,
                style: jText(
                  ku,
                  size: 24,
                  weight: FontWeight.w700,
                  height: 1.3,
                ),
                overflow: TextOverflow.ellipsis,
              ),
            ),
            ?trailing,
          ],
        ),
        sub,
      ],
    );
  }
}
