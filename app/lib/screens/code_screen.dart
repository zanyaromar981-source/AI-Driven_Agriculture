import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../api/api.dart';
import '../app_scope.dart';
import '../config.dart';
import '../phone.dart';
import '../store/session.dart';
import '../theme.dart';
import '../widgets/common.dart';
import 'my_farms_screen.dart';

/// Step 2 of 3: the 6-digit SMS code.
class CodeScreen extends StatefulWidget {
  const CodeScreen({super.key, required this.digits, this.wait = 60});

  /// The number as typed, e.g. 07501234567.
  final String digits;

  /// Seconds before another code may be asked for (the server's retry_after_s).
  final int wait;

  @override
  State<CodeScreen> createState() => _CodeScreenState();
}

class _CodeScreenState extends State<CodeScreen> {
  final _ctrl = TextEditingController();
  final _focus = FocusNode();
  Timer? _timer;
  int _left = 0;
  bool _busy = false;

  @override
  void initState() {
    super.initState();
    _ctrl.addListener(_onChanged);
    _focus.addListener(() => setState(() {}));
    _startTimer();
  }

  @override
  void dispose() {
    _timer?.cancel();
    _ctrl.dispose();
    _focus.dispose();
    super.dispose();
  }

  void _onChanged() {
    final end = _ctrl.text.length;
    if (_ctrl.selection.baseOffset != end ||
        _ctrl.selection.extentOffset != end) {
      _ctrl.selection = TextSelection.collapsed(offset: end);
    }
    setState(() {});
  }

  void _startTimer([int? seconds]) {
    _timer?.cancel();
    _left = seconds ?? widget.wait;
    _timer = Timer.periodic(const Duration(seconds: 1), (t) {
      if (!mounted) return;
      setState(() {
        _left--;
        if (_left <= 0) t.cancel();
      });
    });
  }

  Future<void> _resend() async {
    final scope = AppScope.read(context);
    try {
      final res = await scope.api.sendOtp(
        phone: toE164(widget.digits),
        lang: scope.s.code,
      );
      if (!mounted) return;
      showToast(context, scope.s.sent);
      setState(() => _startTimer(res.retryAfterS));
    } on ApiException catch (e) {
      if (!mounted) return;
      if (e.code == 'rate_limited') {
        setState(() => _startTimer(e.retryAfterS));
      } else {
        showToast(context, scope.s.sendError(e.code));
      }
    }
  }

  Future<void> _verify() async {
    final scope = AppScope.read(context);
    setState(() => _busy = true);
    try {
      final res = await scope.api.verifyOtp(
        phone: toE164(widget.digits),
        code: _ctrl.text,
      );
      await Session(token: res.token, digits: widget.digits).save();
      if (!mounted) return;
      showToast(context, scope.s.welcome);
      await Navigator.of(context).pushAndRemoveUntil(
        MaterialPageRoute<void>(
          builder: (_) => MyFarmsScreen(digits: widget.digits),
        ),
        (_) => false,
      );
    } on ApiException catch (e) {
      if (!mounted) return;
      showToast(
        context,
        e.code == 'bad_code'
            ? scope.s.wrongCode
            : '${scope.s.error} (${e.code})',
      );
      _ctrl.clear();
      _focus.requestFocus();
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    final s = scope.s;
    final ku = scope.ku;
    final canResend = _left <= 0 && !_busy;
    return JutyarPage(
      step: 2,
      children: [
        Padding(
          padding: const EdgeInsets.only(top: 4),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            spacing: 8,
            children: [
              BackLink(onTap: () => Navigator.of(context).pop()),
              Text(
                s.h2,
                style: jText(
                  ku,
                  size: 26,
                  weight: FontWeight.w700,
                  height: 1.35,
                ),
              ),
              Row(
                spacing: 6,
                children: [
                  Text(
                    s.lead2,
                    style: jText(ku, size: 15, color: JColors.muted),
                  ),
                  Text(
                    prettyPhone(widget.digits),
                    textDirection: TextDirection.ltr,
                    style: latText(size: 15, weight: FontWeight.w700),
                  ),
                ],
              ),
            ],
          ),
        ),
        const SizedBox(height: 28),
        OtpBoxes(controller: _ctrl, focusNode: _focus),
        const SizedBox(height: 18),
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Text(s.noCode, style: jText(ku, size: 14, color: JColors.muted)),
            InkWell(
              onTap: canResend ? _resend : null,
              borderRadius: BorderRadius.circular(8),
              child: Row(
                spacing: 6,
                children: [
                  Text(
                    s.resend,
                    style: jText(
                      ku,
                      size: 14,
                      weight: FontWeight.w700,
                      color: canResend ? JColors.accent : JColors.faint,
                    ),
                  ),
                  if (_left > 0)
                    Text(
                      mmss(_left),
                      textDirection: TextDirection.ltr,
                      style: latText(
                        size: 14,
                        weight: FontWeight.w600,
                        color: JColors.muted,
                      ),
                    ),
                ],
              ),
            ),
          ],
        ),
        const SizedBox(height: 28),
        PrimaryButton(
          label: s.verify,
          loading: _busy,
          onPressed: kTestMode || _ctrl.text.length == 6 ? _verify : null,
        ),
      ],
    );
  }
}

/// Six boxes drawn over one invisible text field, so backspace and SMS
/// autofill behave like a normal field.
class OtpBoxes extends StatelessWidget {
  const OtpBoxes({
    super.key,
    required this.controller,
    required this.focusNode,
  });
  final TextEditingController controller;
  final FocusNode focusNode;

  @override
  Widget build(BuildContext context) {
    final value = controller.text;
    final focused = focusNode.hasFocus;
    return Directionality(
      textDirection: TextDirection.ltr,
      child: SizedBox(
        height: 56,
        child: Stack(
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                for (var i = 0; i < 6; i++)
                  _Box(
                    digit: i < value.length ? value[i] : '',
                    active:
                        focused &&
                        (i == value.length || (value.length == 6 && i == 5)),
                  ),
              ],
            ),
            Positioned.fill(
              child: Opacity(
                opacity: 0,
                child: TextField(
                  controller: controller,
                  focusNode: focusNode,
                  autofocus: true,
                  keyboardType: TextInputType.number,
                  inputFormatters: [
                    FilteringTextInputFormatter.digitsOnly,
                    LengthLimitingTextInputFormatter(6),
                  ],
                  autofillHints: const [AutofillHints.oneTimeCode],
                  showCursor: false,
                  enableInteractiveSelection: false,
                  style: const TextStyle(
                    color: Colors.transparent,
                    fontSize: 1,
                  ),
                  decoration: const InputDecoration.collapsed(hintText: ''),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _Box extends StatelessWidget {
  const _Box({required this.digit, required this.active});
  final String digit;
  final bool active;

  @override
  Widget build(BuildContext context) {
    return AnimatedContainer(
      duration: const Duration(milliseconds: 150),
      width: 48,
      height: 56,
      alignment: Alignment.center,
      decoration: BoxDecoration(
        color: JColors.card,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(
          color: digit.isNotEmpty || active ? JColors.accent : JColors.line,
          width: 1.5,
        ),
      ),
      child: Text(digit, style: latText(size: 24, weight: FontWeight.w700)),
    );
  }
}
