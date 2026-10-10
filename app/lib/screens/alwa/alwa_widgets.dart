import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:flutter_map/flutter_map.dart';
import 'package:geolocator/geolocator.dart';
import 'package:latlong2/latlong.dart' show LatLng;

import '../../api/api.dart';
import '../../app_scope.dart';
import '../../l10n/strings.dart';
import '../../phone.dart';
import '../../theme.dart';
import '../../widgets/common.dart';
import '../../widgets/farm_map.dart';
import '../../widgets/header.dart';

// Shared parts of the Alwa screens (Pencil: Alwa screens). English for now,
// like Home; Sorani later.

/// The body in English, whatever the header's language (decision 2026-10-08).
Widget alwaEnglish(BuildContext context, Widget child) {
  final outer = AppScope.of(context);
  return AppScope(
    ku: false,
    s: const S(false),
    api: outer.api,
    setKu: outer.setKu,
    child: child,
  );
}

/// [children] with a [gap] between each, for a ListView.
List<Widget> gapped(List<Widget> children, double gap) => [
  for (final (i, c) in children.indexed) ...[
    if (i > 0) SizedBox(height: gap),
    c,
  ],
];

/// A screen opened from the Alwa tab: header, back to the market, body.
/// No tab bar (the Alwa tab keeps it).
class AlwaPage extends StatelessWidget {
  const AlwaPage({super.key, required this.children, this.gap = 16});
  final List<Widget> children;
  final double gap;

  @override
  Widget build(BuildContext context) {
    return Directionality(
      textDirection: TextDirection.ltr,
      child: Scaffold(
        backgroundColor: JColors.bg,
        body: SafeArea(
          child: Column(
            children: [
              const JutyarHeader(),
              alwaEnglish(context, const _BackRow()),
              Expanded(
                child: alwaEnglish(
                  context,
                  ListView(
                    padding: const EdgeInsets.fromLTRB(20, 8, 20, 24),
                    keyboardDismissBehavior:
                        ScrollViewKeyboardDismissBehavior.onDrag,
                    children: gapped(children, gap),
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
  const _BackRow();

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.fromLTRB(12, 4, 16, 0),
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
              spacing: 4,
              children: [
                const Icon(Icons.chevron_left, size: 18, color: JColors.muted),
                Text(
                  'Alwa market',
                  style: latText(
                    size: 14,
                    weight: FontWeight.w600,
                    color: JColors.muted,
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

/// Title in English with the Sorani name on the right (Title Row).
class AlwaTitle extends StatelessWidget {
  const AlwaTitle({super.key, required this.en, required this.ku, this.sub});
  final String en;
  final String ku;
  final String? sub;

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    spacing: 6,
    children: [
      Row(
        children: [
          Expanded(
            child: Text(
              en,
              style: latText(
                size: 24,
                weight: FontWeight.w700,
              ).copyWith(letterSpacing: -0.4),
            ),
          ),
          Text(
            ku,
            textDirection: TextDirection.rtl,
            style: jText(
              true,
              size: 15,
              weight: FontWeight.w600,
              color: JColors.muted,
            ),
          ),
        ],
      ),
      if (sub != null)
        Text(
          sub!,
          style: latText(
            size: 15,
            weight: FontWeight.w400,
            color: JColors.muted,
            height: 1.45,
          ),
        ),
    ],
  );
}

/// Small grey capitals over a section ("TODAY AT THE ALWA").
class SectionHead extends StatelessWidget {
  const SectionHead(this.label, {super.key, this.trailing});
  final String label;
  final String? trailing;

  @override
  Widget build(BuildContext context) => Row(
    children: [
      Expanded(
        child: Text(
          label.toUpperCase(),
          style: latText(
            size: 12,
            weight: FontWeight.w700,
            color: JColors.muted,
          ).copyWith(letterSpacing: 0.6),
        ),
      ),
      if (trailing != null)
        Text(
          trailing!,
          style: latText(
            size: 12,
            weight: FontWeight.w600,
            color: JColors.muted,
          ),
        ),
    ],
  );
}

/// Label over a form field ("How much, in kg").
class FieldLabel extends StatelessWidget {
  const FieldLabel(this.label, {super.key, this.trailing});
  final String label;
  final String? trailing;

  @override
  Widget build(BuildContext context) => Row(
    children: [
      Expanded(
        child: Text(label, style: latText(size: 13, weight: FontWeight.w700)),
      ),
      if (trailing != null)
        Text(
          trailing!,
          style: latText(
            size: 12,
            weight: FontWeight.w500,
            color: JColors.muted,
          ),
        ),
    ],
  );
}

/// Grey line with a small icon under a field or card (Hint, Board Note).
class HintLine extends StatelessWidget {
  const HintLine(
    this.text, {
    super.key,
    this.icon = Icons.info_outline_rounded,
    this.color = JColors.muted,
  });
  final String text;
  final IconData icon;
  final Color color;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.symmetric(horizontal: 2),
    child: Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      spacing: 6,
      children: [
        Padding(
          padding: const EdgeInsets.only(top: 1.5),
          child: Icon(icon, size: 14, color: color),
        ),
        Expanded(
          child: Text(
            text,
            style: latText(
              size: 12,
              weight: FontWeight.w500,
              color: color,
              height: 1.4,
            ),
          ),
        ),
      ],
    ),
  );
}

/// The crop's picture on a rounded tile (Crop Tile).
class CropTile extends StatelessWidget {
  const CropTile({
    super.key,
    required this.crop,
    this.size = 48,
    this.radius = 12,
    this.color = JColors.bg,
  });
  final String crop;
  final double size;
  final double radius;
  final Color color;

  @override
  Widget build(BuildContext context) => Container(
    width: size,
    height: size,
    alignment: Alignment.center,
    decoration: BoxDecoration(
      color: color,
      borderRadius: BorderRadius.circular(radius),
    ),
    child: Text(alwaCrop(crop).emoji, style: TextStyle(fontSize: size / 2)),
  );
}

/// open / sold / closed (Status Pill).
class StatusPill extends StatelessWidget {
  const StatusPill(this.status, {super.key});
  final String status;

  @override
  Widget build(BuildContext context) {
    final (bg, fg) = switch (status) {
      'open' => (JColors.accentSoft, JColors.accent),
      'sold' => (JColors.levelNormalSoft, JColors.levelNormal),
      _ => (JColors.levelNoneSoft, JColors.muted),
    };
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
      decoration: BoxDecoration(
        color: bg,
        borderRadius: BorderRadius.circular(999),
      ),
      child: Text(
        status,
        style: latText(size: 11, weight: FontWeight.w700, color: fg),
      ),
    );
  }
}

/// White rounded card.
class AlwaCard extends StatelessWidget {
  const AlwaCard({
    super.key,
    required this.child,
    this.padding = const EdgeInsets.all(16),
    this.radius = 14,
    this.border,
    this.borderWidth = 1,
  });
  final Widget child;
  final EdgeInsetsGeometry padding;
  final double radius;
  final Color? border;
  final double borderWidth;

  @override
  Widget build(BuildContext context) => Container(
    width: double.infinity,
    padding: padding,
    decoration: BoxDecoration(
      color: JColors.card,
      borderRadius: BorderRadius.circular(radius),
      border: border == null
          ? null
          : Border.all(color: border!, width: borderWidth),
    ),
    child: child,
  );
}

/// Green button with the icon first (Button/Primary).
class AlwaButton extends StatelessWidget {
  const AlwaButton({
    super.key,
    required this.label,
    required this.icon,
    this.onPressed,
    this.height = 56,
    this.loading = false,
  });
  final String label;
  final IconData icon;
  final VoidCallback? onPressed;
  final double height;
  final bool loading;

  @override
  Widget build(BuildContext context) => Container(
    height: height,
    width: double.infinity,
    decoration: BoxDecoration(
      borderRadius: BorderRadius.circular(14),
      boxShadow: onPressed == null
          ? null
          : const [
              BoxShadow(
                color: Color(0x331E7A5A),
                offset: Offset(0, 6),
                blurRadius: 16,
              ),
            ],
    ),
    child: FilledButton(
      style: FilledButton.styleFrom(
        backgroundColor: JColors.accent,
        disabledBackgroundColor: JColors.accent.withValues(alpha: 0.5),
        foregroundColor: Colors.white,
        disabledForegroundColor: Colors.white,
        elevation: 0,
        padding: const EdgeInsets.symmetric(horizontal: 12),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(14)),
      ),
      onPressed: loading ? null : onPressed,
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
                Icon(icon, size: height > 50 ? 20 : 18),
                Flexible(
                  child: Text(
                    label,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: latText(
                      size: height > 50 ? 16 : 15,
                      weight: FontWeight.w700,
                      color: Colors.white,
                    ),
                  ),
                ),
              ],
            ),
    ),
  );
}

/// White button with a grey edge (Button/Ghost); the label shrinks before
/// it overflows.
class AlwaGhostButton extends StatelessWidget {
  const AlwaGhostButton({
    super.key,
    required this.label,
    required this.icon,
    this.onPressed,
    this.filled = true,
  });
  final String label;
  final IconData icon;
  final VoidCallback? onPressed;

  /// White fill (My listings); off for "Show more", as designed.
  final bool filled;

  @override
  Widget build(BuildContext context) => SizedBox(
    height: 48,
    width: double.infinity,
    child: OutlinedButton(
      style: OutlinedButton.styleFrom(
        backgroundColor: filled ? JColors.card : Colors.transparent,
        foregroundColor: JColors.ink,
        padding: const EdgeInsets.symmetric(horizontal: 12),
        side: const BorderSide(color: JColors.line, width: 1.5),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(14)),
      ),
      onPressed: onPressed,
      child: Row(
        mainAxisSize: MainAxisSize.min,
        spacing: 8,
        children: [
          Icon(icon, size: 18),
          Flexible(
            child: Text(
              label,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: latText(size: 15, weight: FontWeight.w600),
            ),
          ),
        ],
      ),
    ),
  );
}

/// A coloured note with an action (Banner: location off, too many listings,
/// no internet).
class AlwaBanner extends StatelessWidget {
  const AlwaBanner({
    super.key,
    required this.icon,
    required this.title,
    required this.body,
    this.action,
    this.onAction,
    this.alarm = false,
  });
  final IconData icon;
  final String title;
  final String body;
  final String? action;
  final VoidCallback? onAction;

  /// Red (a refusal) instead of yellow (a problem the farmer can fix).
  final bool alarm;

  @override
  Widget build(BuildContext context) => Container(
    width: double.infinity,
    padding: const EdgeInsets.all(14),
    decoration: BoxDecoration(
      color: alarm ? JColors.levelAlarmSoft : JColors.levelWatchSoft,
      borderRadius: BorderRadius.circular(14),
    ),
    child: Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      spacing: 12,
      children: [
        Container(
          width: 28,
          height: 28,
          decoration: BoxDecoration(
            color: JColors.card,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Icon(
            icon,
            size: 16,
            color: alarm ? JColors.levelAlarm : JColors.gold,
          ),
        ),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            spacing: 4,
            children: [
              Text(
                title,
                style: latText(size: 14, weight: FontWeight.w700, height: 1.3),
              ),
              Text(
                body,
                style: latText(size: 13, weight: FontWeight.w500, height: 1.4),
              ),
              if (action != null)
                InkWell(
                  onTap: onAction,
                  borderRadius: BorderRadius.circular(6),
                  child: Padding(
                    padding: const EdgeInsets.only(top: 4, bottom: 2),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      spacing: 4,
                      children: [
                        Text(
                          action!,
                          style: latText(
                            size: 13,
                            weight: FontWeight.w700,
                            color: JColors.accent,
                          ),
                        ),
                        const Icon(
                          Icons.chevron_right_rounded,
                          size: 16,
                          color: JColors.accent,
                        ),
                      ],
                    ),
                  ),
                ),
            ],
          ),
        ),
      ],
    ),
  );
}

/// Dashed box for "nothing here" (Empty/No prices, Empty/No listings).
class DashedEmpty extends StatelessWidget {
  const DashedEmpty({super.key, required this.icon, required this.text});
  final IconData icon;
  final String text;

  @override
  Widget build(BuildContext context) => CustomPaint(
    painter: _DashedBorder(),
    child: Container(
      constraints: const BoxConstraints(minHeight: 56),
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      decoration: BoxDecoration(
        color: Colors.white.withValues(alpha: 0.4),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        spacing: 10,
        children: [
          Icon(icon, size: 18, color: JColors.muted),
          Flexible(
            child: Text(
              text,
              style: latText(
                size: 13,
                weight: FontWeight.w500,
                color: JColors.muted,
              ),
            ),
          ),
        ],
      ),
    ),
  );
}

class _DashedBorder extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = const Color(0x805E6E64)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5;
    final path = Path()
      ..addRRect(
        RRect.fromRectAndRadius(
          Offset.zero & size,
          const Radius.circular(12),
        ).deflate(0.75),
      );
    for (final m in path.computeMetrics()) {
      for (var d = 0.0; d < m.length; d += 10) {
        canvas.drawPath(m.extractPath(d, d + 6), paint);
      }
    }
  }

  @override
  bool shouldRepaint(_DashedBorder old) => false;
}

/// While something loads: a quiet card with a spinner.
class AlwaLoading extends StatelessWidget {
  const AlwaLoading(this.text, {super.key});
  final String text;

  @override
  Widget build(BuildContext context) => AlwaCard(
    child: Row(
      spacing: 12,
      children: [
        const SizedBox(
          width: 18,
          height: 18,
          child: CircularProgressIndicator(
            strokeWidth: 2.2,
            color: JColors.accent,
          ),
        ),
        Expanded(
          child: Text(
            text,
            style: latText(
              size: 13,
              weight: FontWeight.w600,
              color: JColors.muted,
            ),
          ),
        ),
      ],
    ),
  );
}

/// The banner for a failed load: no internet, or the server failed.
Widget loadProblem(Object error, VoidCallback retry, {required String what}) {
  final offline = error is ApiException && error.isOffline;
  return AlwaBanner(
    icon: offline ? Icons.wifi_off_rounded : Icons.error_outline_rounded,
    title: offline ? 'No internet' : 'Could not load $what',
    body: offline
        ? 'The alwa needs internet. It shows again when you are back online.'
        : 'The server did not answer properly. Try again in a moment.',
    action: 'Try again',
    onAction: retry,
  );
}

// ---- Numbers, days and phones ----

/// "12 trays", "4,000 kg": an amount in the product's unit. English, like
/// the rest of the Alwa screens ([alwaEnglish]).
String fmtQty(num n, String unit) =>
    '${fmtInt(n)} ${const S(false).unitWord(unit, n)}';

/// "IQD/tray", "IQD/kg": what a price is for.
String iqdPer(String unit) => 'IQD/${const S(false).unitWord(unit, 1)}';

/// Reads the server's product list once it answers, so a product the app
/// was not built with still gets its name. Quiet when it fails.
Future<void> refreshProducts(Api api) async {
  try {
    final p = await api.products();
    if (p.isNotEmpty) alwaProducts = p;
  } on ApiException {
    // The built-in list stays.
  }
}

/// A round chip to filter or choose with (crop groups, per day or hour).
class AlwaChip extends StatelessWidget {
  const AlwaChip({
    super.key,
    required this.label,
    required this.selected,
    required this.onTap,
    this.emoji,
  });
  final String label;
  final String? emoji;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) => InkWell(
    onTap: onTap,
    borderRadius: BorderRadius.circular(999),
    child: Container(
      height: 34,
      padding: const EdgeInsets.symmetric(horizontal: 12),
      decoration: BoxDecoration(
        color: selected ? JColors.accentSoft : JColors.card,
        borderRadius: BorderRadius.circular(999),
        border: Border.all(
          color: selected ? JColors.accent : JColors.line,
          width: selected ? 1.5 : 1,
        ),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        spacing: 6,
        children: [
          if (emoji != null) Text(emoji!, style: const TextStyle(fontSize: 13)),
          Text(
            label,
            style: latText(
              size: 13,
              weight: selected ? FontWeight.w700 : FontWeight.w600,
              color: selected ? JColors.accent : JColors.ink,
            ),
          ),
        ],
      ),
    ),
  );
}

/// The five product groups as chips in one row that scrolls sideways;
/// [all] adds "All" first (null).
class GroupChips extends StatelessWidget {
  const GroupChips({
    super.key,
    required this.selected,
    required this.onPick,
    this.all = false,
  });
  final String? selected;
  final ValueChanged<String?> onPick;
  final bool all;

  @override
  Widget build(BuildContext context) {
    final s = AppScope.of(context).s;
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      clipBehavior: Clip.none,
      child: Row(
        spacing: 8,
        children: [
          if (all)
            AlwaChip(
              label: s.allGroups,
              selected: selected == null,
              onTap: () => onPick(null),
            ),
          for (final g in kProductGroups)
            AlwaChip(
              emoji: kGroupEmoji[g],
              label: s.groupName(g),
              selected: selected == g,
              onTap: () => onPick(g),
            ),
        ],
      ),
    );
  }
}

/// 4000 -> "4,000".
String fmtInt(num n) {
  final s = n.round().abs().toString();
  final b = StringBuffer(n < 0 ? '-' : '');
  for (var i = 0; i < s.length; i++) {
    if (i > 0 && (s.length - i) % 3 == 0) b.write(',');
    b.write(s[i]);
  }
  return b.toString();
}

const _wd = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];
const _mo = [
  'Jan',
  'Feb',
  'Mar',
  'Apr',
  'May',
  'Jun',
  'Jul',
  'Aug',
  'Sep',
  'Oct',
  'Nov',
  'Dec',
];

/// "Wed 7 Oct".
String fmtDay(DateTime t) =>
    '${_wd[t.weekday - 1]} ${t.day} ${_mo[t.month - 1]}';

/// "Wed 7 Oct, 08:10".
String fmtDayTime(DateTime t) =>
    '${fmtDay(t)}, ${t.hour.toString().padLeft(2, '0')}:'
    '${t.minute.toString().padLeft(2, '0')}';

DateTime _date(DateTime t) => DateTime(t.year, t.month, t.day);

bool isToday(DateTime t, [DateTime? now]) =>
    _date(t) == _date(now ?? DateTime.now());

/// How long a listing stays open: "12 days left", or "Closes in 18 h" when
/// it is the last day ([soon] = true, shown in gold).
({String text, bool soon}) timeLeft(DateTime closes, [DateTime? now]) {
  final n = now ?? DateTime.now();
  final left = closes.difference(n);
  if (left.isNegative) return (text: 'Closed', soon: true);
  if (left.inMinutes < 60) {
    return (text: 'Closes in ${left.inMinutes.clamp(1, 59)} min', soon: true);
  }
  if (left.inHours < 24) {
    // Round up: 17 h 59 min left reads "18 h".
    return (text: 'Closes in ${(left.inMinutes / 60).ceil()} h', soon: true);
  }
  final days = _date(closes).difference(_date(n)).inDays;
  return (text: days == 1 ? '1 day left' : '$days days left', soon: false);
}

/// "2 km", or metres under 1 km ("800 m").
String fmtKm(double km) =>
    km < 1 ? '${((km * 1000) / 10).round() * 10} m' : '${km.round()} km';

/// "+9647704128890" -> "+964 770 412 8890"; anything else as it came.
String fmtPhone(String raw) {
  var d = digitsOnly(raw);
  if (d.startsWith('964')) d = d.substring(3);
  return isValidIraqiMobile(d) ? prettyPhone(d) : raw;
}

/// Opens the phone app with the seller's number typed in, ready to call
/// (user, 2026-10-09). If no phone app opens, the number is copied instead.
Future<void> callPhone(BuildContext context, String phone) async {
  final number = digitsOnly(phone).isEmpty ? phone : '+${digitsOnly(phone)}';
  var opened = false;
  try {
    opened = await launchUrl(Uri(scheme: 'tel', path: number));
  } catch (_) {
    opened = false;
  }
  if (opened || !context.mounted) return;
  await Clipboard.setData(ClipboardData(text: number));
  if (!context.mounted) return;
  showToast(
    context,
    'Number copied: ${fmtPhone(phone)}. Paste it in your phone app to call.',
    long: true,
  );
}

// ---- The phone's place (GPS), shared by Home and Sell ----

enum GpsIssue { serviceOff, denied, deniedForever, noFix }

/// One reading of the phone's GPS.
class GpsFix {
  const GpsFix(this.lat, this.lon, this.accM);
  final double lat;
  final double lon;
  final double accM;
  LatLng get ll => LatLng(lat, lon);
}

class GpsProblem implements Exception {
  const GpsProblem(this.issue);
  final GpsIssue issue;
}

/// The phone's place, with the same package and checks as Add farm.
class AlwaGps {
  /// Tests replace this (widget tests have no GPS).
  static Future<GpsFix> Function({bool ask}) locate = _locate;

  /// The last fix this run, so a screen opened next starts with it.
  static GpsFix? last;

  static Future<GpsFix> _locate({bool ask = true}) async {
    try {
      if (!await Geolocator.isLocationServiceEnabled()) {
        throw const GpsProblem(GpsIssue.serviceOff);
      }
      var perm = await Geolocator.checkPermission();
      if (perm == LocationPermission.denied && ask) {
        perm = await Geolocator.requestPermission();
      }
      if (perm == LocationPermission.deniedForever) {
        throw const GpsProblem(GpsIssue.deniedForever);
      }
      if (perm == LocationPermission.denied) {
        throw const GpsProblem(GpsIssue.denied);
      }
      final p = await Geolocator.getCurrentPosition(
        locationSettings: const LocationSettings(
          accuracy: LocationAccuracy.high,
          timeLimit: Duration(seconds: 20),
        ),
      );
      return last = GpsFix(p.latitude, p.longitude, p.accuracy);
    } on GpsProblem {
      rethrow;
    } catch (_) {
      throw const GpsProblem(GpsIssue.noFix);
    }
  }

  /// Ask again, or open the right settings page.
  static Future<void> fix(GpsIssue issue) async {
    try {
      switch (issue) {
        case GpsIssue.serviceOff:
          await Geolocator.openLocationSettings();
        case GpsIssue.deniedForever:
          await Geolocator.openAppSettings();
        case GpsIssue.denied:
        case GpsIssue.noFix:
          break;
      }
    } catch (_) {
      // No settings page on this platform (tests): nothing to open.
    }
  }
}

/// "Location is off" (design: State/location_off).
class LocationOffBanner extends StatelessWidget {
  const LocationOffBanner({
    super.key,
    required this.issue,
    required this.onFix,
  });
  final GpsIssue issue;
  final VoidCallback onFix;

  @override
  Widget build(BuildContext context) {
    final noFix = issue == GpsIssue.noFix;
    return AlwaBanner(
      icon: Icons.location_off_outlined,
      title: noFix ? 'No GPS fix yet' : 'Location is off',
      body: noFix
          ? 'The phone could not find where you are. Go outside, away from walls, and try again.'
          : 'Jutyar needs it to show crops near you and to put your crop on the map.',
      action: noFix ? 'Try again' : 'Turn on location',
      onAction: onFix,
    );
  }
}

// ---- The small map (Mini Map) ----

/// A small map that does not move: the crop's pin, and "you" when the
/// phone's place is known.
class AlwaMiniMap extends StatelessWidget {
  const AlwaMiniMap({
    super.key,
    this.crop,
    this.item,
    this.you,
    this.accM,
    this.height = 150,
  });

  /// For tests: no map pictures (widget tests have no internet).
  static bool showTiles = true;

  final String? crop;
  final LatLng? item;
  final LatLng? you;

  /// GPS accuracy, shown as a chip (Sell).
  final double? accM;
  final double height;

  @override
  Widget build(BuildContext context) {
    final pts = [?item, ?you];
    final Widget map;
    if (pts.isEmpty) {
      map = const ColoredBox(color: Color(0xFF717A50));
    } else {
      map = StyledMap(
        builder: (context, style) => FlutterMap(
          key: ValueKey(Object.hashAll([...pts, style])),
          options: MapOptions(
            initialCenter: pts.first,
            initialZoom: 15,
            initialCameraFit: pts.length > 1
                ? CameraFit.coordinates(
                    coordinates: pts,
                    padding: const EdgeInsets.fromLTRB(40, 40, 60, 40),
                    maxZoom: 16,
                  )
                : null,
            backgroundColor: const Color(0xFF717A50),
            interactionOptions: const InteractionOptions(
              flags: InteractiveFlag.none,
            ),
          ),
          children: [
            if (showTiles) ...baseLayers(style, false),
            MarkerLayer(
              markers: [
                if (you != null)
                  Marker(
                    point: you!,
                    width: 120,
                    height: 56,
                    child: _YouDot(label: item != null),
                  ),
                if (item != null)
                  Marker(
                    point: item!,
                    width: 44,
                    height: 44,
                    child: _Pin(crop: crop ?? ''),
                  ),
              ],
            ),
            if (showTiles) mapAttribution(style),
          ],
        ),
      );
    }
    return ClipRRect(
      borderRadius: BorderRadius.circular(14),
      child: SizedBox(
        height: height,
        width: double.infinity,
        child: Stack(
          children: [
            Positioned.fill(child: IgnorePointer(child: map)),
            if (accM != null)
              Positioned(
                left: 12,
                top: 12,
                child: Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 12,
                    vertical: 6,
                  ),
                  decoration: BoxDecoration(
                    color: Colors.white.withValues(alpha: 0.92),
                    borderRadius: BorderRadius.circular(999),
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    spacing: 6,
                    children: [
                      Container(
                        width: 8,
                        height: 8,
                        decoration: BoxDecoration(
                          color: accM! <= 10
                              ? const Color(0xFF2BB673)
                              : accM! <= 25
                              ? JColors.levelWatch
                              : JColors.levelAlarm,
                          shape: BoxShape.circle,
                        ),
                      ),
                      Text(
                        'GPS ±${accM!.round()} m',
                        style: latText(size: 13, weight: FontWeight.w700),
                      ),
                    ],
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }
}

class _Pin extends StatelessWidget {
  const _Pin({required this.crop});
  final String crop;

  @override
  Widget build(BuildContext context) => Center(
    child: Container(
      width: 40,
      height: 40,
      alignment: Alignment.center,
      decoration: BoxDecoration(
        color: JColors.accent,
        shape: BoxShape.circle,
        border: Border.all(color: Colors.white, width: 2.5),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withValues(alpha: 0.25),
            blurRadius: 6,
            offset: const Offset(0, 2),
          ),
        ],
      ),
      child: Text(alwaCrop(crop).emoji, style: const TextStyle(fontSize: 19)),
    ),
  );
}

/// The blue "you" dot with its halo; [label] adds the "you" chip.
class _YouDot extends StatelessWidget {
  const _YouDot({required this.label});
  final bool label;

  @override
  Widget build(BuildContext context) => Stack(
    clipBehavior: Clip.none,
    children: [
      Center(
        child: Container(
          width: 56,
          height: 56,
          decoration: const BoxDecoration(
            color: Color(0x2E4A9DFF),
            shape: BoxShape.circle,
          ),
          alignment: Alignment.center,
          child: Container(
            width: 34,
            height: 34,
            decoration: BoxDecoration(
              color: const Color(0x4D4A9DFF),
              shape: BoxShape.circle,
              border: Border.all(color: const Color(0xFF9CCBFF)),
            ),
            alignment: Alignment.center,
            child: Container(
              width: 16,
              height: 16,
              decoration: BoxDecoration(
                color: const Color(0xFF1F7BEF),
                shape: BoxShape.circle,
                border: Border.all(color: Colors.white, width: 2),
              ),
            ),
          ),
        ),
      ),
      if (label)
        Positioned(
          left: 72,
          top: 17,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
            decoration: BoxDecoration(
              color: Colors.white.withValues(alpha: 0.92),
              borderRadius: BorderRadius.circular(999),
            ),
            child: Text(
              'you',
              style: latText(size: 11, weight: FontWeight.w700),
            ),
          ),
        ),
    ],
  );
}
