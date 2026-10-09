import 'package:flutter/material.dart';

import '../api/api.dart';
import '../app_scope.dart';
import '../l10n/strings.dart';
import '../theme.dart';
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

/// Opens [tab] from the tab bar. Home is the farm screen; the other tabs sit
/// on top of it one at a time, so back always returns to the farm. Home on
/// the farm screen itself goes back to My farms.
void openTab(BuildContext context, JTab from, JTab tab, FarmSummary farm) {
  final nav = Navigator.of(context);
  if (tab == JTab.home) {
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
  const TabPage({super.key, required this.tab, required this.farm});
  final JTab tab;
  final FarmSummary farm;

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
            JTab.settings => SettingsScreen(farm: farm),
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

/// Open alerts (not ticked done) per farm id, for the badge on the bell.
/// The Alerts screen updates it when it loads or a tick changes.
final openAlerts = ValueNotifier<Map<String, int>>({});

/// Home, Alerts, Ask the Doctor (raised), Alwa, Settings (design: Tab Bar).
class JutyarTabBar extends StatefulWidget {
  const JutyarTabBar({super.key, required this.current, required this.farm});
  final JTab current;
  final FarmSummary farm;

  @override
  State<JutyarTabBar> createState() => _JutyarTabBarState();
}

class _JutyarTabBarState extends State<JutyarTabBar> {
  @override
  void initState() {
    super.initState();
    _count();
  }

  /// The badge count; no badge when the server has no alerts (404) or
  /// there is no internet.
  Future<void> _count() async {
    final id = widget.farm.id;
    if (openAlerts.value.containsKey(id)) return;
    try {
      final a = await AppScope.read(context).api.getAlerts(id);
      openAlerts.value = {
        ...openAlerts.value,
        id: a.where((x) => !x.done).length,
      };
    } on ApiException {
      // No badge.
    }
  }

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
                  badge: counts[farm.id] ?? 0,
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
                        onTap: () => Navigator.of(context).push(
                          MaterialPageRoute<void>(
                            builder: (_) => AskDoctorScreen(farm: farm),
                          ),
                        ),
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
