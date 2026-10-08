import 'package:flutter/material.dart';

import '../app_scope.dart';
import '../theme.dart';
import 'grain_sun.dart';

/// Brand on the left, language toggle on the right (same in both languages).
class JutyarHeader extends StatelessWidget {
  const JutyarHeader({super.key});

  @override
  Widget build(BuildContext context) {
    return Directionality(
      textDirection: TextDirection.ltr,
      child: SizedBox(
        height: 56,
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 20),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Row(
                spacing: 10,
                children: [
                  const LogoMark(),
                  Text(
                    'Jutyar',
                    style: latText(size: 20, weight: FontWeight.w800),
                  ),
                ],
              ),
              const LanguageToggle(),
            ],
          ),
        ),
      ),
    );
  }
}

class LanguageToggle extends StatelessWidget {
  const LanguageToggle({super.key});

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    return Container(
      padding: const EdgeInsets.all(3),
      decoration: BoxDecoration(
        color: JColors.toggleBg,
        borderRadius: BorderRadius.circular(999),
      ),
      child: Row(
        spacing: 2,
        children: [
          _Option(
            label: 'English',
            selected: !scope.ku,
            style: latText(size: 13, weight: FontWeight.w600),
            onTap: () => scope.setKu(false),
          ),
          _Option(
            label: 'کوردی',
            selected: scope.ku,
            style: jText(true, size: 13, weight: FontWeight.w700),
            onTap: () => scope.setKu(true),
          ),
        ],
      ),
    );
  }
}

class _Option extends StatelessWidget {
  const _Option({
    required this.label,
    required this.selected,
    required this.style,
    required this.onTap,
  });

  final String label;
  final bool selected;
  final TextStyle style;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      behavior: HitTestBehavior.opaque,
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 180),
        height: 32,
        padding: const EdgeInsets.symmetric(horizontal: 14),
        alignment: Alignment.center,
        decoration: BoxDecoration(
          color: selected ? JColors.card : Colors.transparent,
          borderRadius: BorderRadius.circular(999),
          boxShadow: selected
              ? [
                  BoxShadow(
                    color: Colors.black.withValues(alpha: 0.10),
                    blurRadius: 3,
                    offset: const Offset(0, 1),
                  ),
                ]
              : const [],
        ),
        child: Text(
          label,
          style: style.copyWith(
            color: selected ? JColors.ink : JColors.muted,
            fontWeight: selected ? FontWeight.w700 : style.fontWeight,
          ),
        ),
      ),
    );
  }
}
