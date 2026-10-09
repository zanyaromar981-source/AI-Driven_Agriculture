import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_map/flutter_map.dart';
import 'package:geolocator/geolocator.dart';
import 'package:latlong2/latlong.dart' show LatLng;
import 'package:wakelock_plus/wakelock_plus.dart';

import '../../api/api.dart';
import '../../app_scope.dart';
import '../../config.dart';
import '../../geo.dart';
import '../../store/draft.dart';
import '../../theme.dart';
import '../../widgets/common.dart';
import '../../widgets/farm_map.dart';
import 'farm_edit.dart';
import 'paint_screen.dart';

/// Add farm, step 1 of 3: mark the field edge, either by tapping "Dot" at each
/// corner, or by walking the edge while the app places the dots.
class CornersScreen extends StatefulWidget {
  const CornersScreen({super.key, this.edit});

  /// Set when an existing farm is being edited: start from its border.
  final FarmEdit? edit;

  @override
  State<CornersScreen> createState() => _CornersScreenState();
}

enum _GpsIssue { serviceOff, denied, deniedForever }

/// One GPS reading on a walk: where, how good, when, and in UTM metres.
class _TrackPt {
  _TrackPt(this.point, this.x, this.y);
  final GeoPoint point;
  final double x;
  final double y;
}

class _CornersScreenState extends State<CornersScreen>
    with WidgetsBindingObserver {
  static const _slemani = LatLng(35.5613, 45.4374);
  static const _maxDunam = 1000;
  static const _maxCorners = 50;

  /// Walk mode tuning: skip bad fixes and standing jitter, close when back at the start.
  static const _maxAccM = 25.0;
  static const _minStepM = 2.0;
  static const _closeAfterM = 40.0;
  static const _closeWithinM = 10.0;

  final _map = MapController();
  final List<GeoPoint> _points = [];
  StreamSubscription<Position>? _sub;
  Position? _pos;
  String? _gpsProblem;
  _GpsIssue? _gpsIssue;
  bool _mapReady = false;
  bool _centred = false;

  bool _walkMode = false;
  bool _recording = false;

  /// Crops painted on the next screen, kept when the farmer comes back here
  /// to move a corner, so the painting is not lost.
  Map<CellKey, String>? _painted;
  final List<_TrackPt> _track = [];
  double _walkedM = 0;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    final edit = widget.edit;
    if (edit != null) {
      _points.addAll(edit.points);
      _centred = true;
    } else {
      _restoreDraft();
    }
    _startGps();
  }

  /// An edge marked earlier (app closed, battery died) comes back.
  Future<void> _restoreDraft() async {
    final d = await Draft.load();
    if (d == null || !mounted || _points.isNotEmpty) return;
    setState(() {
      _points.addAll(d.points);
      _walkMode = d.walk;
    });
    _centred = true;
    if (_mapReady) {
      WidgetsBinding.instance.addPostFrameCallback((_) => _showAllDots());
    }
    showToast(context, AppScope.read(context).s.draftBack);
  }

  /// Fit the map to every dot (used when a saved edge comes back).
  void _showAllDots() {
    if (_points.length < 2) {
      if (_points.isNotEmpty) _map.move(_ll(_points.first), 18);
      return;
    }
    _map.fitCamera(
      CameraFit.bounds(
        bounds: LatLngBounds.fromPoints([for (final p in _points) _ll(p)]),
        padding: const EdgeInsets.fromLTRB(40, 90, 40, 120),
        maxZoom: 19,
      ),
    );
  }

  /// The draft is for a new farm only; an edit starts from the saved farm.
  void _saveDraft() {
    if (widget.edit == null) Draft.save(_points, walk: _walkMode);
  }

  /// Drag a dot to move it (not while a walk is being recorded).
  int? _dragging;

  /// The dot under [local] (screen position on the map), if any.
  int? _dotAt(Offset local) {
    if (!_mapReady) return null;
    final cam = _map.camera;
    int? best;
    var bestD = 28.0;
    for (final (i, p) in _points.indexed) {
      final d = (cam.latLngToScreenOffset(_ll(p)) - local).distance;
      if (d < bestD) {
        bestD = d;
        best = i;
      }
    }
    return best;
  }

  void _endDrag() {
    if (_dragging == null) return;
    setState(() => _dragging = null);
    _saveDraft();
  }

  void _moveDot(int i, Offset delta) {
    if (i >= _points.length) return;
    final cam = _map.camera;
    final at = cam.latLngToScreenOffset(_ll(_points[i])) + delta;
    final ll = cam.screenOffsetToLatLng(at);
    setState(
      () => _points[i] = GeoPoint(
        lat: ll.latitude,
        lon: ll.longitude,
        accM: 0,
        t: DateTime.now(),
      ),
    );
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    _sub?.cancel();
    if (_recording) WakelockPlus.disable();
    super.dispose();
  }

  /// Coming back from the phone's settings: try GPS again.
  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed && _sub == null) {
      _startGps(ask: false);
    }
  }

  void _setIssue(_GpsIssue issue) {
    if (!mounted) return;
    final s = AppScope.read(context).s;
    setState(() {
      _gpsIssue = issue;
      _gpsProblem = issue == _GpsIssue.serviceOff ? s.gpsOff : s.gpsDenied;
    });
  }

  Future<void> _startGps({bool ask = true}) async {
    if (_sub != null) return;
    if (!await Geolocator.isLocationServiceEnabled()) {
      return _setIssue(_GpsIssue.serviceOff);
    }
    var perm = await Geolocator.checkPermission();
    if (perm == LocationPermission.denied && ask) {
      perm = await Geolocator.requestPermission();
    }
    if (perm == LocationPermission.deniedForever) {
      return _setIssue(_GpsIssue.deniedForever);
    }
    if (perm == LocationPermission.denied) return _setIssue(_GpsIssue.denied);
    if (_sub != null || !mounted) return;
    setState(() => _gpsIssue = null);
    final settings = defaultTargetPlatform == TargetPlatform.android
        ? AndroidSettings(
            accuracy: LocationAccuracy.best,
            distanceFilter: 0,
            intervalDuration: const Duration(seconds: 1),
          )
        : const LocationSettings(
            accuracy: LocationAccuracy.best,
            distanceFilter: 0,
          );
    _sub = Geolocator.getPositionStream(locationSettings: settings).listen(
      onError: (_) {
        _sub?.cancel();
        _sub = null;
      },
      _onPosition,
    );
  }

  void _onPosition(Position p) {
    if (!mounted) return;
    setState(() {
      _pos = p;
      _gpsProblem = null;
    });
    _centreOnce();
    if (_recording) _record(p);
  }

  void _centreOnce() {
    if (_centred || !_mapReady || _pos == null || _points.isNotEmpty) return;
    _centred = true;
    _map.move(LatLng(_pos!.latitude, _pos!.longitude), 18);
  }

  /// Ask again, or open the right settings page when Android will not ask any more.
  Future<void> _fixGps() async {
    switch (_gpsIssue) {
      case _GpsIssue.serviceOff:
        await Geolocator.openLocationSettings();
      case _GpsIssue.deniedForever:
        await Geolocator.openAppSettings();
      case _GpsIssue.denied:
        await _startGps();
      case null:
        break;
    }
  }

  GeoPoint _geo(Position p) => GeoPoint(
    lat: p.latitude,
    lon: p.longitude,
    accM: p.accuracy.round(),
    t: DateTime.now(),
  );

  bool _needGps() {
    if (_pos != null) return false;
    showToast(context, _gpsProblem ?? AppScope.read(context).s.waitingGps);
    _fixGps();
    return true;
  }

  // ---------- tap mode ----------

  void _addGpsDot() {
    if (_needGps()) return;
    setState(() => _points.add(_geo(_pos!)));
    _saveDraft();
  }

  void _addTapped(LatLng ll) {
    if (_recording) return;
    setState(
      () => _points.add(
        GeoPoint(
          lat: ll.latitude,
          lon: ll.longitude,
          accM: 0,
          t: DateTime.now(),
        ),
      ),
    );

    _saveDraft();
  }

  void _undo() {
    if (_points.isNotEmpty && !_recording) {
      setState(() => _points.removeLast());
      _saveDraft();
    }
  }

  // ---------- walk mode ----------

  void _startWalk() {
    if (_needGps()) return;
    WakelockPlus.enable();
    setState(() {
      _points.clear();
      _track.clear();
      _walkedM = 0;
      _recording = true;
    });
    _record(_pos!);
  }

  void _record(Position p) {
    if (p.accuracy > _maxAccM) return;
    final (x, y) = Utm.fromLatLng(p.latitude, p.longitude);
    if (_track.isNotEmpty) {
      final last = _track.last;
      final step = math.sqrt(
        (x - last.x) * (x - last.x) + (y - last.y) * (y - last.y),
      );
      if (step < _minStepM) return;
      _walkedM += step;
    }
    _track.add(_TrackPt(_geo(p), x, y));
    _map.move(LatLng(p.latitude, p.longitude), _map.camera.zoom);
    final first = _track.first;
    final home = math.sqrt(
      (x - first.x) * (x - first.x) + (y - first.y) * (y - first.y),
    );
    if (_walkedM >= _closeAfterM && home <= _closeWithinM) {
      _stopWalk(closed: true);
    } else {
      setState(_refreshDots);
      _saveDraft();
    }
  }

  /// Dots = the walked track with straight stretches trimmed (at most 50).
  void _refreshDots() {
    final keep = simplifyTrack([for (final t in _track) (t.x, t.y)]);
    _points
      ..clear()
      ..addAll([for (final i in keep) _track[i].point]);
  }

  void _stopWalk({bool closed = false}) {
    WakelockPlus.disable();
    setState(() {
      _recording = false;
      _refreshDots();
      // The outline closes itself, so a last dot right next to the first one is not needed.
      if (_points.length > 3 &&
          metresBetween(_ll(_points.first), _ll(_points.last)) <
              _closeWithinM) {
        _points.removeLast();
      }
    });
    if (closed) showToast(context, AppScope.read(context).s.shapeClosed);

    _saveDraft();
  }

  LatLng _ll(GeoPoint p) => LatLng(p.lat, p.lon);

  void _setMode(bool walk) {
    if (_recording || walk == _walkMode) return;
    setState(() => _walkMode = walk);
    _saveDraft();
  }

  // ---------- next ----------

  void _close() {
    final s = AppScope.read(context).s;
    if (_recording) return showToast(context, s.walkFirst);
    final outline = [for (final p in _points) _ll(p)];
    if (outline.length < 3) return showToast(context, s.needThree);
    // The same limits as the server (BACKEND.md 2.2), so a farm is never
    // refused after the farmer has walked and painted it.
    if (outline.length > _maxCorners) return showToast(context, s.tooManyCorners);
    if (selfIntersects(outline)) return showToast(context, s.crosses);
    if (polygonAreaM2(outline) / 2500 > _maxDunam) {
      return showToast(context, s.tooBig);
    }
    if (cellsInside(outline).isEmpty) return showToast(context, s.tooSmall);
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => PaintScreen(
          points: List.of(_points),
          edit: widget.edit,
          crops: _painted,
          onCrops: (c) => _painted = c,
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    final s = scope.s;
    final ku = scope.ku;
    final dots = [for (final p in _points) _ll(p)];
    final you = _pos == null ? null : LatLng(_pos!.latitude, _pos!.longitude);
    final path = [for (final t in _track) _ll(t.point)];

    final Widget centreButton = !_walkMode
        ? _BigButton(icon: Icons.add_rounded, label: s.dot, onTap: _addGpsDot)
        : _recording
        ? _BigButton(
            icon: Icons.stop_rounded,
            label: s.stop,
            onTap: _stopWalk,
            colour: JColors.levelAlarm,
          )
        : _BigButton(
            icon: Icons.directions_walk_rounded,
            label: s.start,
            onTap: _startWalk,
          );
    final String? countText = _recording
        ? s.walked(_walkedM.round(), _points.length)
        : _points.isEmpty
        ? null
        : _points.length < 3
        ? s.corners(_points.length)
        : '${s.corners(_points.length)} · '
              '\u2066${fmtM2(polygonAreaM2(dots))}\u2069 ${s.m2}';

    return MapPage(
      step: 1,
      heading: MapHeading(
        ku: ku,
        title: widget.edit == null ? s.cornersTitle : s.editTitle,
        sub: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          spacing: 10,
          children: [
            Text(
              _walkMode
                  ? s.walkSub
                  : widget.edit != null
                  ? s.dragHint
                  : s.cornersSub,
              style: jText(ku, size: 15, color: JColors.muted),
            ),
            _ModeToggle(
              walk: _walkMode,
              locked: _recording,
              onChanged: _setMode,
            ),
          ],
        ),
      ),
      map: Stack(
        children: [
          StyledMap(
            builder: (context, style) => FlutterMap(
              mapController: _map,
              options: MapOptions(
                initialCenter: _slemani,
                initialZoom: 16,
                // Editing: start already framed on the farm, so its map
                // pictures load straight away.
                initialCameraFit: widget.edit == null
                    ? null
                    : CameraFit.bounds(
                        bounds: LatLngBounds.fromPoints([
                          for (final p in widget.edit!.points) _ll(p),
                        ]),
                        padding: const EdgeInsets.fromLTRB(40, 90, 40, 120),
                        maxZoom: 19,
                      ),
                minZoom: 5,
                maxZoom: 20,
                backgroundColor: const Color(0xFF717A50),
                // While a finger holds a dot the map stays still.
                interactionOptions: InteractionOptions(
                  flags: _dragging != null
                      ? InteractiveFlag.none
                      : InteractiveFlag.all & ~InteractiveFlag.rotate,
                ),
                onTap: kTestMode ? (_, ll) => _addTapped(ll) : null,
                onMapReady: () {
                  _mapReady = true;
                  if (_points.isNotEmpty) {
                    if (widget.edit == null) {
                      WidgetsBinding.instance.addPostFrameCallback(
                        (_) => _showAllDots(),
                      );
                    }
                  } else {
                    _centreOnce();
                  }
                },
              ),
              children: [
                ...baseLayers(style, ku),
                if (path.length >= 2)
                  PolylineLayer(
                    polylines: [
                      Polyline(
                        points: path,
                        color: Colors.white.withValues(alpha: 0.75),
                        strokeWidth: 2,
                      ),
                    ],
                  ),
                if (dots.length >= 2)
                  PolylineLayer(
                    polylines: [
                      Polyline(
                        points: dots,
                        color: const Color(0xFF3FD08F),
                        strokeWidth: 3,
                        pattern: StrokePattern.dashed(segments: const [10, 6]),
                      ),
                    ],
                  ),
                if (dots.length >= 3 && !_recording)
                  PolylineLayer(
                    polylines: [
                      Polyline(
                        points: [dots.last, dots.first],
                        color: Colors.white.withValues(alpha: 0.67),
                        strokeWidth: 2,
                        pattern: StrokePattern.dashed(segments: const [6, 6]),
                      ),
                    ],
                  ),
                if (you != null)
                  CircleLayer(
                    circles: [
                      CircleMarker(
                        point: you,
                        radius: _pos!.accuracy.clamp(3, 200).toDouble(),
                        useRadiusInMeter: true,
                        color: const Color(0x2E4A9DFF),
                        borderColor: const Color(0xFF9CCBFF),
                        borderStrokeWidth: 1,
                      ),
                    ],
                  ),
                MarkerLayer(
                  markers: [
                    for (final (i, d) in dots.indexed)
                      Marker(
                        point: d,
                        width: 30,
                        height: 30,
                        child: IgnorePointer(
                          child: Center(
                            child: AnimatedContainer(
                              duration: const Duration(milliseconds: 120),
                              width: _dragging == i ? 26 : 16,
                              height: _dragging == i ? 26 : 16,
                              decoration: BoxDecoration(
                                color: Colors.white,
                                shape: BoxShape.circle,
                                border: Border.all(
                                  color: JColors.accent,
                                  width: 3,
                                ),
                              ),
                            ),
                          ),
                        ),
                      ),
                    if (you != null)
                      Marker(
                        point: you,
                        width: 18,
                        height: 18,
                        child: Container(
                          decoration: BoxDecoration(
                            color: const Color(0xFF1F7BEF),
                            shape: BoxShape.circle,
                            border: Border.all(color: Colors.white, width: 2.5),
                          ),
                        ),
                      ),
                  ],
                ),
                // A finger that lands on a dot moves that dot; any other
                // touch still pans and zooms the map.
                Listener(
                  behavior: HitTestBehavior.translucent,
                  onPointerDown: (e) {
                    if (_recording || _dragging != null) return;
                    final i = _dotAt(e.localPosition);
                    if (i != null) setState(() => _dragging = i);
                  },
                  onPointerMove: (e) {
                    final i = _dragging;
                    if (i != null) _moveDot(i, e.delta);
                  },
                  onPointerUp: (_) => _endDrag(),
                  onPointerCancel: (_) => _endDrag(),
                  child: const SizedBox.expand(),
                ),
                mapAttribution(style),
              ],
            ),
          ),
          Positioned(
            top: 12,
            left: 12,
            right: 12,
            child: Directionality(
              textDirection: TextDirection.ltr,
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                spacing: 8,
                children: [
                  if (kTestMode && !_walkMode)
                    Flexible(
                      child: _Chip(
                        child: Text(
                          s.testTap,
                          textDirection: ku
                              ? TextDirection.rtl
                              : TextDirection.ltr,
                          style: jText(ku, size: 11, weight: FontWeight.w600),
                        ),
                      ),
                    ),
                  Flexible(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.end,
                      spacing: 8,
                      children: [
                        GestureDetector(
                          onTap: _gpsIssue == null ? null : _fixGps,
                          child: _GpsChip(
                            accuracy: _pos?.accuracy,
                            problem: _gpsProblem,
                          ),
                        ),
                        const MapStyleButton(),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          ),
          Positioned(
            left: 0,
            right: 0,
            bottom: 16,
            child: Directionality(
              textDirection: TextDirection.ltr,
              child: Row(
                children: [
                  Expanded(
                    child: Align(
                      alignment: Alignment.centerRight,
                      child: Padding(
                        padding: const EdgeInsets.only(right: 14),
                        child: _RoundAction(
                          icon: Icons.undo_rounded,
                          label: s.undo,
                          onTap: _points.isEmpty || _recording ? null : _undo,
                        ),
                      ),
                    ),
                  ),
                  centreButton,
                  Expanded(
                    child: Align(
                      alignment: Alignment.centerLeft,
                      child: Padding(
                        padding: const EdgeInsets.only(left: 14),
                        child: countText == null
                            ? const SizedBox.shrink()
                            : _Chip(
                                child: Text(
                                  countText,
                                  textDirection: ku
                                      ? TextDirection.rtl
                                      : TextDirection.ltr,
                                  style: jText(
                                    ku,
                                    size: 12,
                                    weight: FontWeight.w700,
                                  ),
                                ),
                              ),
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
      below: [
        PrimaryButton(
          label: s.closeShape,
          icon: Icons.check_rounded,
          onPressed: _close,
        ),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          spacing: 6,
          children: [
            const Icon(Icons.wifi_off_rounded, size: 15, color: JColors.muted),
            Text(
              s.offlineHint,
              style: jText(ku, size: 13, color: JColors.muted),
            ),
          ],
        ),
      ],
    );
  }
}

/// "Tap corners" | "Walk the edge".
class _ModeToggle extends StatelessWidget {
  const _ModeToggle({
    required this.walk,
    required this.locked,
    required this.onChanged,
  });
  final bool walk;
  final bool locked;
  final ValueChanged<bool> onChanged;

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    Widget option(bool isWalk, IconData icon, String label) {
      final on = walk == isWalk;
      return Expanded(
        child: GestureDetector(
          onTap: locked ? null : () => onChanged(isWalk),
          behavior: HitTestBehavior.opaque,
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 180),
            height: 36,
            decoration: BoxDecoration(
              color: on ? JColors.card : Colors.transparent,
              borderRadius: BorderRadius.circular(999),
              boxShadow: on
                  ? const [
                      BoxShadow(
                        color: Color(0x1A000000),
                        blurRadius: 3,
                        offset: Offset(0, 1),
                      ),
                    ]
                  : const [],
            ),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              spacing: 6,
              children: [
                Icon(
                  icon,
                  size: 16,
                  color: on ? JColors.accent : JColors.muted,
                ),
                Text(
                  label,
                  style: jText(
                    scope.ku,
                    size: 13,
                    weight: on ? FontWeight.w700 : FontWeight.w500,
                    color: on ? JColors.ink : JColors.muted,
                  ),
                ),
              ],
            ),
          ),
        ),
      );
    }

    return Opacity(
      opacity: locked ? 0.5 : 1,
      child: Container(
        padding: const EdgeInsets.all(3),
        decoration: BoxDecoration(
          color: JColors.toggleBg,
          borderRadius: BorderRadius.circular(999),
        ),
        child: Row(
          spacing: 2,
          children: [
            option(false, Icons.touch_app_outlined, scope.s.modeTap),
            option(true, Icons.directions_walk_rounded, scope.s.modeWalk),
          ],
        ),
      ),
    );
  }
}

class _Chip extends StatelessWidget {
  const _Chip({required this.child});
  final Widget child;

  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
    decoration: BoxDecoration(
      color: Colors.white.withValues(alpha: 0.92),
      borderRadius: BorderRadius.circular(999),
      boxShadow: const [
        BoxShadow(
          color: Color(0x22000000),
          blurRadius: 4,
          offset: Offset(0, 1),
        ),
      ],
    ),
    child: child,
  );
}

class _GpsChip extends StatelessWidget {
  const _GpsChip({required this.accuracy, required this.problem});
  final double? accuracy;
  final String? problem;

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    final a = accuracy;
    final colour = a == null
        ? JColors.levelAlarm
        : a <= 10
        ? const Color(0xFF2BB673)
        : a <= 25
        ? JColors.levelWatch
        : JColors.levelAlarm;
    return _Chip(
      child: Row(
        mainAxisSize: MainAxisSize.min,
        spacing: 6,
        children: [
          Container(
            width: 8,
            height: 8,
            decoration: BoxDecoration(color: colour, shape: BoxShape.circle),
          ),
          if (a != null)
            Text(
              'GPS ±${a.round()} ${scope.s.metres}',
              textDirection: TextDirection.ltr,
              style: latText(size: 13, weight: FontWeight.w700),
            )
          else
            Flexible(
              child: Text(
                problem ?? scope.s.waitingGps,
                textDirection: scope.ku ? TextDirection.rtl : TextDirection.ltr,
                style: jText(scope.ku, size: 12, weight: FontWeight.w600),
              ),
            ),
        ],
      ),
    );
  }
}

/// The big round button in the middle: Dot, Start or Stop.
class _BigButton extends StatelessWidget {
  const _BigButton({
    required this.icon,
    required this.label,
    required this.onTap,
    this.colour = JColors.accent,
  });
  final IconData icon;
  final String label;
  final VoidCallback onTap;
  final Color colour;

  @override
  Widget build(BuildContext context) {
    final ku = AppScope.of(context).ku;
    return Material(
      color: colour,
      shape: const CircleBorder(
        side: BorderSide(color: Colors.white, width: 3),
      ),
      elevation: 4,
      child: InkWell(
        customBorder: const CircleBorder(),
        onTap: onTap,
        child: SizedBox(
          width: 80,
          height: 80,
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(icon, size: 28, color: Colors.white),
              Text(
                label,
                style: jText(
                  ku,
                  size: 14,
                  weight: FontWeight.w700,
                  color: Colors.white,
                  height: 1.1,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _RoundAction extends StatelessWidget {
  const _RoundAction({required this.icon, required this.label, this.onTap});
  final IconData icon;
  final String label;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final ku = AppScope.of(context).ku;
    return Opacity(
      opacity: onTap == null ? 0.5 : 1,
      child: Material(
        color: Colors.white.withValues(alpha: 0.92),
        borderRadius: BorderRadius.circular(999),
        child: InkWell(
          borderRadius: BorderRadius.circular(999),
          onTap: onTap,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 9),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              spacing: 6,
              children: [
                Icon(icon, size: 16, color: JColors.ink),
                Text(
                  label,
                  style: jText(ku, size: 12, weight: FontWeight.w700),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
