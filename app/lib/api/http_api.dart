import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'api.dart';

/// The real server (BACKEND.md section 2). Same methods as [FakeApi], so no
/// screen changes when the server comes up. Chosen at build time with
/// `--dart-define=API_URL=https://...` (see config.dart).
///
/// Errors follow BACKEND.md section 4: the HTTP status plus the JSON `error`
/// code. No answer at all (no internet, server down, timeout) becomes
/// `ApiException(0, 'offline')`, which the screens and the Outbox treat as
/// "keep the saved copy and try again later".
class HttpApi implements Api {
  HttpApi(
    String baseUrl, {
    HttpClient? client,
    this.timeout = const Duration(seconds: 20),
    this.onUnauthorized,
  }) : _base = Uri.parse(baseUrl.endsWith('/') ? baseUrl : '$baseUrl/'),
       _client =
           client ??
           (HttpClient()..connectionTimeout = const Duration(seconds: 10));

  final Uri _base;
  final HttpClient _client;

  /// Longest wait for one answer.
  final Duration timeout;

  /// Called when the server refuses the saved token (401 after sign-in):
  /// the app signs the farmer out instead of getting stuck.
  final void Function()? onUnauthorized;

  String? _token;

  @override
  void useToken(String token) => _token = token;

  static String _codeFor(int status) => switch (status) {
    400 => 'bad_request',
    401 => 'unauthorized',
    404 => 'not_found',
    422 => 'invalid',
    429 => 'rate_limited',
    503 => 'upstream_down',
    _ => 'http_$status',
  };

  Future<Map<String, dynamic>> _call(
    String method,
    String path, {
    Map<String, dynamic>? body,
    String? idempotencyKey,
  }) async {
    final uri = _base.resolve(path);
    final sentToken = _token;
    final int status;
    final String text;
    try {
      final req = await _client.openUrl(method, uri).timeout(timeout);
      req.headers.set(HttpHeaders.acceptHeader, 'application/json');
      if (sentToken != null) {
        req.headers.set(HttpHeaders.authorizationHeader, 'Bearer $sentToken');
      }
      if (idempotencyKey != null) {
        req.headers.set('Idempotency-Key', idempotencyKey);
      }
      if (body != null) {
        final bytes = utf8.encode(jsonEncode(body));
        req.headers.contentType = ContentType.json;
        req.contentLength = bytes.length;
        req.add(bytes);
      }
      final res = await req.close().timeout(timeout);
      status = res.statusCode;
      text = await res.transform(utf8.decoder).join().timeout(timeout);
    } on SocketException {
      throw ApiException(0, 'offline');
    } on TimeoutException {
      throw ApiException(0, 'offline');
    } on HandshakeException {
      throw ApiException(0, 'offline');
    } on HttpException {
      throw ApiException(0, 'offline');
    }

    Object? json;
    try {
      json = text.trim().isEmpty ? const <String, dynamic>{} : jsonDecode(text);
    } on FormatException {
      json = null; // e.g. an HTML error page from a proxy
    }

    if (status >= 200 && status < 300) {
      if (json is Map<String, dynamic>) return json;
      throw ApiException(status, 'bad_response');
    }
    final extra = json is Map<String, dynamic>
        ? Map<String, dynamic>.of(json)
        : <String, dynamic>{};
    final code = extra.remove('error') as String? ?? _codeFor(status);
    if (status == 401 && sentToken != null) {
      _token = null;
      onUnauthorized?.call();
    }
    throw ApiException(status, code, extra.isEmpty ? null : extra);
  }

  static String _farm(String id) => 'farms/${Uri.encodeComponent(id)}';

  @override
  Future<OtpSendResult> sendOtp({
    required String phone,
    required String lang,
  }) async {
    final j = await _call(
      'POST',
      'auth/otp/send',
      body: {'phone': phone, 'lang': lang},
    );
    return OtpSendResult(
      sent: j['sent'] as bool? ?? true,
      retryAfterS: (j['retry_after_s'] as num?)?.toInt() ?? 60,
    );
  }

  @override
  Future<OtpVerifyResult> verifyOtp({
    required String phone,
    required String code,
  }) async {
    final j = await _call(
      'POST',
      'auth/otp/verify',
      body: {'phone': phone, 'code': code},
    );
    final token = j['token'] as String;
    _token = token;
    return OtpVerifyResult(
      token: token,
      farmsCount: (j['farms_count'] as num?)?.toInt() ?? 0,
    );
  }

  @override
  Future<List<FarmSummary>> getFarms() async {
    final j = await _call('GET', 'farms');
    return [
      for (final f in j['farms'] as List? ?? const [])
        FarmSummary.fromJson(f as Map<String, dynamic>),
    ];
  }

  @override
  Future<CreateFarmResult> createFarm(
    NewFarmRequest request, {
    String? idempotencyKey,
  }) async {
    final j = await _call(
      'POST',
      'farms',
      body: request.toJson(),
      idempotencyKey: idempotencyKey,
    );
    return CreateFarmResult(
      farm: Farm.fromJson(j['farm'] as Map<String, dynamic>),
      droppedCells: (j['dropped_cells'] as List?)?.length ?? 0,
    );
  }

  @override
  Future<CreateFarmResult> updateFarm(
    String id,
    NewFarmRequest request, {
    String? idempotencyKey,
  }) async {
    final j = await _call(
      'PUT',
      _farm(id),
      body: request.toJson(),
      idempotencyKey: idempotencyKey,
    );
    return CreateFarmResult(
      farm: Farm.fromJson(j['farm'] as Map<String, dynamic>),
      droppedCells: (j['dropped_cells'] as List?)?.length ?? 0,
    );
  }

  @override
  Future<void> deleteFarm(String id) async {
    await _call('DELETE', _farm(id));
  }

  @override
  Future<Farm> getFarm(String id) async {
    final j = await _call('GET', _farm(id));
    return Farm.fromJson(j['farm'] as Map<String, dynamic>);
  }

  @override
  Future<FarmStatusReport> getFarmStatus(String id) async =>
      FarmStatusReport.fromJson(await _call('GET', '${_farm(id)}/status'));

  @override
  Future<FarmPlan> getPlan(String id) async =>
      FarmPlan.fromJson(await _call('GET', '${_farm(id)}/plan'));
}
