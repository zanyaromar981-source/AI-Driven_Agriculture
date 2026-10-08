import 'package:flutter/widgets.dart';

import 'api/api.dart';
import 'l10n/strings.dart';

/// Shared app state: language, strings and the API in use.
class AppScope extends InheritedWidget {
  const AppScope({
    super.key,
    required this.ku,
    required this.s,
    required this.api,
    required this.setKu,
    required super.child,
  });

  final bool ku;
  final S s;
  final Api api;
  final ValueChanged<bool> setKu;

  /// Use inside build(): rebuilds when the language changes.
  static AppScope of(BuildContext c) =>
      c.dependOnInheritedWidgetOfExactType<AppScope>()!;

  /// Use inside callbacks and initState(): no rebuild dependency.
  static AppScope read(BuildContext c) =>
      c.getInheritedWidgetOfExactType<AppScope>()!;

  @override
  bool updateShouldNotify(AppScope old) => ku != old.ku || api != old.api;
}
