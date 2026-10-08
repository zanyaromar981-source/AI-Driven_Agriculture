import 'package:flutter/material.dart';

import '../../api/api.dart';
import '../../app_scope.dart';
import '../../theme.dart';
import 'farm_drawing.dart';

/// Small pill with the level name on its soft colour.
class LevelPill extends StatelessWidget {
  const LevelPill({super.key, required this.level});
  final FarmStatus level;

  @override
  Widget build(BuildContext context) {
    final s = AppScope.of(context).s;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 9, vertical: 3),
      decoration: BoxDecoration(
        color: levelSoft(level),
        borderRadius: BorderRadius.circular(999),
      ),
      child: Text(
        s.levelName(level),
        style: jText(
          false,
          size: 12,
          weight: FontWeight.w700,
          color: level == FarmStatus.none ? JColors.muted : levelColor(level),
        ),
      ),
    );
  }
}

/// The card that opens over the drawing when a cell or a crop plot is tapped.
class CellCard extends StatelessWidget {
  const CellCard({
    super.key,
    required this.title,
    required this.level,
    this.pct,
    this.lines = const [],
    this.emptyText,
    required this.onClose,
    this.onAsk,
    this.onReport,
  });

  final String title;
  final FarmStatus level;
  final int? pct;
  final List<String> lines;

  /// Shown instead of the % when there is no reading (default: no reading yet).
  final String? emptyText;
  final VoidCallback onClose;
  final VoidCallback? onAsk;
  final VoidCallback? onReport;

  @override
  Widget build(BuildContext context) {
    final s = AppScope.of(context).s;
    final p = pct;
    return Material(
      color: JColors.card,
      elevation: 6,
      shadowColor: const Color(0x44000000),
      borderRadius: BorderRadius.circular(14),
      child: Padding(
        padding: const EdgeInsets.fromLTRB(14, 10, 6, 12),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          spacing: 4,
          children: [
            Row(
              children: [
                Expanded(
                  child: Text(
                    title,
                    style: jText(
                      false,
                      size: 13,
                      weight: FontWeight.w600,
                      color: JColors.muted,
                    ),
                  ),
                ),
                InkWell(
                  onTap: onClose,
                  customBorder: const CircleBorder(),
                  child: const Padding(
                    padding: EdgeInsets.all(6),
                    child: Icon(
                      Icons.close_rounded,
                      size: 18,
                      color: JColors.muted,
                    ),
                  ),
                ),
              ],
            ),
            Padding(
              padding: const EdgeInsets.only(right: 8),
              child: Row(
                spacing: 8,
                children: [
                  Flexible(
                    child: Text(
                      p == null ? (emptyText ?? s.noReading) : s.pctOfNormal(p),
                      style: jText(
                        false,
                        size: p == null ? 14 : 21,
                        weight: FontWeight.w800,
                      ),
                    ),
                  ),
                  if (p != null) LevelPill(level: level),
                ],
              ),
            ),
            for (final l in lines)
              if (l.isNotEmpty)
                Text(l, style: jText(false, size: 12.5, color: JColors.muted)),
            if (onAsk != null || onReport != null)
              Padding(
                padding: const EdgeInsets.only(top: 6, right: 8),
                child: Row(
                  spacing: 8,
                  children: [
                    if (onAsk != null)
                      Expanded(
                        child: FilledButton(
                          onPressed: onAsk,
                          style: FilledButton.styleFrom(
                            backgroundColor: JColors.accent,
                            minimumSize: const Size(0, 40),
                            padding: const EdgeInsets.symmetric(horizontal: 8),
                            shape: RoundedRectangleBorder(
                              borderRadius: BorderRadius.circular(10),
                            ),
                          ),
                          child: Text(
                            s.askSpot,
                            style: jText(
                              false,
                              size: 13,
                              weight: FontWeight.w700,
                              color: Colors.white,
                            ),
                          ),
                        ),
                      ),
                    if (onReport != null)
                      Expanded(
                        child: OutlinedButton(
                          onPressed: onReport,
                          style: OutlinedButton.styleFrom(
                            minimumSize: const Size(0, 40),
                            padding: const EdgeInsets.symmetric(horizontal: 8),
                            side: const BorderSide(color: JColors.line),
                            shape: RoundedRectangleBorder(
                              borderRadius: BorderRadius.circular(10),
                            ),
                          ),
                          child: Text(
                            s.reportHere,
                            style: jText(
                              false,
                              size: 13,
                              weight: FontWeight.w700,
                              color: JColors.ink,
                            ),
                          ),
                        ),
                      ),
                  ],
                ),
              ),
          ],
        ),
      ),
    );
  }
}
