import 'dart:math';

import 'package:flutter/foundation.dart';

import '../api/api.dart';
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

/// What one upload try did.
class FlushResult {
  const FlushResult({
    this.sent = 0,
    this.rejected = const [],
    this.offline = false,
  });
  final int sent;
  final List<String> rejected;
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
    final rejected = <String>[];
    var offline = false;
    // Keep and try later: no internet, not signed in, timeout, too many
    // requests (429) or a server error. Losing a walked farm because the
    // server was busy would be far worse than waiting.
    // 405/501: the server does not have this call yet (e.g. editing a
    // farm's border before the backend adds PUT /farms/{id}): keep it too.
    bool later(ApiException e) =>
        e.isOffline ||
        const {401, 405, 408, 429, 501}.contains(e.status) ||
        e.status >= 500;
    try {
      for (final id in List.of(_deletes)) {
        try {
          await api.deleteFarm(id);
        } on ApiException catch (e) {
          if (later(e)) {
            offline = e.isOffline;
            return FlushResult(
              sent: sent,
              rejected: rejected,
              offline: offline,
            );
          }
          // 404: already gone. Any other refusal will not change on retry.
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
            // The old farm's copies on the phone are no longer needed.
            for (final name in ['farm_$oldId', 'insights_$oldId']) {
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
          // The server refused it (e.g. bad_polygon): sending again will not help.
          _items.remove(item);
          rejected.add(item.request.name);
          changed = true;
        }
      }
    } finally {
      _flushing = false;
      if (changed) {
        await _persist();
        notifyListeners();
      }
    }
    return FlushResult(sent: sent, rejected: rejected, offline: offline);
  }
}
