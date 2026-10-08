import 'package:flutter/material.dart';

import '../../api/api.dart';
import '../../app_scope.dart';
import '../../l10n/strings.dart';
import '../../theme.dart';
import '../../widgets/common.dart';
import '../../widgets/header.dart';
import 'farm_section.dart';

/// Home: every farm of this number stacked (decision 2026-10-08), opened at
/// the farm that was tapped in My farms. English for now; Sorani later.
class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key, required this.farms, this.openId});
  final List<FarmSummary> farms;
  final String? openId;

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  late final _keys = {for (final f in widget.farms) f.id: GlobalKey()};
  final _loaded = <String>{};
  bool _scrolled = false;

  /// Farms load at different speeds and change height; scroll to the opened
  /// farm only once every farm above it (and itself) has its final size.
  void _onLoaded(String id) {
    _loaded.add(id);
    final open = widget.openId;
    if (_scrolled || open == null) return;
    final above = widget.farms.takeWhile((f) => f.id != open).map((f) => f.id);
    if (!_loaded.contains(open) || !above.every(_loaded.contains)) return;
    _scrolled = true;
    if (above.isEmpty) return;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      final c = _keys[open]?.currentContext;
      if (c != null && c.mounted) {
        Scrollable.ensureVisible(
          c,
          duration: const Duration(milliseconds: 450),
          curve: Curves.easeOutCubic,
        );
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final outer = AppScope.of(context);
    final s = outer.s;
    // Home is English for now (decision 2026-10-08). Only the body and tab bar
    // switch; the header keeps the language toggle in the farmer's choice.
    Widget english(Widget child) => AppScope(
      ku: false,
      s: const S(false),
      api: outer.api,
      setKu: outer.setKu,
      child: child,
    );
    return Directionality(
      textDirection: TextDirection.ltr,
      child: Scaffold(
        backgroundColor: JColors.bg,
        body: SafeArea(
          bottom: false,
          child: Column(
            children: [
              const JutyarHeader(),
              Expanded(
                child: english(
                  SingleChildScrollView(
                    padding: const EdgeInsets.fromLTRB(16, 8, 16, 24),
                    child: Column(
                      spacing: 16,
                      children: [
                        for (final f in widget.farms)
                          FarmSection(
                            key: _keys[f.id],
                            summary: f,
                            onLoaded: () => _onLoaded(f.id),
                          ),
                        Text(
                          '${s.farmingAssistant} · Jutyar',
                          style: jText(false, size: 11.5, color: JColors.faint),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            ],
          ),
        ),
        bottomNavigationBar: english(const _TabBar()),
      ),
    );
  }
}

/// Home, Alerts, Ask the Doctor (raised), Settings. Only Home is built so far.
class _TabBar extends StatelessWidget {
  const _TabBar();

  @override
  Widget build(BuildContext context) {
    final s = AppScope.of(context).s;
    void later(String name) => showToast(context, '$name: ${s.notBuilt}');
    Widget tab(IconData icon, String label, {bool on = false}) => Expanded(
      child: InkWell(
        onTap: on ? null : () => later(label),
        child: Padding(
          padding: const EdgeInsets.only(top: 10, bottom: 6),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            spacing: 3,
            children: [
              Icon(icon, size: 22, color: on ? JColors.accent : JColors.muted),
              Text(
                label,
                style: jText(
                  false,
                  size: 11,
                  weight: on ? FontWeight.w700 : FontWeight.w500,
                  color: on ? JColors.accent : JColors.muted,
                ),
              ),
            ],
          ),
        ),
      ),
    );
    return Container(
      decoration: const BoxDecoration(
        color: JColors.card,
        border: Border(top: BorderSide(color: JColors.cardLine)),
      ),
      child: SafeArea(
        top: false,
        child: SizedBox(
          height: 64,
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              tab(Icons.home_rounded, s.tabHome, on: true),
              tab(Icons.notifications_none_rounded, s.tabAlerts),
              Expanded(
                child: InkWell(
                  onTap: () => later(s.tabAsk),
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.end,
                    spacing: 3,
                    children: [
                      Container(
                        width: 44,
                        height: 44,
                        decoration: BoxDecoration(
                          color: JColors.accent,
                          shape: BoxShape.circle,
                          border: Border.all(
                            color: JColors.accentSoft,
                            width: 3,
                          ),
                        ),
                        child: const Icon(
                          Icons.medical_services_outlined,
                          size: 20,
                          color: Colors.white,
                        ),
                      ),
                      Padding(
                        padding: const EdgeInsets.only(bottom: 6),
                        child: Text(
                          s.tabAsk,
                          style: jText(
                            false,
                            size: 11,
                            weight: FontWeight.w600,
                            color: JColors.accent,
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
              tab(Icons.settings_outlined, s.tabSettings),
            ],
          ),
        ),
      ),
    );
  }
}
