import 'package:flutter/material.dart';

import '../api/api.dart';
import '../app_scope.dart';
import '../geo.dart';
import '../l10n/strings.dart';
import '../theme.dart';
import '../widgets/common.dart';
import 'alerts/alerts_screen.dart';
import 'alwa/alwa_home_screen.dart';
import 'doctor/ask_doctor_screen.dart';
import 'settings/settings_screen.dart';

/// The places in the bottom tab bar (design: Tab Bar). Ask the Doctor is
/// the raised button in the middle; it opens on top instead of being a tab.
enum JTab { home, alerts, alwa, settings }

/// Body and tab bar in English for now (decision 2026-10-08); the header
/// keeps the language toggle in the farmer's choice.
Widget englishScope(BuildContext context, Widget child) {
  final outer = AppScope.of(context);
  return AppScope(
    ku: false,
    s: const S(false),
    api: outer.api,
    setKu: outer.setKu,
    child: child,
  );
}

/// Opens [tab] from the tab bar. Home is My farms or the open farm; the
/// other tabs sit on top of it one at a time, so back always returns there.
/// Home on a farm goes back to My farms. [farm] is null on My farms.
void openTab(BuildContext context, JTab from, JTab tab, FarmSummary? farm) {
  final nav = Navigator.of(context);
  if (tab == JTab.home) {
    if (from == JTab.home && farm == null) return;
    nav.maybePop();
    return;
  }
  if (tab == from) return;
  final route = PageRouteBuilder<void>(
    pageBuilder: (_, _, _) => TabPage(tab: tab, farm: farm),
    transitionDuration: Duration.zero,
    reverseTransitionDuration: Duration.zero,
  );
  from == JTab.home ? nav.push(route) : nav.pushReplacement(route);
}

/// Alerts, Alwa or Settings with the tab bar under it.
class TabPage extends StatelessWidget {
  const TabPage({super.key, required this.tab, this.farm});
  final JTab tab;

  /// The open farm, or null when opened from My farms (all farms).
  final FarmSummary? farm;

  @override
  Widget build(BuildContext context) {
    return Directionality(
      textDirection: TextDirection.ltr,
      child: Scaffold(
        backgroundColor: JColors.bg,
        body: SafeArea(
          bottom: false,
          child: switch (tab) {
            JTab.alerts => AlertsScreen(farm: farm),
            JTab.alwa => const AlwaHomeScreen(),
            JTab.settings => const SettingsScreen(),
            JTab.home => const SizedBox.shrink(),
          },
        ),
        bottomNavigationBar: englishScope(
          context,
          JutyarTabBar(current: tab, farm: farm),
        ),
      ),
    );
  }
}

/// The [openAlerts] key for all farms (My farms).
const kAllFarms = '*';

/// Open alerts (not ticked done) per farm id, for the badge on the bell.
/// The Alerts screen updates it when it loads or a tick changes.
final openAlerts = ValueNotifier<Map<String, int>>({});

/// Home, Alerts, Ask the Doctor (raised), Alwa, Settings (design: Tab Bar).
class JutyarTabBar extends StatefulWidget {
  const JutyarTabBar({super.key, required this.current, this.farm});
  final JTab current;

  /// The open farm, or null on My farms and the tabs opened from it.
  final FarmSummary? farm;

  @override
  State<JutyarTabBar> createState() => _JutyarTabBarState();
}

class _JutyarTabBarState extends State<JutyarTabBar> {
  @override
  void initState() {
    super.initState();
    _count();
  }

  /// The badge count, for the open farm or all farms; no badge when the
  /// server has no alerts (404) or there is no internet.
  Future<void> _count() async {
    final id = widget.farm?.id ?? kAllFarms;
    if (openAlerts.value.containsKey(id)) return;
    final api = AppScope.read(context).api;
    try {
      final ids = widget.farm == null
          ? [for (final f in await api.getFarms()) f.id]
          : [id];
      var n = 0;
      for (final a in await Future.wait(ids.map(api.getAlerts))) {
        n += a.where((x) => !x.done).length;
      }
      openAlerts.value = {...openAlerts.value, id: n};
    } on ApiException {
      // No badge.
    }
  }

  /// Ask the Doctor needs a farm: the open one, the only one, or the one
  /// the farmer picks.
  Future<void> _ask() async {
    final nav = Navigator.of(context);
    var farm = widget.farm;
    if (farm == null) {
      List<FarmSummary> farms;
      try {
        farms = await AppScope.read(context).api.getFarms();
      } on ApiException catch (e) {
        if (mounted) {
          showToast(
            context,
            e.isOffline
                ? 'No internet. Open a farm to ask the Doctor.'
                : 'Could not load your farms. Please try again.',
          );
        }
        return;
      }
      if (!mounted) return;
      if (farms.isEmpty) {
        showToast(context, 'Add a farm first, then ask the Doctor about it.');
        return;
      }
      farm = farms.length == 1 ? farms.first : await _pickFarm(farms);
      if (farm == null || !mounted) return;
    }
    final f = farm;
    nav.push(MaterialPageRoute<void>(builder: (_) => AskDoctorScreen(farm: f)));
  }

  Future<FarmSummary?> _pickFarm(List<FarmSummary> farms) =>
      showModalBottomSheet<FarmSummary>(
        context: context,
        backgroundColor: JColors.card,
        shape: const RoundedRectangleBorder(
          borderRadius: BorderRadius.vertical(top: Radius.circular(20)),
        ),
        builder: (c) => SafeArea(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Padding(
                padding: const EdgeInsets.fromLTRB(20, 18, 20, 8),
                child: Text(
                  'Which farm is the question about?',
                  style: latText(size: 16, weight: FontWeight.w700),
                ),
              ),
              for (final f in farms)
                ListTile(
                  leading: const Icon(
                    Icons.agriculture_outlined,
                    color: JColors.accent,
                  ),
                  title: Text(
                    f.name,
                    style: jText(true, size: 15, weight: FontWeight.w600),
                  ),
                  subtitle: Text(
                    '${fmtM2(f.areaDunam * 2500)} m²',
                    style: latText(size: 12.5, color: JColors.muted),
                  ),
                  onTap: () => Navigator.of(c).pop(f),
                ),
              const SizedBox(height: 8),
            ],
          ),
        ),
      );

  @override
  Widget build(BuildContext context) {
    final s = AppScope.of(context).s;
    final current = widget.current;
    final farm = widget.farm;
    Widget tab(JTab t, IconData icon, String label, {int badge = 0}) {
      final on = t == current;
      return Expanded(
        child: Semantics(
          button: true,
          selected: on,
          label: badge > 0 ? '$label, $badge new' : label,
          excludeSemantics: true,
          child: InkWell(
            onTap: () => openTab(context, current, t, farm),
            child: Padding(
              padding: const EdgeInsets.only(top: 10, bottom: 6),
              child: FittedBox(
                fit: BoxFit.scaleDown,
                alignment: Alignment.topCenter,
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  spacing: 4,
                  children: [
                    Stack(
                      clipBehavior: Clip.none,
                      children: [
                        Icon(
                          icon,
                          size: 22,
                          color: on ? JColors.accent : JColors.muted,
                        ),
                        if (badge > 0)
                          Positioned(
                            left: 12,
                            top: -5,
                            child: Container(
                              width: 16,
                              height: 16,
                              alignment: Alignment.center,
                              decoration: BoxDecoration(
                                color: JColors.levelAlarm,
                                shape: BoxShape.circle,
                                border: Border.all(
                                  color: JColors.card,
                                  width: 2,
                                ),
                              ),
                              child: Text(
                                badge > 9 ? '9+' : '$badge',
                                style: latText(
                                  size: 10,
                                  weight: FontWeight.w800,
                                  color: Colors.white,
                                  height: 1,
                                ),
                              ),
                            ),
                          ),
                      ],
                    ),
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
          ),
        ),
      );
    }

    return Container(
      decoration: const BoxDecoration(
        color: JColors.card,
        border: Border(top: BorderSide(color: JColors.line)),
      ),
      child: SafeArea(
        top: false,
        child: SizedBox(
          height: 64,
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              tab(JTab.home, Icons.home_rounded, s.tabHome),
              ValueListenableBuilder(
                valueListenable: openAlerts,
                builder: (_, counts, _) => tab(
                  JTab.alerts,
                  Icons.notifications_none_rounded,
                  s.tabAlerts,
                  badge: counts[farm?.id ?? kAllFarms] ?? 0,
                ),
              ),
              // The raised button sits 15 px above the bar (design: y -15).
              SizedBox(
                width: 96,
                child: OverflowBox(
                  alignment: Alignment.topCenter,
                  maxHeight: 100,
                  child: Transform.translate(
                    offset: const Offset(0, -19),
                    child: Semantics(
                      button: true,
                      label: s.tabAsk,
                      excludeSemantics: true,
                      child: GestureDetector(
                        behavior: HitTestBehavior.opaque,
                        onTap: _ask,
                        child: Column(
                          mainAxisSize: MainAxisSize.min,
                          spacing: 4,
                          children: [
                            Container(
                              width: 64,
                              height: 64,
                              padding: const EdgeInsets.all(4),
                              decoration: const BoxDecoration(
                                color: JColors.card,
                                shape: BoxShape.circle,
                              ),
                              child: Container(
                                decoration: const BoxDecoration(
                                  color: JColors.accent,
                                  shape: BoxShape.circle,
                                  boxShadow: [
                                    BoxShadow(
                                      color: Color(0x4D1E7A5A),
                                      offset: Offset(0, 6),
                                      blurRadius: 14,
                                    ),
                                  ],
                                ),
                                child: const Icon(
                                  Icons.medical_services_outlined,
                                  size: 26,
                                  color: Colors.white,
                                ),
                              ),
                            ),
                            Text(
                              s.tabAsk,
                              maxLines: 1,
                              style: jText(
                                false,
                                size: 11,
                                weight: FontWeight.w700,
                                color: JColors.accent,
                              ),
                            ),
                          ],
                        ),
                      ),
                    ),
                  ),
                ),
              ),
              tab(JTab.alwa, Icons.storefront_outlined, s.tabAlwa),
              tab(JTab.settings, Icons.settings_outlined, s.tabSettings),
            ],
          ),
        ),
      ),
    );
  }
}
