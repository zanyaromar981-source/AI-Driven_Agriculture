import 'package:flutter/material.dart';

import '../app_scope.dart';
import '../theme.dart';
import 'header.dart';

/// Page frame used by every sign-in screen: header, progress, content, footer.
/// Scrolls when the keyboard takes the space; otherwise the footer sits at the bottom.
class JutyarPage extends StatelessWidget {
  const JutyarPage({super.key, required this.step, required this.children});

  /// 1 to 3, lights up the progress bars.
  final int step;
  final List<Widget> children;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: JColors.bg,
      body: SafeArea(
        child: Column(
          children: [
            const JutyarHeader(),
            Expanded(
              child: LayoutBuilder(
                builder: (context, box) => SingleChildScrollView(
                  keyboardDismissBehavior:
                      ScrollViewKeyboardDismissBehavior.onDrag,
                  child: ConstrainedBox(
                    constraints: BoxConstraints(minHeight: box.maxHeight),
                    child: IntrinsicHeight(
                      child: Padding(
                        padding: const EdgeInsets.fromLTRB(20, 12, 20, 28),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            ProgressSteps(step: step),
                            const SizedBox(height: 28),
                            ...children,
                            const Spacer(),
                            const SizedBox(height: 28),
                            const JutyarFooter(),
                          ],
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class ProgressSteps extends StatelessWidget {
  const ProgressSteps({super.key, required this.step, this.total = 3});
  final int step;
  final int total;

  @override
  Widget build(BuildContext context) {
    return Row(
      spacing: 6,
      children: [
        for (var i = 0; i < total; i++)
          Expanded(
            child: AnimatedContainer(
              duration: const Duration(milliseconds: 300),
              height: 4,
              decoration: BoxDecoration(
                color: i < step ? JColors.accent : JColors.line,
                borderRadius: BorderRadius.circular(2),
              ),
            ),
          ),
      ],
    );
  }
}

class PrimaryButton extends StatelessWidget {
  const PrimaryButton({
    super.key,
    required this.label,
    this.onPressed,
    this.loading = false,
    this.icon = Icons.arrow_forward_rounded,
  });

  final String label;
  final VoidCallback? onPressed;
  final bool loading;
  final IconData icon;

  @override
  Widget build(BuildContext context) {
    final ku = AppScope.of(context).ku;
    final enabled = onPressed != null && !loading;
    return AnimatedOpacity(
      opacity: enabled || loading ? 1 : 0.4,
      duration: const Duration(milliseconds: 200),
      child: SizedBox(
        height: 56,
        width: double.infinity,
        child: FilledButton(
          style: FilledButton.styleFrom(
            backgroundColor: JColors.accent,
            disabledBackgroundColor: JColors.accent,
            foregroundColor: Colors.white,
            disabledForegroundColor: Colors.white,
            elevation: 0,
            padding: EdgeInsets.zero,
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(14),
            ),
          ),
          onPressed: enabled ? onPressed : null,
          child: loading
              ? const SizedBox(
                  width: 20,
                  height: 20,
                  child: CircularProgressIndicator(
                    strokeWidth: 2.2,
                    color: Colors.white,
                  ),
                )
              : Row(
                  mainAxisSize: MainAxisSize.min,
                  spacing: 10,
                  children: [
                    Text(
                      label,
                      style: jText(
                        ku,
                        size: 17,
                        weight: FontWeight.w700,
                        color: Colors.white,
                      ),
                    ),
                    Icon(icon, size: 20),
                  ],
                ),
        ),
      ),
    );
  }
}

class InfoNote extends StatelessWidget {
  const InfoNote({
    super.key,
    this.title,
    required this.body,
    this.icon = Icons.layers_outlined,
  });
  final String? title;
  final String body;
  final IconData icon;

  @override
  Widget build(BuildContext context) {
    final ku = AppScope.of(context).ku;
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: JColors.accentSoft,
        borderRadius: BorderRadius.circular(14),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        spacing: 12,
        children: [
          Container(
            width: 32,
            height: 32,
            decoration: BoxDecoration(
              color: JColors.card,
              borderRadius: BorderRadius.circular(10),
            ),
            child: Icon(icon, size: 18, color: JColors.accent),
          ),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              spacing: 4,
              children: [
                if (title != null)
                  Text(
                    title!,
                    style: jText(ku, size: 14, weight: FontWeight.w700),
                  ),
                Text(
                  body,
                  style: jText(
                    ku,
                    size: 13,
                    color: title == null ? JColors.ink : JColors.muted,
                    height: 1.6,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class JutyarFooter extends StatelessWidget {
  const JutyarFooter({super.key});

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      spacing: 6,
      children: [
        Text(
          'Jutyar',
          style: latText(
            size: 12,
            weight: FontWeight.w700,
            color: JColors.faint,
          ),
        ),
        Text('·', style: latText(size: 12, color: JColors.faint)),
        Text(
          scope.s.tagline,
          style: jText(scope.ku, size: 12, color: JColors.faint),
        ),
      ],
    );
  }
}

/// Small text button with a chevron, used for "Back".
class BackLink extends StatelessWidget {
  const BackLink({super.key, required this.onTap});
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(8),
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 4),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          spacing: 4,
          children: [
            const Icon(Icons.chevron_left, size: 18, color: JColors.muted),
            Text(
              scope.s.back,
              style: jText(
                scope.ku,
                size: 14,
                weight: FontWeight.w600,
                color: JColors.muted,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// Short dark pill at the bottom of the screen.
/// [long]: a message the farmer must be able to read in full (a reason).
void showToast(BuildContext context, String message, {bool long = false}) {
  final ku = AppScope.read(context).ku;
  ScaffoldMessenger.of(context)
    ..hideCurrentSnackBar()
    ..showSnackBar(
      SnackBar(
        content: Text(
          message,
          textAlign: TextAlign.center,
          style: jText(ku, size: 13.5, color: Colors.white),
        ),
        backgroundColor: JColors.ink,
        behavior: SnackBarBehavior.floating,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
        margin: const EdgeInsets.fromLTRB(40, 0, 40, 92),
        duration: Duration(milliseconds: long ? 5000 : 1800),
      ),
    );
}

class GhostButton extends StatelessWidget {
  const GhostButton({
    super.key,
    required this.label,
    required this.icon,
    this.onPressed,
  });
  final String label;
  final IconData icon;
  final VoidCallback? onPressed;

  @override
  Widget build(BuildContext context) {
    final ku = AppScope.of(context).ku;
    return SizedBox(
      height: 48,
      width: double.infinity,
      child: OutlinedButton(
        style: OutlinedButton.styleFrom(
          backgroundColor: JColors.card,
          foregroundColor: JColors.accent,
          side: const BorderSide(color: JColors.line),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(14),
          ),
        ),
        onPressed: onPressed,
        child: Row(
          mainAxisSize: MainAxisSize.min,
          spacing: 8,
          children: [
            Icon(icon, size: 18),
            Text(
              label,
              style: jText(
                ku,
                size: 15,
                weight: FontWeight.w600,
                color: JColors.ink,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
