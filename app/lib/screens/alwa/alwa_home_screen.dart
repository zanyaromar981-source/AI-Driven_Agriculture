import 'package:flutter/material.dart';

import '../../api/api.dart';
import '../../app_scope.dart';
import '../../l10n/strings.dart';
import '../../theme.dart';
import '../../widgets/common.dart';
import '../../widgets/header.dart';
import 'alwa_listing_screen.dart';
import 'alwa_my_listings_screen.dart';
import 'alwa_sell_screen.dart';
import 'alwa_widgets.dart';
import 'workers_screen.dart';

/// The Alwa tab (Pencil: Screen/Alwa Home): today's prices at the nearest
/// alwa, then crops for sale near the phone, nearest first. Buyers call the
/// seller. The shared tab bar sits around this screen, not in it.
class AlwaHomeScreen extends StatefulWidget {
  const AlwaHomeScreen({super.key});

  @override
  State<AlwaHomeScreen> createState() => _AlwaHomeScreenState();
}

class _AlwaHomeScreenState extends State<AlwaHomeScreen>
    with WidgetsBindingObserver {
  GpsFix? _fix = AlwaGps.last;
  GpsIssue? _gpsIssue;

  AlwaPriceBoard? _board;
  Object? _boardError;
  bool _boardLoading = true;

  List<AlwaListing>? _listings;
  Object? _listingsError;

  /// null = every product group.
  String? _group;

  /// How many cards show; "Show 10 more" adds ten.
  int _shown = 10;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    _loadAll();
    _locate(ask: true);
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  /// Back from the phone's settings: try the GPS again.
  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed && _gpsIssue != null) _locate();
  }

  Future<void> _locate({bool ask = false}) async {
    try {
      final f = await AlwaGps.locate(ask: ask);
      if (!mounted) return;
      final first = _fix == null;
      setState(() {
        _fix = f;
        _gpsIssue = null;
      });
      // With a place, ask again so the server can sort and pick the alwa.
      if (first) _loadAll();
    } on GpsProblem catch (e) {
      if (mounted) setState(() => _gpsIssue = e.issue);
    }
  }

  Future<void> _loadAll() =>
      Future.wait([_loadBoard(), _loadListings(), _loadProducts()]);

  /// The server's product names, for a product the app was not built with.
  Future<void> _loadProducts() async {
    await refreshProducts(AppScope.read(context).api);
    if (mounted) setState(() {});
  }

  Future<void> _loadBoard() async {
    setState(() {
      _boardLoading = true;
      _boardError = null;
    });
    try {
      final b = await AppScope.read(
        context,
      ).api.alwaPriceBoard(lat: _fix?.lat, lon: _fix?.lon);
      if (mounted) setState(() => _board = b);
    } catch (e) {
      if (mounted) setState(() => _boardError = e);
    } finally {
      if (mounted) setState(() => _boardLoading = false);
    }
  }

  Future<void> _loadListings() async {
    setState(() => _listingsError = null);
    final group = _group;
    try {
      final l = await AppScope.read(
        context,
      ).api.alwaListings(lat: _fix?.lat, lon: _fix?.lon, group: group);
      // An answer for a group the farmer has already left is dropped.
      if (mounted && group == _group) setState(() => _listings = l);
    } catch (e) {
      if (mounted && group == _group) setState(() => _listingsError = e);
    }
  }

  Future<void> _fixGps() async {
    final issue = _gpsIssue;
    if (issue != null) await AlwaGps.fix(issue);
    await _locate(ask: true);
  }

  Future<void> _openSell() async {
    final made = await Navigator.of(context).push<AlwaListing>(
      MaterialPageRoute(builder: (_) => AlwaSellScreen(board: _board)),
    );
    if (made == null || !mounted) return;
    showToast(
      context,
      'Your ${alwaCrop(made.crop).en.toLowerCase()} is on sale.',
    );
    _loadListings();
  }

  Future<void> _openMine() async {
    await Navigator.of(context).push<void>(
      MaterialPageRoute(builder: (_) => AlwaMyListingsScreen(board: _board)),
    );
    if (mounted) _loadListings();
  }

  void _open(Widget screen) => Navigator.of(
    context,
  ).push<void>(MaterialPageRoute(builder: (_) => screen));

  void _openListing(AlwaListing l) => Navigator.of(context).push<void>(
    MaterialPageRoute(builder: (_) => AlwaListingScreen(listing: l)),
  );

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        const JutyarHeader(),
        Expanded(
          child: alwaEnglish(
            context,
            RefreshIndicator(
              color: JColors.accent,
              onRefresh: () => Future.wait([_loadAll(), _locate()]),
              child: ListView(
                padding: const EdgeInsets.fromLTRB(20, 12, 20, 24),
                children: gapped([
                  const AlwaTitle(
                    en: 'Alwa market',
                    ku: 'بازاڕی عەلوە',
                    sub:
                        'Today\'s price at the alwa, and crops for sale near you. Call the seller directly.',
                  ),
                  Row(
                    spacing: 10,
                    children: [
                      Expanded(
                        child: AlwaButton(
                          label: 'Sell a crop',
                          icon: Icons.add_rounded,
                          height: 48,
                          onPressed: _openSell,
                        ),
                      ),
                      Expanded(
                        child: AlwaGhostButton(
                          label: 'My listings',
                          icon: Icons.assignment_outlined,
                          onPressed: _openMine,
                        ),
                      ),
                    ],
                  ),
                  // Workers for hire (FRONTEND.md 5B).
                  Row(
                    spacing: 10,
                    children: [
                      Expanded(
                        child: AlwaGhostButton(
                          label: 'Find workers',
                          icon: Icons.groups_outlined,
                          onPressed: () => _open(const WorkersScreen()),
                        ),
                      ),
                      Expanded(
                        child: AlwaGhostButton(
                          label: 'Offer my work',
                          icon: Icons.handyman_outlined,
                          onPressed: () => _open(const WorkerOfferScreen()),
                        ),
                      ),
                    ],
                  ),
                  if (_gpsIssue != null && _fix == null)
                    LocationOffBanner(issue: _gpsIssue!, onFix: _fixGps),
                  _priceBoard(),
                  _forSale(),
                ], 16),
              ),
            ),
          ),
        ),
      ],
    );
  }

  // ---- Today at the alwa ----

  Widget _priceBoard() {
    final b = _board;
    final Widget body;
    if (_boardLoading && b == null) {
      body = const AlwaLoading('Loading today\'s prices');
    } else if (_boardError != null && b == null) {
      body = loadProblem(_boardError!, _loadBoard, what: 'today\'s prices');
    } else if (b == null || b.prices.isEmpty || b.day == null) {
      body = const DashedEmpty(
        icon: Icons.content_paste_off_outlined,
        text: 'No prices posted today',
      );
    } else {
      body = Column(
        spacing: 8,
        children: [
          AlwaCard(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
            child: Column(
              children: [
                for (final (i, p) in b.prices.indexed)
                  _PriceRow(price: p, last: i == b.prices.length - 1),
              ],
            ),
          ),
          HintLine(
            b.nearest
                ? 'Typed in by Ministry staff at the alwa nearest to you. Change is against 7 days ago.'
                : 'Typed in by Ministry staff at the ${b.market.nameEn} alwa. Change is against 7 days ago.',
          ),
        ],
      );
    }
    final day = b?.day;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      spacing: 8,
      children: [
        SectionHead(
          day == null || isToday(day)
              ? 'Today at the alwa'
              : 'Latest at the alwa',
          trailing: b == null
              ? null
              : day == null
              ? b.market.nameEn
              : '${b.market.nameEn} · ${fmtDay(day)}',
        ),
        body,
      ],
    );
  }

  // ---- For sale near you ----

  Widget _forSale() {
    final all = _listings;
    final Widget body;
    var sorted = false;
    if (all == null) {
      body = _listingsError != null
          ? loadProblem(_listingsError!, _loadListings, what: 'crops for sale')
          : const AlwaLoading('Loading crops for sale');
    } else {
      // Nearest first: the server's distance, else worked out on the phone.
      final open = [
        for (final l in all)
          if (l.isOpen) (l: l, km: l.kmFrom(_fix?.lat, _fix?.lon)),
      ];
      if (open.any((e) => e.km != null)) {
        sorted = true;
        open.sort((a, b) => (a.km ?? 1e9).compareTo(b.km ?? 1e9));
      }
      // The server filters with `group=`; this keeps the list right while
      // its answer is on the way.
      final picked = [
        for (final e in open)
          if (_group == null || e.l.group == _group) e,
      ];
      if (picked.isEmpty) {
        body = DashedEmpty(
          icon: Icons.storefront_outlined,
          text: _group == null
              ? 'No crops for sale near you yet'
              : 'Nothing in "${const S(false).groupName(_group!)}" for sale near you yet',
        );
      } else {
        final more = (picked.length - _shown).clamp(0, 10);
        body = Column(
          spacing: 10,
          children: [
            for (final e in picked.take(_shown))
              _ListingCard(
                listing: e.l,
                km: e.km,
                onTap: () => _openListing(e.l),
              ),
            if (more > 0)
              AlwaGhostButton(
                label: 'Show $more more',
                filled: false,
                icon: Icons.keyboard_double_arrow_down_rounded,
                onPressed: () => setState(() => _shown += 10),
              ),
          ],
        );
      }
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      spacing: 10,
      children: [
        SectionHead(
          'For sale near you',
          trailing: sorted ? 'nearest first' : null,
        ),
        GroupChips(
          all: true,
          selected: _group,
          onPick: (g) {
            setState(() {
              _group = g;
              _shown = 10;
            });
            _loadListings();
          },
        ),
        body,
      ],
    );
  }
}

/// One crop on the price board (Price Row).
class _PriceRow extends StatelessWidget {
  const _PriceRow({required this.price, required this.last});
  final AlwaPrice price;
  final bool last;

  @override
  Widget build(BuildContext context) {
    final crop = alwaCrop(price.crop);
    final pct = price.changePct7d?.round();
    return Container(
      constraints: const BoxConstraints(minHeight: 52),
      decoration: BoxDecoration(
        border: last
            ? null
            : const Border(bottom: BorderSide(color: JColors.line)),
      ),
      child: Row(
        spacing: 12,
        children: [
          CropTile(crop: price.crop, size: 32, radius: 10),
          Expanded(
            child: Row(
              spacing: 12,
              children: [
                Flexible(
                  flex: 2,
                  child: Text(
                    crop.en,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: latText(size: 15, weight: FontWeight.w600),
                  ),
                ),
                Flexible(
                  child: Text(
                    crop.ku,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    textDirection: TextDirection.rtl,
                    style: jText(true, size: 13, color: JColors.muted),
                  ),
                ),
              ],
            ),
          ),
          if (price.fixed)
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
              decoration: BoxDecoration(
                color: JColors.goldSoft,
                borderRadius: BorderRadius.circular(999),
              ),
              child: Text(
                'gov. price',
                style: latText(
                  size: 11,
                  weight: FontWeight.w700,
                  color: JColors.gold,
                ),
              ),
            )
          else if (pct != null)
            Text(
              pct > 0 ? '+$pct%' : '$pct%',
              style: latText(
                size: 12,
                weight: FontWeight.w600,
                color: pct > 0
                    ? JColors.levelNormal
                    : pct < 0
                    ? JColors.gold
                    : JColors.muted,
              ),
            ),
          Row(
            crossAxisAlignment: CrossAxisAlignment.baseline,
            textBaseline: TextBaseline.alphabetic,
            spacing: 3,
            children: [
              Text(
                fmtInt(price.priceIqdPerKg),
                style: latText(size: 16, weight: FontWeight.w700),
              ),
              Text(
                'IQD/kg',
                style: latText(
                  size: 11,
                  weight: FontWeight.w600,
                  color: JColors.muted,
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

/// One crop for sale (Listing Card): tap to open, round button to call.
class _ListingCard extends StatelessWidget {
  const _ListingCard({
    required this.listing,
    required this.km,
    required this.onTap,
  });
  final AlwaListing listing;
  final double? km;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final crop = alwaCrop(listing.crop);
    final left = timeLeft(listing.closesAt);
    final phone = listing.sellerPhone;
    return Material(
      color: JColors.card,
      borderRadius: BorderRadius.circular(16),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(16),
        child: Container(
          padding: const EdgeInsets.all(14),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(16),
            border: Border.all(color: JColors.cardLine),
          ),
          child: Row(
            spacing: 12,
            children: [
              CropTile(crop: listing.crop),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  spacing: 5,
                  children: [
                    Row(
                      spacing: 6,
                      children: [
                        Flexible(
                          child: Text(
                            crop.en,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: latText(size: 16, weight: FontWeight.w700),
                          ),
                        ),
                        Flexible(
                          child: Text(
                            crop.ku,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            textDirection: TextDirection.rtl,
                            style: jText(true, size: 13, color: JColors.muted),
                          ),
                        ),
                      ],
                    ),
                    Text.rich(
                      TextSpan(
                        children: [
                          TextSpan(
                            text: fmtInt(listing.priceIqd),
                            style: latText(
                              size: 18,
                              weight: FontWeight.w800,
                            ).copyWith(letterSpacing: -0.3),
                          ),
                          TextSpan(
                            text: ' ${iqdPer(listing.unit)}',
                            style: latText(
                              size: 12,
                              weight: FontWeight.w600,
                              color: JColors.muted,
                            ),
                          ),
                          TextSpan(
                            text:
                                ' · ${fmtQty(listing.quantity, listing.unit)}',
                            style: latText(size: 13, weight: FontWeight.w600),
                          ),
                        ],
                      ),
                    ),
                    Wrap(
                      spacing: 12,
                      runSpacing: 4,
                      children: [
                        if (km != null)
                          _Meta(
                            icon: Icons.location_on_outlined,
                            text: fmtKm(km!),
                            color: JColors.accent,
                            bold: true,
                          ),
                        _Meta(
                          icon: Icons.timer_outlined,
                          text: left.text,
                          color: left.soon ? JColors.gold : JColors.muted,
                          bold: left.soon,
                        ),
                      ],
                    ),
                  ],
                ),
              ),
              if (phone != null)
                Semantics(
                  button: true,
                  label: 'Call the seller',
                  child: InkWell(
                    onTap: () => callPhone(context, phone),
                    customBorder: const CircleBorder(),
                    child: Container(
                      width: 44,
                      height: 44,
                      decoration: const BoxDecoration(
                        color: JColors.accent,
                        shape: BoxShape.circle,
                      ),
                      child: const Icon(
                        Icons.phone_outlined,
                        size: 19,
                        color: Colors.white,
                      ),
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

class _Meta extends StatelessWidget {
  const _Meta({
    required this.icon,
    required this.text,
    required this.color,
    this.bold = false,
  });
  final IconData icon;
  final String text;
  final Color color;
  final bool bold;

  @override
  Widget build(BuildContext context) => Row(
    mainAxisSize: MainAxisSize.min,
    spacing: 4,
    children: [
      Icon(icon, size: 13, color: color),
      Text(
        text,
        style: latText(
          size: 12,
          weight: bold ? FontWeight.w700 : FontWeight.w500,
          color: color,
        ),
      ),
    ],
  );
}
