import 'dart:async';

import 'package:flutter/material.dart';

import '../../api/api.dart';
import '../../app_scope.dart';
import '../../crops.dart';
import '../../l10n/strings.dart';
import '../../store/insights_copy.dart';
import '../../store/local_store.dart';
import '../../theme.dart';
import '../../widgets/header.dart';
import 'crop_fit.dart';
import 'history_text.dart';

// English for now, like the rest of Home (decision 2026-10-08, BACKEND.md 2.9);
// Sorani after a native speaker check. Design: design/jutyar_app.pen, page
// "Field history" (revised after design/field_history_codex_review.md).

/// The field's history keeps arriving while the analysis runs; this loads the
/// copy on the phone first, then the server's, and checks again every 30 s
/// until all five topics are in, or until the analysis looks stuck.
class InsightsLoader {
  InsightsLoader(this.api, this.farmId);
  final Api api;
  final String farmId;
  String get _cache => InsightsCopy.name(farmId);
  Map<String, dynamic>? _copy;

  /// After [cached] or [fresh]: unfinished for longer than
  /// [InsightsCopy.giveUpAfter], so stop promising minutes.
  bool get stalled => InsightsCopy.stalled(_copy);

  /// After [cached] or [fresh]: topics still shown from before an edit.
  Set<String> get carried => InsightsCopy.carriedTopics(_copy);

  /// Keep checking: the server's analysis is not complete and not stuck.
  bool get keepChecking =>
      !stalled &&
      (carried.isNotEmpty ||
          _serverTopics < FarmInsights.all.length ||
          _copy == null);

  int get _serverTopics {
    final d = _copy?['data'];
    return d is Map<String, dynamic> ? FarmInsights.fromJson(d).ready : 0;
  }

  Future<FarmInsights?> cached() async {
    try {
      _copy = await LocalStore.read(_cache);
    } catch (_) {}
    final d = InsightsCopy.merged(_copy);
    return d == null ? null : FarmInsights.fromJson(d);
  }

  Future<FarmInsights> fresh() async {
    final f = await api.getInsights(farmId);
    Map<String, dynamic>? old = _copy;
    try {
      old = await LocalStore.read(_cache);
    } catch (_) {}
    final done = f.ready >= FarmInsights.all.length;
    final copy = <String, dynamic>{
      'saved_at': DateTime.now().toUtc().toIso8601String(),
      'data': f.json,
      if (!done)
        'waiting_since':
            old?['waiting_since'] ?? DateTime.now().toUtc().toIso8601String(),
      if (!done && old?['carried'] != null) 'carried': old!['carried'],
    };
    _copy = copy;
    try {
      await LocalStore.write(_cache, copy);
    } catch (_) {}
    return FarmInsights.fromJson(InsightsCopy.merged(copy)!);
  }
}

class FieldHistoryScreen extends StatefulWidget {
  const FieldHistoryScreen({super.key, required this.farm});
  final FarmSummary farm;

  @override
  State<FieldHistoryScreen> createState() => _FieldHistoryScreenState();
}

class _FieldHistoryScreenState extends State<FieldHistoryScreen> {
  FarmInsights? _data;
  bool _busy = false;
  String? _error;
  Timer? _poll;

  /// The analysis has not finished for a long time: say so, stop the clocks.
  bool _stalled = false;

  /// Topics still shown from the farm before an edit.
  Set<String> _carried = const {};

  @override
  void initState() {
    super.initState();
    _refresh();
  }

  @override
  void dispose() {
    _poll?.cancel();
    super.dispose();
  }

  Future<void> _refresh() async {
    final loader = InsightsLoader(AppScope.read(context).api, widget.farm.id);
    final c = await loader.cached();
    if (c != null && mounted && _data == null) setState(() => _data = c);
    if (!mounted) return;
    setState(() => _busy = true);
    try {
      final f = await loader.fresh();
      if (!mounted) return;
      setState(() {
        _data = f;
        _error = null;
      });
    } on ApiException catch (e) {
      if (mounted) {
        setState(() => _error = e.isOffline ? 'offline' : e.code);
      }
    } finally {
      if (mounted) {
        setState(() {
          _busy = false;
          _stalled = loader.stalled;
          _carried = loader.carried;
        });
      }
    }
    // Keep checking while the satellite part is still being read.
    _poll?.cancel();
    if (mounted && loader.keepChecking) {
      _poll = Timer(const Duration(seconds: 30), () {
        if (mounted) _refresh();
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final outer = AppScope.of(context);
    Widget english(Widget child) => AppScope(
      ku: false,
      s: const S(false),
      api: outer.api,
      setKu: outer.setKu,
      child: child,
    );
    final d = _data;
    final upTo = d == null ? null : dataUpTo(d);
    final views = {
      for (final v in d == null ? const <TopicView>[] : topicViews(d)) v.key: v,
    };
    final fit = d == null ? const <CropGroup>[] : cropFit(d);
    final tips = d == null ? const <String>[] : suggestions(d);
    final none = d == null || d.ready == 0;
    return Directionality(
      textDirection: TextDirection.ltr,
      child: Scaffold(
        backgroundColor: JColors.bg,
        body: SafeArea(
          child: Column(
            children: [
              const JutyarHeader(),
              english(
                Builder(
                  builder: (context) => _BackRow(label: widget.farm.name),
                ),
              ),
              Expanded(
                child: english(
                  RefreshIndicator(
                    color: JColors.accent,
                    onRefresh: _refresh,
                    child: ListView(
                      padding: const EdgeInsets.fromLTRB(20, 4, 20, 28),
                      children: [
                        Text(
                          'Field history',
                          style: latText(size: 26, weight: FontWeight.w800),
                        ),
                        const SizedBox(height: 4),
                        Text(
                          none
                              ? _stalled
                                    ? 'The analysis of this field has not finished on the server. Pull down to check again.'
                                    : 'Rain, frost and soil arrive in about a minute; the satellite pictures take about 7 minutes, on the server.'
                              : "Past patterns of this field, not this season's forecast${upTo == null ? '' : ' · data up to $upTo'}",
                          style: latText(
                            size: 13.5,
                            weight: FontWeight.w500,
                            color: JColors.muted,
                            height: 1.4,
                          ),
                        ),
                        const SizedBox(height: 10),
                        SizedBox(
                          height: 3,
                          child: _busy
                              ? ClipRRect(
                                  borderRadius: BorderRadius.circular(2),
                                  child: const LinearProgressIndicator(
                                    minHeight: 3,
                                    color: JColors.accent,
                                    backgroundColor: JColors.accentSoft,
                                  ),
                                )
                              : null,
                        ),
                        const SizedBox(height: 10),
                        if (_error != null && d == null)
                          _Notice(error: _error!, onRetry: _refresh),
                        if (d != null) ...[
                          if (_error != null) ...[
                            _Notice(
                              error: _error!,
                              onRetry: _refresh,
                              small: true,
                            ),
                            const SizedBox(height: 12),
                          ],
                          if (_carried.isNotEmpty) ...[
                            Text(
                              _stalled
                                  ? 'Some parts are from before you edited this farm: the new analysis has not finished on the server.'
                                  : 'Some parts are from before you edited this farm. They update when the new analysis is done.',
                              style: latText(
                                size: 13,
                                weight: FontWeight.w600,
                                color: JColors.gold,
                                height: 1.4,
                              ),
                            ),
                            const SizedBox(height: 12),
                          ],
                          const _HowToUse(),
                          const SizedBox(height: 14),
                          if (fit.isNotEmpty) ...[
                            _FitCard(groups: fit, tips: tips),
                            const SizedBox(height: 14),
                          ],
                          for (final (key, title, icon) in _order) ...[
                            views[key] == null
                                ? _WaitingCard(
                                    title: title,
                                    icon: icon,
                                    stalled: _stalled,
                                    waiting:
                                        key == 'dryness' &&
                                        views['greenness'] == null,
                                    satellite:
                                        key == 'greenness' || key == 'dryness',
                                  )
                                : _TopicCard(view: views[key]!, icon: icon),
                            const SizedBox(height: 14),
                          ],
                          // Comes from a daily job, not the field's analysis:
                          // shown when the server has it, never waited for.
                          if (views['groundwater'] != null) ...[
                            _TopicCard(
                              view: views['groundwater']!,
                              icon: Icons.waves_outlined,
                            ),
                            const SizedBox(height: 14),
                          ],
                        ],
                      ],
                    ),
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

const _order = [
  ('rain', 'Rain', Icons.water_drop_outlined),
  ('weather', 'Frost and heat', Icons.ac_unit_rounded),
  ('greenness', 'Greenness', Icons.eco_outlined),
  ('soil', 'Soil and land', Icons.layers_outlined),
  ('dryness', 'Dryness and fire', Icons.wb_sunny_outlined),
];

(Color, Color, String) _evidence(Evidence e) => switch (e) {
  Evidence.measured => (
    JColors.levelNormalSoft,
    JColors.levelNormal,
    'Measured from space',
  ),
  Evidence.area => (JColors.goldSoft, JColors.gold, 'Area estimate'),
  Evidence.notTested => (
    JColors.levelNoneSoft,
    JColors.muted,
    'Estimate, not tested',
  ),
  Evidence.derived => (JColors.goldSoft, JColors.gold, 'Derived'),
  Evidence.ruleOfThumb => (JColors.goldSoft, JColors.gold, 'Rule of thumb'),
};

class _BackRow extends StatelessWidget {
  const _BackRow({required this.label});
  final String label;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.fromLTRB(6, 0, 16, 0),
    child: Align(
      alignment: Alignment.centerLeft,
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
                    label,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: jText(true, size: 15, weight: FontWeight.w600),
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

class _Card extends StatelessWidget {
  const _Card({
    required this.children,
    this.fill = JColors.card,
    this.outlined = false,
  });
  final List<Widget> children;
  final Color fill;
  final bool outlined;

  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.all(14),
    decoration: BoxDecoration(
      color: fill,
      borderRadius: BorderRadius.circular(16),
      border: outlined ? Border.all(color: JColors.line) : null,
    ),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      spacing: 10,
      children: children,
    ),
  );
}

class _Head extends StatelessWidget {
  const _Head({required this.title, required this.icon, required this.pill});
  final String title;
  final IconData icon;
  final (Color, Color, String) pill;

  @override
  Widget build(BuildContext context) => Row(
    spacing: 10,
    children: [
      Container(
        width: 32,
        height: 32,
        decoration: BoxDecoration(
          color: pill.$1,
          borderRadius: BorderRadius.circular(10),
        ),
        child: Icon(
          icon,
          size: 18,
          color: pill.$2 == JColors.muted ? JColors.ink : pill.$2,
        ),
      ),
      Expanded(
        child: Text(title, style: latText(size: 16, weight: FontWeight.w700)),
      ),
      Container(
        padding: const EdgeInsets.symmetric(horizontal: 9, vertical: 3),
        decoration: BoxDecoration(
          color: pill.$1,
          borderRadius: BorderRadius.circular(999),
        ),
        child: Text(
          pill.$3,
          style: latText(size: 11.5, weight: FontWeight.w700, color: pill.$2),
        ),
      ),
    ],
  );
}

class _Chips extends StatelessWidget {
  const _Chips({required this.chips});
  final List<(String, String)> chips;

  @override
  Widget build(BuildContext context) => IntrinsicHeight(
    child: Row(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      spacing: 6,
      children: [
        for (final (v, l) in chips)
          Expanded(
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 7),
              decoration: BoxDecoration(
                color: JColors.bg,
                borderRadius: BorderRadius.circular(10),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                spacing: 1,
                children: [
                  FittedBox(
                    fit: BoxFit.scaleDown,
                    alignment: Alignment.centerLeft,
                    child: Text(
                      v,
                      style: latText(size: 15, weight: FontWeight.w800),
                    ),
                  ),
                  Text(
                    l,
                    style: latText(size: 11, color: JColors.muted, height: 1.3),
                  ),
                ],
              ),
            ),
          ),
      ],
    ),
  );
}

class _Source extends StatelessWidget {
  const _Source(this.text);
  final String text;

  @override
  Widget build(BuildContext context) => Row(
    crossAxisAlignment: CrossAxisAlignment.start,
    spacing: 6,
    children: [
      const Padding(
        padding: EdgeInsets.only(top: 1),
        child: Icon(Icons.info_outline_rounded, size: 13, color: JColors.muted),
      ),
      Expanded(
        child: Text(
          text,
          style: latText(size: 11, color: JColors.muted, height: 1.35),
        ),
      ),
    ],
  );
}

class _TopicCard extends StatefulWidget {
  const _TopicCard({required this.view, required this.icon});
  final TopicView view;
  final IconData icon;

  @override
  State<_TopicCard> createState() => _TopicCardState();
}

class _TopicCardState extends State<_TopicCard> {
  bool _open = false;

  @override
  Widget build(BuildContext context) {
    final v = widget.view;
    return _Card(
      children: [
        _Head(title: v.title, icon: widget.icon, pill: _evidence(v.evidence)),
        Text(
          v.summary,
          style: latText(size: 14, weight: FontWeight.w500, height: 1.45),
        ),
        if (v.chips.isNotEmpty) _Chips(chips: v.chips),
        _Source(v.source),
        if (v.all.isNotEmpty)
          InkWell(
            onTap: () => setState(() => _open = !_open),
            borderRadius: BorderRadius.circular(8),
            child: Padding(
              padding: const EdgeInsets.symmetric(vertical: 4),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                spacing: 4,
                children: [
                  Text(
                    'All numbers',
                    style: latText(
                      size: 13,
                      weight: FontWeight.w700,
                      color: JColors.accent,
                    ),
                  ),
                  Icon(
                    _open
                        ? Icons.expand_less_rounded
                        : Icons.expand_more_rounded,
                    size: 18,
                    color: JColors.accent,
                  ),
                ],
              ),
            ),
          ),
        if (_open)
          Column(
            children: [
              for (final (label, value) in v.all)
                Padding(
                  padding: const EdgeInsets.symmetric(vertical: 4),
                  child: Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    spacing: 10,
                    children: [
                      Expanded(
                        child: Text(
                          label,
                          style: latText(
                            size: 12.5,
                            color: JColors.muted,
                            height: 1.3,
                          ),
                        ),
                      ),
                      Text(
                        value,
                        style: latText(size: 12.5, weight: FontWeight.w700),
                      ),
                    ],
                  ),
                ),
            ],
          ),
      ],
    );
  }
}

class _WaitingCard extends StatelessWidget {
  const _WaitingCard({
    required this.title,
    required this.icon,
    required this.waiting,
    required this.satellite,
    this.stalled = false,
  });
  final String title;
  final IconData icon;

  /// The analysis has not finished for a long time: no promise of minutes.
  final bool stalled;

  /// Waiting for another topic first (dryness waits for greenness).
  final bool waiting;

  /// Comes from the satellite pictures (minutes), not from weather (a minute).
  final bool satellite;

  @override
  Widget build(BuildContext context) => _Card(
    outlined: true,
    children: [
      _Head(
        title: title,
        icon: icon,
        pill: stalled
            ? (JColors.levelNoneSoft, JColors.muted, 'Not read')
            : waiting
            ? (JColors.levelNoneSoft, JColors.muted, 'Waiting')
            : (JColors.accentSoft, JColors.accent, 'Reading'),
      ),
      Text(
        stalled
            ? 'This part could not be read on the server yet. Pull down to check again.'
            : waiting
            ? 'Starts when the greenness is ready: compares the field in dry and wet seasons.'
            : satellite
            ? 'Reading every Landsat and Sentinel-2 picture of this field since 1984.'
            : 'Reading the weather and soil records for this field.',
        style: latText(
          size: 14,
          weight: FontWeight.w500,
          color: JColors.muted,
          height: 1.45,
        ),
      ),
      if (!waiting && !stalled)
        ClipRRect(
          borderRadius: BorderRadius.circular(999),
          child: const LinearProgressIndicator(
            minHeight: 6,
            color: JColors.accent,
            backgroundColor: JColors.line,
          ),
        ),
      if (!stalled)
        Text(
          waiting
              ? 'Waiting for the greenness'
              : satellite
              ? 'About 7 minutes · it keeps going on the server if you close the app'
              : 'About a minute',
          style: latText(
            size: 12,
            weight: FontWeight.w600,
            color: JColors.muted,
            height: 1.35,
          ),
        ),
    ],
  );
}

class _HowToUse extends StatelessWidget {
  const _HowToUse();

  @override
  Widget build(BuildContext context) => _Card(
    fill: JColors.accentSoft,
    children: [
      Text(
        'How to use this',
        style: latText(size: 15, weight: FontWeight.w700),
      ),
      for (final (icon, text) in const [
        (
          Icons.calendar_month_outlined,
          "Before sowing or spraying, check this week's weather on the farm screen.",
        ),
        (
          Icons.science_outlined,
          'Before changing fertiliser, get a soil test: the soil numbers here are estimates.',
        ),
      ])
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          spacing: 8,
          children: [
            Icon(icon, size: 16, color: JColors.accent),
            Expanded(
              child: Text(
                text,
                style: latText(
                  size: 13.5,
                  weight: FontWeight.w500,
                  height: 1.4,
                ),
              ),
            ),
          ],
        ),
    ],
  );
}

class _FitCard extends StatelessWidget {
  const _FitCard({required this.groups, required this.tips});
  final List<CropGroup> groups;
  final List<String> tips;

  @override
  Widget build(BuildContext context) {
    final s = AppScope.of(context).s;
    return _Card(
      children: [
        _Head(
          title: 'What fits this field',
          icon: Icons.grass_rounded,
          pill: _evidence(Evidence.ruleOfThumb),
        ),
        for (final g in groups) _group(g, s),
        const Divider(height: 8, color: JColors.line),
        if (tips.isNotEmpty)
          Text(
            'Suggestions',
            style: latText(size: 14, weight: FontWeight.w700),
          ),
        for (final (i, t) in tips.indexed)
          Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            spacing: 8,
            children: [
              Icon(
                const [
                  Icons.water_drop_outlined,
                  Icons.ac_unit_rounded,
                  Icons.opacity_rounded,
                ][i % 3],
                size: 16,
                color: JColors.accent,
              ),
              Expanded(
                child: Text(
                  t,
                  style: latText(
                    size: 13.5,
                    weight: FontWeight.w500,
                    height: 1.4,
                  ),
                ),
              ),
            ],
          ),
        const _Source(
          "Crop needs from FAO EcoCrop, matched with this field's soil, rain, frost and heat below. "
          'Not checked against salinity, soil depth or your water source. "?" = may suffer if the clay stays wet.',
        ),
      ],
    );
  }

  Widget _group(CropGroup g, S s) {
    final (fg, bg, label) = switch (g.kind) {
      'fits' => (
        JColors.levelNormal,
        JColors.levelNormalSoft,
        'Usually fits, rain-fed',
      ),
      'irrigation' => (
        JColors.levelWatch,
        JColors.levelWatchSoft,
        'Only with irrigation',
      ),
      _ => (JColors.levelAlarm, JColors.levelAlarmSoft, 'Usually a poor fit'),
    };
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      spacing: 6,
      children: [
        Row(
          spacing: 6,
          children: [
            Container(
              width: 8,
              height: 8,
              decoration: BoxDecoration(color: fg, shape: BoxShape.circle),
            ),
            Text(
              label,
              style: latText(size: 12.5, weight: FontWeight.w700, color: fg),
            ),
          ],
        ),
        if (g.crops.isNotEmpty)
          Wrap(
            spacing: 6,
            runSpacing: 6,
            children: [
              for (final c in g.crops)
                Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 10,
                    vertical: 5,
                  ),
                  decoration: BoxDecoration(
                    color: bg,
                    borderRadius: BorderRadius.circular(999),
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    spacing: 5,
                    children: [
                      Text(
                        cropOf(c.replaceAll('?', '')).emoji,
                        style: const TextStyle(fontSize: 14),
                      ),
                      Text(
                        '${s.crop(c.replaceAll('?', ''))}${c.endsWith('?') ? ' ?' : ''}',
                        style: latText(size: 13, weight: FontWeight.w600),
                      ),
                    ],
                  ),
                ),
            ],
          ),
        Text(
          g.why,
          style: latText(size: 12.5, color: JColors.muted, height: 1.4),
        ),
      ],
    );
  }
}

class _Notice extends StatelessWidget {
  const _Notice({
    required this.error,
    required this.onRetry,
    this.small = false,
  });
  final String error;
  final VoidCallback onRetry;
  final bool small;

  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.all(12),
    decoration: BoxDecoration(
      color: JColors.levelWatchSoft,
      borderRadius: BorderRadius.circular(12),
    ),
    child: Row(
      spacing: 8,
      children: [
        const Icon(Icons.cloud_off_rounded, size: 18, color: JColors.gold),
        Expanded(
          child: Text(
            error == 'offline'
                ? (small
                      ? 'No internet. Showing the copy saved on this phone.'
                      : 'No internet, and no copy on this phone yet.')
                : 'Could not load the field history ($error).',
            style: latText(size: 13, weight: FontWeight.w600, height: 1.35),
          ),
        ),
        TextButton(
          onPressed: onRetry,
          child: Text(
            'Retry',
            style: latText(
              size: 13,
              weight: FontWeight.w700,
              color: JColors.accent,
            ),
          ),
        ),
      ],
    ),
  );
}

/// The card on the farm screen that opens Field history.
class FieldHistoryEntry extends StatefulWidget {
  const FieldHistoryEntry({super.key, required this.farm});
  final FarmSummary farm;

  @override
  State<FieldHistoryEntry> createState() => _FieldHistoryEntryState();
}

class _FieldHistoryEntryState extends State<FieldHistoryEntry> {
  FarmInsights? _data;
  Timer? _poll;
  bool _stalled = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  @override
  void dispose() {
    _poll?.cancel();
    super.dispose();
  }

  Future<void> _load() async {
    final loader = InsightsLoader(AppScope.read(context).api, widget.farm.id);
    final c = await loader.cached();
    if (c != null && mounted && _data == null) setState(() => _data = c);
    try {
      final f = await loader.fresh();
      if (mounted) setState(() => _data = f);
    } on ApiException {
      // The copy (or nothing) stays; the card still opens the screen.
    }
    if (mounted) setState(() => _stalled = loader.stalled);
    _poll?.cancel();
    if (mounted && loader.keepChecking) {
      _poll = Timer(const Duration(seconds: 30), () {
        if (mounted) _load();
      });
    }
  }

  Future<void> _open() async {
    await Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => FieldHistoryScreen(farm: widget.farm),
      ),
    );
    if (mounted) _load();
  }

  @override
  Widget build(BuildContext context) {
    final d = _data;
    final ready = d?.ready ?? 0;
    final seasons = d?.topic('greenness')?.m('seasons_measured');
    final sub = d == null
        ? 'Rain, frost, soil and greenness of this field over the years'
        : ready == 0
        ? _stalled
              ? 'The analysis has not finished on the server · tap to check again'
              : 'Starting the analysis of this field · about 7 minutes'
        : ready < FarmInsights.all.length
        ? _stalled
              ? '$ready of ${FarmInsights.all.length} topics ready · the rest could not be read yet'
              : '$ready of ${FarmInsights.all.length} topics ready · reading satellite pictures'
        : '${seasons == null ? 'Years' : '${seasons.round()} seasons'} of rain, frost, soil and greenness · 5 topics';
    return Material(
      color: JColors.card,
      borderRadius: BorderRadius.circular(16),
      child: InkWell(
        onTap: _open,
        borderRadius: BorderRadius.circular(16),
        child: Container(
          padding: const EdgeInsets.all(14),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(16),
            border: Border.all(color: JColors.line),
          ),
          child: Row(
            spacing: 12,
            children: [
              Container(
                width: 40,
                height: 40,
                decoration: BoxDecoration(
                  color: JColors.accentSoft,
                  borderRadius: BorderRadius.circular(12),
                ),
                child: const Icon(
                  Icons.history_rounded,
                  size: 22,
                  color: JColors.accent,
                ),
              ),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  spacing: 2,
                  children: [
                    Text(
                      'Field history',
                      style: latText(size: 16, weight: FontWeight.w700),
                    ),
                    Text(
                      sub,
                      style: latText(
                        size: 12.5,
                        color: JColors.muted,
                        height: 1.35,
                      ),
                    ),
                  ],
                ),
              ),
              if (d != null && ready < FarmInsights.all.length)
                const SizedBox(
                  width: 18,
                  height: 18,
                  child: CircularProgressIndicator(
                    strokeWidth: 2.2,
                    color: JColors.accent,
                  ),
                )
              else
                const Icon(
                  Icons.chevron_right_rounded,
                  size: 22,
                  color: JColors.muted,
                ),
            ],
          ),
        ),
      ),
    );
  }
}
