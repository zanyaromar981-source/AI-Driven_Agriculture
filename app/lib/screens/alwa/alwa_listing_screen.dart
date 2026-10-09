import 'package:flutter/material.dart';
import 'package:latlong2/latlong.dart';

import '../../api/api.dart';
import '../../app_scope.dart';
import '../../theme.dart';
import 'alwa_widgets.dart';

/// One crop for sale (Pencil: Screen/Alwa Listing): the price, where it is,
/// the facts and the seller's phone. Buyers call the seller; there are no
/// offers in the app.
class AlwaListingScreen extends StatefulWidget {
  const AlwaListingScreen({super.key, required this.listing});
  final AlwaListing listing;

  @override
  State<AlwaListingScreen> createState() => _AlwaListingScreenState();
}

class _AlwaListingScreenState extends State<AlwaListingScreen> {
  late AlwaListing _l = widget.listing;
  GpsFix? _fix = AlwaGps.last;

  @override
  void initState() {
    super.initState();
    _refresh();
  }

  /// The latest copy from the server; on failure the list's copy stays.
  Future<void> _refresh() async {
    try {
      final l = await AppScope.read(context).api.alwaListing(_l.id);
      if (mounted) setState(() => _l = l);
    } on ApiException {
      // Offline or gone: keep what the list showed.
    }
    if (_fix == null) {
      try {
        final f = await AlwaGps.locate(ask: false);
        if (mounted) setState(() => _fix = f);
      } on GpsProblem {
        // No place: the distance line says so.
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final l = _l;
    final crop = alwaCrop(l.crop);
    final km = l.kmFrom(_fix?.lat, _fix?.lon);
    final left = timeLeft(l.closesAt);
    return AlwaPage(
      children: [
        // Hero: crop, status, posted.
        Row(
          spacing: 14,
          children: [
            CropTile(crop: l.crop, size: 56, radius: 14, color: JColors.card),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                spacing: 6,
                children: [
                  Row(
                    spacing: 8,
                    children: [
                      Flexible(
                        child: Text(
                          crop.en,
                          style: latText(
                            size: 24,
                            weight: FontWeight.w700,
                          ).copyWith(letterSpacing: -0.4),
                        ),
                      ),
                      Text(
                        crop.ku,
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
                  Row(
                    spacing: 8,
                    children: [
                      StatusPill(l.status),
                      Flexible(
                        child: Text(
                          l.createdAt == null
                              ? 'Posted'
                              : 'Posted ${fmtDay(l.createdAt!)}',
                          style: latText(
                            size: 13,
                            weight: FontWeight.w500,
                            color: JColors.muted,
                          ),
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ],
        ),
        // Price card.
        AlwaCard(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            spacing: 10,
            children: [
              Row(
                crossAxisAlignment: CrossAxisAlignment.baseline,
                textBaseline: TextBaseline.alphabetic,
                spacing: 6,
                children: [
                  Text(
                    fmtInt(l.priceIqdPerKg),
                    style: latText(
                      size: 32,
                      weight: FontWeight.w800,
                    ).copyWith(letterSpacing: -0.6),
                  ),
                  Text(
                    'IQD/kg asking',
                    style: latText(
                      size: 13,
                      weight: FontWeight.w600,
                      color: JColors.muted,
                    ),
                  ),
                ],
              ),
              Text(
                '${fmtInt(l.quantityKg)} kg · '
                '${fmtInt(l.quantityKg * l.priceIqdPerKg)} IQD for all of it',
                style: latText(size: 14, weight: FontWeight.w500),
              ),
            ],
          ),
        ),
        // Where it is.
        Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          spacing: 8,
          children: [
            const SectionHead('Where it is'),
            AlwaCard(
              padding: const EdgeInsets.all(12),
              child: l.hasPoint
                  ? Column(
                      spacing: 12,
                      children: [
                        AlwaMiniMap(
                          crop: l.crop,
                          item: LatLng(l.lat!, l.lon!),
                          you: _fix?.ll,
                        ),
                        Row(
                          spacing: 8,
                          children: [
                            const Icon(
                              Icons.location_on_outlined,
                              size: 16,
                              color: JColors.accent,
                            ),
                            Expanded(
                              child: Text(
                                km != null
                                    ? '${fmtKm(km)} from you'
                                    : 'Turn on location to see how far it is',
                                style: latText(
                                  size: 15,
                                  weight: FontWeight.w700,
                                  color: km != null
                                      ? JColors.ink
                                      : JColors.muted,
                                ),
                              ),
                            ),
                          ],
                        ),
                      ],
                    )
                  : const DashedEmpty(
                      icon: Icons.location_off_outlined,
                      text: 'The seller\'s place is not shared yet',
                    ),
            ),
          ],
        ),
        // Facts.
        AlwaCard(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
          child: Column(
            children: [
              _Fact(
                icon: Icons.inventory_2_outlined,
                label: 'Quantity',
                value: '${fmtInt(l.quantityKg)} kg',
              ),
              _Fact(
                icon: Icons.calendar_today_outlined,
                label: 'Posted',
                value: l.createdAt == null ? '-' : fmtDayTime(l.createdAt!),
              ),
              _Fact(
                icon: Icons.timer_outlined,
                label: 'Closes',
                value: left.soon
                    ? left.text
                    : '${fmtDay(l.closesAt)} · ${left.text}',
                last: true,
              ),
            ],
          ),
        ),
        // Seller's phone.
        Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          spacing: 8,
          children: [
            const SectionHead('Seller\'s phone'),
            AlwaCard(
              radius: 16,
              border: JColors.accent,
              borderWidth: 1.5,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                spacing: 14,
                children: [
                  if (l.sellerPhone != null) ...[
                    SelectableText(
                      fmtPhone(l.sellerPhone!),
                      style: latText(
                        size: 26,
                        weight: FontWeight.w800,
                      ).copyWith(letterSpacing: 0.3),
                    ),
                    // Opens the phone app with the number typed in.
                    AlwaButton(
                      label: 'Call the seller',
                      icon: Icons.phone_rounded,
                      onPressed: () => callPhone(context, l.sellerPhone!),
                    ),
                  ] else
                    Text(
                      'The seller\'s phone is not shown yet. It will be, once the server sends it.',
                      style: latText(
                        size: 14,
                        weight: FontWeight.w600,
                        color: JColors.muted,
                        height: 1.4,
                      ),
                    ),
                  const HintLine(
                    'Agree the pickup and the money on the phone. Jutyar never takes payment.',
                    icon: Icons.verified_user_outlined,
                  ),
                ],
              ),
            ),
          ],
        ),
      ],
    );
  }
}

class _Fact extends StatelessWidget {
  const _Fact({
    required this.icon,
    required this.label,
    required this.value,
    this.last = false,
  });
  final IconData icon;
  final String label;
  final String value;
  final bool last;

  @override
  Widget build(BuildContext context) => Container(
    constraints: const BoxConstraints(minHeight: 44),
    decoration: BoxDecoration(
      border: last
          ? null
          : const Border(bottom: BorderSide(color: JColors.line)),
    ),
    child: Row(
      spacing: 10,
      children: [
        Icon(icon, size: 16, color: JColors.muted),
        Text(
          label,
          style: latText(
            size: 14,
            weight: FontWeight.w500,
            color: JColors.muted,
          ),
        ),
        Expanded(
          child: Text(
            value,
            textAlign: TextAlign.right,
            style: latText(size: 14, weight: FontWeight.w600),
          ),
        ),
      ],
    ),
  );
}
