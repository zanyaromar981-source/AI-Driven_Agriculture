import 'package:flutter/material.dart';

import '../../api/api.dart';
import '../../app_scope.dart';
import '../../store/local_store.dart';
import '../../store/session.dart';
import '../../theme.dart';
import '../../widgets/common.dart';
import '../../widgets/header.dart';
import '../phone_screen.dart';
import '../tabs.dart';

/// Settings (Pencil: Screen/Settings): language, phone number, farms,
/// notifications, about, and deleting the account. Push and account delete
/// are not on the server yet (FRONTEND.md 14): the switches are kept on the
/// phone and delete says so instead of pretending.
class SettingsScreen extends StatefulWidget {
  const SettingsScreen({super.key, required this.farm});
  final FarmSummary farm;

  @override
  State<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends State<SettingsScreen> {
  static const _notifyFile = 'notify';
  String? _phone;
  int? _farms;
  bool _redAlerts = true;
  bool _weeklyPlan = true;
  bool _deleting = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    _loadNotify();
    final api = AppScope.read(context).api;
    try {
      final me = await api.getMe();
      if (mounted) setState(() => _phone = me.phone);
    } on ApiException {
      // Offline or refused: the row shows a dash.
    }
    try {
      final f = await api.getFarms();
      if (mounted) setState(() => _farms = f.length);
    } on ApiException {
      // Same.
    }
  }

  /// The switches as saved on this phone (push is not on the server yet).
  Future<void> _loadNotify() async {
    Map<String, dynamic>? n;
    try {
      n = await LocalStore.read(_notifyFile);
    } catch (_) {
      // No file store (tests): the switches start on.
    }
    final saved = n;
    if (mounted && saved != null) {
      setState(() {
        _redAlerts = saved['red_alerts'] != false;
        _weeklyPlan = saved['weekly_plan'] != false;
      });
    }
  }

  Future<void> _saveNotify() async {
    try {
      await LocalStore.write(_notifyFile, {
        'red_alerts': _redAlerts,
        'weekly_plan': _weeklyPlan,
      });
    } catch (_) {
      // Not saved: the switch still shows the choice until the app closes.
    }
  }

  /// "+9647501234567" as "+964 750 123 4567".
  static String _fmtPhone(String p) => p.startsWith('+964') && p.length == 14
      ? '+964 ${p.substring(4, 7)} ${p.substring(7, 10)} ${p.substring(10)}'
      : p;

  Future<void> _signOut() async {
    await Session.clear();
    if (!mounted) return;
    Navigator.of(context).pushAndRemoveUntil(
      MaterialPageRoute<void>(builder: (_) => const PhoneScreen()),
      (_) => false,
    );
  }

  Future<void> _changeNumber() async {
    final yes = await _confirm(
      title: 'Change number',
      body:
          'Sign out, then sign in with the new number. Your farms stay with '
          'the old number until the office moves them.',
      action: 'Sign out',
    );
    if (yes) await _signOut();
  }

  Future<void> _delete() async {
    final yes = await _confirm(
      title: 'Delete my account and farms?',
      body:
          'This removes your phone number, farms, reports and questions. '
          'It cannot be undone.',
      action: 'Delete',
      danger: true,
    );
    if (!yes || !mounted) return;
    setState(() => _deleting = true);
    try {
      await AppScope.read(context).api.deleteAccount();
      await _signOut();
    } on ApiException catch (e) {
      if (!mounted) return;
      showToast(context, switch (e) {
        _ when e.isOffline => 'No internet. Nothing was deleted.',
        _ when e.status == 404 =>
          'Deleting is not on the server yet. Nothing was deleted. '
              'Call the office to delete your account.',
        _ => 'Could not delete. Nothing was deleted. Please try again.',
      }, long: true);
    } finally {
      if (mounted) setState(() => _deleting = false);
    }
  }

  Future<bool> _confirm({
    required String title,
    required String body,
    required String action,
    bool danger = false,
  }) async =>
      await showDialog<bool>(
        context: context,
        builder: (c) => AlertDialog(
          title: Text(title, style: latText(size: 18, weight: FontWeight.w700)),
          content: Text(body, style: latText(size: 14, height: 1.4)),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(c).pop(false),
              child: Text('Cancel', style: latText(size: 14)),
            ),
            TextButton(
              onPressed: () => Navigator.of(c).pop(true),
              child: Text(
                action,
                style: latText(
                  size: 14,
                  weight: FontWeight.w700,
                  color: danger ? JColors.levelAlarm : JColors.accent,
                ),
              ),
            ),
          ],
        ),
      ) ??
      false;

  void _about() => showDialog<void>(
    context: context,
    builder: (c) => AlertDialog(
      title: Text(
        'About Jutyar',
        style: latText(size: 18, weight: FontWeight.w700),
      ),
      content: Text(
        'Version 0.1. Jutyar reads your field from free Sentinel-2 satellite '
        'pictures and the weather forecast, and turns them into farm work. '
        'Check your field and your label before you act.',
        style: latText(size: 14, height: 1.4),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(c).pop(),
          child: Text('OK', style: latText(size: 14, color: JColors.accent)),
        ),
      ],
    ),
  );

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    final farms = _farms;
    return Column(
      children: [
        const JutyarHeader(),
        Expanded(
          child: englishScope(
            context,
            ListView(
              padding: const EdgeInsets.fromLTRB(20, 12, 20, 24),
              children: [
                Text(
                  'Settings',
                  style: latText(
                    size: 24,
                    weight: FontWeight.w700,
                    letterSpacing: -0.4,
                  ),
                ),
                const SizedBox(height: 16),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  decoration: BoxDecoration(
                    color: JColors.card,
                    borderRadius: BorderRadius.circular(14),
                  ),
                  child: Column(
                    children: [
                      _Row(
                        icon: Icons.translate_rounded,
                        label: 'Language',
                        value: Row(
                          mainAxisSize: MainAxisSize.min,
                          spacing: 4,
                          children: [
                            Flexible(
                              child: Text(
                                'کوردی',
                                style: jText(
                                  true,
                                  size: 13,
                                  color: JColors.muted,
                                ),
                              ),
                            ),
                            Flexible(
                              child: Text(
                                '· English',
                                maxLines: 1,
                                overflow: TextOverflow.ellipsis,
                                style: latText(
                                  size: 13,
                                  weight: FontWeight.w400,
                                  color: JColors.muted,
                                ),
                              ),
                            ),
                          ],
                        ),
                        onTap: () => scope.setKu(!scope.ku),
                      ),
                      _Row(
                        icon: Icons.phone_outlined,
                        label: 'Phone number',
                        value: Column(
                          crossAxisAlignment: CrossAxisAlignment.end,
                          spacing: 2,
                          children: [
                            Text(
                              _phone == null ? '-' : _fmtPhone(_phone!),
                              textDirection: TextDirection.ltr,
                              style: latText(
                                size: 13,
                                weight: FontWeight.w400,
                                color: JColors.muted,
                              ),
                            ),
                            Text(
                              'Change number',
                              style: latText(
                                size: 12,
                                weight: FontWeight.w600,
                                color: JColors.accent,
                              ),
                            ),
                          ],
                        ),
                        onTap: _changeNumber,
                      ),
                      _Row(
                        icon: Icons.map_outlined,
                        label: 'My farms',
                        value: Text(
                          farms == null
                              ? '-'
                              : '$farms ${farms == 1 ? 'farm' : 'farms'}',
                          style: latText(
                            size: 13,
                            weight: FontWeight.w400,
                            color: JColors.muted,
                          ),
                        ),
                        // Back past the farm screen to My farms.
                        onTap: () => Navigator.of(context)
                          ..pop()
                          ..maybePop(),
                      ),
                      const _Row(
                        icon: Icons.notifications_none_rounded,
                        label: 'Notifications',
                      ),
                      _Toggle(
                        label: 'Red alerts (sent to your phone)',
                        value: _redAlerts,
                        onChanged: (v) {
                          setState(() => _redAlerts = v);
                          _saveNotify();
                        },
                      ),
                      _Toggle(
                        label: 'Weekly plan, Sunday morning',
                        value: _weeklyPlan,
                        onChanged: (v) {
                          setState(() => _weeklyPlan = v);
                          _saveNotify();
                        },
                      ),
                      _Row(
                        icon: Icons.info_outline_rounded,
                        label: 'About Jutyar',
                        value: Text(
                          'v0.1 · data from satellites and weather',
                          textAlign: TextAlign.end,
                          style: latText(
                            size: 13,
                            weight: FontWeight.w400,
                            color: JColors.muted,
                          ),
                        ),
                        onTap: _about,
                        last: true,
                      ),
                    ],
                  ),
                ),
                const SizedBox(height: 24),
                SizedBox(
                  height: 48,
                  child: OutlinedButton(
                    style: OutlinedButton.styleFrom(
                      backgroundColor: Colors.transparent,
                      foregroundColor: JColors.levelAlarm,
                      side: const BorderSide(
                        color: JColors.levelAlarm,
                        width: 1.5,
                      ),
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(14),
                      ),
                    ),
                    onPressed: _deleting ? null : _delete,
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      spacing: 8,
                      children: [
                        _deleting
                            ? const SizedBox(
                                width: 16,
                                height: 16,
                                child: CircularProgressIndicator(
                                  strokeWidth: 2,
                                  color: JColors.levelAlarm,
                                ),
                              )
                            : const Icon(
                                Icons.delete_outline_rounded,
                                size: 18,
                              ),
                        Flexible(
                          child: Text(
                            'Delete my account and farms',
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: latText(
                              size: 15,
                              weight: FontWeight.w600,
                              color: JColors.levelAlarm,
                            ),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  'Removes your phone number, farms, reports and questions.',
                  textAlign: TextAlign.center,
                  style: latText(
                    size: 12,
                    weight: FontWeight.w400,
                    color: JColors.muted,
                  ),
                ),
                const SizedBox(height: 16),
                const JutyarFooter(),
              ],
            ),
          ),
        ),
      ],
    );
  }
}

/// One settings row: icon tile, label, value on the right, chevron. A
/// line under every row but the last (design: Settings Card).
class _Row extends StatelessWidget {
  const _Row({
    required this.icon,
    required this.label,
    this.value,
    this.onTap,
    this.last = false,
  });
  final IconData icon;
  final String label;
  final Widget? value;
  final VoidCallback? onTap;
  final bool last;

  @override
  Widget build(BuildContext context) => InkWell(
    onTap: onTap,
    child: Container(
      constraints: const BoxConstraints(minHeight: 56),
      padding: const EdgeInsets.symmetric(vertical: 10),
      decoration: BoxDecoration(
        border: last
            ? null
            : const Border(bottom: BorderSide(color: JColors.line)),
      ),
      child: Row(
        spacing: 12,
        children: [
          Container(
            width: 28,
            height: 28,
            decoration: BoxDecoration(
              color: JColors.accentSoft,
              borderRadius: BorderRadius.circular(8),
            ),
            child: Icon(icon, size: 16, color: JColors.accent),
          ),
          Text(label, style: latText(size: 15, weight: FontWeight.w600)),
          Expanded(
            child: Align(
              alignment: Alignment.centerRight,
              child: value ?? const SizedBox.shrink(),
            ),
          ),
          if (onTap != null)
            const Icon(
              Icons.chevron_right_rounded,
              size: 16,
              color: JColors.muted,
            ),
        ],
      ),
    ),
  );
}

/// A notification switch, indented under the Notifications row.
class _Toggle extends StatelessWidget {
  const _Toggle({
    required this.label,
    required this.value,
    required this.onChanged,
  });
  final String label;
  final bool value;
  final ValueChanged<bool> onChanged;

  @override
  Widget build(BuildContext context) => Semantics(
    toggled: value,
    label: label,
    excludeSemantics: true,
    child: InkWell(
      onTap: () => onChanged(!value),
      child: Container(
        constraints: const BoxConstraints(minHeight: 56),
        padding: const EdgeInsets.only(left: 40),
        decoration: const BoxDecoration(
          border: Border(bottom: BorderSide(color: JColors.line)),
        ),
        child: Row(
          spacing: 12,
          children: [
            Expanded(
              child: Text(
                label,
                style: latText(size: 14, weight: FontWeight.w500),
              ),
            ),
            // Design: 44 x 26 track, 20 px white knob.
            AnimatedContainer(
              duration: const Duration(milliseconds: 150),
              width: 44,
              height: 26,
              padding: const EdgeInsets.all(3),
              alignment: value ? Alignment.centerRight : Alignment.centerLeft,
              decoration: BoxDecoration(
                color: value ? JColors.accent : JColors.levelNone,
                borderRadius: BorderRadius.circular(13),
              ),
              child: Container(
                width: 20,
                height: 20,
                decoration: const BoxDecoration(
                  color: Colors.white,
                  shape: BoxShape.circle,
                ),
              ),
            ),
          ],
        ),
      ),
    ),
  );
}
