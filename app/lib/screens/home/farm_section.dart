import 'package:flutter/material.dart';

import '../../api/api.dart';
import '../../app_scope.dart';
import '../../crops.dart';
import '../../geo.dart';
import '../../store/local_store.dart';
import '../../theme.dart';
import '../../widgets/common.dart';
import '../../widgets/farm_card.dart';
import 'cell_card.dart';
import 'farm_drawing.dart';
import 'week_strip.dart';

/// One farm on Home: name, picture date, Cells / Crops / Farm, the drawing,
/// the weak-cell line and this week's plan. Keeps a copy on the phone and
/// shows it with its date when there is no internet.
class FarmSection extends StatefulWidget {
  const FarmSection({super.key, required this.summary, this.onLoaded});
  final FarmSummary summary;

  /// Called once, when the first load finishes (data, cached copy or error).
  final VoidCallback? onLoaded;

  @override
  State<FarmSection> createState() => _FarmSectionState();
}

class _FarmSectionState extends State<FarmSection> {
  Farm? _farm;
  FarmShape? _shape;
  FarmStatusReport? _status;
  FarmPlan? _plan;
  bool _planDown = false;
  bool _loading = true;

  /// True while fresh data loads over the copy already on screen.
  bool _refreshing = false;

  /// When the copy on screen was saved (for the banner if fresh data fails).
  DateTime? _shownSavedAt;
  String? _error;
  DateTime? _offlineSince;
  bool _told = false;

  FarmView _view = FarmView.cells;
  CellKey? _cell;
  String? _crop;

  String get _cacheName => 'farm_${widget.summary.id}';

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final api = AppScope.read(context).api;
    final s = AppScope.read(context).s;
    final id = widget.summary.id;
    Farm? farm;
    FarmStatusReport? status;
    FarmPlan? plan;
    var planDown = false;
    DateTime? offlineSince;
    // Show the copy saved on the phone at once; fresh data replaces it below.
    if (_farm == null) {
      final c = await LocalStore.read(_cacheName);
      final cf = c?['farm'];
      if (cf is Map<String, dynamic> && mounted) {
        try {
          final cfarm = Farm.fromJson(cf);
          final cs = c!['status'];
          final cst = cs is Map<String, dynamic>
              ? FarmStatusReport.fromJson(cs)
              : null;
          final cp = c['plan'];
          setState(() {
            _farm = cfarm;
            _status = cst;
            _shape = cfarm.outline.length < 3 ? null : FarmShape(cfarm, cst);
            _plan = cp is Map<String, dynamic> ? FarmPlan.fromJson(cp) : null;
            _shownSavedAt = DateTime.tryParse(
              c['saved_at'] as String? ?? '',
            )?.toLocal();
            _loading = false;
          });
        } catch (_) {
          // A damaged copy is skipped; the spinner stays until fresh data.
        }
      }
    }
    if (!mounted) return;
    if (_farm != null) setState(() => _refreshing = true);
    String? error;
    try {
      final got = await Future.wait<Object>([
        api.getFarm(id),
        api.getFarmStatus(id),
      ]);
      farm = got[0] as Farm;
      status = got[1] as FarmStatusReport;
      try {
        plan = await api.getPlan(id);
      } on ApiException {
        planDown = true;
        final old = await LocalStore.read(_cacheName);
        final p = old?['plan'] as Map<String, dynamic>?;
        if (p != null) plan = FarmPlan.fromJson(p);
      }
      await LocalStore.write(_cacheName, {
        'saved_at': DateTime.now().toUtc().toIso8601String(),
        'farm': farm.toJson(),
        'status': status.json,
        'plan': plan?.json,
      });
    } on ApiException catch (e) {
      if (e.isOffline) {
        final j = await LocalStore.read(_cacheName);
        if (j == null) {
          error = s.noInternet;
        } else {
          farm = Farm.fromJson(j['farm'] as Map<String, dynamic>);
          status = FarmStatusReport.fromJson(
            j['status'] as Map<String, dynamic>,
          );
          final p = j['plan'] as Map<String, dynamic>?;
          plan = p == null ? null : FarmPlan.fromJson(p);
          offlineSince = DateTime.parse(j['saved_at'] as String);
        }
      } else {
        error = '${s.loadFailed} (${e.code})';
      }
    } catch (e) {
      error = '${s.loadFailed} ($e)';
    }
    if (!mounted) return;
    if (farm == null && _farm != null) {
      // Fresh data failed but a copy is on screen: keep it, say how old it is.
      setState(() {
        _loading = false;
        _refreshing = false;
        _offlineSince = offlineSince ?? _shownSavedAt;
      });
      if (!_told) {
        _told = true;
        widget.onLoaded?.call();
      }
      return;
    }
    setState(() {
      _loading = false;
      _refreshing = false;
      _error = error;
      _farm = farm;
      _status = status;
      _shape = farm == null || farm.outline.length < 3
          ? null
          : FarmShape(farm, status);
      _plan = plan;
      _planDown = planDown;
      _offlineSince = offlineSince;
    });
    if (!_told) {
      _told = true;
      widget.onLoaded?.call();
    }
  }

  void _retry() {
    setState(() => _loading = true);
    _load();
  }

  void _setView(FarmView v) => setState(() {
    _view = v;
    _cell = null;
    _crop = null;
  });

  @override
  Widget build(BuildContext context) {
    final s = AppScope.of(context).s;
    final sum = _farm?.summary ?? widget.summary;
    return Container(
      padding: const EdgeInsets.fromLTRB(16, 16, 16, 18),
      decoration: BoxDecoration(
        color: JColors.card,
        borderRadius: BorderRadius.circular(18),
        border: Border.all(color: JColors.cardLine),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        spacing: 12,
        children: [
          Row(
            spacing: 8,
            children: [
              Expanded(
                child: Text(
                  sum.name,
                  style: jText(false, size: 20, weight: FontWeight.w800),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              StatusPill(
                status: sum.status,
                hasPicture: sum.lastPicture != null,
              ),
            ],
          ),
          // Thin bar while fresh data loads over the saved copy.
          SizedBox(
            height: 3,
            child: _refreshing
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
          if (_loading)
            const Padding(
              padding: EdgeInsets.symmetric(vertical: 60),
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
            )
          else if (_error != null || _shape == null)
            _ErrorBox(
              text: _error ?? s.loadFailed,
              retry: s.retry,
              onRetry: _retry,
            )
          else
            ..._body(context),
        ],
      ),
    );
  }

  List<Widget> _body(BuildContext context) {
    final s = AppScope.of(context).s;
    final shape = _shape!;
    final st = _status;
    final picture = st?.pictureDate;
    final next = st?.nextPictureExpected;
    final measured = [
      for (final e in shape.reading.entries)
        if (e.value.greennessPct != null) e.key,
    ];
    final weakM2 = shape.areaM2(
      measured.where((k) => shape.levelAt(k) != FarmStatus.normal),
    );
    // Same number as the crop list and the Farm view: the server's crop areas.
    final measuredCrops = (st?.crops ?? const <CropReading>[]).where(
      (c) => c.greennessPctOfNormal != null,
    );
    final measuredM2 = measuredCrops.isEmpty
        ? shape.areaM2(measured)
        : measuredCrops.fold(0.0, (a, c) => a + c.dunam * 2500);
    final whole = st?.greennessPctOfNormal;
    final since = _offlineSince;
    return [
      Row(
        spacing: 6,
        children: [
          const Icon(
            Icons.satellite_alt_outlined,
            size: 15,
            color: JColors.muted,
          ),
          Expanded(
            child: Text(
              picture == null
                  ? s.waitingFirst(next == null ? null : fmtDay(next))
                  : s.fromSpace(
                      fmtDay(picture),
                      next == null ? null : fmtDay(next),
                    ),
              style: jText(false, size: 12.5, color: JColors.muted),
            ),
          ),
        ],
      ),
      if (since != null) _Banner(text: s.offlineCopy(fmtWhen(since))),
      _ViewToggle(view: _view, onChanged: _setView),
      FarmDrawing(
        shape: shape,
        view: _view,
        farmLevel: levelFromPct(whole),
        selectedCell: _cell,
        selectedCrop: _crop,
        onCell: (k) => setState(() => _cell = k),
        onCrop: (c) => setState(() => _crop = c),
        overlay: _overlay(context, shape),
      ),
      if (_view == FarmView.crops)
        _CropList(status: st, farm: _farm!)
      else
        const _Legend(),
      if (_view != FarmView.crops)
        Text(
          measured.isEmpty
              ? s.notMeasured
              : s.weakLine(weakM2, measuredM2, st?.weakWhere),
          style: jText(false, size: 13.5, weight: FontWeight.w600),
        ),
      const Divider(height: 8, color: JColors.cardLine),
      Text(
        s.thisWeek,
        style: latText(size: 11, weight: FontWeight.w800, color: JColors.faint),
      ),
      if (_plan != null) WeekStrip(plan: _plan!),
      if (_planDown)
        Text(
          s.weatherDown,
          style: jText(false, size: 12.5, color: JColors.levelAlarm),
        ),
    ];
  }

  Widget? _overlay(BuildContext context, FarmShape shape) {
    final s = AppScope.of(context).s;
    final st = _status;
    final whole = st?.greennessPctOfNormal;
    final sum = _farm!.summary;
    switch (_view) {
      case FarmView.farm:
        final measuredDunam = (st?.crops ?? const <CropReading>[])
            .where((c) => c.greennessPctOfNormal != null)
            .fold(0.0, (a, c) => a + c.dunam);
        final all = measuredDunam >= sum.areaDunam * 0.95;
        return IgnorePointer(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            spacing: 2,
            children: [
              Text(
                whole == null ? s.notMeasured : s.pctOfNormal(whole),
                style: jText(
                  false,
                  size: whole == null ? 18 : 26,
                  weight: FontWeight.w800,
                ),
              ),
              Text(
                [
                  s.wholeFarmLabel,
                  '${fmtM2(sum.areaDunam * 2500)} ${s.m2}',
                  if (whole != null) s.levelName(levelFromPct(whole)),
                ].join(' · '),
                style: jText(false, size: 12.5, color: JColors.muted),
              ),
              if (whole != null && !all)
                Text(
                  s.measuredOn(
                    fmtM2(measuredDunam * 2500),
                    fmtM2(sum.areaDunam * 2500),
                  ),
                  style: jText(false, size: 12, color: JColors.muted),
                ),
            ],
          ),
        );
      case FarmView.cells:
        final k = _cell;
        if (k == null) return null;
        final r = shape.reading[k];
        final crop = shape.crop[k] ?? 'empty';
        final days = r?.since == null
            ? null
            : DateTime.now().difference(DateTime.parse(r!.since!)).inDays;
        return CellCard(
          title:
              '${s.cellName(shape.label(k))} · ${cropOf(crop).emoji} ${s.crop(crop)}',
          level: r?.level ?? FarmStatus.none,
          pct: r?.greennessPct,
          emptyText: st?.pictureDate == null ? s.noReading : s.cloudOrNotSown,
          lines: [
            if (days != null) s.sinceLine(fmtDay(r!.since!), days),
            if (r?.greennessPct != null)
              s.compareLine(shape.neighbours(k), whole),
          ],
          onClose: () => setState(() => _cell = null),
          onAsk: () => showToast(context, '${s.askSpot}: ${s.notBuilt}'),
          onReport: () => showToast(context, '${s.reportHere}: ${s.notBuilt}'),
        );
      case FarmView.crops:
        final c = _crop;
        if (c == null) return null;
        final r = (st?.crops ?? const <CropReading>[])
            .where((x) => x.crop == c)
            .firstOrNull;
        return CellCard(
          title:
              '${cropOf(c).emoji} ${s.crop(c)} · ${fmtM2((r?.dunam ?? 0) * 2500)} ${s.m2}',
          level: r?.level ?? FarmStatus.none,
          pct: r?.greennessPctOfNormal,
          emptyText: st?.pictureDate == null ? s.noReading : s.notSownCap,
          lines: [
            if (r?.greennessPctOfNormal != null) s.compareLine(null, whole),
          ],
          onClose: () => setState(() => _crop = null),
        );
    }
  }
}

class _ViewToggle extends StatelessWidget {
  const _ViewToggle({required this.view, required this.onChanged});
  final FarmView view;
  final ValueChanged<FarmView> onChanged;

  @override
  Widget build(BuildContext context) {
    final s = AppScope.of(context).s;
    return Container(
      padding: const EdgeInsets.all(3),
      decoration: BoxDecoration(
        color: JColors.toggleBg,
        borderRadius: BorderRadius.circular(11),
      ),
      child: Row(
        children: [
          for (final v in FarmView.values)
            Expanded(
              child: GestureDetector(
                onTap: () => onChanged(v),
                child: AnimatedContainer(
                  duration: const Duration(milliseconds: 180),
                  height: 32,
                  alignment: Alignment.center,
                  decoration: BoxDecoration(
                    color: v == view ? JColors.card : Colors.transparent,
                    borderRadius: BorderRadius.circular(8),
                    boxShadow: v == view
                        ? const [
                            BoxShadow(
                              color: Color(0x1A000000),
                              blurRadius: 3,
                              offset: Offset(0, 1),
                            ),
                          ]
                        : null,
                  ),
                  child: Text(
                    switch (v) {
                      FarmView.cells => s.viewCells,
                      FarmView.crops => s.viewCrops,
                      FarmView.farm => s.viewFarm,
                    },
                    style: jText(
                      false,
                      size: 13.5,
                      weight: v == view ? FontWeight.w700 : FontWeight.w500,
                      color: v == view ? JColors.ink : JColors.muted,
                    ),
                  ),
                ),
              ),
            ),
        ],
      ),
    );
  }
}

class _Legend extends StatelessWidget {
  const _Legend();

  @override
  Widget build(BuildContext context) {
    final s = AppScope.of(context).s;
    return Wrap(
      spacing: 14,
      runSpacing: 6,
      children: [
        for (final l in [
          FarmStatus.normal,
          FarmStatus.watch,
          FarmStatus.alarm,
          FarmStatus.none,
        ])
          Row(
            mainAxisSize: MainAxisSize.min,
            spacing: 5,
            children: [
              Container(
                width: 10,
                height: 10,
                decoration: BoxDecoration(
                  color: levelColor(l),
                  borderRadius: BorderRadius.circular(2),
                ),
              ),
              Text(
                s.levelName(l),
                style: jText(false, size: 12, color: JColors.muted),
              ),
            ],
          ),
      ],
    );
  }
}

class _CropList extends StatelessWidget {
  const _CropList({required this.status, required this.farm});
  final FarmStatusReport? status;
  final Farm farm;

  @override
  Widget build(BuildContext context) {
    final s = AppScope.of(context).s;
    final rows =
        status?.crops ??
        [
          for (final c in farm.summary.crops)
            CropReading(crop: c.crop, dunam: c.dunam, level: FarmStatus.none),
        ];
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      spacing: 6,
      children: [
        for (final c in rows)
          Row(
            spacing: 8,
            children: [
              Container(
                width: 10,
                height: 10,
                decoration: BoxDecoration(
                  color: cropOf(c.crop).color,
                  borderRadius: BorderRadius.circular(2),
                ),
              ),
              Text(cropOf(c.crop).emoji, style: const TextStyle(fontSize: 13)),
              Expanded(
                child: Text(
                  [
                    '${s.crop(c.crop)} ${fmtM2(c.dunam * 2500)} ${s.m2}',
                    c.greennessPctOfNormal != null
                        ? s.pctOfNormal(c.greennessPctOfNormal!)
                        : status?.pictureDate == null
                        ? s.noReadingYet
                        : s.notSownYet,
                    if (c.greennessPctOfNormal != null) s.levelName(c.level),
                  ].join(' · '),
                  style: jText(false, size: 13, color: JColors.ink),
                ),
              ),
            ],
          ),
      ],
    );
  }
}

class _Banner extends StatelessWidget {
  const _Banner({required this.text});
  final String text;

  @override
  Widget build(BuildContext context) => Container(
    width: double.infinity,
    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
    decoration: BoxDecoration(
      color: JColors.levelWatchSoft,
      borderRadius: BorderRadius.circular(10),
    ),
    child: Row(
      spacing: 8,
      children: [
        const Icon(Icons.cloud_off_rounded, size: 16, color: JColors.gold),
        Expanded(
          child: Text(
            text,
            style: jText(false, size: 12.5, weight: FontWeight.w600),
          ),
        ),
      ],
    ),
  );
}

class _ErrorBox extends StatelessWidget {
  const _ErrorBox({
    required this.text,
    required this.retry,
    required this.onRetry,
  });
  final String text;
  final String retry;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 24),
    child: Column(
      spacing: 10,
      children: [
        Text(
          text,
          textAlign: TextAlign.center,
          style: jText(false, size: 14, color: JColors.levelAlarm),
        ),
        OutlinedButton(onPressed: onRetry, child: Text(retry)),
      ],
    ),
  );
}
