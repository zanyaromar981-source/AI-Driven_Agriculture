import 'dart:io';
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/foundation.dart';
import 'package:flutter/painting.dart';
import 'package:flutter_map/flutter_map.dart';
import 'package:latlong2/latlong.dart' show LatLng;
import 'package:path_provider/path_provider.dart';

/// Map pictures (tiles) kept on the phone. Every tile the app shows is saved
/// the first time, and the area around each farm is downloaded ahead, so a
/// farm can be seen on its map with no internet.
class TileCache {
  static Directory? _dir;
  static final HttpClient _http = HttpClient()
    ..connectionTimeout = const Duration(seconds: 10);
  static const _userAgent = 'flutter_map (krd.jutyar)';

  static Future<Directory> _folder() async {
    _dir ??= Directory(
      '${(await getApplicationSupportDirectory()).path}/tiles',
    );
    await _dir!.create(recursive: true);
    return _dir!;
  }

  /// One file per tile URL, with a name that is safe on every phone.
  static String _name(String url) => url
      .replaceFirst(RegExp(r'^https?://'), '')
      .replaceAll(RegExp(r'[^A-Za-z0-9._-]'), '_');

  static Future<File> _file(String url) async =>
      File('${(await _folder()).path}/${_name(url)}');

  static Future<bool> has(String url) async => (await _file(url)).exists();

  /// The tile from the phone, or from the internet (then saved on the phone).
  static Future<Uint8List> get(
    String url, {
    String userAgent = _userAgent,
  }) async {
    final f = await _file(url);
    try {
      if (await f.exists()) return await f.readAsBytes();
    } catch (_) {}
    final req = await _http.getUrl(Uri.parse(url));
    req.headers.set(HttpHeaders.userAgentHeader, userAgent);
    final res = await req.close().timeout(const Duration(seconds: 20));
    if (res.statusCode != 200) {
      throw HttpException('tile ${res.statusCode}', uri: Uri.parse(url));
    }
    final bytes = await consolidateHttpClientResponseBytes(res);
    try {
      final tmp = File(
        '${f.path}.${DateTime.now().microsecondsSinceEpoch}.tmp',
      );
      await tmp.writeAsBytes(bytes, flush: true);
      await tmp.rename(f.path);
    } catch (_) {
      // Saving is a bonus; showing the tile is what matters.
    }
    return bytes;
  }

  static final Set<String> _prefetched = {};

  /// Download the map around [outline] (plus [padM] metres) for zoom
  /// [minZoom]..[maxZoom], for each URL template ({z}, {x}, {y}). Tiles already
  /// on the phone are skipped. Stops quietly with no internet. Returns how
  /// many tiles were downloaded.
  static Future<int> prefetchArea(
    List<LatLng> outline,
    List<({String url, int maxNativeZoom})> templates, {
    int minZoom = 14,
    int maxZoom = 18,
    double padM = 80,
    int maxTiles = 400,
  }) async {
    if (outline.length < 3) return 0;
    final key =
        '${outline.first.latitude.toStringAsFixed(5)},${outline.first.longitude.toStringAsFixed(5)},'
        '${outline.length},${templates.map((t) => t.url).join('|')}';
    if (!_prefetched.add(key)) return 0;
    final lats = outline.map((p) => p.latitude);
    final lons = outline.map((p) => p.longitude);
    final dLat = padM / 111320;
    final dLon = padM / (111320 * math.cos(lats.first * math.pi / 180));
    final south = lats.reduce(math.min) - dLat,
        north = lats.reduce(math.max) + dLat;
    final west = lons.reduce(math.min) - dLon,
        east = lons.reduce(math.max) + dLon;
    var done = 0, seen = 0;
    try {
      for (final t in templates) {
        for (var z = minZoom; z <= math.min(maxZoom, t.maxNativeZoom); z++) {
          final x0 = _tileX(west, z), x1 = _tileX(east, z);
          final y0 = _tileY(north, z), y1 = _tileY(south, z);
          for (var x = x0; x <= x1; x++) {
            for (var y = y0; y <= y1; y++) {
              if (++seen > maxTiles) return done;
              final url = t.url
                  .replaceAll('{z}', '$z')
                  .replaceAll('{x}', '$x')
                  .replaceAll('{y}', '$y');
              if (await has(url)) continue;
              await get(url);
              done++;
            }
          }
        }
      }
    } catch (_) {
      _prefetched.remove(key); // no internet: try again next time
    }
    return done;
  }

  static int _tileX(double lon, int z) =>
      ((lon + 180) / 360 * (1 << z)).floor();
  static int _tileY(double lat, int z) {
    final r = lat * math.pi / 180;
    return ((1 - math.log(math.tan(r) + 1 / math.cos(r)) / math.pi) /
            2 *
            (1 << z))
        .floor();
  }
}

/// Tile source for flutter_map that reads from [TileCache] first.
class CachedTileProvider extends TileProvider {
  CachedTileProvider({super.headers});

  @override
  ImageProvider getImage(TileCoordinates coordinates, TileLayer options) =>
      _CachedTile(
        getTileUrl(coordinates, options),
        headers['User-Agent'] ?? TileCache._userAgent,
      );
}

class _CachedTile extends ImageProvider<_CachedTile> {
  const _CachedTile(this.url, this.userAgent);
  final String url;
  final String userAgent;

  @override
  Future<_CachedTile> obtainKey(ImageConfiguration configuration) =>
      SynchronousFuture(this);

  @override
  ImageStreamCompleter loadImage(
    _CachedTile key,
    ImageDecoderCallback decode,
  ) => MultiFrameImageStreamCompleter(
    codec: _load(decode),
    scale: 1,
    debugLabel: url,
  );

  Future<ui.Codec> _load(ImageDecoderCallback decode) async {
    final bytes = await TileCache.get(url, userAgent: userAgent);
    return decode(await ui.ImmutableBuffer.fromUint8List(bytes));
  }

  @override
  bool operator ==(Object other) => other is _CachedTile && other.url == url;

  @override
  int get hashCode => url.hashCode;
}
