import 'dart:convert';
import 'dart:io';

import 'package:path_provider/path_provider.dart';

/// Small JSON files in the app's private folder on the phone. Works without
/// internet and survives app restarts. Writes go to a temp file first, so a
/// crash mid-write never leaves a broken file, and writes to the same file
/// wait for each other (two at once used to fail with "cannot rename").
class LocalStore {
  static Directory? _dir;
  static final Map<String, Future<void>> _queue = {};
  static int _seq = 0;

  /// Run [job] after every earlier write or delete of [name] has finished.
  static Future<void> _serial(String name, Future<void> Function() job) {
    final next = (_queue[name] ?? Future<void>.value())
        .catchError((Object _) {})
        .then((_) => job());
    _queue[name] = next;
    return next;
  }

  static Future<File> _file(String name) async {
    _dir ??= await getApplicationSupportDirectory();
    return File('${_dir!.path}/$name.json');
  }

  static Future<Map<String, dynamic>?> read(String name) async {
    try {
      final f = await _file(name);
      if (!await f.exists()) return null;
      return jsonDecode(await f.readAsString()) as Map<String, dynamic>;
    } catch (_) {
      return null;
    }
  }

  static Future<void> write(String name, Map<String, dynamic> data) =>
      _serial(name, () async {
        final f = await _file(name);
        final tmp = File('${f.path}.${_seq++}.tmp');
        await tmp.writeAsString(jsonEncode(data), flush: true);
        await tmp.rename(f.path);
      });

  static Future<void> delete(String name) => _serial(name, () async {
    final f = await _file(name);
    if (await f.exists()) await f.delete();
  });
}
