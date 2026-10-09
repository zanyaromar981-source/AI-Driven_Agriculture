import 'dart:async';

import 'package:flutter/material.dart';

import '../../api/api.dart';
import '../../app_scope.dart';
import '../../theme.dart';
import 'field_history_screen.dart';
import 'now_text.dart';

/// "How it's doing now": the farm's state from the latest clear satellite
/// picture, at the top of the farm screen. Design: design/jutyar_app.pen,
/// page "Field history", row "Now Cards". English for now.
class NowCard extends StatefulWidget {
  const NowCard({super.key, required this.farm});
  final FarmSummary farm;

  @override
  State<NowCard> createState() => _NowCardState();
}

class _NowCardState extends State<NowCard> {
  FarmInsights? _data;
  Timer? _poll;

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
      // The copy on the phone (or the waiting state) stays.
    }
    _poll?.cancel();
    // The latest picture comes with the greenness topic. Once that topic is
    // in without it, it will not come later (the farm is read once), so
    // checking again only makes sense while the greenness is still read.
    if (mounted && _data?.topic('greenness') == null && loader.keepChecking) {
      _poll = Timer(const Duration(seconds: 30), () {
        if (mounted) _load();
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final v = nowView(_data, today: DateTime.now());
    // No latest picture for this farm: show nothing rather than a card that
    // says "Reading" for ever. The Field history card shows the progress.
    if (v.kind == NowKind.waiting) return const SizedBox.shrink();
    final (soft, color, icon) = switch (v.kind) {
      NowKind.waiting => (
        JColors.accentSoft,
        JColors.accent,
        Icons.autorenew_rounded,
      ),
      NowKind.bare => (JColors.goldSoft, JColors.gold, Icons.wb_sunny_outlined),
      NowKind.comingUp => (
        JColors.levelNormalSoft,
        JColors.levelNormal,
        Icons.grass_rounded,
      ),
      NowKind.growing => (
        JColors.levelNormalSoft,
        JColors.levelNormal,
        Icons.eco_outlined,
      ),
      NowKind.behind => (
        JColors.levelWatchSoft,
        JColors.levelWatch,
        Icons.eco_outlined,
      ),
      NowKind.harvested => (
        JColors.goldSoft,
        JColors.gold,
        Icons.agriculture_outlined,
      ),
      NowKind.cloudy => (
        JColors.levelNoneSoft,
        JColors.muted,
        Icons.cloud_outlined,
      ),
    };
    final titleColor = switch (v.kind) {
      NowKind.growing || NowKind.comingUp => JColors.levelNormal,
      NowKind.behind => JColors.levelWatch,
      _ => JColors.ink,
    };
    return Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: JColors.card,
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: JColors.line),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        spacing: 10,
        children: [
          Row(
            spacing: 10,
            children: [
              Container(
                width: 32,
                height: 32,
                decoration: BoxDecoration(
                  color: soft,
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Icon(icon, size: 18, color: color),
              ),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  spacing: 1,
                  children: [
                    Text(
                      "HOW IT'S DOING NOW",
                      style: latText(
                        size: 11,
                        weight: FontWeight.w700,
                        color: JColors.muted,
                      ).copyWith(letterSpacing: 0.6),
                    ),
                    Text(
                      v.title,
                      style: latText(
                        size: 18,
                        weight: FontWeight.w800,
                        color: titleColor,
                      ),
                    ),
                  ],
                ),
              ),
              if (v.date != null)
                Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 9,
                    vertical: 3,
                  ),
                  decoration: BoxDecoration(
                    color: JColors.bg,
                    borderRadius: BorderRadius.circular(999),
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    spacing: 4,
                    children: [
                      const Icon(
                        Icons.satellite_alt_outlined,
                        size: 12,
                        color: JColors.muted,
                      ),
                      Text(
                        v.date!,
                        style: latText(
                          size: 11.5,
                          weight: FontWeight.w700,
                          color: JColors.muted,
                        ),
                      ),
                    ],
                  ),
                ),
            ],
          ),
          Text(
            v.line,
            style: latText(size: 14, weight: FontWeight.w500, height: 1.45),
          ),
          if (v.chips.isNotEmpty)
            IntrinsicHeight(
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                spacing: 6,
                children: [
                  for (final (val, label) in v.chips)
                    Expanded(
                      child: Container(
                        padding: const EdgeInsets.symmetric(
                          horizontal: 10,
                          vertical: 7,
                        ),
                        decoration: BoxDecoration(
                          color: JColors.bg,
                          borderRadius: BorderRadius.circular(10),
                        ),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          spacing: 1,
                          children: [
                            Text(
                              val,
                              style: latText(size: 15, weight: FontWeight.w800),
                            ),
                            Text(
                              label,
                              style: latText(
                                size: 11,
                                color: JColors.muted,
                                height: 1.3,
                              ),
                            ),
                          ],
                        ),
                      ),
                    ),
                ],
              ),
            ),
          if (v.next != null)
            Row(
              spacing: 6,
              children: [
                const Icon(
                  Icons.schedule_rounded,
                  size: 13,
                  color: JColors.muted,
                ),
                Expanded(
                  child: Text(
                    v.next!,
                    style: latText(
                      size: 11.5,
                      color: JColors.muted,
                      height: 1.35,
                    ),
                  ),
                ),
              ],
            ),
        ],
      ),
    );
  }
}
