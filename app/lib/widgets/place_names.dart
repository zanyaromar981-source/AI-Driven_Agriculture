import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_map/flutter_map.dart';
import 'package:latlong2/latlong.dart' show LatLng;

import '../theme.dart';

/// A named place from OpenStreetMap (city, town, village, ...).
class Place {
  const Place({
    required this.at,
    required this.kind,
    required this.name,
    this.ckb,
    this.en,
  });
  final LatLng at;
  final String kind;
  final String name;
  final String? ckb;
  final String? en;

  String label(bool ku) => ku ? (ckb ?? name) : (en ?? name);

  /// Zoom from which this kind of place gets a label.
  double get minZoom => switch (kind) {
    'city' => 8,
    'town' => 10,
    'village' || 'suburb' => 12,
    'locality' => 15,
    _ => 14,
  };
}

/// Loads place names for the area on screen from OpenStreetMap (Overpass API)
/// and keeps them in memory. Squares of 0.1 degree are fetched once each.
class PlaceNames extends ChangeNotifier {
  PlaceNames._();
  static final instance = PlaceNames._();

  static const _cell = 0.1;
  static const _endpoint = 'https://overpass-api.de/api/interpreter';

  final Map<int, Place> _places = {};
  final Set<String> _done = {};
  final Set<String> _loading = {};

  Future<void> load(LatLngBounds b, double zoom) async {
    if (zoom < 9) return;
    final s0 = (b.south / _cell).floor(), s1 = (b.north / _cell).floor();
    final w0 = (b.west / _cell).floor(), w1 = (b.east / _cell).floor();
    if ((s1 - s0 + 1) * (w1 - w0 + 1) > 30) return;
    final missing = <(int, int)>[
      for (var i = s0; i <= s1; i++)
        for (var j = w0; j <= w1; j++)
          if (!_done.contains('$i,$j') && !_loading.contains('$i,$j')) (i, j),
    ];
    if (missing.isEmpty) return;
    final keys = [for (final (i, j) in missing) '$i,$j'];
    _loading.addAll(keys);
    final south =
        missing.map((c) => c.$1).reduce((a, b) => a < b ? a : b) * _cell;
    final north =
        (missing.map((c) => c.$1).reduce((a, b) => a > b ? a : b) + 1) * _cell;
    final west =
        missing.map((c) => c.$2).reduce((a, b) => a < b ? a : b) * _cell;
    final east =
        (missing.map((c) => c.$2).reduce((a, b) => a > b ? a : b) + 1) * _cell;
    final query =
        '[out:json][timeout:15];'
        'node["place"~"^(city|town|village|hamlet|suburb|neighbourhood|quarter|locality)\$"]["name"]'
        '(${south.toStringAsFixed(4)},${west.toStringAsFixed(4)},${north.toStringAsFixed(4)},${east.toStringAsFixed(4)});'
        'out body 800;';
    try {
      final json = await _post(query).timeout(const Duration(seconds: 20));
      for (final e in (json['elements'] as List? ?? const [])) {
        final m = e as Map<String, dynamic>;
        final tags = (m['tags'] as Map?)?.cast<String, dynamic>() ?? const {};
        final name = tags['name'] as String?;
        if (name == null || m['lat'] == null) continue;
        _places[m['id'] as int] = Place(
          at: LatLng(
            (m['lat'] as num).toDouble(),
            (m['lon'] as num).toDouble(),
          ),
          kind: tags['place'] as String? ?? 'locality',
          name: name,
          ckb: tags['name:ckb'] as String?,
          en: tags['name:en'] as String?,
        );
      }
      _done.addAll(keys);
      notifyListeners();
    } catch (_) {
      // No internet or Overpass busy: labels stay missing and are retried on the next move.
    } finally {
      _loading.removeAll(keys);
    }
  }

  Future<Map<String, dynamic>> _post(String query) async {
    final client = HttpClient()
      ..connectionTimeout = const Duration(seconds: 10);
    try {
      final req = await client.postUrl(Uri.parse(_endpoint));
      req.headers.set(
        HttpHeaders.userAgentHeader,
        'krd.jutyar (SmartSuli farm app prototype)',
      );
      req.headers.contentType = ContentType(
        'application',
        'x-www-form-urlencoded',
        charset: 'utf-8',
      );
      req.write('data=${Uri.encodeQueryComponent(query)}');
      final res = await req.close();
      if (res.statusCode != 200) {
        throw HttpException('overpass ${res.statusCode}');
      }
      return jsonDecode(await res.transform(utf8.decoder).join())
          as Map<String, dynamic>;
    } finally {
      client.close();
    }
  }

  List<Place> visible(LatLngBounds b, double zoom) => [
    for (final p in _places.values)
      if (zoom >= p.minZoom && b.contains(p.at)) p,
  ];
}

/// Draws place names on the map; loads more when the map stops moving.
class PlaceLabelLayer extends StatefulWidget {
  const PlaceLabelLayer({super.key, required this.ku});
  final bool ku;

  @override
  State<PlaceLabelLayer> createState() => _PlaceLabelLayerState();
}

class _PlaceLabelLayerState extends State<PlaceLabelLayer> {
  Timer? _debounce;

  @override
  void initState() {
    super.initState();
    PlaceNames.instance.addListener(_changed);
  }

  void _changed() {
    if (mounted) setState(() {});
  }

  @override
  void dispose() {
    _debounce?.cancel();
    PlaceNames.instance.removeListener(_changed);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final cam = MapCamera.of(context);
    final bounds = cam.visibleBounds;
    final zoom = cam.zoom;
    _debounce?.cancel();
    _debounce = Timer(
      const Duration(milliseconds: 500),
      () => PlaceNames.instance.load(bounds, zoom),
    );
    return MarkerLayer(
      markers: [
        for (final p in PlaceNames.instance.visible(bounds, zoom))
          Marker(
            point: p.at,
            width: 200,
            height: 30,
            child: IgnorePointer(
              child: Center(
                child: Text(
                  p.label(widget.ku),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style:
                      jText(
                        widget.ku,
                        size: switch (p.kind) {
                          'city' => 16,
                          'town' => 15,
                          'village' => 13.5,
                          _ => 12,
                        },
                        weight: p.kind == 'city' || p.kind == 'town'
                            ? FontWeight.w700
                            : FontWeight.w600,
                        color: Colors.white,
                      ).copyWith(
                        shadows: const [
                          Shadow(color: Color(0xCC000000), blurRadius: 3),
                          Shadow(
                            color: Color(0x99000000),
                            blurRadius: 1,
                            offset: Offset(0, 1),
                          ),
                        ],
                      ),
                ),
              ),
            ),
          ),
      ],
    );
  }
}
