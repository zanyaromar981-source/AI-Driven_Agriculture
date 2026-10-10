import 'package:flutter/material.dart';

import '../../api/api.dart';
import '../../app_scope.dart';
import '../../l10n/strings.dart';
import '../../store/outbox.dart';
import '../add_farm/farm_actions.dart';
import '../history/field_history_screen.dart';
import '../history/now_card.dart';
import '../../theme.dart';
import '../../widgets/header.dart';
import '../tabs.dart';
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

  /// The field's history as last read by the Field history card; feeds the
  /// crop advice card.
  FarmInsights? _insights;

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
                        // The field's state from the latest clear picture.
                        NowCard(
                          key: ValueKey('now-${farm.id}#$_version'),
                          farm: farm,
                        ),
                        // Which crops fit this field (user, 2026-10-10).
                        HomeFitCard(data: _insights, zoneSlug: farm.zoneSlug),
                        FarmSection(
                          key: ValueKey('${farm.id}#$_version'),
                          summary: farm,
                        ),
                        const _SeasonOutlookCard(),
                        // The field's 20+ year history (Field history screen).
                        FieldHistoryEntry(
                          key: ValueKey('history-${farm.id}#$_version'),
                          farm: farm,
                          onData: (d) {
                            if (mounted) setState(() => _insights = d);
                          },
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
        bottomNavigationBar: english(
          JutyarTabBar(current: JTab.home, farm: _farm),
        ),
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

/// The winter outlook for the region (GET /outlooks). Shows nothing while it
/// loads, when none has been issued, or when the call fails.
class _SeasonOutlookCard extends StatefulWidget {
  const _SeasonOutlookCard();

  @override
  State<_SeasonOutlookCard> createState() => _SeasonOutlookCardState();
}

class _SeasonOutlookCardState extends State<_SeasonOutlookCard> {
  SeasonOutlook? _outlook;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      // The Farm model carries no zone; the signal is the same for every zone.
      final o = await AppScope.read(context).api.seasonOutlook();
      if (mounted && o != null) setState(() => _outlook = o);
    } catch (_) {
      // No card: the outlook is extra, never an error on Home.
    }
  }

  @override
  Widget build(BuildContext context) {
    final o = _outlook;
    if (o == null) return const SizedBox.shrink();
    final scope = AppScope.of(context);
    final s = scope.s;
    final ku = o.reasonKu?.trim() ?? '';
    final reason = scope.ku && ku.isNotEmpty ? ku : (o.reasonEn ?? ku);
    final colour = switch (o.outlook) {
      'good' => JColors.levelNormal,
      'bad' => JColors.levelAlarm,
      _ => JColors.ink,
    };
    final pct = o.confidencePct;
    final tested = o.seasonsTested;
    final right = o.seasonsRight;
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.fromLTRB(16, 16, 16, 18),
      decoration: BoxDecoration(
        color: JColors.card,
        borderRadius: BorderRadius.circular(18),
        border: Border.all(color: JColors.cardLine),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        spacing: 6,
        children: [
          Text(
            s.seasonOutlookTitle(o.season).toUpperCase(),
            style: latText(
              size: 11,
              weight: FontWeight.w800,
              color: JColors.faint,
            ),
          ),
          Text(
            s.outlookHeadline(o.outlook),
            style: jText(
              false,
              size: 20,
              weight: FontWeight.w800,
              color: colour,
            ),
          ),
          if (pct != null)
            Text(
              s.pctSure(pct.round()),
              style: jText(false, size: 13.5, weight: FontWeight.w600),
            ),
          if (reason.isNotEmpty)
            Text(reason, style: jText(false, size: 13.5, color: JColors.muted)),
          if (tested != null && right != null && tested > 0)
            Text(
              s.calledRight(right, tested),
              style: jText(false, size: 12.5, color: JColors.muted),
            ),
        ],
      ),
    );
  }
}
