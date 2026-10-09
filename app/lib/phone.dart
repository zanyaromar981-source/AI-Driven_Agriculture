import 'package:flutter/services.dart';

String digitsOnly(String s) => s.replaceAll(RegExp(r'\D'), '');

/// Iraqi mobile: 07XX XXX XXXX (11 digits) or, since +964 is already shown,
/// 7XX XXX XXXX (10 digits).
bool isValidIraqiMobile(String d) =>
    (d.length == 11 && d.startsWith('07')) ||
    (d.length == 10 && d.startsWith('7'));

/// +9647501234567 for the API.
String toE164(String d) => '+964${d.startsWith('0') ? d.substring(1) : d}';

/// +964 750 123 4567 for the screen.
String prettyPhone(String d) {
  final n = d.startsWith('0') ? d.substring(1) : d;
  final parts = <String>[];
  if (n.isNotEmpty) parts.add(n.substring(0, n.length < 3 ? n.length : 3));
  if (n.length > 3) parts.add(n.substring(3, n.length < 6 ? n.length : 6));
  if (n.length > 6) parts.add(n.substring(6));
  return '+964 ${parts.join(' ')}';
}

/// Seconds as m:ss for a countdown: 42 -> "0:42", 60 -> "1:00".
String mmss(int seconds) =>
    '${seconds ~/ 60}:${(seconds % 60).toString().padLeft(2, '0')}';

/// Groups typed digits as 0750 123 4567 (or 750 123 4567) and keeps the cursor at the end.
class PhoneFormatter extends TextInputFormatter {
  @override
  TextEditingValue formatEditUpdate(
    TextEditingValue oldValue,
    TextEditingValue newValue,
  ) {
    var d = digitsOnly(newValue.text);
    final lead = d.startsWith('0') ? 1 : 0;
    if (d.length > 10 + lead) d = d.substring(0, 10 + lead);
    final b = StringBuffer();
    for (var i = 0; i < d.length; i++) {
      if (i == 3 + lead || i == 6 + lead) b.write(' ');
      b.write(d[i]);
    }
    final t = b.toString();
    return TextEditingValue(
      text: t,
      selection: TextSelection.collapsed(offset: t.length),
    );
  }
}
