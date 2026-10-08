import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../theme.dart';

/// Logo 36 "Grain Sun": 21 grain-shaped rays (the Kurdish sun) around a centre.
class GrainSun extends StatelessWidget {
  const GrainSun({super.key, this.size = 26});
  final double size;

  @override
  Widget build(BuildContext context) =>
      CustomPaint(size: Size.square(size), painter: const _GrainSunPainter());
}

class _GrainSunPainter extends CustomPainter {
  const _GrainSunPainter();

  @override
  void paint(Canvas canvas, Size size) {
    canvas.scale(size.width / 100);
    final ray = Paint()..color = JColors.rayGold;
    final lens = Path()
      ..moveTo(50, 30)
      ..arcToPoint(
        const Offset(50, 7),
        radius: const Radius.circular(30),
        clockwise: true,
      )
      ..arcToPoint(
        const Offset(50, 30),
        radius: const Radius.circular(30),
        clockwise: true,
      )
      ..close();
    for (var i = 0; i < 21; i++) {
      canvas.save();
      canvas.translate(50, 50);
      canvas.rotate(i * 2 * math.pi / 21);
      canvas.translate(-50, -50);
      canvas.drawPath(lens, ray);
      canvas.restore();
    }
    canvas.drawCircle(
      const Offset(50, 50),
      14,
      Paint()..color = JColors.sunCentre,
    );
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}

/// The 34x34 rounded tile with the sun inside, used in the header.
class LogoMark extends StatelessWidget {
  const LogoMark({super.key});

  @override
  Widget build(BuildContext context) => Container(
    width: 34,
    height: 34,
    decoration: BoxDecoration(
      color: JColors.goldSoft,
      borderRadius: BorderRadius.circular(10),
    ),
    child: const Center(child: GrainSun(size: 26)),
  );
}
