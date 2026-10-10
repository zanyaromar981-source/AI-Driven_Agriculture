import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../api/api.dart';
import '../../app_scope.dart';
import '../../l10n/strings.dart';
import '../../theme.dart';
import '../../widgets/common.dart';
import 'alwa_my_listings_screen.dart';
import 'alwa_widgets.dart';

/// Put a crop on sale (Pencil: Screen/Alwa Sell): crop, kg, price per kg,
/// the phone's GPS place, 1 to 14 days, and the sign-in phone. Answers the
/// new listing to the screen that opened it.
class AlwaSellScreen extends StatefulWidget {
  const AlwaSellScreen({super.key, this.board});

  /// Today's prices, for the hint under the price field.
  final AlwaPriceBoard? board;

  @override
  State<AlwaSellScreen> createState() => _AlwaSellScreenState();
}

class _AlwaSellScreenState extends State<AlwaSellScreen>
    with WidgetsBindingObserver {
  String _group = 'crops';
  String? _crop;
  bool _allCrops = false;

  /// The unit of the picked product; kg until one is picked.
  String get _unit => _crop == null ? 'kg' : alwaProduct(_crop!)?.unit ?? 'kg';

  /// "trays" and "tray", for the labels.
  String get _many => const S(false).unitWord(_unit);
  String get _one => const S(false).unitWord(_unit, 1);

  /// More than the server takes for this unit (FRONTEND.md 5).
  bool get _tooMuch => (_num(_kg) ?? 0) > kUnitMax[_unit]!;
  final _kg = TextEditingController();
  final _price = TextEditingController();
  final _priceFocus = FocusNode();
  int _days = kAlwaMaxDays;

  GpsFix? _fix = AlwaGps.last;
  GpsIssue? _gpsIssue;
  bool _locating = false;

  String? _phone;
  int? _openCount;

  bool _sending = false;
  bool _tried = false;
  bool _tooMany = false;

  /// Kept for retries of the same form; a change to the form makes a new one.
  String? _key;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    _locate(ask: true);
    _loadMine();
    refreshProducts(AppScope.read(context).api).then((_) {
      if (mounted) setState(() {});
    });
    _priceFocus.addListener(() => setState(() {}));
    for (final c in [_kg, _price]) {
      c.addListener(() {
        _key = null;
        if (_tried) setState(() {});
      });
    }
  }

  /// Back from the phone's settings: try the GPS again.
  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed && _gpsIssue != null) _locate();
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    _kg.dispose();
    _price.dispose();
    _priceFocus.dispose();
    super.dispose();
  }

  Future<void> _locate({bool ask = false}) async {
    setState(() => _locating = true);
    try {
      final f = await AlwaGps.locate(ask: ask);
      if (!mounted) return;
      setState(() {
        _fix = f;
        _gpsIssue = null;
        _key = null;
      });
    } on GpsProblem catch (e) {
      if (mounted) setState(() => _gpsIssue = e.issue);
    } finally {
      if (mounted) setState(() => _locating = false);
    }
  }

  Future<void> _fixGps() async {
    final issue = _gpsIssue;
    if (issue != null) await AlwaGps.fix(issue);
    await _locate(ask: true);
  }

  /// The sign-in phone and how many listings are open (footnote).
  Future<void> _loadMine() async {
    final api = AppScope.read(context).api;
    try {
      final me = await api.getMe();
      if (mounted && me.phone.isNotEmpty) setState(() => _phone = me.phone);
    } on ApiException {
      // Shown as "the phone you signed in with".
    }
    try {
      final mine = await api.myAlwaListings();
      if (mounted) {
        setState(() => _openCount = mine.where((l) => l.isOpen).length);
      }
    } on ApiException {
      // The footnote leaves the count out.
    }
  }

  double? _num(TextEditingController c) {
    final d = c.text.replaceAll(RegExp(r'\D'), '');
    final v = double.tryParse(d);
    return v == null || v <= 0 ? null : v;
  }

  Future<void> _submit() async {
    setState(() {
      _tried = true;
      _tooMany = false;
    });
    final kg = _num(_kg);
    final price = _num(_price);
    if (_crop == null || kg == null || price == null) {
      showToast(context, 'Fill in what you sell, how much and your price.');
      return;
    }
    if (_tooMuch) {
      showToast(context, 'At most ${fmtQty(kUnitMax[_unit]!, _unit)}.');
      return;
    }
    final fix = _fix;
    if (fix == null) {
      showToast(
        context,
        'Your place is needed. Turn on location and try again.',
      );
      _locate(ask: true);
      return;
    }
    _key ??= 'alwa-${DateTime.now().microsecondsSinceEpoch}';
    setState(() => _sending = true);
    try {
      final made = await AppScope.read(context).api.createAlwaListing(
        NewAlwaListing(
          product: _crop!,
          quantity: kg,
          priceIqd: price,
          lat: fix.lat,
          lon: fix.lon,
          days: _days,
          sellerPhone: _phone,
        ),
        idempotencyKey: _key,
      );
      if (mounted) Navigator.of(context).pop(made);
    } on ApiException catch (e) {
      if (!mounted) return;
      if (e.code == 'too_many_listings') {
        setState(() => _tooMany = true);
      } else {
        showToast(
          context,
          e.isOffline
              ? 'No internet. Nothing was put on sale. Try again when you are online.'
              : e.status == 422 || e.status == 400
              ? 'The server did not take these numbers. Check them and try again.'
              : 'Could not put it on sale. Try again in a moment.',
          long: true,
        );
      }
    } finally {
      if (mounted) setState(() => _sending = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final crop = _crop;
    final today = crop == null ? null : widget.board?.priceOf(crop);
    final closes = DateTime.now().add(Duration(days: _days));
    return AlwaPage(
      gap: 18,
      children: [
        const AlwaTitle(
          en: 'Sell a crop',
          ku: 'بەرهەمەکەت بفرۆشە',
          sub:
              'Buyers near you see your crop, the amount and your price, and call you.',
        ),
        // Crop.
        Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          spacing: 8,
          children: [
            const FieldLabel('What do you sell'),
            GroupChips(
              selected: _group,
              onPick: (g) => setState(() {
                if (g == null || g == _group) return;
                _group = g;
                _crop = null;
                _key = null;
              }),
            ),
            _CropPicker(
              // Crops keep the app's order; the other groups the server's.
              items: [
                if (_group == 'crops') ...kAlwaCrops,
                for (final p in alwaProducts)
                  if (p.group == _group &&
                      !(_group == 'crops' &&
                          kAlwaCrops.any((c) => c.code == p.code)))
                    alwaCrop(p.code),
              ],
              selected: crop,
              all:
                  _allCrops ||
                  kAlwaCrops.indexWhere((c) => c.code == crop) > 10,
              onAll: () => setState(() => _allCrops = true),
              onPick: (c) => setState(() {
                _crop = c;
                _key = null;
              }),
            ),
            if (_tried && crop == null)
              _Error(_group == 'crops' ? 'Pick a crop.' : 'Pick a product.'),
          ],
        ),
        // Quantity.
        Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          spacing: 8,
          children: [
            FieldLabel('How much, in $_many'),
            _NumberField(
              key: const ValueKey('alwa-kg'),
              controller: _kg,
              unit: _many,
            ),
            if (_tried && _num(_kg) == null)
              _Error('Type how many $_many you sell.')
            else if (_tooMuch)
              _Error('At most ${fmtQty(kUnitMax[_unit]!, _unit)}.'),
          ],
        ),
        // Asking price.
        Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          spacing: 8,
          children: [
            FieldLabel('Asking price per $_one'),
            _NumberField(
              key: const ValueKey('alwa-price'),
              controller: _price,
              focusNode: _priceFocus,
              unit: iqdPer(_unit),
            ),
            if (_tried && _num(_price) == null)
              _Error('Type your price per $_one.'),
            if (today != null)
              HintLine(
                'Alwa today: ${alwaCrop(crop!).en.toLowerCase()} '
                '${fmtInt(today.priceIqdPerKg)} ${iqdPer(_unit)}',
              ),
          ],
        ),
        // Location.
        Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          spacing: 8,
          children: [
            const FieldLabel('Location'),
            if (_fix == null && _gpsIssue != null && !_locating)
              LocationOffBanner(issue: _gpsIssue!, onFix: _fixGps)
            else
              AlwaCard(
                padding: const EdgeInsets.all(12),
                child: Column(
                  spacing: 12,
                  children: [
                    AlwaMiniMap(you: _fix?.ll, accM: _fix?.accM, height: 130),
                    Row(
                      spacing: 8,
                      children: [
                        const Icon(
                          Icons.my_location_rounded,
                          size: 16,
                          color: JColors.accent,
                        ),
                        Expanded(
                          child: Text(
                            _fix == null
                                ? 'Finding where you are'
                                : 'Your current location',
                            style: latText(size: 15, weight: FontWeight.w700),
                          ),
                        ),
                        InkWell(
                          onTap: _locating ? null : () => _locate(ask: true),
                          borderRadius: BorderRadius.circular(8),
                          child: Padding(
                            padding: const EdgeInsets.symmetric(
                              horizontal: 4,
                              vertical: 2,
                            ),
                            child: Row(
                              mainAxisSize: MainAxisSize.min,
                              spacing: 4,
                              children: [
                                _locating
                                    ? const SizedBox(
                                        width: 14,
                                        height: 14,
                                        child: CircularProgressIndicator(
                                          strokeWidth: 2,
                                          color: JColors.accent,
                                        ),
                                      )
                                    : const Icon(
                                        Icons.refresh_rounded,
                                        size: 14,
                                        color: JColors.accent,
                                      ),
                                Text(
                                  'Update',
                                  style: latText(
                                    size: 13,
                                    weight: FontWeight.w700,
                                    color: JColors.accent,
                                  ),
                                ),
                              ],
                            ),
                          ),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            const HintLine(
              'From your phone\'s GPS. Buyers see it on a map, with how far it is.',
              icon: Icons.location_on_outlined,
            ),
          ],
        ),
        // Closes in.
        Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          spacing: 8,
          children: [
            const FieldLabel('Closes in'),
            Container(
              height: 56,
              padding: const EdgeInsets.all(6),
              decoration: BoxDecoration(
                color: JColors.card,
                borderRadius: BorderRadius.circular(14),
                border: Border.all(color: JColors.line),
              ),
              child: Row(
                children: [
                  _StepButton(
                    icon: Icons.remove_rounded,
                    label: 'One day less',
                    onTap: _days > 1 ? () => setState(() => _days--) : null,
                  ),
                  Expanded(
                    child: Row(
                      mainAxisAlignment: MainAxisAlignment.center,
                      crossAxisAlignment: CrossAxisAlignment.baseline,
                      textBaseline: TextBaseline.alphabetic,
                      spacing: 6,
                      children: [
                        Text(
                          '$_days',
                          style: latText(size: 20, weight: FontWeight.w800),
                        ),
                        Text(
                          _days == 1 ? 'day' : 'days',
                          style: latText(
                            size: 14,
                            weight: FontWeight.w600,
                            color: JColors.muted,
                          ),
                        ),
                      ],
                    ),
                  ),
                  _StepButton(
                    icon: Icons.add_rounded,
                    label: 'One day more',
                    onTap: _days < kAlwaMaxDays
                        ? () => setState(() => _days++)
                        : null,
                  ),
                ],
              ),
            ),
            HintLine(
              'Closes ${fmtDayTime(closes)}. Shorter if you like, 1 to 14 days.',
              icon: Icons.calendar_today_outlined,
            ),
          ],
        ),
        // Your phone.
        Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          spacing: 8,
          children: [
            const FieldLabel('Your phone', trailing: 'from your sign-in'),
            Container(
              height: 52,
              padding: const EdgeInsets.symmetric(horizontal: 16),
              decoration: BoxDecoration(
                color: Colors.white.withValues(alpha: 0.4),
                borderRadius: BorderRadius.circular(14),
                border: Border.all(color: JColors.line),
              ),
              child: Row(
                spacing: 10,
                children: [
                  const Icon(
                    Icons.phone_outlined,
                    size: 16,
                    color: JColors.muted,
                  ),
                  Expanded(
                    child: Text(
                      _phone == null
                          ? 'The phone you signed in with'
                          : fmtPhone(_phone!),
                      style: latText(
                        size: 15,
                        weight: FontWeight.w700,
                        color: _phone == null ? JColors.muted : JColors.ink,
                      ),
                    ),
                  ),
                  const Icon(
                    Icons.lock_outline_rounded,
                    size: 14,
                    color: JColors.muted,
                  ),
                ],
              ),
            ),
            const HintLine('Buyers will call this number.'),
          ],
        ),
        if (_tooMany)
          AlwaBanner(
            alarm: true,
            icon: Icons.playlist_remove_rounded,
            title: 'You have $kAlwaMaxOpen open listings',
            body:
                'That is the most one phone can have. Mark one sold, delete one, or wait for one to close.',
            action: 'Open My listings',
            onAction: () => Navigator.of(context).pushReplacement(
              MaterialPageRoute<void>(
                builder: (_) => AlwaMyListingsScreen(board: widget.board),
              ),
            ),
          ),
        AlwaButton(
          label: 'Put on sale',
          icon: Icons.storefront_outlined,
          loading: _sending,
          onPressed: _submit,
        ),
        Text(
          _openCount == null
              ? 'Up to $kAlwaMaxOpen open listings per phone.'
              : 'Up to $kAlwaMaxOpen open listings per phone. You have $_openCount open.',
          textAlign: TextAlign.center,
          style: latText(
            size: 12,
            weight: FontWeight.w500,
            color: JColors.muted,
            height: 1.4,
          ),
        ),
      ],
    );
  }
}

class _Error extends StatelessWidget {
  const _Error(this.text);
  final String text;

  @override
  Widget build(BuildContext context) => Text(
    text,
    style: latText(
      size: 12,
      weight: FontWeight.w600,
      color: JColors.levelAlarm,
    ),
  );
}

/// 6 products a row; when there are more than 12, the first 11 and
/// "All 16", or all of them.
class _CropPicker extends StatelessWidget {
  const _CropPicker({
    required this.items,
    required this.selected,
    required this.all,
    required this.onAll,
    required this.onPick,
  });
  final List<AlwaCrop> items;
  final String? selected;
  final bool all;
  final VoidCallback onAll;
  final ValueChanged<String> onPick;

  @override
  Widget build(BuildContext context) {
    final all = this.all || items.length <= 12;
    final chips = <Widget>[
      for (final c in all ? items : items.take(11))
        _CropChip(
          label: c.en,
          selected: c.code == selected,
          onTap: () => onPick(c.code),
          child: Text(c.emoji, style: const TextStyle(fontSize: 24)),
        ),
      if (!all)
        _CropChip(
          label: 'All ${items.length}',
          selected: false,
          onTap: onAll,
          child: const Icon(
            Icons.more_horiz_rounded,
            size: 22,
            color: JColors.muted,
          ),
        ),
    ];
    return Column(
      spacing: 8,
      children: [
        for (var i = 0; i < chips.length; i += 6)
          Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              for (var j = i; j < i + 6; j++)
                Expanded(child: j < chips.length ? chips[j] : const SizedBox()),
            ],
          ),
      ],
    );
  }
}

class _CropChip extends StatelessWidget {
  const _CropChip({
    required this.label,
    required this.selected,
    required this.onTap,
    required this.child,
  });
  final String label;
  final bool selected;
  final VoidCallback onTap;
  final Widget child;

  @override
  Widget build(BuildContext context) => Semantics(
    button: true,
    selected: selected,
    label: label,
    excludeSemantics: true,
    child: GestureDetector(
      onTap: onTap,
      behavior: HitTestBehavior.opaque,
      child: Column(
        spacing: 4,
        children: [
          Container(
            width: 52,
            height: 52,
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: selected ? JColors.accentSoft : JColors.card,
              borderRadius: BorderRadius.circular(14),
              border: Border.all(
                color: selected ? JColors.accent : Colors.transparent,
                width: 2,
              ),
            ),
            child: child,
          ),
          Text(
            label,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: latText(
              size: 11,
              weight: selected ? FontWeight.w700 : FontWeight.w600,
              color: selected ? JColors.accent : JColors.ink,
            ),
          ),
        ],
      ),
    ),
  );
}

/// Big number field with its unit on the right (Field).
class _NumberField extends StatelessWidget {
  const _NumberField({
    super.key,
    required this.controller,
    required this.unit,
    this.focusNode,
  });
  final TextEditingController controller;
  final String unit;
  final FocusNode? focusNode;

  @override
  Widget build(BuildContext context) {
    final focused = focusNode?.hasFocus ?? false;
    return Container(
      height: 56,
      padding: const EdgeInsets.symmetric(horizontal: 16),
      decoration: BoxDecoration(
        color: JColors.card,
        borderRadius: BorderRadius.circular(14),
        border: Border.all(
          color: focused ? JColors.accent : JColors.line,
          width: focused ? 1.5 : 1,
        ),
      ),
      child: Row(
        spacing: 10,
        children: [
          Expanded(
            child: TextField(
              controller: controller,
              focusNode: focusNode,
              keyboardType: TextInputType.number,
              inputFormatters: [_Thousands()],
              style: latText(size: 20, weight: FontWeight.w700),
              decoration: InputDecoration(
                border: InputBorder.none,
                isDense: true,
                hintText: '0',
                hintStyle: latText(
                  size: 20,
                  weight: FontWeight.w700,
                  color: JColors.placeholder,
                ),
              ),
            ),
          ),
          Text(
            unit,
            style: latText(
              size: 14,
              weight: FontWeight.w600,
              color: JColors.muted,
            ),
          ),
        ],
      ),
    );
  }
}

/// Digits only, grouped as 4,000; at most 9 digits.
class _Thousands extends TextInputFormatter {
  @override
  TextEditingValue formatEditUpdate(
    TextEditingValue oldValue,
    TextEditingValue newValue,
  ) {
    var d = newValue.text.replaceAll(RegExp(r'\D'), '');
    d = d.replaceFirst(RegExp(r'^0+'), '');
    if (d.length > 9) d = d.substring(0, 9);
    final t = d.isEmpty ? '' : fmtInt(int.parse(d));
    return TextEditingValue(
      text: t,
      selection: TextSelection.collapsed(offset: t.length),
    );
  }
}

class _StepButton extends StatelessWidget {
  const _StepButton({required this.icon, required this.label, this.onTap});
  final IconData icon;
  final String label;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) => Semantics(
    button: true,
    label: label,
    child: Opacity(
      // A step that cannot be taken is faded whole (design: 40%).
      opacity: onTap == null ? 0.4 : 1,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(10),
        child: Container(
          width: 44,
          height: 44,
          decoration: BoxDecoration(
            color: JColors.bg,
            borderRadius: BorderRadius.circular(10),
          ),
          child: Icon(icon, size: 18, color: JColors.ink),
        ),
      ),
    ),
  );
}
