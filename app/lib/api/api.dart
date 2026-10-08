// The contract with the backend, shaped exactly like BACKEND.md sections 2.1 and 2.2.
// Screens only talk to [Api]. The demo uses FakeApi; the real server will be
// another class with the same methods.

enum FarmStatus { normal, watch, alarm, none }

FarmStatus farmStatusFrom(String? s) => switch (s) {
  'normal' => FarmStatus.normal,
  'watch' => FarmStatus.watch,
  'alarm' => FarmStatus.alarm,
  _ => FarmStatus.none,
};

class CropShare {
  const CropShare({required this.crop, required this.dunam});

  /// BACKEND.md crop code: wheat, barley, tomato, ...
  final String crop;
  final double dunam;

  factory CropShare.fromJson(Map<String, dynamic> j) => CropShare(
    crop: j['crop'] as String,
    dunam: (j['dunam'] as num).toDouble(),
  );
}

/// FarmSummary from `GET /farms`.
class FarmSummary {
  const FarmSummary({
    required this.id,
    required this.name,
    required this.areaDunam,
    required this.crops,
    required this.status,
    this.lastPicture,
    this.lat,
    this.lon,
  });

  final String id;
  final String name;
  final double areaDunam;
  final List<CropShare> crops;
  final FarmStatus status;
  final String? lastPicture;
  final double? lat;
  final double? lon;

  factory FarmSummary.fromJson(Map<String, dynamic> j) {
    final centroid = j['centroid'] as Map<String, dynamic>?;
    return FarmSummary(
      id: j['id'] as String,
      name: j['name'] as String,
      areaDunam: (j['area_dunam'] as num).toDouble(),
      crops: [
        for (final c in (j['crops'] as List? ?? const []))
          CropShare.fromJson(c as Map<String, dynamic>),
      ],
      status: farmStatusFrom(j['status'] as String?),
      lastPicture: j['last_picture'] as String?,
      lat: (centroid?['lat'] as num?)?.toDouble(),
      lon: (centroid?['lon'] as num?)?.toDouble(),
    );
  }

  Map<String, dynamic> toJson() => {
    'id': id,
    'name': name,
    'area_dunam': areaDunam,
    'crops': [
      for (final c in crops) {'crop': c.crop, 'dunam': c.dunam},
    ],
    'status': status.name,
    'last_picture': lastPicture,
    if (lat != null && lon != null) 'centroid': {'lat': lat, 'lon': lon},
  };

  /// The crop with the most dunams, shown on the farm card.
  String? get mainCrop => crops.isEmpty
      ? null
      : crops.reduce((a, b) => a.dunam >= b.dunam ? a : b).crop;
}

/// A GPS corner the farmer tapped (BACKEND.md "Point").
class GeoPoint {
  const GeoPoint({
    required this.lat,
    required this.lon,
    required this.accM,
    required this.t,
  });
  final double lat;
  final double lon;

  /// GPS accuracy in metres (0 = placed by hand in test mode).
  final int accM;
  final DateTime t;

  Map<String, dynamic> toJson() => {
    'lat': lat,
    'lon': lon,
    'acc_m': accM,
    't': isoUtc(t),
  };

  factory GeoPoint.fromJson(Map<String, dynamic> j) => GeoPoint(
    lat: (j['lat'] as num).toDouble(),
    lon: (j['lon'] as num).toDouble(),
    accM: (j['acc_m'] as num).toInt(),
    t: DateTime.parse(j['t'] as String),
  );
}

/// One painted cell for `POST /farms`.
class CellCrop {
  const CellCrop({required this.e, required this.n, required this.crop});
  final int e;
  final int n;
  final String crop;

  Map<String, dynamic> toJson() => {'e': e, 'n': n, 'crop': crop};

  factory CellCrop.fromJson(Map<String, dynamic> j) => CellCrop(
    e: (j['e'] as num).toInt(),
    n: (j['n'] as num).toInt(),
    crop: j['crop'] as String,
  );
}

/// Body of `POST /farms` (BACKEND.md 2.2).
class NewFarmRequest {
  const NewFarmRequest({
    required this.name,
    required this.points,
    required this.cells,
    required this.createdOfflineAt,
  });

  final String name;
  final List<GeoPoint> points;
  final List<CellCrop> cells;
  final DateTime createdOfflineAt;

  Map<String, dynamic> toJson() => {
    'name': name,
    'points': [for (final p in points) p.toJson()],
    'cells': [for (final c in cells) c.toJson()],
    'created_offline_at': isoUtc(createdOfflineAt),
  };

  factory NewFarmRequest.fromJson(Map<String, dynamic> j) => NewFarmRequest(
    name: j['name'] as String,
    points: [
      for (final p in j['points'] as List)
        GeoPoint.fromJson(p as Map<String, dynamic>),
    ],
    cells: [
      for (final c in j['cells'] as List)
        CellCrop.fromJson(c as Map<String, dynamic>),
    ],
    createdOfflineAt: DateTime.parse(j['created_offline_at'] as String),
  );
}

/// A cell as the server returns it.
class FarmCell {
  const FarmCell({
    required this.e,
    required this.n,
    required this.crop,
    this.greennessPct,
    required this.level,
    this.insidePct = 100,
  });
  final int e;
  final int n;
  final String crop;
  final int? greennessPct;
  final FarmStatus level;

  /// Share of the cell inside the farm outline, 0 to 100 (edge cells are cut).
  final double insidePct;

  factory FarmCell.fromJson(Map<String, dynamic> j) => FarmCell(
    e: (j['e'] as num).toInt(),
    n: (j['n'] as num).toInt(),
    crop: j['crop'] as String,
    greennessPct: (j['greenness_pct'] as num?)?.toInt(),
    level: farmStatusFrom(j['level'] as String?),
    insidePct: (j['inside_pct'] as num?)?.toDouble() ?? 100,
  );
}

/// Full farm (BACKEND.md 2.2 "Farm" = FarmSummary + outline, cells, picture_date).
class Farm {
  const Farm({
    required this.summary,
    required this.outline,
    required this.cells,
    this.pictureDate,
  });
  final FarmSummary summary;
  final List<({double lat, double lon})> outline;
  final List<FarmCell> cells;
  final String? pictureDate;

  factory Farm.fromJson(Map<String, dynamic> j) => Farm(
    summary: FarmSummary.fromJson(j),
    outline: [
      for (final p in j['outline'] as List)
        (
          lat: ((p as Map)['lat'] as num).toDouble(),
          lon: (p['lon'] as num).toDouble(),
        ),
    ],
    cells: [
      for (final c in j['cells'] as List)
        FarmCell.fromJson(c as Map<String, dynamic>),
    ],
    pictureDate: j['picture_date'] as String?,
  );
}

/// Result of `POST /farms`: the farm plus any cells the server dropped.
class CreateFarmResult {
  const CreateFarmResult({required this.farm, required this.droppedCells});
  final Farm farm;
  final int droppedCells;
}

/// ISO 8601 UTC to the second: 2026-10-08T14:03:11Z
String isoUtc(DateTime t) => '${t.toUtc().toIso8601String().split('.').first}Z';

class OtpSendResult {
  const OtpSendResult({required this.sent, required this.retryAfterS});
  final bool sent;
  final int retryAfterS;
}

class OtpVerifyResult {
  const OtpVerifyResult({required this.token, required this.farmsCount});
  final String token;
  final int farmsCount;
}

/// Error shape from BACKEND.md section 4: HTTP status + error code.
class ApiException implements Exception {
  ApiException(this.status, this.code, [this.extra]);
  final int status;
  final String code;
  final Map<String, dynamic>? extra;

  /// No internet or server unreachable: keep the data and try again later.
  bool get isOffline => status == 0;

  @override
  String toString() => 'ApiException($status $code)';
}

abstract class Api {
  /// POST /auth/otp/send
  Future<OtpSendResult> sendOtp({required String phone, required String lang});

  /// POST /auth/otp/verify
  Future<OtpVerifyResult> verifyOtp({
    required String phone,
    required String code,
  });

  /// GET /farms (needs the token from verifyOtp)
  Future<List<FarmSummary>> getFarms();

  /// POST /farms. [idempotencyKey] goes in the `Idempotency-Key` header, so a
  /// farm sent twice (retry after a lost answer) is only created once.
  Future<CreateFarmResult> createFarm(
    NewFarmRequest request, {
    String? idempotencyKey,
  });

  /// Use a token saved on the phone from an earlier sign-in.
  void useToken(String token);
}
