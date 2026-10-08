import 'package:flutter/material.dart';

import '../../api/api.dart';
import '../../app_scope.dart';
import '../../theme.dart';
import 'farm_drawing.dart';

const _wd = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];
const _mon = [
  'Jan',
  'Feb',
  'Mar',
  'Apr',
  'May',
  'Jun',
  'Jul',
  'Aug',
  'Sep',
  'Oct',
  'Nov',
  'Dec',
];

/// "2026-10-05" -> "5 Oct".
String fmtDay(String ymd) {
  final d = DateTime.parse(ymd);
  return '${d.day} ${_mon[d.month - 1]}';
}

/// Local time as "8 Oct 06:00".
String fmtWhen(DateTime t) {
  final l = t.toLocal();
  String two(int v) => v.toString().padLeft(2, '0');
  return '${l.day} ${_mon[l.month - 1]} ${two(l.hour)}:${two(l.minute)}';
}

/// Ten day chips (rain and alert dots) and the list of things to do.
class WeekStrip extends StatelessWidget {
  const WeekStrip({super.key, required this.plan});
  final FarmPlan plan;

  @override
  Widget build(BuildContext context) {
    final s = AppScope.of(context).s;
    final n = plan.rainMm.length;
    String ymd(int i) =>
        plan.from.add(Duration(days: i)).toIso8601String().substring(0, 10);
    FarmStatus worst(String day) {
      var w = FarmStatus.none;
      for (final a in plan.alerts.where((a) => a.day == day)) {
        if (a.level == FarmStatus.alarm) return FarmStatus.alarm;
        w = FarmStatus.watch;
      }
      return w;
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      spacing: 10,
      children: [
        SingleChildScrollView(
          scrollDirection: Axis.horizontal,
          child: Row(
            spacing: 6,
            children: [
              for (var i = 0; i < n; i++)
                _DayChip(
                  date: plan.from.add(Duration(days: i)),
                  rain: plan.rainMm[i],
                  dot: worst(ymd(i)),
                  today: i == 0,
                ),
            ],
          ),
        ),
        if (plan.decisions.isEmpty)
          Text(
            s.nothingToDo,
            style: jText(false, size: 13.5, color: JColors.muted),
          ),
        for (final d in plan.decisions) _DecisionRow(decision: d),
        Text(
          s.forecastSource(plan.source, fmtWhen(plan.issued)),
          style: jText(false, size: 11, color: JColors.faint),
        ),
      ],
    );
  }
}

class _DayChip extends StatelessWidget {
  const _DayChip({
    required this.date,
    required this.rain,
    required this.dot,
    required this.today,
  });
  final DateTime date;
  final double? rain;
  final FarmStatus dot;
  final bool today;

  @override
  Widget build(BuildContext context) {
    final r = rain ?? 0;
    return Container(
      width: 46,
      height: 68,
      padding: const EdgeInsets.symmetric(vertical: 6),
      decoration: BoxDecoration(
        color: today ? JColors.accentSoft : JColors.bg,
        borderRadius: BorderRadius.circular(10),
        border: Border.all(
          color: today
              ? JColors.accent.withValues(alpha: 0.4)
              : JColors.cardLine,
        ),
      ),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(
            _wd[date.weekday - 1],
            style: latText(size: 10.5, color: JColors.muted),
          ),
          Text(
            '${date.day}',
            style: latText(size: 15, weight: FontWeight.w800),
          ),
          if (r >= 1)
            Text(
              '${r.round()} mm',
              style: latText(
                size: 9.5,
                weight: FontWeight.w700,
                color: const Color(0xFF2F6FB0),
              ),
            )
          else if (dot != FarmStatus.none)
            Container(
              width: 7,
              height: 7,
              decoration: BoxDecoration(
                color: levelColor(dot),
                shape: BoxShape.circle,
              ),
            )
          else
            const SizedBox(height: 7),
        ],
      ),
    );
  }
}

class _DecisionRow extends StatelessWidget {
  const _DecisionRow({required this.decision});
  final PlanDecision decision;

  @override
  Widget build(BuildContext context) {
    final good = const {
      'sow_go',
      'urea_go',
      'spray_ok',
    }.contains(decision.code);
    final icon = switch (decision.code) {
      'sow_go' || 'sow_wait' => Icons.grass_rounded,
      'urea_go' || 'urea_hold' => Icons.scatter_plot_outlined,
      'spray_ok' => Icons.sanitizer_outlined,
      'frost_check' => Icons.ac_unit_rounded,
      'heat_check' => Icons.wb_sunny_outlined,
      'check_rust' => Icons.eco_outlined,
      'count_sunn_pest' => Icons.bug_report_outlined,
      'dust_delay' => Icons.blur_on_rounded,
      _ => Icons.info_outline_rounded,
    };
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      spacing: 10,
      children: [
        Container(
          width: 28,
          height: 28,
          decoration: BoxDecoration(
            color: good ? JColors.accentSoft : JColors.levelWatchSoft,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Icon(
            icon,
            size: 16,
            color: good ? JColors.accent : JColors.gold,
          ),
        ),
        Expanded(
          child: Padding(
            padding: const EdgeInsets.only(top: 4),
            child: Text(
              decision.text(false),
              style: jText(false, size: 13.5, height: 1.35),
            ),
          ),
        ),
      ],
    );
  }
}
