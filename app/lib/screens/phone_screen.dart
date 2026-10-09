import 'package:flutter/material.dart';

import '../api/api.dart';
import '../app_scope.dart';
import '../phone.dart';
import '../theme.dart';
import '../widgets/common.dart';
import 'code_screen.dart';

/// Step 1 of 3: phone number.
class PhoneScreen extends StatefulWidget {
  const PhoneScreen({super.key});

  @override
  State<PhoneScreen> createState() => _PhoneScreenState();
}

class _PhoneScreenState extends State<PhoneScreen> {
  final _ctrl = TextEditingController();
  bool _busy = false;

  String get _digits => digitsOnly(_ctrl.text);

  @override
  void initState() {
    super.initState();
    _ctrl.addListener(() => setState(() {}));
  }

  @override
  void dispose() {
    _ctrl.dispose();
    super.dispose();
  }

  Future<void> _send() async {
    final scope = AppScope.read(context);
    final digits = _digits;
    setState(() => _busy = true);
    try {
      await scope.api.sendOtp(phone: toE164(digits), lang: scope.s.code);
      if (!mounted) return;
      showToast(context, scope.s.sent);
      await Navigator.of(context).push(
        MaterialPageRoute<void>(builder: (_) => CodeScreen(digits: digits)),
      );
    } on ApiException catch (e) {
      if (!mounted) return;
      showToast(context, '${scope.s.error} (${e.code})');
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    final s = scope.s;
    final ku = scope.ku;
    return JutyarPage(
      step: 1,
      children: [
        Padding(
          padding: const EdgeInsets.only(top: 4),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            spacing: 10,
            children: [
              Text(
                s.h1,
                style: jText(
                  ku,
                  size: 26,
                  weight: FontWeight.w700,
                  height: 1.35,
                ),
              ),
              Text(
                s.lead1,
                style: jText(ku, size: 15, color: JColors.muted, height: 1.65),
              ),
            ],
          ),
        ),
        const SizedBox(height: 28),
        PhoneField(controller: _ctrl),
        const SizedBox(height: 28),
        PrimaryButton(
          label: s.send,
          loading: _busy,
          // Always checked, also in test mode: the server refuses any other
          // number (422 invalid) now that it sends real codes.
          onPressed: isValidIraqiMobile(_digits) ? _send : null,
        ),
        const SizedBox(height: 16),
        InfoNote(title: s.noteTitle, body: s.noteBody),
      ],
    );
  }
}

class PhoneField extends StatefulWidget {
  const PhoneField({super.key, required this.controller});
  final TextEditingController controller;

  @override
  State<PhoneField> createState() => _PhoneFieldState();
}

class _PhoneFieldState extends State<PhoneField> {
  final _focus = FocusNode();

  @override
  void initState() {
    super.initState();
    _focus.addListener(() => setState(() {}));
  }

  @override
  void dispose() {
    _focus.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    final s = scope.s;
    final ku = scope.ku;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      spacing: 8,
      children: [
        Text(s.tel, style: jText(ku, size: 14, weight: FontWeight.w600)),
        Directionality(
          textDirection: TextDirection.ltr,
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 150),
            height: 56,
            padding: const EdgeInsets.symmetric(horizontal: 18),
            decoration: BoxDecoration(
              color: JColors.card,
              borderRadius: BorderRadius.circular(14),
              border: Border.all(
                color: _focus.hasFocus ? JColors.accent : JColors.line,
                width: 1.5,
              ),
            ),
            child: Row(
              spacing: 14,
              children: [
                Text('+964', style: latText(size: 17, weight: FontWeight.w700)),
                Container(width: 1.5, height: 24, color: JColors.line),
                Expanded(
                  child: TextField(
                    controller: widget.controller,
                    focusNode: _focus,
                    keyboardType: TextInputType.phone,
                    inputFormatters: [PhoneFormatter()],
                    autofillHints: const [
                      AutofillHints.telephoneNumberNational,
                    ],
                    style: latText(size: 17, weight: FontWeight.w700),
                    decoration: InputDecoration.collapsed(
                      hintText: '07XX XXX XXXX',
                      hintStyle: latText(size: 17, color: JColors.placeholder),
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
        Text.rich(
          TextSpan(
            style: jText(ku, size: 13, color: JColors.muted),
            children: [
              TextSpan(text: '${s.hintStart} '),
              TextSpan(
                text: 'SMS',
                style: latText(
                  size: 12,
                  weight: FontWeight.w600,
                  color: JColors.muted,
                ),
              ),
              if (s.hintEnd.isNotEmpty) TextSpan(text: ' ${s.hintEnd}'),
            ],
          ),
        ),
      ],
    );
  }
}
