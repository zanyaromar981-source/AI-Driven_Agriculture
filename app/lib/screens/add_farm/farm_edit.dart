import '../../api/api.dart';
import '../../geo.dart';

/// A farm opened for editing: its id, name, border and painted crops. The Add
/// farm screens take it to start from the farm as it is, instead of empty.
class FarmEdit {
  const FarmEdit({
    required this.id,
    required this.name,
    required this.points,
    required this.crops,
  });

  final String id;
  final String name;
  final List<GeoPoint> points;

  /// Painted cells only; cells not listed are "empty".
  final Map<CellKey, String> crops;

  /// The server keeps only lat/lon of the border, so accuracy is unknown (0).
  factory FarmEdit.fromFarm(Farm f) {
    final now = DateTime.now();
    return FarmEdit(
      id: f.summary.id,
      name: f.summary.name,
      points: [
        for (final p in f.outline)
          GeoPoint(lat: p.lat, lon: p.lon, accM: 0, t: now),
      ],
      crops: {
        for (final c in f.cells)
          if (c.crop != 'empty') (e: c.e, n: c.n): c.crop,
      },
    );
  }
}
