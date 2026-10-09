import 'package:flutter/material.dart';

import '../../app_scope.dart';
import '../../l10n/strings.dart';
import '../../theme.dart';
import '../../widgets/header.dart';

/// The frame both Doctor screens share: header, back to the farm, and an
/// English body (the Doctor's screens are English for now, like Home).
class DoctorPage extends StatelessWidget {
  const DoctorPage({
    super.key,
    required this.backLabel,
    required this.children,
  });
  final String backLabel;
  final List<Widget> children;

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
    return Directionality(
      textDirection: TextDirection.ltr,
      child: Scaffold(
        backgroundColor: JColors.bg,
        body: SafeArea(
          child: Column(
            children: [
              const JutyarHeader(),
              english(_BackRow(label: backLabel)),
              Expanded(
                child: english(
                  ListView(
                    padding: const EdgeInsets.fromLTRB(20, 4, 20, 28),
                    keyboardDismissBehavior:
                        ScrollViewKeyboardDismissBehavior.onDrag,
                    children: children,
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

/// Shown while the server works. No fake progress: the server sends no
/// steps, so this only says what is being read and how long it usually takes.
class ReadingTheField extends StatelessWidget {
  const ReadingTheField({super.key});

  @override
  Widget build(BuildContext context) => Semantics(
    liveRegion: true,
    label: 'The Doctor is reading your field. About 30 seconds.',
    excludeSemantics: true,
    child: Container(
      padding: const EdgeInsets.all(16),
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
            spacing: 12,
            children: [
              const SizedBox(
                width: 22,
                height: 22,
                child: CircularProgressIndicator(
                  strokeWidth: 2.6,
                  color: JColors.accent,
                ),
              ),
              Expanded(
                child: Text(
                  'Reading your field',
                  style: latText(size: 16, weight: FontWeight.w700),
                ),
              ),
            ],
          ),
          Text(
            'The latest satellite picture, the 10-day weather, the season\'s rain and your photos. Usually about 30 seconds; keep the app open.',
            style: latText(
              size: 13,
              weight: FontWeight.w500,
              color: JColors.muted,
              height: 1.4,
            ),
          ),
        ],
      ),
    ),
  );
}

/// A problem with the last ask, in plain words.
class DoctorNotice extends StatelessWidget {
  const DoctorNotice({super.key, required this.text});
  final String text;

  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.all(14),
    decoration: BoxDecoration(
      color: JColors.levelWatchSoft,
      borderRadius: BorderRadius.circular(14),
    ),
    child: Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      spacing: 10,
      children: [
        const Icon(Icons.info_outline_rounded, size: 20, color: JColors.ink),
        Expanded(
          child: Text(
            text,
            style: latText(size: 13.5, weight: FontWeight.w600, height: 1.4),
          ),
        ),
      ],
    ),
  );
}

/// How sure the Doctor is: three segments (Pencil: Confidence Bar).
class ConfidenceBar extends StatelessWidget {
  const ConfidenceBar({super.key, required this.confidence});

  /// `sure`, `likely` or `unsure`.
  final String confidence;

  @override
  Widget build(BuildContext context) {
    final filled = switch (confidence) {
      'sure' => 3,
      'likely' => 2,
      _ => 1,
    };
    final color = filled == 1 ? JColors.levelWatch : JColors.accent;
    return Semantics(
      label: 'How sure: $confidence',
      excludeSemantics: true,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        spacing: 8,
        children: [
          Text(
            'How sure: $confidence',
            style: latText(size: 13, weight: FontWeight.w600),
          ),
          Row(
            spacing: 4,
            children: [
              for (var i = 0; i < 3; i++)
                Expanded(
                  child: Container(
                    height: 8,
                    decoration: BoxDecoration(
                      color: i < filled ? color : JColors.line,
                      borderRadius: BorderRadius.circular(4),
                    ),
                  ),
                ),
            ],
          ),
          Text(
            'unsure · likely · sure',
            style: latText(
              size: 10.5,
              weight: FontWeight.w500,
              color: JColors.muted,
            ),
          ),
        ],
      ),
    );
  }
}
