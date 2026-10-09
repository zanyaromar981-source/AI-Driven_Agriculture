import 'dart:math';

import 'package:flutter/foundation.dart';

import '../api/api.dart';
import 'draft.dart';
import 'insights_copy.dart';
import 'local_store.dart';

/// A farm saved on the phone, waiting to be sent to the server.
class PendingFarm {
  const PendingFarm({
    required this.key,
    required this.request,
    required this.summary,
    required this.queuedAt,
    this.farmId,
  });

  /// Sent as `Idempotency-Key`, so a retry never makes a second farm.
  final String key;
  final NewFarmRequest request;

  /// What My farms shows until the server has it.
  final FarmSummary summary;
  final DateTime queuedAt;

  /// Set when this is a change to an existing farm (PUT), null for a new farm.
  final String? farmId;

  Map<String, dynamic> toJson() => {
    'key': key,
    'request': request.toJson(),
    'summary': summary.toJson(),
    'queued_at': queuedAt.toUtc().toIso8601String(),
    if (farmId != null) 'farm_id': farmId,
  };

  factory PendingFarm.fromJson(Map<String, dynamic> j) => PendingFarm(
    key: j['key'] as String,
    request: NewFarmRequest.fromJson(j['request'] as Map<String, dynamic>),
    summary: FarmSummary.fromJson(j['summary'] as Map<String, dynamic>),
    queuedAt: DateTime.parse(j['queued_at'] as String),
    farmId: j['farm_id'] as String?,
  );
}

/// A farm the server will not take, with the server's reason
/// (`bad_polygon`, `farm_too_large`, `too_many_farms`, ...).
class Refused {
  const Refused(this.item, this.code, {this.edgeBack = false});
  final PendingFarm item;
  final String code;

  /// The farm's walked edge was put back as the Add farm draft.
  final bool edgeBack;
}

/// What one upload try did.
class FlushResult {
  const FlushResult({
    this.sent = 0,
    this.rejected = const [],
    this.offline = false,
  });
  final int sent;
  final List<Refused> rejected;
  final bool offline;
}

/// Farms are written here first, then uploaded. Anything that cannot be sent
/// (no internet) stays on the phone and is tried again later.
class Outbox extends ChangeNotifier {
  Outbox._();
  static final instance = Outbox._();

  static const _name = 'outbox';
  final List<PendingFarm> _items = [];

  /// Farms deleted on the phone, waiting to be deleted on the server.
  final List<String> _deletes = [];

  /// An edited farm is saved as a new farm (the server has no edit call yet,
  /// and a new farm gets its 20-year analysis again): old id -> new id.
  final Map<String, String> _replaced = {};

  /// The farm that replaced [id] after an edit (follows chains of edits).
  String? replacedBy(String id) {
    var cur = _replaced[id];
    for (var i = 0; i < 20 && cur != null && _replaced[cur] != null; i++) {
      cur = _replaced[cur];
    }
    return cur;
  }

  List<String> get deletes => List.unmodifiable(_deletes);
  bool _loaded = false;
  bool _flushing = false;

  /// Uploads and refusals not yet shown to the farmer (see [takeNews]).
  int _newSent = 0;
  final List<Refused> _newRefused = [];

  /// What went up or was refused since the last call, so My farms tells the
  /// farmer once, with the real count and reason.
  ({int sent, List<Refused> refused}) takeNews() {
    final news = (sent: _newSent, refused: List.of(_newRefused));
    _newSent = 0;
    _newRefused.clear();
    return news;
  }

  List<PendingFarm> get items => List.unmodifiable(_items);

  Future<void> load() async {
    if (_loaded) return;
    _loaded = true;
    final j = await LocalStore.read(_name);
    (j?['replaced'] as Map?)?.forEach(
      (k, v) => _replaced[k as String] = v as String,
    );
    _deletes.addAll([
      for (final d in (j?['deletes'] as List? ?? const [])) d as String,
    ]);
    for (final e in (j?['items'] as List? ?? const [])) {
      try {
        _items.add(PendingFarm.fromJson(e as Map<String, dynamic>));
      } catch (_) {}
    }
    notifyListeners();
  }

  Future<void> _persist() => LocalStore.write(_name, {
    'items': [for (final i in _items) i.toJson()],
    'deletes': _deletes,
    'replaced': _replaced,
  });

  static String newKey() {
    final r = Random.secure();
    return '${DateTime.now().microsecondsSinceEpoch.toRadixString(36)}-'
        '${List.generate(8, (_) => r.nextInt(36).toRadixString(36)).join()}';
  }

  /// Delete a farm: any waiting change to it is dropped, and the delete goes
  /// to the server now or when there is internet.
  Future<void> delete(String farmId) async {
    await load();
    _items.removeWhere((i) => i.farmId == farmId);
    if (!_deletes.contains(farmId)) _deletes.add(farmId);
    await _persist();
    notifyListeners();
  }

  /// Queue a new farm, or with [farmId] a change to an existing farm. A newer
  /// change replaces an older one of the same farm that is still waiting.
  Future<PendingFarm> add(
    NewFarmRequest request,
    FarmSummary summary, {
    String? farmId,
  }) async {
    await load();
    if (farmId != null) _items.removeWhere((i) => i.farmId == farmId);
    final item = PendingFarm(
      farmId: farmId,
      key: newKey(),
      request: request,
      summary: summary,
      queuedAt: DateTime.now(),
    );
    _items.add(item);
    await _persist();
    notifyListeners();
    return item;
  }

  /// Send everything waiting: deletes first, then new farms and edits, oldest
  /// first. Stops at the first sign that the server cannot be reached now.
  Future<FlushResult> flush(Api api) async {
    await load();
    if (_flushing || (_items.isEmpty && _deletes.isEmpty)) {
      return const FlushResult();
    }
    _flushing = true;
    var sent = 0;
    var changed = false;
    final rejected = <Refused>[];
    var offline = false;
    // Our server answers a refusal with its JSON error code. A Wi-Fi login
    // page or a used-up data bundle answers with a redirect, a 403 or a web
    // page instead: that is not our server, so nothing is decided by it.
    bool fromServer(ApiException e) =>
        e.extra != null ||
        !(const {
              'bad_request',
              'not_found',
              'invalid',
              'bad_response',
            }).contains(e.code) &&
            !e.code.startsWith('http_');
    // Keep and try later: no internet, not signed in, timeout, too many
    // requests (429), a server error, or an answer that is not our server.
    // Losing a walked farm because the server was busy would be far worse
    // than waiting.
    // 405/501: the server does not have this call yet (e.g. editing a
    // farm's border before the backend adds PUT /farms/{id}): keep it too.
    bool later(ApiException e) =>
        e.isOffline ||
        e.status < 400 ||
        const {401, 403, 405, 408, 429, 501}.contains(e.status) ||
        e.status >= 500 ||
        !fromServer(e);
    try {
      for (final id in List.of(_deletes)) {
        try {
          await api.deleteFarm(id);
        } on ApiException catch (e) {
          // 404 is "already gone", whatever sent it: deleting is done.
          if (e.status != 404 && later(e)) {
            offline = e.isOffline;
            return FlushResult(
              sent: sent,
              rejected: rejected,
              offline: offline,
            );
          }
          // Any other refusal will not change on retry.
        }
        _deletes.remove(id);
        changed = true;
      }
      for (final item in List.of(_items)) {
        try {
          final oldId = item.farmId;
          final made = await api.createFarm(
            item.request,
            idempotencyKey: item.key,
          );
          if (oldId != null) {
            // An edit: the new farm replaces the old one, which is deleted
            // now or, without internet, on the next try (deletes go first).
            _replaced[oldId] = made.farm.summary.id;
            if (!_deletes.contains(oldId)) _deletes.add(oldId);
            try {
              await api.deleteFarm(oldId);
              _deletes.remove(oldId);
            } on ApiException catch (e) {
              if (e.status == 404) _deletes.remove(oldId);
            }
            // The field's history stays on show while the new farm is
            // analysed; the old farm's copies are then no longer needed.
            await InsightsCopy.carry(oldId, made.farm.summary.id);
            for (final name in ['farm_$oldId', InsightsCopy.name(oldId)]) {
              try {
                await LocalStore.delete(name);
              } catch (_) {}
            }
          }
          _items.remove(item);
          sent++;
          changed = true;
        } on ApiException catch (e) {
          if (later(e)) {
            offline = e.isOffline;
            break;
          }
          // The server refused it (e.g. bad_polygon): sending again will not
          // help. A new farm's walked border goes back to Add farm, so the
          // walk is not lost; an edit leaves the farm as it was on the server.
          _items.remove(item);
          final edgeBack = item.farmId == null && await Draft.load() == null;
          if (edgeBack) await Draft.save(item.request.points, walk: false);
          rejected.add(Refused(item, e.code, edgeBack: edgeBack));
          changed = true;
        }
      }
    } finally {
      _flushing = false;
      _newSent += sent;
      _newRefused.addAll(rejected);
      if (changed) {
        await _persist();
        notifyListeners();
      }
    }
    return FlushResult(sent: sent, rejected: rejected, offline: offline);
  }
}
