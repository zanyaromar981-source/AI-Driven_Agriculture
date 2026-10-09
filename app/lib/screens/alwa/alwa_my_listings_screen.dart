import 'package:flutter/material.dart';

import '../../api/api.dart';
import '../../app_scope.dart';
import '../../theme.dart';
import '../../widgets/common.dart';
import 'alwa_sell_screen.dart';
import 'alwa_widgets.dart';

/// The farmer's own listings (Pencil: Screen/Alwa My Listings), grouped as
/// open, sold and closed. Open ones can be marked sold or deleted.
class AlwaMyListingsScreen extends StatefulWidget {
  const AlwaMyListingsScreen({super.key, this.board});

  /// Passed on to Sell, for the price hint.
  final AlwaPriceBoard? board;

  @override
  State<AlwaMyListingsScreen> createState() => _AlwaMyListingsScreenState();
}

class _AlwaMyListingsScreenState extends State<AlwaMyListingsScreen> {
  List<AlwaListing>? _mine;
  Object? _error;

  /// The listing a button is working on.
  String? _busy;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() => _error = null);
    try {
      final m = await AppScope.read(context).api.myAlwaListings();
      if (mounted) setState(() => _mine = m);
    } catch (e) {
      if (mounted) setState(() => _error = e);
    }
  }

  Future<void> _sell() async {
    final made = await Navigator.of(context).push<AlwaListing>(
      MaterialPageRoute(builder: (_) => AlwaSellScreen(board: widget.board)),
    );
    if (made == null || !mounted) return;
    showToast(
      context,
      'Your ${alwaCrop(made.crop).en.toLowerCase()} is on sale.',
    );
    _load();
  }

  Future<void> _markSold(AlwaListing l) async {
    setState(() => _busy = l.id);
    try {
      await AppScope.read(context).api.markAlwaListingSold(l.id);
      if (!mounted) return;
      showToast(context, 'Marked sold. Buyers no longer see it.');
      await _load();
    } on ApiException catch (e) {
      if (!mounted) return;
      showToast(
        context,
        e.isOffline
            ? 'No internet. Try again when you are online.'
            : e.status == 404
            ? 'Mark as sold is not on the server yet. Use Delete to take it off sale.'
            : 'Could not mark it sold. Try again in a moment.',
        long: true,
      );
    } finally {
      if (mounted) setState(() => _busy = null);
    }
  }

  Future<void> _delete(AlwaListing l) async {
    final crop = alwaCrop(l.crop).en.toLowerCase();
    final ok = await showDialog<bool>(
      context: context,
      builder: (c) => AlertDialog(
        backgroundColor: JColors.card,
        title: Text(
          'Delete this listing?',
          style: latText(size: 18, weight: FontWeight.w700),
        ),
        content: Text(
          l.isOpen
              ? 'Your $crop comes off sale. Buyers no longer see it.'
              : 'Your $crop is removed from this list.',
          style: latText(size: 14, color: JColors.muted, height: 1.4),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(c).pop(false),
            child: Text(
              'Keep it',
              style: latText(
                size: 14,
                weight: FontWeight.w700,
                color: JColors.muted,
              ),
            ),
          ),
          TextButton(
            onPressed: () => Navigator.of(c).pop(true),
            child: Text(
              'Delete',
              style: latText(
                size: 14,
                weight: FontWeight.w700,
                color: JColors.levelAlarm,
              ),
            ),
          ),
        ],
      ),
    );
    if (ok != true || !mounted) return;
    setState(() => _busy = l.id);
    try {
      await AppScope.read(context).api.cancelAlwaListing(l.id);
      if (!mounted) return;
      showToast(context, 'Deleted.');
      await _load();
    } on ApiException catch (e) {
      if (!mounted) return;
      showToast(
        context,
        e.isOffline
            ? 'No internet. Try again when you are online.'
            : e.status == 409
            ? 'The server keeps sold and closed listings for now. Only open ones can be deleted.'
            : 'Could not delete it. Try again in a moment.',
        long: true,
      );
    } finally {
      if (mounted) setState(() => _busy = null);
    }
  }

  @override
  Widget build(BuildContext context) {
    final mine = _mine;
    final open = [...?mine?.where((l) => l.isOpen)];
    final sold = [...?mine?.where((l) => l.status == 'sold')];
    final closed = [...?mine?.where((l) => l.status == 'closed')];
    return AlwaPage(
      children: [
        const AlwaTitle(en: 'My listings', ku: 'بەرهەمەکانم'),
        if (mine == null)
          _error != null
              ? loadProblem(_error!, _load, what: 'your listings')
              : const AlwaLoading('Loading your listings')
        else if (open.isEmpty && sold.isEmpty && closed.isEmpty)
          const DashedEmpty(
            icon: Icons.storefront_outlined,
            text: 'You have nothing on sale yet',
          )
        else ...[
          if (open.isNotEmpty)
            _Group(
              label: 'Open · ${open.length} of $kAlwaMaxOpen',
              children: [
                for (final l in open)
                  _MyCard(
                    listing: l,
                    busy: _busy == l.id,
                    onSold: () => _markSold(l),
                    onDelete: () => _delete(l),
                  ),
              ],
            ),
          if (sold.isNotEmpty)
            _Group(
              label: 'Sold',
              children: [
                for (final l in sold)
                  _MyCard(
                    listing: l,
                    busy: _busy == l.id,
                    onDelete: () => _delete(l),
                  ),
              ],
            ),
          if (closed.isNotEmpty)
            _Group(
              label: 'Closed',
              children: [
                for (final l in closed)
                  _MyCard(
                    listing: l,
                    busy: _busy == l.id,
                    onDelete: () => _delete(l),
                  ),
              ],
            ),
        ],
        AlwaButton(
          label: mine == null || mine.isEmpty
              ? 'Sell a crop'
              : 'Sell another crop',
          icon: Icons.add_rounded,
          onPressed: _sell,
        ),
      ],
    );
  }
}

class _Group extends StatelessWidget {
  const _Group({required this.label, required this.children});
  final String label;
  final List<Widget> children;

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    spacing: 8,
    children: [SectionHead(label), ...children],
  );
}

/// One of the farmer's listings (My Listing card).
class _MyCard extends StatelessWidget {
  const _MyCard({
    required this.listing,
    required this.busy,
    required this.onDelete,
    this.onSold,
  });
  final AlwaListing listing;
  final bool busy;
  final VoidCallback onDelete;
  final VoidCallback? onSold;

  @override
  Widget build(BuildContext context) {
    final l = listing;
    final crop = alwaCrop(l.crop);
    final left = timeLeft(l.closesAt);
    final days = (l.closesAt.difference(l.createdAt).inHours / 24).round();
    return Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: JColors.card,
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: JColors.cardLine),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        spacing: 12,
        children: [
          Row(
            spacing: 10,
            children: [
              CropTile(crop: l.crop, size: 40, radius: 10),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  spacing: 2,
                  children: [
                    Text(
                      '${crop.en} · ${fmtInt(l.quantityKg)} kg',
                      style: latText(size: 15, weight: FontWeight.w700),
                    ),
                    Text(
                      '${fmtInt(l.priceIqdPerKg)} IQD/kg · posted ${fmtDay(l.createdAt)}',
                      style: latText(
                        size: 12,
                        weight: FontWeight.w500,
                        color: JColors.muted,
                      ),
                    ),
                  ],
                ),
              ),
              StatusPill(l.status),
            ],
          ),
          if (l.isOpen) ...[
            Row(
              spacing: 6,
              children: [
                Icon(
                  Icons.timer_outlined,
                  size: 14,
                  color: left.soon ? JColors.gold : JColors.muted,
                ),
                Expanded(
                  child: Text(
                    left.soon
                        ? left.text
                        : '${left.text}, closes ${fmtDay(l.closesAt)}',
                    style: latText(
                      size: 13,
                      weight: left.soon ? FontWeight.w700 : FontWeight.w500,
                      color: left.soon ? JColors.gold : JColors.muted,
                    ),
                  ),
                ),
              ],
            ),
            Container(
              padding: const EdgeInsets.only(top: 12),
              decoration: const BoxDecoration(
                border: Border(top: BorderSide(color: JColors.line)),
              ),
              child: Row(
                spacing: 10,
                children: [
                  Expanded(
                    child: _CardButton(
                      icon: Icons.check_circle_outline_rounded,
                      label: 'Mark as sold',
                      color: JColors.accent,
                      fill: JColors.accentSoft,
                      bold: true,
                      onTap: busy ? null : onSold,
                    ),
                  ),
                  _CardButton(
                    icon: Icons.delete_outline_rounded,
                    label: 'Delete',
                    color: JColors.levelAlarm,
                    onTap: busy ? null : onDelete,
                  ),
                ],
              ),
            ),
          ] else
            Container(
              padding: const EdgeInsets.only(top: 12),
              decoration: const BoxDecoration(
                border: Border(top: BorderSide(color: JColors.line)),
              ),
              child: Row(
                spacing: 8,
                children: [
                  Icon(
                    l.status == 'sold'
                        ? Icons.check_circle_outline_rounded
                        : Icons.timer_off_outlined,
                    size: 14,
                    color: JColors.muted,
                  ),
                  Expanded(
                    child: Text(
                      l.status == 'sold'
                          ? l.soldAt == null
                                ? 'Sold. Buyers no longer see it.'
                                : 'You marked it sold on ${fmtDay(l.soldAt!)}. Buyers no longer see it.'
                          : 'Closed by itself on ${fmtDay(l.closesAt)}, after $days ${days == 1 ? 'day' : 'days'}.',
                      style: latText(
                        size: 12,
                        weight: FontWeight.w500,
                        color: JColors.muted,
                        height: 1.4,
                      ),
                    ),
                  ),
                  _CardButton(
                    icon: Icons.delete_outline_rounded,
                    label: 'Delete',
                    color: JColors.levelAlarm,
                    small: true,
                    onTap: busy ? null : onDelete,
                  ),
                ],
              ),
            ),
        ],
      ),
    );
  }
}

class _CardButton extends StatelessWidget {
  const _CardButton({
    required this.icon,
    required this.label,
    required this.color,
    required this.onTap,
    this.fill,
    this.bold = false,
    this.small = false,
  });
  final IconData icon;
  final String label;
  final Color color;
  final Color? fill;
  final bool bold;
  final bool small;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) => Material(
    color: fill ?? Colors.transparent,
    borderRadius: BorderRadius.circular(small ? 10 : 12),
    child: InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(small ? 10 : 12),
      child: Container(
        height: small ? 32 : 40,
        padding: EdgeInsets.symmetric(horizontal: small ? 10 : 14),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(small ? 10 : 12),
          border: Border.all(color: color),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          mainAxisAlignment: MainAxisAlignment.center,
          spacing: 6,
          children: [
            Icon(icon, size: small ? 13 : 15, color: color),
            Flexible(
              child: Text(
                label,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: latText(
                  size: small ? 12 : 14,
                  weight: bold ? FontWeight.w700 : FontWeight.w600,
                  color: color,
                ),
              ),
            ),
          ],
        ),
      ),
    ),
  );
}
