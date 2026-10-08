import 'package:flutter/material.dart';
import 'package:flutter_map/flutter_map.dart';
import 'package:latlong2/latlong.dart' show LatLng;

import '../../api/api.dart';
import '../../app_scope.dart';
import '../../crops.dart';
import '../../geo.dart';
import '../../store/draft.dart';
import '../../store/outbox.dart';
import '../../theme.dart';
import '../../widgets/common.dart';
import '../../widgets/farm_map.dart';

/// Add farm, step 3 of 3: check the crops, name the farm, save it.
class FarmReadyScreen extends StatefulWidget {
  const FarmReadyScreen({
    super.key,
    required this.points,
    required this.outline,
    required this.cells,
    required this.inside,
    required this.corners,
    required this.crops,
  });

  final List<GeoPoint> points;
  final List<LatLng> outline;
  final List<CellKey> cells;

  /// m² of each cell inside the border.
  final Map<CellKey, double> inside;
  final Map<CellKey, List<LatLng>> corners;
  final Map<CellKey, String> crops;

  @override
  State<FarmReadyScreen> createState() => _FarmReadyScreenState();
}

class _FarmReadyScreenState extends State<FarmReadyScreen> {
  String? _name;
  bool _busy = false;

  /// crop code -> its cells, biggest first; unpainted cells count as "empty".
  late final List<MapEntry<String, List<CellKey>>> _byCrop = () {
    final m = <String, List<CellKey>>{};
    for (final c in widget.cells) {
      m.putIfAbsent(widget.crops[c] ?? 'empty', () => []).add(c);
    }
    final list = m.entries.toList()
      ..sort((a, b) {
        if (a.key == 'empty') return 1;
        if (b.key == 'empty') return -1;
        return _m2(b.value).compareTo(_m2(a.value));
      });
    return list;
  }();

  int get _cropTypes => _byCrop.where((e) => e.key != 'empty').length;

  double _m2(List<CellKey> cells) =>
      cells.fold(0.0, (a, c) => a + (widget.inside[c] ?? 0));

  /// Exact area inside the walked border.
  late final double _totalM2 = polygonAreaM2(widget.outline);

  LatLng _centreOf(List<CellKey> cells) {
    var e = 0.0, n = 0.0, w = 0.0;
    for (final c in cells) {
      final k = widget.inside[c] ?? 100;
      e += (c.e * 10.0 + 5) * k;
      n += (c.n * 10.0 + 5) * k;
      w += k;
    }
    return Utm.toLatLng(e / w, n / w);
  }

  Future<void> _rename() async {
    final scope = AppScope.read(context);
    final ctrl = TextEditingController(text: _name ?? scope.s.newFarmName);
    final v = await showDialog<String>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text(
          scope.s.farmName,
          style: jText(scope.ku, size: 18, weight: FontWeight.w700),
        ),
        content: TextField(
          controller: ctrl,
          autofocus: true,
          style: jText(scope.ku, size: 16),
          onSubmitted: (t) => Navigator.pop(ctx, t),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: Text(scope.s.cancel, style: jText(scope.ku, size: 14)),
          ),
          FilledButton(
            style: FilledButton.styleFrom(backgroundColor: JColors.accent),
            onPressed: () => Navigator.pop(ctx, ctrl.text),
            child: Text(
              scope.s.ok,
              style: jText(scope.ku, size: 14, color: Colors.white),
            ),
          ),
        ],
      ),
    );
    if (v != null && v.trim().isNotEmpty) setState(() => _name = v.trim());
  }

  /// Save on the phone first (works with no internet), then try to upload.
  Future<void> _save() async {
    final scope = AppScope.read(context);
    final s = scope.s;
    setState(() => _busy = true);
    final name = _name ?? s.newFarmName;
    final request = NewFarmRequest(
      name: name,
      points: widget.points,
      cells: [
        for (final c in widget.cells)
          CellCrop(e: c.e, n: c.n, crop: widget.crops[c] ?? 'empty'),
      ],
      createdOfflineAt: DateTime.now(),
    );
    final centre = Utm.toLatLng(
      widget.outline
              .map((p) => Utm.fromLatLng(p.latitude, p.longitude).$1)
              .reduce((a, b) => a + b) /
          widget.outline.length,
      widget.outline
              .map((p) => Utm.fromLatLng(p.latitude, p.longitude).$2)
              .reduce((a, b) => a + b) /
          widget.outline.length,
    );
    final summary = FarmSummary(
      id: 'local',
      name: name,
      areaDunam: _round2(_totalM2 / 2500),
      crops: [
        for (final e in _byCrop)
          if (e.key != 'empty')
            CropShare(crop: e.key, dunam: _round2(_m2(e.value) / 2500)),
      ],
      status: FarmStatus.none,
      lat: centre.latitude,
      lon: centre.longitude,
    );
    await Outbox.instance.add(request, summary);
    await Draft.clear();
    final result = await Outbox.instance.flush(scope.api);
    if (!mounted) return;
    if (result.rejected.contains(name)) {
      showToast(context, s.crosses);
      setState(() => _busy = false);
      return;
    }
    showToast(
      context,
      Outbox.instance.items.isEmpty ? s.saved : s.savedOffline,
    );
    Navigator.of(context).popUntil((r) => r.isFirst);
  }

  static double _round2(double v) => (v * 100).round() / 100;

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    final s = scope.s;
    final ku = scope.ku;
    final muted = jText(ku, size: 15, color: JColors.muted);
    final mutedNum = latText(
      size: 15,
      weight: FontWeight.w700,
      color: JColors.muted,
    );

    return MapPage(
      step: 3,
      heading: MapHeading(
        ku: ku,
        title: _name ?? s.newFarmName,
        trailing: Material(
          color: JColors.accentSoft,
          shape: const CircleBorder(),
          child: InkWell(
            customBorder: const CircleBorder(),
            onTap: _rename,
            child: const SizedBox(
              width: 32,
              height: 32,
              child: Icon(Icons.edit_outlined, size: 16, color: JColors.accent),
            ),
          ),
        ),
        sub: Text.rich(
          TextSpan(
            children: [
              TextSpan(text: fmtM2(_totalM2), style: mutedNum),
              TextSpan(text: ' ${s.m2} · ', style: muted),
              TextSpan(text: '$_cropTypes', style: mutedNum),
              TextSpan(text: ' ${s.cropTypes}', style: muted),
            ],
          ),
        ),
      ),
      map: Stack(
        children: [
          StyledMap(
            builder: (context, style) => FlutterMap(
              options: MapOptions(
                initialCameraFit: CameraFit.bounds(
                  bounds: LatLngBounds.fromPoints(widget.outline),
                  padding: const EdgeInsets.all(20),
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
                CellLayer(
                  corners: widget.corners,
                  clipTo: widget.outline,
                  style: (c) {
                    final crop = widget.crops[c];
                    return crop == null
                        ? Colors.white.withValues(alpha: 0.12)
                        : cropOf(crop).color.withValues(alpha: 0.85);
                  },
                ),
                outlineLayer(widget.outline),
                MarkerLayer(
                  markers: [
                    for (final e in _byCrop)
                      if (e.key != 'empty')
                        Marker(
                          point: _centreOf(e.value),
                          width: 28,
                          height: 28,
                          child: Container(
                            alignment: Alignment.center,
                            decoration: BoxDecoration(
                              color: Colors.white.withValues(alpha: 0.95),
                              shape: BoxShape.circle,
                            ),
                            child: Text(
                              cropOf(e.key).emoji,
                              style: const TextStyle(fontSize: 15),
                            ),
                          ),
                        ),
                  ],
                ),
                mapAttribution(style),
              ],
            ),
          ),
          const Positioned(top: 10, right: 10, child: MapStyleButton()),
        ],
      ),
      below: [
        ConstrainedBox(
          constraints: const BoxConstraints(maxHeight: 112),
          child: Container(
            decoration: BoxDecoration(
              color: JColors.card,
              borderRadius: BorderRadius.circular(14),
            ),
            child: ListView.separated(
              shrinkWrap: true,
              padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 4),
              itemCount: _byCrop.length,
              separatorBuilder: (_, _) =>
                  const Divider(height: 1, color: JColors.line),
              itemBuilder: (_, i) {
                final e = _byCrop[i];
                return SizedBox(
                  height: 34,
                  child: Row(
                    spacing: 8,
                    children: [
                      Container(
                        width: 26,
                        height: 26,
                        alignment: Alignment.center,
                        decoration: BoxDecoration(
                          color: cropOf(e.key).color.withValues(alpha: 0.25),
                          borderRadius: BorderRadius.circular(7),
                        ),
                        child: Text(
                          e.key == 'empty' ? '' : cropOf(e.key).emoji,
                          style: const TextStyle(fontSize: 14),
                        ),
                      ),
                      Expanded(
                        child: Text(
                          s.crop(e.key),
                          style: jText(ku, size: 15, weight: FontWeight.w700),
                        ),
                      ),
                      Text(
                        fmtM2(_m2(e.value)),
                        style: latText(
                          size: 14,
                          weight: FontWeight.w700,
                          color: JColors.muted,
                        ),
                      ),
                      Text(
                        s.m2,
                        style: jText(
                          ku,
                          size: 14,
                          weight: FontWeight.w600,
                          color: JColors.muted,
                        ),
                      ),
                    ],
                  ),
                );
              },
            ),
          ),
        ),
        InfoNote(body: s.satNote, icon: Icons.satellite_alt_outlined),
        GhostButton(
          label: s.changeCrops,
          icon: Icons.grass_rounded,
          onPressed: _busy ? null : () => Navigator.of(context).pop(),
        ),
        PrimaryButton(
          label: s.save,
          icon: Icons.check_rounded,
          loading: _busy,
          onPressed: _save,
        ),
      ],
    );
  }
}
