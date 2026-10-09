import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

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
    ({String type, List<int> bytes})? raw,
    Duration? wait,
  }) async {
    final limit = wait ?? timeout;
    final uri = _base.resolve(path);
    final sentToken = _token;
    final int status;
    final String text;
    try {
      final req = await _client.openUrl(method, uri).timeout(limit);
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
      } else if (raw != null) {
        req.headers.set(HttpHeaders.contentTypeHeader, raw.type);
        req.contentLength = raw.bytes.length;
        req.add(raw.bytes);
      }
      final res = await req.close().timeout(limit);
      status = res.statusCode;
      text = await res.transform(utf8.decoder).join().timeout(limit);
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
  Future<FarmInsights> getInsights(String id) async =>
      FarmInsights.fromJson(await _call('GET', '${_farm(id)}/insights'));

  @override
  Future<FarmPlan> getPlan(String id) async =>
      FarmPlan.fromJson(await _call('GET', '${_farm(id)}/plan'));

  /// Multipart by hand (no extra package): question, lang, cell, photos.
  /// The Doctor reads the field and the weather first, so it waits longer.
  @override
  Future<DoctorAnswer> askDoctor(String farmId, DoctorQuestion q) async {
    final boundary = 'jutyar${DateTime.now().microsecondsSinceEpoch}';
    final out = BytesBuilder(copy: false);
    void field(String name, String value) => out.add(
      utf8.encode(
        '--$boundary\r\nContent-Disposition: form-data; name="$name"'
        '\r\n\r\n$value\r\n',
      ),
    );
    final text = q.text?.trim() ?? '';
    if (text.isNotEmpty) field('question', text);
    field('lang', q.lang);
    if (q.cellE != null && q.cellN != null) {
      field('cell', jsonEncode({'e': q.cellE, 'n': q.cellN}));
    }
    for (final (i, p) in q.photos.indexed) {
      final ext = p.mime == 'image/png' ? 'png' : 'jpg';
      out.add(
        utf8.encode(
          '--$boundary\r\nContent-Disposition: form-data; name="photos"; '
          'filename="photo_${i + 1}.$ext"\r\nContent-Type: ${p.mime}\r\n\r\n',
        ),
      );
      out.add(p.bytes);
      out.add(utf8.encode('\r\n'));
    }
    out.add(utf8.encode('--$boundary--\r\n'));
    final j = await _call(
      'POST',
      '${_farm(farmId)}/ask',
      raw: (
        type: 'multipart/form-data; boundary=$boundary',
        bytes: out.takeBytes(),
      ),
      wait: const Duration(seconds: 120),
    );
    return DoctorAnswer.fromJson(j);
  }

  @override
  Future<FarmerProfile> getMe() async =>
      FarmerProfile.fromJson(await _call('GET', 'me'));

  @override
  Future<List<FarmAlert>> getAlerts(String farmId) async {
    final j = await _call('GET', '${_farm(farmId)}/alerts?days=30');
    return [
      for (final a in j['alerts'] as List? ?? const [])
        FarmAlert.fromJson(a as Map<String, dynamic>),
    ];
  }

  /// Multipart by hand, like [askDoctor]: kind, text, farm_id, photos.
  @override
  Future<FarmerMessage> sendReport(
    NewReport r, {
    String? idempotencyKey,
  }) async {
    final boundary = 'jutyar${DateTime.now().microsecondsSinceEpoch}';
    final out = BytesBuilder(copy: false);
    void field(String name, String value) => out.add(
      utf8.encode(
        '--$boundary\r\nContent-Disposition: form-data; name="$name"'
        '\r\n\r\n$value\r\n',
      ),
    );
    field('kind', 'report');
    field('text', r.text);
    field('farm_id', r.farmId);
    for (final (i, p) in r.photos.indexed) {
      final ext = p.mime == 'image/png' ? 'png' : 'jpg';
      out.add(
        utf8.encode(
          '--$boundary\r\nContent-Disposition: form-data; name="photos"; '
          'filename="photo_${i + 1}.$ext"\r\nContent-Type: ${p.mime}\r\n\r\n',
        ),
      );
      out.add(p.bytes);
      out.add(utf8.encode('\r\n'));
    }
    out.add(utf8.encode('--$boundary--\r\n'));
    final j = await _call(
      'POST',
      'messages',
      raw: (
        type: 'multipart/form-data; boundary=$boundary',
        bytes: out.takeBytes(),
      ),
      idempotencyKey: idempotencyKey,
      wait: const Duration(seconds: 60),
    );
    return FarmerMessage.fromJson(j['message'] as Map<String, dynamic>);
  }

  @override
  Future<List<FarmerMessage>> getMyMessages() async {
    final j = await _call('GET', 'messages/mine');
    return [
      for (final m in j['messages'] as List? ?? const [])
        FarmerMessage.fromJson(m as Map<String, dynamic>),
    ];
  }

  @override
  Future<void> deleteAccount() async {
    await _call('DELETE', 'account');
  }

  // ---- Alwa market (FRONTEND.md 5; BACKEND.md 2.14 for what is missing) ----

  static String _listing(String id) =>
      'alwa/listings/${Uri.encodeComponent(id)}';

  @override
  Future<List<AlwaListing>> alwaListings({double? lat, double? lon}) async {
    // Server: not built yet (BACKEND.md 2.14): lat/lon are ignored today, so
    // the list comes unsorted and without distance_km.
    final q = <String, String>{
      'status': 'open',
      'rows_per_page': '100',
      if (lat != null && lon != null) ...{'lat': '$lat', 'lon': '$lon'},
    };
    final j = await _call(
      'GET',
      Uri(path: 'alwa/listings', queryParameters: q).toString(),
    );
    return [
      for (final l in j['listings'] as List? ?? const [])
        AlwaListing.fromJson(l as Map<String, dynamic>),
    ];
  }

  @override
  Future<AlwaListing> alwaListing(String id) async {
    final j = await _call('GET', _listing(id));
    return AlwaListing.fromJson(j['listing'] as Map<String, dynamic>);
  }

  @override
  Future<AlwaListing> createAlwaListing(
    NewAlwaListing listing, {
    String? idempotencyKey,
  }) async {
    final body = listing.toJson(DateTime.now());
    // Server: not built yet (BACKEND.md 2.14): `market` and `pickup` are still
    // required, so send the alwa the price board uses and "farm" (the crop
    // is where the farmer stands). Drop both once 2.14 #1 is built.
    final markets = await _alwaMarkets();
    final market = AlwaMarket.pick(markets, listing.lat, listing.lon);
    if (market != null) body['market'] = market.slug;
    body['pickup'] = 'farm';
    final j = await _call(
      'POST',
      'alwa/listings',
      body: body,
      idempotencyKey: idempotencyKey,
    );
    return AlwaListing.fromJson(j['listing'] as Map<String, dynamic>);
  }

  @override
  Future<List<AlwaListing>> myAlwaListings() async {
    final j = await _call('GET', 'alwa/listings/mine');
    return [
      for (final l in j['listings'] as List? ?? const [])
        AlwaListing.fromJson(l as Map<String, dynamic>),
    ];
  }

  @override
  Future<void> cancelAlwaListing(String id) async {
    await _call('DELETE', _listing(id));
  }

  // Server: not built yet (BACKEND.md 2.14 #4): answers 404 today.
  @override
  Future<AlwaListing> markAlwaListingSold(String id) async {
    final j = await _call('POST', '${_listing(id)}/sold');
    return AlwaListing.fromJson(j['listing'] as Map<String, dynamic>);
  }

  Future<List<AlwaMarket>> _alwaMarkets() async {
    final j = await _call('GET', 'alwa/markets');
    return [
      for (final m in j['markets'] as List? ?? const [])
        AlwaMarket.fromJson(m as Map<String, dynamic>),
    ];
  }

  @override
  Future<AlwaPriceBoard?> alwaPriceBoard({double? lat, double? lon}) async {
    final markets = await _alwaMarkets();
    final market = AlwaMarket.pick(markets, lat, lon);
    if (market == null) return null;
    final j = await _call(
      'GET',
      'alwa/markets/${Uri.encodeComponent(market.slug)}/prices',
    );
    // Server: not built yet (BACKEND.md 2.14 #5): markets have no point, so
    // "nearest" is only true once they do.
    return AlwaPriceBoard.fromJson(
      j,
      market,
      nearest: lat != null && market.lat != null,
    );
  }
}
