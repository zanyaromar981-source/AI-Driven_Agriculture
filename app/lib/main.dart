import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'api/api.dart';
import 'api/fake_api.dart';
import 'app_scope.dart';
import 'l10n/strings.dart';
import 'screens/my_farms_screen.dart';
import 'screens/phone_screen.dart';
import 'store/outbox.dart';
import 'store/session.dart';
import 'theme.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  SystemChrome.setSystemUIOverlayStyle(
    const SystemUiOverlayStyle(
      statusBarColor: Colors.transparent,
      statusBarIconBrightness: Brightness.dark,
      statusBarBrightness: Brightness.light,
    ),
  );
  final session = await Session.load();
  await Outbox.instance.load();
  runApp(JutyarApp(session: session));
}

class JutyarApp extends StatefulWidget {
  const JutyarApp({super.key, this.api, this.session});

  /// Swap in the real server here later; FakeApi is the demo default.
  final Api? api;

  /// Signed in earlier on this phone: open straight on My farms (works offline).
  final Session? session;

  @override
  State<JutyarApp> createState() => _JutyarAppState();
}

class _JutyarAppState extends State<JutyarApp> with WidgetsBindingObserver {
  bool _ku = true;
  late final Api _api = widget.api ?? FakeApi();
  Timer? _retry;

  @override
  void initState() {
    super.initState();
    final s = widget.session;
    if (s != null) _api.useToken(s.token);
    WidgetsBinding.instance.addObserver(this);
    // Farms saved without internet go up by themselves when it comes back.
    _retry = Timer.periodic(const Duration(seconds: 30), (_) => _flush());
    _flush();
  }

  @override
  void dispose() {
    _retry?.cancel();
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) _flush();
  }

  void _flush() {
    if (Outbox.instance.items.isNotEmpty) Outbox.instance.flush(_api);
  }

  @override
  Widget build(BuildContext context) {
    final session = widget.session;
    return AppScope(
      ku: _ku,
      s: S(_ku),
      api: _api,
      setKu: (v) => setState(() => _ku = v),
      child: MaterialApp(
        title: 'Jutyar',
        debugShowCheckedModeBanner: false,
        theme: jutyarTheme(),
        builder: (context, child) => Directionality(
          textDirection: _ku ? TextDirection.rtl : TextDirection.ltr,
          child: child ?? const SizedBox.shrink(),
        ),
        home: session == null
            ? const PhoneScreen()
            : MyFarmsScreen(digits: session.digits),
      ),
    );
  }
}
