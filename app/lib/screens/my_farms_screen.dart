import 'package:flutter/material.dart';

import '../api/api.dart';
import '../app_scope.dart';
import '../phone.dart';
import '../store/local_store.dart';
import '../store/outbox.dart';
import '../theme.dart';
import '../widgets/common.dart';
import '../widgets/farm_card.dart';
import 'add_farm/corners_screen.dart';
import 'add_farm/farm_actions.dart';
import 'home/home_screen.dart';

/// What the list shows: from the server, or the last copy saved on the phone.
class _FarmList {
  const _FarmList(this.farms, {this.offlineSince});
  final List<FarmSummary> farms;

  /// Set when there is no internet: when this copy was saved.
  final DateTime? offlineSince;
}

/// Step 3 of 3: every farm under this number, plus farms still waiting to upload.
class MyFarmsScreen extends StatefulWidget {
  const MyFarmsScreen({super.key, required this.digits});
  final String digits;

  @override
  State<MyFarmsScreen> createState() => _MyFarmsScreenState();
}

class _MyFarmsScreenState extends State<MyFarmsScreen> {
  static const _cacheName = 'farms_cache';
  int _waiting = Outbox.instance.items.length;

  /// What is on screen: the copy saved on the phone first, then fresh data.
  _FarmList? _list;

  /// True while fresh data is loading (a thin bar shows at the top).
  bool _busy = false;

  /// Only shown when there is nothing at all to show.
  Object? _error;

  @override
  void initState() {
    super.initState();
    Outbox.instance.addListener(_outboxChanged);
    _refresh();
  }

  @override
  void dispose() {
    Outbox.instance.removeListener(_outboxChanged);
    super.dispose();
  }

  /// A waiting farm went up (or a new one was saved): show the fresh list.
  void _outboxChanged() {
    if (!mounted) return;
    final now = Outbox.instance.items.length;
    if (now < _waiting) {
      showToast(context, AppScope.read(context).s.uploaded(_waiting - now));
    }
    _waiting = now;
    _reload();
  }

  void _reload() => _refresh();

  /// Show the saved copy straight away (first time only), then load fresh
  /// data and swap it in. The screen never goes blank while loading.
  Future<void> _refresh() async {
    if (_list == null) {
      final j = await LocalStore.read(_cacheName);
      if (j != null && mounted && _list == null) {
        try {
          setState(
            () => _list = _FarmList([
              for (final f in (j['farms'] as List? ?? const []))
                FarmSummary.fromJson(f as Map<String, dynamic>),
            ]),
          );
        } catch (_) {
          // A damaged copy is simply skipped.
        }
      }
    }
    if (!mounted) return;
    setState(() => _busy = true);
    try {
      final fresh = await _load();
      if (!mounted) return;
      setState(() {
        _list = fresh;
        _error = null;
      });
    } catch (e) {
      if (mounted && _list == null) setState(() => _error = e);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<_FarmList> _load() async {
    final api = AppScope.read(context).api;
    try {
      final farms = await api.getFarms();
      try {
        await LocalStore.write(_cacheName, {
          'saved_at': DateTime.now().toUtc().toIso8601String(),
          'farms': [for (final f in farms) f.toJson()],
        });
      } catch (_) {
        // The offline copy is a nice-to-have; never let it hide the real list.
      }
      return _FarmList(farms);
    } on ApiException catch (e) {
      if (!e.isOffline) rethrow;
      final j = await LocalStore.read(_cacheName);
      return _FarmList(
        [
          for (final f in (j?['farms'] as List? ?? const []))
            FarmSummary.fromJson(f as Map<String, dynamic>),
        ],
        offlineSince: j == null
            ? DateTime.now()
            : DateTime.parse(j['saved_at'] as String).toLocal(),
      );
    }
  }

  /// Each farm opens on its own screen (user, 2026-10-08).
  Future<void> _openFarm(FarmSummary farm) async {
    await Navigator.of(
      context,
    ).push(MaterialPageRoute<void>(builder: (_) => HomeScreen(farm: farm)));
    if (!mounted) return;
    _reload();
  }

  Future<void> _addFarm() async {
    await Navigator.of(
      context,
    ).push(MaterialPageRoute<void>(builder: (_) => const CornersScreen()));
    if (!mounted) return;
    _reload();
  }

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    final s = scope.s;
    final ku = scope.ku;
    final pending = Outbox.instance.items;
    return JutyarPage(
      step: 3,
      children: [
        Padding(
          padding: const EdgeInsets.only(top: 4),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            spacing: 8,
            children: [
              Text(
                s.h3,
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
                    s.lead3,
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
        Builder(
          builder: (context) {
            final current = _list;
            if (current == null) {
              if (_error != null) {
                return Text(
                  '${s.error}: $_error',
                  style: jText(ku, size: 14, color: JColors.levelAlarm),
                );
              }
              return const Padding(
                padding: EdgeInsets.symmetric(vertical: 40),
                child: Center(
                  child: SizedBox(
                    width: 24,
                    height: 24,
                    child: CircularProgressIndicator(
                      strokeWidth: 2.4,
                      color: JColors.accent,
                    ),
                  ),
                ),
              );
            }
            final list = current;
            final gone = Outbox.instance.deletes.toSet();
            final shown = [
              for (final f in list.farms)
                if (!gone.contains(f.id)) f,
            ];
            final edits = {
              for (final p in pending)
                if (p.farmId != null) p.farmId!: p,
            };
            final since = list.offlineSince;
            return Column(
              spacing: 12,
              children: [
                // Fresh data on its way; what is shown is the saved copy.
                SizedBox(
                  height: 3,
                  child: _busy
                      ? ClipRRect(
                          borderRadius: BorderRadius.circular(2),
                          child: const LinearProgressIndicator(
                            minHeight: 3,
                            color: JColors.accent,
                            backgroundColor: JColors.accentSoft,
                          ),
                        )
                      : null,
                ),
                if (since != null)
                  _OfflineBanner(
                    text: s.offlineList('\u2066${_fmtWhen(since)}\u2069'),
                  ),
                if (shown.isEmpty && !pending.any((p) => p.farmId == null))
                  const EmptyFarms(),
                // New farms not uploaded yet.
                for (final p in pending)
                  if (p.farmId == null)
                    FarmCard(farm: p.summary, waiting: true, onTap: () {}),
                // Server farms; a waiting edit shows its new version, and a
                // farm deleted on the phone is hidden at once.
                // Slide a farm sideways to show Delete; letting go asks first.
                for (final f in shown)
                  Dismissible(
                    key: ValueKey('farm-${f.id}'),
                    direction: DismissDirection.horizontal,
                    background: const _DeleteBehind(alignStart: true),
                    secondaryBackground: const _DeleteBehind(alignStart: false),
                    confirmDismiss: (_) =>
                        confirmDeleteFarm(context, edits[f.id]?.summary ?? f),
                    child: FarmCard(
                      farm: edits[f.id]?.summary ?? f,
                      waiting: edits.containsKey(f.id),
                      onTap: () => _openFarm(edits[f.id]?.summary ?? f),
                    ),
                  ),
                AddFarmCard(onTap: _addFarm),
              ],
            );
          },
        ),
      ],
    );
  }

  static String _fmtWhen(DateTime t) =>
      '${t.year}-${_two(t.month)}-${_two(t.day)} ${_two(t.hour)}:${_two(t.minute)}';
  static String _two(int v) => v.toString().padLeft(2, '0');
}

class _OfflineBanner extends StatelessWidget {
  const _OfflineBanner({required this.text});
  final String text;

  @override
  Widget build(BuildContext context) {
    final ku = AppScope.of(context).ku;
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
      decoration: BoxDecoration(
        color: JColors.levelWatchSoft,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        spacing: 8,
        children: [
          const Icon(Icons.cloud_off_rounded, size: 18, color: JColors.gold),
          Expanded(
            child: Text(
              text,
              style: jText(
                ku,
                size: 13,
                weight: FontWeight.w600,
                color: JColors.ink,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// The red Delete shown behind a farm card while it is slid sideways.
class _DeleteBehind extends StatelessWidget {
  const _DeleteBehind({required this.alignStart});

  /// True: Delete sits at the side the card is slid away from first.
  final bool alignStart;

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 24),
      alignment: alignStart
          ? AlignmentDirectional.centerStart
          : AlignmentDirectional.centerEnd,
      decoration: BoxDecoration(
        color: JColors.levelAlarm,
        borderRadius: BorderRadius.circular(16),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        spacing: 4,
        children: [
          const Icon(
            Icons.delete_outline_rounded,
            color: Colors.white,
            size: 26,
          ),
          Text(
            scope.s.delete,
            style: jText(
              scope.ku,
              size: 13,
              weight: FontWeight.w700,
              color: Colors.white,
            ),
          ),
        ],
      ),
    );
  }
}
