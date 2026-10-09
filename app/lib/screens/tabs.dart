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

/// Home, Alerts, Ask the Doctor (raised), Alwa, Settings (design: Tab Bar).
class JutyarTabBar extends StatelessWidget {
  const JutyarTabBar({super.key, required this.current, required this.farm});
  final JTab current;
  final FarmSummary farm;

  @override
  Widget build(BuildContext context) {
    final s = AppScope.of(context).s;
    Widget tab(JTab t, IconData icon, String label) {
      final on = t == current;
      return Expanded(
        child: Semantics(
          button: true,
          selected: on,
          label: label,
          excludeSemantics: true,
          child: InkWell(
            onTap: () => openTab(context, current, t, farm),
            child: Padding(
              padding: const EdgeInsets.only(top: 8, bottom: 6),
              child: FittedBox(
                fit: BoxFit.scaleDown,
                alignment: Alignment.bottomCenter,
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  spacing: 4,
                  children: [
                    Icon(
                      icon,
                      size: 22,
                      color: on ? JColors.accent : JColors.muted,
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
          height: 80,
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              tab(JTab.home, Icons.home_rounded, s.tabHome),
              tab(JTab.alerts, Icons.notifications_none_rounded, s.tabAlerts),
              SizedBox(
                width: 96,
                child: Semantics(
                  button: true,
                  label: s.tabAsk,
                  excludeSemantics: true,
                  child: InkWell(
                    onTap: () => Navigator.of(context).push(
                      MaterialPageRoute<void>(
                        builder: (_) => AskDoctorScreen(farm: farm),
                      ),
                    ),
                    child: Padding(
                      padding: const EdgeInsets.only(top: 2, bottom: 6),
                      child: FittedBox(
                        fit: BoxFit.scaleDown,
                        alignment: Alignment.bottomCenter,
                        child: Column(
                          mainAxisSize: MainAxisSize.min,
                          spacing: 4,
                          children: [
                            Container(
                              width: 56,
                              height: 56,
                              decoration: BoxDecoration(
                                color: JColors.accent,
                                shape: BoxShape.circle,
                                border: Border.all(
                                  color: JColors.card,
                                  width: 3,
                                ),
                              ),
                              child: const Icon(
                                Icons.medical_services_outlined,
                                size: 26,
                                color: Colors.white,
                              ),
                            ),
                            Text(
                              s.tabAsk,
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
