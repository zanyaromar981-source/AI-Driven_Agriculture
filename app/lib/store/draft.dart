import '../api/api.dart';
import 'local_store.dart';

/// The field edge being marked right now. Saved on every new dot, so closing
/// the app, a dead battery or no internet never loses a walked border.
class Draft {
  static const _name = 'draft_edge';

  static Future<({List<GeoPoint> points, bool walk})?> load() async {
    final j = await LocalStore.read(_name);
    if (j == null) return null;
    final points = [
      for (final p in j['points'] as List)
        GeoPoint.fromJson(p as Map<String, dynamic>),
    ];
    if (points.isEmpty) return null;
    return (points: points, walk: j['walk'] as bool? ?? false);
  }

  /// Never throws: losing one draft save must not disturb marking the edge.
  static Future<void> save(List<GeoPoint> points, {required bool walk}) async {
    try {
      if (points.isEmpty) return await clear();
      await LocalStore.write(_name, {
        'walk': walk,
        'points': [for (final p in points) p.toJson()],
      });
    } catch (_) {}
  }

  static Future<void> clear() => LocalStore.delete(_name);
}
