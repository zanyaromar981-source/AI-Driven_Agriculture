import 'package:flutter/material.dart';

import '../../api/api.dart';
import '../../theme.dart';
import '../../widgets/common.dart';
import 'doctor_widgets.dart';

/// The Doctor's answer (Pencil: Screen/Answer): what is most likely and how
/// sure, why (each reason names its input), what to do this week, and what
/// it cannot tell. The Doctor's own words come in Sorani and English.
class DoctorAnswerScreen extends StatefulWidget {
  const DoctorAnswerScreen({
    super.key,
    required this.farm,
    required this.answer,
    this.showKu = true,
  });
  final FarmSummary farm;
  final DoctorAnswer answer;

  /// Which language the Doctor's words open in.
  final bool showKu;

  @override
  State<DoctorAnswerScreen> createState() => _DoctorAnswerScreenState();
}

class _DoctorAnswerScreenState extends State<DoctorAnswerScreen> {
  late bool _ku = widget.showKu && widget.answer.ku.isNotEmpty;

  @override
  Widget build(BuildContext context) {
    final a = widget.answer;
    final words = _ku ? a.ku : a.en;
    return DoctorPage(
      backLabel: 'Ask again',
      children: [
        Text(
          'The Doctor says',
          style: latText(size: 26, weight: FontWeight.w800),
        ),
        const SizedBox(height: 14),
        _Card(
          children: [
            Text(
              'MOST LIKELY',
              style: latText(
                size: 11,
                weight: FontWeight.w700,
                color: JColors.muted,
              ),
            ),
            Text(
              a.likely.isEmpty ? 'The Doctor could not say' : a.likely,
              style: latText(size: 19, weight: FontWeight.w700, height: 1.3),
            ),
            const SizedBox(height: 2),
            ConfidenceBar(confidence: a.confidence),
          ],
        ),
        if (words.isNotEmpty) ...[
          const SizedBox(height: 14),
          _Card(
            children: [
              Row(
                children: [
                  Expanded(
                    child: Text(
                      "The Doctor's words",
                      style: latText(size: 15, weight: FontWeight.w700),
                    ),
                  ),
                  if (a.ku.isNotEmpty && a.en.isNotEmpty)
                    _LangPill(
                      ku: _ku,
                      onChanged: (v) => setState(() => _ku = v),
                    ),
                ],
              ),
              Directionality(
                textDirection: _ku ? TextDirection.rtl : TextDirection.ltr,
                child: SizedBox(
                  width: double.infinity,
                  child: Text(
                    words,
                    style: jText(_ku, size: _ku ? 16 : 15, height: 1.6),
                  ),
                ),
              ),
            ],
          ),
        ],
        if (a.why.isNotEmpty) ...[
          const SizedBox(height: 14),
          _Card(
            children: [
              Text('Why', style: latText(size: 15, weight: FontWeight.w700)),
              for (final w in a.why) _WhyRow(line: w),
            ],
          ),
        ],
        if (a.actions.isNotEmpty) ...[
          const SizedBox(height: 14),
          _Card(
            children: [
              Text(
                'This week',
                style: latText(size: 15, weight: FontWeight.w700),
              ),
              for (final (i, step) in a.actions.indexed)
                _Step(number: i + 1, text: step),
            ],
          ),
        ],
        if (a.cannotTell.isNotEmpty) ...[
          const SizedBox(height: 14),
          Container(
            width: double.infinity,
            padding: const EdgeInsets.fromLTRB(16, 12, 16, 12),
            decoration: BoxDecoration(
              color: JColors.toggleBg,
              borderRadius: BorderRadius.circular(14),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              spacing: 4,
              children: [
                Text(
                  'What I cannot tell',
                  style: latText(size: 13, weight: FontWeight.w700),
                ),
                Text(
                  a.cannotTell.join('. ').replaceAll('..', '.'),
                  style: latText(
                    size: 13,
                    weight: FontWeight.w400,
                    color: JColors.muted,
                    height: 1.45,
                  ),
                ),
              ],
            ),
          ),
        ],
        if (a.referToOfficer) ...[
          const SizedBox(height: 14),
          Container(
            padding: const EdgeInsets.all(14),
            decoration: BoxDecoration(
              color: JColors.goldSoft,
              borderRadius: BorderRadius.circular(14),
            ),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              spacing: 10,
              children: [
                const Icon(
                  Icons.support_agent_rounded,
                  size: 22,
                  color: JColors.rayGold,
                ),
                Expanded(
                  child: Text(
                    'Show this to the extension or plant-protection office before you spray or spread anything.',
                    style: latText(
                      size: 13.5,
                      weight: FontWeight.w600,
                      height: 1.4,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ],
        const SizedBox(height: 18),
        GhostButton(
          label: 'New question',
          icon: Icons.chat_bubble_outline_rounded,
          onPressed: () => Navigator.of(context).maybePop(),
        ),
        const SizedBox(height: 14),
        Center(
          child: Text(
            'Check your field and your label.',
            style: latText(size: 12, color: JColors.muted),
          ),
        ),
      ],
    );
  }
}

class _Card extends StatelessWidget {
  const _Card({required this.children});
  final List<Widget> children;

  @override
  Widget build(BuildContext context) => Container(
    width: double.infinity,
    padding: const EdgeInsets.all(16),
    decoration: BoxDecoration(
      color: JColors.card,
      borderRadius: BorderRadius.circular(16),
    ),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      spacing: 12,
      children: children,
    ),
  );
}

class _LangPill extends StatelessWidget {
  const _LangPill({required this.ku, required this.onChanged});
  final bool ku;
  final ValueChanged<bool> onChanged;

  @override
  Widget build(BuildContext context) {
    Widget option(String label, bool value) => Semantics(
      button: true,
      selected: ku == value,
      label: label,
      excludeSemantics: true,
      child: InkWell(
        onTap: () => onChanged(value),
        borderRadius: BorderRadius.circular(999),
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
          decoration: BoxDecoration(
            color: ku == value ? JColors.card : Colors.transparent,
            borderRadius: BorderRadius.circular(999),
          ),
          child: Text(
            label,
            style: latText(
              size: 12.5,
              weight: ku == value ? FontWeight.w700 : FontWeight.w500,
              color: ku == value ? JColors.ink : JColors.muted,
            ),
          ),
        ),
      ),
    );
    return Container(
      padding: const EdgeInsets.all(3),
      decoration: BoxDecoration(
        color: JColors.toggleBg,
        borderRadius: BorderRadius.circular(999),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        spacing: 2,
        children: [option('Sorani', true), option('English', false)],
      ),
    );
  }
}

/// One reason. The Doctor writes "input -> conclusion"; the farmer sees
/// "Weather: conclusion" with an icon for the input.
class _WhyRow extends StatelessWidget {
  const _WhyRow({required this.line});
  final String line;

  @override
  Widget build(BuildContext context) {
    final (icon, text) = whyParts(line);
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      spacing: 10,
      children: [
        Container(
          width: 28,
          height: 28,
          decoration: BoxDecoration(
            color: JColors.accentSoft,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Icon(icon, size: 16, color: JColors.accent),
        ),
        Expanded(
          child: Padding(
            padding: const EdgeInsets.only(top: 5),
            child: Text(
              text,
              style: latText(size: 13.5, weight: FontWeight.w500, height: 1.4),
            ),
          ),
        ),
      ],
    );
  }
}

/// The icon and the farmer-facing text for one "input -> conclusion" line.
(IconData, String) whyParts(String line) {
  final at = line.indexOf('->');
  if (at < 0) return (Icons.lightbulb_outline_rounded, line.trim());
  final input = line.substring(0, at).trim();
  final rest = line.substring(at + 2).trim();
  final k = input.toLowerCase().replaceAll(RegExp(r'[\s_-]+'), ' ');
  final (IconData icon, String name) = switch (k) {
    _ when k.contains('history') => (Icons.history_rounded, 'Field history'),
    _ when k.contains('field') || k.contains('satellite') => (
      Icons.satellite_alt_outlined,
      'Field eye',
    ),
    _ when k.contains('weather') => (Icons.cloud_outlined, 'Weather'),
    _ when k.contains('photo') => (Icons.photo_camera_outlined, 'Your photo'),
    _ when k.contains('season') => (Icons.calendar_month_outlined, 'Season'),
    _ when k.contains('dam') => (Icons.water_outlined, 'Dams'),
    _ when k.contains('question') || k.contains('farmer') => (
      Icons.chat_bubble_outline_rounded,
      'You said',
    ),
    _ => (Icons.lightbulb_outline_rounded, input),
  };
  return (icon, rest.isEmpty ? name : '$name: $rest');
}

class _Step extends StatelessWidget {
  const _Step({required this.number, required this.text});
  final int number;
  final String text;

  @override
  Widget build(BuildContext context) => Row(
    crossAxisAlignment: CrossAxisAlignment.start,
    spacing: 10,
    children: [
      Container(
        width: 26,
        height: 26,
        alignment: Alignment.center,
        decoration: const BoxDecoration(
          color: JColors.accent,
          shape: BoxShape.circle,
        ),
        child: Text(
          '$number',
          style: latText(
            size: 13,
            weight: FontWeight.w700,
            color: Colors.white,
          ),
        ),
      ),
      Expanded(
        child: Padding(
          padding: const EdgeInsets.only(top: 3),
          child: Text(
            text,
            style: latText(size: 14, weight: FontWeight.w500, height: 1.4),
          ),
        ),
      ),
    ],
  );
}
