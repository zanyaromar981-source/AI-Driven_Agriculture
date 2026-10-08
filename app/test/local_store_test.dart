import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/store/local_store.dart';
import 'package:path_provider_platform_interface/path_provider_platform_interface.dart';
import 'package:plugin_platform_interface/plugin_platform_interface.dart';

class _TempPaths extends PathProviderPlatform with MockPlatformInterfaceMixin {
  _TempPaths(this.dir);
  final String dir;

  @override
  Future<String?> getApplicationSupportPath() async => dir;
}

void main() {
  test(
    '20 writes to the same file at once all succeed, last one wins',
    () async {
      final dir = await Directory.systemTemp.createTemp('jutyar_store');
      PathProviderPlatform.instance = _TempPaths(dir.path);

      await Future.wait([
        for (var i = 0; i < 20; i++) LocalStore.write('farms_cache', {'n': i}),
      ]);

      expect((await LocalStore.read('farms_cache'))!['n'], 19);
      final leftovers = dir.listSync().where((f) => f.path.endsWith('.tmp'));
      expect(leftovers, isEmpty);

      await Future.wait([
        LocalStore.write('draft_edge', {'a': 1}),
        LocalStore.delete('draft_edge'),
      ]);
      expect(await LocalStore.read('draft_edge'), isNull);
      await dir.delete(recursive: true);
    },
  );
}
