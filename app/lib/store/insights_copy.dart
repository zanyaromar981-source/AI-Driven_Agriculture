import 'local_store.dart';

/// The phone's copy of a farm's Field history (`insights_<id>`): what the
/// server sent last (`data`), when this phone first saw it unfinished
/// (`waiting_since`) and, after an edit, the history of the farm it replaced
/// (`carried`), shown until the new farm's analysis has the same topics.
class InsightsCopy {
  static String name(String farmId) => 'insights_$farmId';

  /// The server has no "failed" mark: a topic that could not be read simply
  /// never arrives. After this long the screens stop promising minutes.
  static const giveUpAfter = Duration(minutes: 20);

  static List<Map<String, dynamic>> _topics(Object? j) => [
    if (j is Map<String, dynamic>)
      for (final t in (j['topics'] as List? ?? const []))
        if (t is Map<String, dynamic>) t,
  ];

  /// The server's topics, then the carried ones it does not have yet.
  static Map<String, dynamic>? merged(Map<String, dynamic>? copy) {
    final data = copy?['data'];
    final carried = copy?['carried'];
    if (carried is! Map<String, dynamic>) {
      return data is Map<String, dynamic> ? data : null;
    }
    final base = data is Map<String, dynamic> ? data : <String, dynamic>{};
    final have = {for (final t in _topics(base)) t['topic']};
    return {
      ...carried,
      ...base,
      'topics': [
        ..._topics(base),
        for (final t in _topics(carried))
          if (!have.contains(t['topic'])) t,
      ],
    };
  }

  /// Topics on show that still come from the farm before the edit.
  static Set<String> carriedTopics(Map<String, dynamic>? copy) {
    final have = {for (final t in _topics(copy?['data'])) t['topic']};
    return {
      for (final t in _topics(copy?['carried']))
        if (!have.contains(t['topic'])) t['topic'] as String,
    };
  }

  /// True when the analysis has been unfinished for longer than
  /// [giveUpAfter] since this phone first saw it.
  static bool stalled(Map<String, dynamic>? copy, {DateTime? now}) {
    final s = copy?['waiting_since'];
    final t = s is String ? DateTime.tryParse(s) : null;
    return t != null &&
        (now ?? DateTime.now()).toUtc().difference(t) > giveUpAfter;
  }

  /// An edit saved the farm [fromId] as the new farm [toId] (the server has
  /// no edit call, and a new farm is analysed from nothing). Its history
  /// stays on show, labelled, until the new analysis catches up.
  static Future<void> carry(String fromId, String toId) async {
    try {
      final old = merged(await LocalStore.read(name(fromId)));
      if (old == null || _topics(old).isEmpty) return;
      await LocalStore.write(name(toId), {
        'carried': old,
        'waiting_since': DateTime.now().toUtc().toIso8601String(),
      });
    } catch (_) {}
  }
}
