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
  });

  /// Sent as `Idempotency-Key`, so a retry never makes a second farm.
  final String key;
  final NewFarmRequest request;

  /// What My farms shows until the server has it.
  final FarmSummary summary;
  final DateTime queuedAt;

  Map<String, dynamic> toJson() => {
    'key': key,
    'request': request.toJson(),
    'summary': summary.toJson(),
    'queued_at': queuedAt.toUtc().toIso8601String(),
  };

  factory PendingFarm.fromJson(Map<String, dynamic> j) => PendingFarm(
    key: j['key'] as String,
    request: NewFarmRequest.fromJson(j['request'] as Map<String, dynamic>),
    summary: FarmSummary.fromJson(j['summary'] as Map<String, dynamic>),
    queuedAt: DateTime.parse(j['queued_at'] as String),
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
  bool _loaded = false;
  bool _flushing = false;

  List<PendingFarm> get items => List.unmodifiable(_items);

  Future<void> load() async {
    if (_loaded) return;
    _loaded = true;
    final j = await LocalStore.read(_name);
    for (final e in (j?['items'] as List? ?? const [])) {
      try {
        _items.add(PendingFarm.fromJson(e as Map<String, dynamic>));
      } catch (_) {}
    }
    notifyListeners();
  }

  Future<void> _persist() => LocalStore.write(_name, {
    'items': [for (final i in _items) i.toJson()],
  });

  static String newKey() {
    final r = Random.secure();
    return '${DateTime.now().microsecondsSinceEpoch.toRadixString(36)}-'
        '${List.generate(8, (_) => r.nextInt(36).toRadixString(36)).join()}';
  }

  Future<PendingFarm> add(NewFarmRequest request, FarmSummary summary) async {
    await load();
    final item = PendingFarm(
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

  /// Try to send everything waiting, oldest first. Stops at the first sign of no internet.
  Future<FlushResult> flush(Api api) async {
    await load();
    if (_flushing || _items.isEmpty) return const FlushResult();
    _flushing = true;
    var sent = 0;
    final rejected = <String>[];
    var offline = false;
    try {
      for (final item in List.of(_items)) {
        try {
          await api.createFarm(item.request, idempotencyKey: item.key);
          _items.remove(item);
          sent++;
        } on ApiException catch (e) {
          // Keep the farm and try later: no internet, not signed in, timeout,
          // too many requests (429) or a server error. Losing a walked farm
          // because the server was busy would be far worse than waiting.
          if (e.isOffline ||
              const {401, 408, 429}.contains(e.status) ||
              e.status >= 500) {
            offline = e.isOffline;
            break;
          }
          // The server refused it (e.g. bad_polygon): sending again will not help.
          _items.remove(item);
          rejected.add(item.request.name);
        }
      }
    } finally {
      _flushing = false;
      if (sent > 0 || rejected.isNotEmpty) {
        await _persist();
        notifyListeners();
      }
    }
    return FlushResult(sent: sent, rejected: rejected, offline: offline);
  }
}
