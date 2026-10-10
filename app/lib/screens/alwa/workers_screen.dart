import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../api/api.dart';
import '../../app_scope.dart';
import '../../theme.dart';
import '../../widgets/common.dart';
import 'alwa_widgets.dart';

// Workers for hire (FRONTEND.md 5B): people who do farm work put up a card
// with their name and cost; farmers browse the cards and call. No booking.

/// "25,000 IQD per day".
String workerCost(BuildContext context, Worker w) =>
    '${fmtInt(w.costIqd)} IQD ${AppScope.of(context).s.costPer(w.costPer)}';

/// Find workers: the cards, nearest first when the phone's place is known.
class WorkersScreen extends StatefulWidget {
  const WorkersScreen({super.key});

  @override
  State<WorkersScreen> createState() => _WorkersScreenState();
}

class _WorkersScreenState extends State<WorkersScreen> {
  GpsFix? _fix = AlwaGps.last;
  List<Worker>? _workers;
  Object? _error;

  @override
  void initState() {
    super.initState();
    _load();
    if (_fix == null) _locate();
  }

  /// Quietly: the list works without a place, only unsorted by distance.
  Future<void> _locate() async {
    try {
      final f = await AlwaGps.locate(ask: false);
      if (!mounted) return;
      setState(() => _fix = f);
      _load();
    } on GpsProblem {
      // No distance shown.
    }
  }

  Future<void> _load() async {
    setState(() => _error = null);
    try {
      final w = await AppScope.read(
        context,
      ).api.workers(lat: _fix?.lat, lon: _fix?.lon);
      if (mounted) setState(() => _workers = w);
    } catch (e) {
      if (mounted) setState(() => _error = e);
    }
  }

  @override
  Widget build(BuildContext context) {
    final s = AppScope.of(context).s;
    final all = _workers;
    return AlwaPage(
      children: [
        AlwaTitle(
          en: s.findWorkers,
          ku: 'کرێکار',
          sub:
              'People who do farm work, with their cost. Call them directly.',
        ),
        if (all == null)
          _error != null
              ? loadProblem(_error!, _load, what: 'workers')
              : const AlwaLoading('Loading workers')
        else if (all.isEmpty)
          DashedEmpty(icon: Icons.groups_outlined, text: s.noWorkers)
        else
          Column(
            spacing: 10,
            children: [for (final w in all) _WorkerCard(worker: w)],
          ),
      ],
    );
  }
}

/// One worker: name, cost, note, distance, and Call.
class _WorkerCard extends StatelessWidget {
  const _WorkerCard({required this.worker});
  final Worker worker;

  @override
  Widget build(BuildContext context) {
    final s = AppScope.of(context).s;
    final w = worker;
    final note = w.note?.trim() ?? '';
    final phone = w.phone;
    return AlwaCard(
      padding: const EdgeInsets.all(14),
      radius: 16,
      border: JColors.cardLine,
      child: Row(
        spacing: 12,
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              spacing: 4,
              children: [
                Text(
                  w.name,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: latText(size: 16, weight: FontWeight.w700),
                ),
                Text(
                  workerCost(context, w),
                  style: latText(size: 15, weight: FontWeight.w700),
                ),
                if (note.isNotEmpty)
                  Text(
                    note,
                    style: latText(
                      size: 13,
                      weight: FontWeight.w500,
                      color: JColors.muted,
                      height: 1.35,
                    ),
                  ),
                if (w.distanceKm != null)
                  Row(
                    mainAxisSize: MainAxisSize.min,
                    spacing: 4,
                    children: [
                      const Icon(
                        Icons.location_on_outlined,
                        size: 13,
                        color: JColors.accent,
                      ),
                      Text(
                        fmtKm(w.distanceKm!),
                        style: latText(
                          size: 12,
                          weight: FontWeight.w700,
                          color: JColors.accent,
                        ),
                      ),
                    ],
                  ),
              ],
            ),
          ),
          if (phone != null && phone.isNotEmpty)
            SizedBox(
              width: 100,
              child: AlwaButton(
                label: s.call,
                icon: Icons.phone_outlined,
                height: 44,
                onPressed: () => callPhone(context, phone),
              ),
            ),
        ],
      ),
    );
  }
}

/// Offer my work: the caller's one card (name, cost, per day or hour, a
/// note, available or paused). The phone is the sign-in phone, never typed.
class WorkerOfferScreen extends StatefulWidget {
  const WorkerOfferScreen({super.key});

  @override
  State<WorkerOfferScreen> createState() => _WorkerOfferScreenState();
}

class _WorkerOfferScreenState extends State<WorkerOfferScreen> {
  final _name = TextEditingController();
  final _cost = TextEditingController();
  final _note = TextEditingController();
  String _per = 'day';
  bool _available = true;

  GpsFix? _fix = AlwaGps.last;

  /// Whether a card is on the server (Remove shows only then).
  bool _has = false;
  bool _loading = true;
  bool _busy = false;
  bool _tried = false;

  @override
  void initState() {
    super.initState();
    _loadCard();
    _locate();
  }

  @override
  void dispose() {
    _name.dispose();
    _cost.dispose();
    _note.dispose();
    super.dispose();
  }

  /// The phone's place, the way the Sell screen gets it; the card is saved
  /// without a place when there is none.
  Future<void> _locate() async {
    try {
      final f = await AlwaGps.locate(ask: true);
      if (mounted) setState(() => _fix = f);
    } on GpsProblem {
      // Saved without a place.
    }
  }

  Future<void> _loadCard() async {
    try {
      final c = await AppScope.read(context).api.myWorkerCard();
      if (!mounted || c == null) return;
      setState(() {
        _has = true;
        _name.text = c.name;
        _cost.text = '${c.costIqd}';
        _note.text = c.note ?? '';
        _per = c.costPer == 'hour' ? 'hour' : 'day';
        _available = c.available;
      });
    } on ApiException {
      // An empty form: saving makes or replaces the card anyway.
    } finally {
      if (mounted) setState(() => _loading = false);
    }
  }

  String get _nameText => _name.text.trim();
  int? get _costValue => int.tryParse(_cost.text);
  bool get _nameOk => _nameText.isNotEmpty && _nameText.length <= 80;
  bool get _costOk {
    final c = _costValue;
    return c != null && c >= 1000 && c <= 10000000;
  }

  Future<void> _save() async {
    setState(() => _tried = true);
    if (!_nameOk || !_costOk) return;
    setState(() => _busy = true);
    try {
      await AppScope.read(context).api.saveWorkerCard(
        name: _nameText,
        costIqd: _costValue!,
        costPer: _per,
        note: _note.text.trim(),
        lat: _fix?.lat,
        lon: _fix?.lon,
        available: _available,
      );
      if (!mounted) return;
      showToast(
        context,
        _available
            ? 'Your card is saved. Farmers can call you.'
            : 'Your card is saved and paused. Farmers do not see it.',
      );
      Navigator.of(context).pop();
    } on ApiException catch (e) {
      if (mounted) _failed(e, 'save your card');
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _remove() async {
    setState(() => _busy = true);
    try {
      await AppScope.read(context).api.deleteWorkerCard();
      if (!mounted) return;
      showToast(context, 'Your card is removed.');
      Navigator.of(context).pop();
    } on ApiException catch (e) {
      if (mounted) _failed(e, 'remove your card');
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  void _failed(ApiException e, String what) => showToast(
    context,
    e.isOffline
        ? 'No internet. Try again when you are online.'
        : e.status == 422 || e.status == 400
        ? 'The server did not take this. Check the name and the cost.'
        : 'Could not $what. Try again in a moment.',
    long: true,
  );

  @override
  Widget build(BuildContext context) {
    final s = AppScope.of(context).s;
    return AlwaPage(
      gap: 18,
      children: [
        AlwaTitle(
          en: s.offerMyWork,
          ku: 'کرێکار',
          sub:
              'Put up a card with your name and your cost. Farmers near you call you.',
        ),
        if (_loading) const AlwaLoading('Loading your card'),
        _field(
          s.workerName,
          _TextBox(
            key: const ValueKey('worker-name'),
            controller: _name,
            maxLength: 80,
          ),
          error: _tried && !_nameOk ? 'Type your name.' : null,
        ),
        _field(
          s.workerCost,
          _TextBox(
            key: const ValueKey('worker-cost'),
            controller: _cost,
            number: true,
            maxLength: 8,
            unit: 'IQD ${s.costPer(_per)}',
          ),
          error: _tried && !_costOk
              ? 'Type your cost, from 1,000 to 10,000,000 IQD.'
              : null,
        ),
        Row(
          spacing: 8,
          children: [
            for (final p in const ['day', 'hour'])
              AlwaChip(
                label: s.costPer(p),
                selected: _per == p,
                onTap: () => setState(() => _per = p),
              ),
          ],
        ),
        _field(
          s.workerNote,
          _TextBox(
            key: const ValueKey('worker-note'),
            controller: _note,
            maxLength: 200,
            lines: 3,
          ),
        ),
        AlwaCard(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 6),
          border: JColors.line,
          child: Row(
            children: [
              Expanded(
                child: Text(
                  s.workerAvailable,
                  style: latText(size: 15, weight: FontWeight.w700),
                ),
              ),
              Switch(
                value: _available,
                activeTrackColor: JColors.accent,
                onChanged: (v) => setState(() => _available = v),
              ),
            ],
          ),
        ),
        HintLine(
          _fix == null
              ? 'Farmers call the phone you signed in with. Turn on location to show how far you are.'
              : 'Farmers call the phone you signed in with, and see how far you are from your phone\'s GPS.',
          icon: Icons.phone_outlined,
        ),
        AlwaButton(
          label: s.save,
          icon: Icons.check_rounded,
          loading: _busy,
          onPressed: _save,
        ),
        if (_has)
          AlwaGhostButton(
            label: s.remove,
            icon: Icons.delete_outline_rounded,
            onPressed: _busy ? null : _remove,
          ),
      ],
    );
  }

  Widget _field(String label, Widget box, {String? error}) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    spacing: 8,
    children: [
      FieldLabel(label),
      box,
      if (error != null)
        Text(
          error,
          style: latText(
            size: 12,
            weight: FontWeight.w600,
            color: JColors.levelAlarm,
          ),
        ),
    ],
  );
}

/// A white field like the Sell screen's, for text or a whole number.
class _TextBox extends StatelessWidget {
  const _TextBox({
    super.key,
    required this.controller,
    required this.maxLength,
    this.number = false,
    this.lines = 1,
    this.unit,
  });
  final TextEditingController controller;
  final int maxLength;
  final bool number;
  final int lines;
  final String? unit;

  @override
  Widget build(BuildContext context) => Container(
    constraints: const BoxConstraints(minHeight: 56),
    padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
    decoration: BoxDecoration(
      color: JColors.card,
      borderRadius: BorderRadius.circular(14),
      border: Border.all(color: JColors.line),
    ),
    child: Row(
      spacing: 10,
      children: [
        Expanded(
          child: TextField(
            controller: controller,
            keyboardType: number ? TextInputType.number : TextInputType.text,
            minLines: 1,
            maxLines: lines,
            inputFormatters: [
              if (number) FilteringTextInputFormatter.digitsOnly,
              LengthLimitingTextInputFormatter(maxLength),
            ],
            style: latText(size: 16, weight: FontWeight.w600),
            decoration: const InputDecoration(
              border: InputBorder.none,
              isDense: true,
            ),
          ),
        ),
        if (unit != null)
          Text(
            unit!,
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
