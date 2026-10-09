import 'package:flutter/material.dart';

import '../../api/api.dart';
import '../../app_scope.dart';
import '../../theme.dart';
import '../../widgets/farm_card.dart';
import '../../widgets/header.dart';
import '../tabs.dart';

/// Alerts for the open farm (Pencil: Screen/Alerts), newest first, grouped
/// by day. Red ones were pushed to the phone; yellow ones are only here.
/// The server has no alerts yet (FRONTEND.md 14): its 404 shows a calm
/// "coming soon", never made-up alerts.
class AlertsScreen extends StatefulWidget {
  const AlertsScreen({super.key, this.farm});

  /// The open farm, or null for all the farmer's farms (from My farms).
  final FarmSummary? farm;

  @override
  State<AlertsScreen> createState() => _AlertsScreenState();
}

class _AlertsScreenState extends State<AlertsScreen> {
  List<FarmAlert>? _alerts;
  ApiException? _error;

  /// The farms shown, and each alert's farm name when there are several.
  List<FarmSummary> _farms = const [];
  final Map<FarmAlert, String> _farmOf = {};

  /// Ticked as done on this phone (the server has no "done" route yet).
  final Set<String> _done = {};

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final api = AppScope.read(context).api;
    try {
      final farms = widget.farm == null ? await api.getFarms() : [widget.farm!];
      final lists = await Future.wait(farms.map((f) => api.getAlerts(f.id)));
      _farms = farms;
      _farmOf.clear();
      final a = <FarmAlert>[];
      for (final (i, l) in lists.indexed) {
        for (final x in l) {
          a.add(x);
          if (farms.length > 1) _farmOf[x] = farms[i].name;
        }
      }
      a.sort((x, y) => (y.day ?? DateTime(0)).compareTo(x.day ?? DateTime(0)));
      if (mounted) {
        setState(() {
          _alerts = a;
          _error = null;
        });
        _publishCount();
      }
    } on ApiException catch (e) {
      if (mounted) setState(() => _error = e);
    }
  }

  /// The badge on the bell counts what is not ticked done.
  void _publishCount() {
    final a = _alerts ?? const <FarmAlert>[];
    openAlerts.value = {
      ...openAlerts.value,
      widget.farm?.id ?? kAllFarms: a
          .where((x) => !x.done && !_done.contains(x.id))
          .length,
    };
  }

  @override
  Widget build(BuildContext context) {
    final alerts = _alerts;
    final e = _error;
    return Column(
      children: [
        const JutyarHeader(),
        Expanded(
          child: englishScope(
            context,
            RefreshIndicator(
              color: JColors.accent,
              onRefresh: _load,
              child: ListView(
                padding: const EdgeInsets.fromLTRB(20, 12, 20, 24),
                children: [
                  Text(
                    'Alerts',
                    style: latText(
                      size: 24,
                      weight: FontWeight.w700,
                      letterSpacing: -0.4,
                    ),
                  ),
                  const SizedBox(height: 6),
                  Text(
                    'Red was sent to your phone. Yellow is only here.',
                    style: latText(
                      size: 15,
                      weight: FontWeight.w400,
                      color: JColors.muted,
                      height: 1.45,
                    ),
                  ),
                  const SizedBox(height: 16),
                  if (e != null)
                    _Empty(
                      icon: e.isOffline
                          ? Icons.wifi_off_rounded
                          : Icons.notifications_off_outlined,
                      text: switch (e) {
                        _ when e.isOffline =>
                          'No internet. Pull down to try again.',
                        _ when e.status == 404 =>
                          'Alerts are coming soon. None are sent yet, so you have missed nothing.',
                        _ => 'Could not load alerts. Pull down to try again.',
                      },
                    )
                  else if (alerts == null)
                    const Padding(
                      padding: EdgeInsets.only(top: 40),
                      child: Center(
                        child: CircularProgressIndicator(color: JColors.accent),
                      ),
                    )
                  else if (alerts.isEmpty)
                    _Empty(
                      icon: Icons.notifications_off_outlined,
                      text: widget.farm == null
                          ? 'No alerts for your farms'
                          : 'No alerts for ${widget.farm!.name}',
                    )
                  else ...[
                    for (final (label, group) in _byDay(alerts)) ...[
                      Text(
                        label,
                        style: latText(
                          size: 12,
                          weight: FontWeight.w700,
                          color: JColors.muted,
                          letterSpacing: 0.6,
                        ),
                      ),
                      const SizedBox(height: 8),
                      for (final a in group) ...[
                        _AlertRow(
                          alert: a,
                          farmName: _farmOf[a],
                          done: a.done || _done.contains(a.id),
                          onDone: () {
                            setState(
                              () => _done.contains(a.id)
                                  ? _done.remove(a.id)
                                  : _done.add(a.id),
                            );
                            _publishCount();
                          },
                        ),
                        const SizedBox(height: 8),
                      ],
                      const SizedBox(height: 8),
                    ],
                    // All farms: say which farms have nothing (design:
                    // "No alerts for Tomato plot").
                    if (_farms.length > 1)
                      for (final f in _farms)
                        if (!_farmOf.containsValue(f.name)) ...[
                          _Empty(
                            icon: Icons.notifications_off_outlined,
                            text: 'No alerts for ${f.name}',
                          ),
                          const SizedBox(height: 8),
                        ],
                  ],
                ],
              ),
            ),
          ),
        ),
      ],
    );
  }
}

const _months = [
  'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', //
  'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec',
];

/// "TODAY, 13 OCT", "YESTERDAY", "LAST WEEK", then "EARLIER".
List<(String, List<FarmAlert>)> _byDay(List<FarmAlert> alerts) {
  final now = DateTime.now();
  final today = DateTime(now.year, now.month, now.day);
  String label(DateTime? d) {
    if (d == null) return 'EARLIER';
    final days = today.difference(DateTime(d.year, d.month, d.day)).inDays;
    if (days <= 0) {
      return 'TODAY, ${d.day} ${_months[d.month - 1].toUpperCase()}';
    }
    if (days == 1) return 'YESTERDAY';
    if (days <= 7) return 'LAST WEEK';
    return 'EARLIER';
  }

  final out = <(String, List<FarmAlert>)>[];
  for (final a in alerts) {
    final l = label(a.day);
    if (out.isEmpty || out.last.$1 != l) out.add((l, []));
    out.last.$2.add(a);
  }
  return out;
}

/// One alert (design: Alert Row): level bar, icon, title, what to do, how
/// sure and when, and a tick to mark it done.
class _AlertRow extends StatelessWidget {
  const _AlertRow({
    required this.alert,
    required this.done,
    required this.onDone,
    this.farmName,
  });
  final FarmAlert alert;

  /// Shown when the list holds several farms.
  final String? farmName;
  final bool done;
  final VoidCallback onDone;

  static IconData _icon(String type) => switch (type) {
    'frost' => Icons.ac_unit_rounded,
    'rust_weather' => Icons.grain_rounded,
    'dust' || 'wind' => Icons.air_rounded,
    'urea_rain' || 'rain' => Icons.umbrella_outlined,
    'field_drop' => Icons.trending_down_rounded,
    'heat' => Icons.wb_sunny_outlined,
    _ => Icons.warning_amber_rounded,
  };

  @override
  Widget build(BuildContext context) {
    final a = alert;
    final c = a.isAlarm ? JColors.levelAlarm : JColors.levelWatch;
    final soft = a.isAlarm ? JColors.levelAlarmSoft : JColors.levelWatchSoft;
    final d = a.day;
    final when = d == null
        ? ''
        : '${d.day} ${_months[d.month - 1]}, '
              '${d.hour.toString().padLeft(2, '0')}:'
              '${d.minute.toString().padLeft(2, '0')}';
    return Container(
      clipBehavior: Clip.antiAlias,
      decoration: BoxDecoration(
        color: JColors.card,
        borderRadius: BorderRadius.circular(14),
      ),
      child: IntrinsicHeight(
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Container(width: 4, color: c),
            Expanded(
              child: Padding(
                padding: const EdgeInsets.fromLTRB(16, 12, 12, 12),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  spacing: 12,
                  children: [
                    Container(
                      width: 28,
                      height: 28,
                      decoration: BoxDecoration(
                        color: soft,
                        borderRadius: BorderRadius.circular(8),
                      ),
                      child: Icon(_icon(a.type), size: 16, color: c),
                    ),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        spacing: 4,
                        children: [
                          Text(
                            a.en,
                            style: latText(
                              size: 14,
                              weight: FontWeight.w700,
                              height: 1.3,
                            ),
                          ),
                          if (a.actionEn.isNotEmpty)
                            Text(
                              a.actionEn,
                              style: latText(
                                size: 13,
                                weight: FontWeight.w500,
                                height: 1.35,
                              ),
                            ),
                          Padding(
                            padding: const EdgeInsets.only(top: 2),
                            child: Row(
                              spacing: 8,
                              children: [
                                Container(
                                  padding: const EdgeInsets.symmetric(
                                    horizontal: 8,
                                    vertical: 2,
                                  ),
                                  decoration: BoxDecoration(
                                    color: JColors.accentSoft,
                                    borderRadius: BorderRadius.circular(999),
                                  ),
                                  child: Text(
                                    a.confidence,
                                    style: latText(
                                      size: 11,
                                      weight: FontWeight.w700,
                                      color: JColors.accent,
                                    ),
                                  ),
                                ),
                                if (when.isNotEmpty || farmName != null)
                                  Text(
                                    [
                                      if (when.isNotEmpty) when,
                                      ?farmName,
                                    ].join(' · '),
                                    style: latText(
                                      size: 12,
                                      weight: FontWeight.w500,
                                      color: JColors.muted,
                                    ),
                                  ),
                              ],
                            ),
                          ),
                        ],
                      ),
                    ),
                    Semantics(
                      button: true,
                      checked: done,
                      label: done ? 'Done' : 'Mark as done',
                      excludeSemantics: true,
                      child: InkWell(
                        onTap: onDone,
                        customBorder: const CircleBorder(),
                        child: Container(
                          width: 24,
                          height: 24,
                          decoration: BoxDecoration(
                            color: done ? JColors.accent : null,
                            shape: BoxShape.circle,
                            border: Border.all(
                              color: done ? JColors.accent : JColors.line,
                              width: 1.5,
                            ),
                          ),
                          child: done
                              ? const Icon(
                                  Icons.check_rounded,
                                  size: 14,
                                  color: Colors.white,
                                )
                              : null,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// The dashed empty box of the design ("No alerts for Tomato plot").
class _Empty extends StatelessWidget {
  const _Empty({required this.icon, required this.text});
  final IconData icon;
  final String text;

  @override
  Widget build(BuildContext context) => CustomPaint(
    painter: const DashedBorderPainter(color: Color(0x805E6E64), radius: 12),
    child: Container(
      width: double.infinity,
      constraints: const BoxConstraints(minHeight: 56),
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
      decoration: BoxDecoration(
        color: const Color(0x66FFFFFF),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        spacing: 10,
        children: [
          Icon(icon, size: 18, color: JColors.muted),
          Flexible(
            child: Text(
              text,
              textAlign: TextAlign.center,
              style: latText(
                size: 13,
                weight: FontWeight.w500,
                color: JColors.muted,
              ),
            ),
          ),
        ],
      ),
    ),
  );
}
