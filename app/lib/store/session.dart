import 'local_store.dart';

/// The signed-in phone, kept on the phone so the app opens in the field
/// without internet and without signing in again.
class Session {
  const Session({required this.token, required this.digits});
  final String token;

  /// The number as typed, e.g. 07501234567.
  final String digits;

  static const _name = 'session';

  static Future<Session?> load() async {
    final j = await LocalStore.read(_name);
    if (j == null) return null;
    return Session(token: j['token'] as String, digits: j['digits'] as String);
  }

  Future<void> save() =>
      LocalStore.write(_name, {'token': token, 'digits': digits});

  static Future<void> clear() => LocalStore.delete(_name);
}
