import 'package:flutter/material.dart';

import '../../api/api.dart';
import '../../app_scope.dart';
import '../../l10n/strings.dart';
import '../../store/outbox.dart';
import '../add_farm/farm_actions.dart';
import '../history/field_history_screen.dart';
import '../../theme.dart';
import '../../widgets/common.dart';
import '../../widgets/header.dart';
import 'farm_section.dart';

/// Home: one farm on its own screen (user, 2026-10-08: "every farm should be
/// opened separately"). The back arrow, the Android back gesture and the Home
/// tab all return to My farms. English for now; Sorani later.
class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key, required this.farm});
  final FarmSummary farm;

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  late FarmSummary _farm = widget.farm;

  /// Bumped after an edit, so the farm section loads the farm again.
  int _version = 0;

  /// After the editor closes: fresh name and size from the server, or the
  /// waiting change on the phone when there is no internet.
  Future<void> _afterEdit() async {
    // A saved edit becomes a new farm (with a new id and a fresh 20-year
    // analysis); follow it so the screen shows the new border and history.
    final newId = Outbox.instance.replacedBy(_farm.id);
    try {
      final list = await AppScope.read(context).api.getFarms();
      final f = list.where((x) => x.id == (newId ?? _farm.id));
      if (f.isNotEmpty) _farm = f.first;
    } on ApiException {
      // Offline: fall through to the waiting change below.
    }
    final waiting = Outbox.instance.items.where((i) => i.farmId == _farm.id);
    if (waiting.isNotEmpty) _farm = waiting.last.summary;
    if (mounted) setState(() => _version++);
  }

  @override
  Widget build(BuildContext context) {
    final farm = _farm;
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
              english(
                _BackBar(
                  trailing: FarmMenuButton(
                    farm: farm,
                    onEdited: _afterEdit,
                    onDeleted: () => Navigator.of(context).maybePop(),
                  ),
                ),
              ),
              Expanded(
                child: english(
                  SingleChildScrollView(
                    padding: const EdgeInsets.fromLTRB(16, 4, 16, 24),
                    child: Column(
                      spacing: 16,
                      children: [
                        FarmSection(
                          key: ValueKey('${farm.id}#$_version'),
                          summary: farm,
                        ),
                        // The field's 20+ year history (Field history screen).
                        FieldHistoryEntry(
                          key: ValueKey('history-${farm.id}#$_version'),
                          farm: farm,
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

/// Back arrow to My farms on the left; the farm's Edit and Delete menu on
/// the right.
class _BackBar extends StatelessWidget {
  const _BackBar({required this.trailing});
  final Widget trailing;

  @override
  Widget build(BuildContext context) {
    final s = AppScope.of(context).s;
    return Padding(
      padding: const EdgeInsets.fromLTRB(6, 0, 16, 0),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Flexible(
            child: Semantics(
              button: true,
              label: s.backToFarms,
              excludeSemantics: true,
              child: InkWell(
                onTap: () => Navigator.of(context).maybePop(),
                borderRadius: BorderRadius.circular(10),
                child: ConstrainedBox(
                  constraints: const BoxConstraints(minHeight: 44),
                  child: Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 8),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      spacing: 6,
                      children: [
                        const Icon(
                          Icons.arrow_back_rounded,
                          size: 22,
                          color: JColors.ink,
                        ),
                        Flexible(
                          child: Text(
                            s.backToFarms,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: jText(
                              false,
                              size: 15,
                              weight: FontWeight.w600,
                              color: JColors.ink,
                            ),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            ),
          ),
          trailing,
        ],
      ),
    );
  }
}

/// Home, Alerts, Ask the Doctor (raised), Settings. Only Home is built so far;
/// tapping it from a farm goes back to My farms.
class _TabBar extends StatelessWidget {
  const _TabBar();

  @override
  Widget build(BuildContext context) {
    final s = AppScope.of(context).s;
    void later(String name) => showToast(context, '$name: ${s.notBuilt}');
    Widget tab(
      IconData icon,
      String label, {
      bool on = false,
      VoidCallback? onTap,
    }) => Expanded(
      child: InkWell(
        onTap: onTap ?? () => later(label),
        child: Padding(
          padding: const EdgeInsets.only(top: 8, bottom: 6),
          child: FittedBox(
            fit: BoxFit.scaleDown,
            alignment: Alignment.bottomCenter,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              spacing: 3,
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
    );
    return Container(
      decoration: const BoxDecoration(
        color: JColors.card,
        border: Border(top: BorderSide(color: JColors.cardLine)),
      ),
      child: SafeArea(
        top: false,
        child: SizedBox(
          height: 74,
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              tab(
                Icons.home_rounded,
                s.tabHome,
                on: true,
                onTap: () => Navigator.of(context).maybePop(),
              ),
              tab(Icons.notifications_none_rounded, s.tabAlerts),
              Expanded(
                child: InkWell(
                  onTap: () => later(s.tabAsk),
                  child: Padding(
                    padding: const EdgeInsets.only(top: 4, bottom: 6),
                    child: FittedBox(
                      fit: BoxFit.scaleDown,
                      alignment: Alignment.bottomCenter,
                      child: Column(
                        mainAxisSize: MainAxisSize.min,
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
                          Text(
                            s.tabAsk,
                            style: jText(
                              false,
                              size: 11,
                              weight: FontWeight.w600,
                              color: JColors.accent,
                            ),
                          ),
                        ],
                      ),
                    ),
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
