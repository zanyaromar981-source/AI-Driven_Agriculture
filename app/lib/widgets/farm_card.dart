import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../api/api.dart';
import '../app_scope.dart';
import '../geo.dart';
import '../theme.dart';

class FarmCard extends StatelessWidget {
  const FarmCard({
    super.key,
    required this.farm,
    required this.onTap,
    this.waiting = false,
  });
  final FarmSummary farm;
  final VoidCallback onTap;

  /// Saved on the phone, not uploaded yet.
  final bool waiting;

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    final s = scope.s;
    final ku = scope.ku;
    final meta = jText(ku, size: 13, color: JColors.muted);
    final metaNum = latText(
      size: 13,
      weight: FontWeight.w600,
      color: JColors.muted,
    );
    return Material(
      color: JColors.card,
      borderRadius: BorderRadius.circular(16),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(16),
        child: Container(
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            border: Border.all(color: JColors.cardLine),
            borderRadius: BorderRadius.circular(16),
          ),
          child: Row(
            spacing: 14,
            children: [
              FarmThumbnail(status: farm.status),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  spacing: 4,
                  children: [
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      spacing: 8,
                      children: [
                        Expanded(
                          child: Text(
                            farm.name,
                            style: jText(ku, size: 17, weight: FontWeight.w700),
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                        (waiting
                            ? const WaitingPill()
                            : StatusPill(
                                status: farm.status,
                                hasPicture: farm.lastPicture != null,
                              )),
                      ],
                    ),
                    Text.rich(
                      TextSpan(
                        children: [
                          TextSpan(
                            text: fmtDunam(farm.areaDunam),
                            style: metaNum,
                          ),
                          TextSpan(
                            text: farm.mainCrop == null
                                ? ' ${s.dunam}'
                                : ' ${s.dunam} · ${s.crop(farm.mainCrop)}',
                            style: meta,
                          ),
                        ],
                      ),
                    ),
                    if (farm.lastPicture != null)
                      Text.rich(
                        TextSpan(
                          children: [
                            TextSpan(text: '${s.last}: ', style: meta),
                            TextSpan(
                              text: '\u2066${farm.lastPicture}\u2069',
                              style: metaNum,
                            ),
                          ],
                        ),
                      ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class WaitingPill extends StatelessWidget {
  const WaitingPill({super.key});

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
      decoration: BoxDecoration(
        color: JColors.levelNoneSoft,
        borderRadius: BorderRadius.circular(999),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        spacing: 4,
        children: [
          const Icon(
            Icons.cloud_upload_outlined,
            size: 13,
            color: JColors.muted,
          ),
          Text(
            scope.s.waitingUpload,
            style: jText(
              scope.ku,
              size: 12,
              weight: FontWeight.w600,
              color: JColors.muted,
            ),
          ),
        ],
      ),
    );
  }
}

class StatusPill extends StatelessWidget {
  const StatusPill({super.key, required this.status, this.hasPicture = true});
  final FarmStatus status;
  final bool hasPicture;

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    final (bg, fg) = switch (status) {
      FarmStatus.normal => (JColors.accentSoft, JColors.accent),
      FarmStatus.watch => (JColors.levelWatchSoft, JColors.levelWatch),
      FarmStatus.alarm => (JColors.levelAlarmSoft, JColors.levelAlarm),
      FarmStatus.none => (JColors.goldSoft, JColors.gold),
    };
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
      decoration: BoxDecoration(
        color: bg,
        borderRadius: BorderRadius.circular(999),
      ),
      child: Text(
        scope.s.status(status, hasPicture: hasPicture),
        style: jText(scope.ku, size: 12, weight: FontWeight.w600, color: fg),
      ),
    );
  }
}

/// 56x56 field picture placeholder: soil or crop gradient with furrows.
class FarmThumbnail extends StatelessWidget {
  const FarmThumbnail({super.key, required this.status});
  final FarmStatus status;

  @override
  Widget build(BuildContext context) {
    final colors = switch (status) {
      FarmStatus.normal => const [Color(0xFF8DBF5A), Color(0xFF4F8A3C)],
      FarmStatus.watch => const [Color(0xFFC5A54A), Color(0xFF8D6E3A)],
      FarmStatus.alarm => const [Color(0xFFC96A4A), Color(0xFF8A3A2A)],
      FarmStatus.none => const [Color(0xFFA9845A), Color(0xFF7A5A3A)],
    };
    return ClipRRect(
      borderRadius: BorderRadius.circular(12),
      child: Container(
        width: 56,
        height: 56,
        decoration: BoxDecoration(
          gradient: LinearGradient(
            begin: Alignment.topLeft,
            end: Alignment.bottomRight,
            colors: colors,
          ),
        ),
        child: const CustomPaint(painter: _FurrowPainter()),
      ),
    );
  }
}

class _FurrowPainter extends CustomPainter {
  const _FurrowPainter();

  @override
  void paint(Canvas canvas, Size size) {
    final p = Paint()
      ..color = Colors.white.withValues(alpha: 0.14)
      ..strokeWidth = 2.5;
    for (var y = -size.width; y < size.height + size.width; y += 11) {
      canvas.drawLine(
        Offset(-10, y),
        Offset(size.width + 10, y + size.width * 0.45),
        p,
      );
    }
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}

class AddFarmCard extends StatelessWidget {
  const AddFarmCard({super.key, required this.onTap});
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    return SizedBox(
      height: 64,
      child: CustomPaint(
        painter: DashedBorderPainter(
          color: JColors.accent.withValues(alpha: 0.5),
        ),
        child: Material(
          color: Colors.white.withValues(alpha: 0.4),
          borderRadius: BorderRadius.circular(16),
          child: InkWell(
            onTap: onTap,
            borderRadius: BorderRadius.circular(16),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              spacing: 10,
              children: [
                const Icon(Icons.add_rounded, size: 20, color: JColors.accent),
                Text(
                  scope.s.add,
                  style: jText(
                    scope.ku,
                    size: 15,
                    weight: FontWeight.w600,
                    color: JColors.accent,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class DashedBorderPainter extends CustomPainter {
  const DashedBorderPainter({
    required this.color,
    this.radius = 16,
    this.dash = 6,
    this.gap = 5,
    this.width = 1.5,
  });

  final Color color;
  final double radius;
  final double dash;
  final double gap;
  final double width;

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = color
      ..style = PaintingStyle.stroke
      ..strokeWidth = width;
    final rect = Rect.fromLTWH(
      width / 2,
      width / 2,
      size.width - width,
      size.height - width,
    );
    final path = Path()
      ..addRRect(RRect.fromRectAndRadius(rect, Radius.circular(radius)));
    for (final m in path.computeMetrics()) {
      var d = 0.0;
      while (d < m.length) {
        final e = math.min(d + dash, m.length);
        canvas.drawPath(m.extractPath(d, e), paint);
        d = e + gap;
      }
    }
  }

  @override
  bool shouldRepaint(covariant DashedBorderPainter old) =>
      old.color != color ||
      old.radius != radius ||
      old.dash != dash ||
      old.gap != gap ||
      old.width != width;
}

/// Shown when the number has no farms yet.
class EmptyFarms extends StatelessWidget {
  const EmptyFarms({super.key});

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 30, horizontal: 10),
      child: Column(
        spacing: 10,
        children: [
          Container(
            width: 72,
            height: 72,
            decoration: const BoxDecoration(
              color: JColors.accentSoft,
              shape: BoxShape.circle,
            ),
            child: const Icon(
              Icons.add_location_alt_outlined,
              size: 32,
              color: JColors.accent,
            ),
          ),
          Text(
            scope.s.empty1,
            style: jText(scope.ku, size: 16, weight: FontWeight.w700),
          ),
          Text(
            scope.s.empty2,
            textAlign: TextAlign.center,
            style: jText(scope.ku, size: 14, color: JColors.muted, height: 1.6),
          ),
        ],
      ),
    );
  }
}
