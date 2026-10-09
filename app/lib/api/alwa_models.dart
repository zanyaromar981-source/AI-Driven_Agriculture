// The Alwa market as the app uses it (BACKEND.md 2.14): a listing is only a
// crop, how many kg, the asking price per kg, where it is (the phone's GPS),
// the seller's phone, when it was posted and when it closes. No offers.
//
// The server today (FRONTEND.md 5) has no point, no seller phone, no
// distance and no "sold" date on a listing: those fields are null here and
// the screens show a calm empty state for them.

import 'dart:math' as math;

/// The 16 Alwa crop codes (FRONTEND.md 5), with names and a picture.
class AlwaCrop {
  const AlwaCrop(this.code, this.en, this.ku, this.emoji);
  final String code;
  final String en;
  final String ku;
  final String emoji;
}

/// In the order of the Sell screen's picker: the first 11 show at once,
/// the rest behind "All 16".
const kAlwaCrops = <AlwaCrop>[
  AlwaCrop('wheat', 'Wheat', 'گەنم', '🌾'),
  AlwaCrop('barley', 'Barley', 'جۆ', '🌿'),
  AlwaCrop('tomato', 'Tomato', 'تەماتە', '🍅'),
  AlwaCrop('cucumber', 'Cucumber', 'خەیار', '🥒'),
  AlwaCrop('potato', 'Potato', 'پەتاتە', '🥔'),
  AlwaCrop('onion', 'Onion', 'پیاز', '🧅'),
  AlwaCrop('chickpea', 'Chickpea', 'نۆک', '🫘'),
  AlwaCrop('grape', 'Grape', 'ترێ', '🍇'),
  AlwaCrop('olive', 'Olive', 'زەیتوون', '🫒'),
  AlwaCrop('eggplant', 'Eggplant', 'باینجان', '🍆'),
  AlwaCrop('pepper', 'Pepper', 'بیبەر', '🌶'),
  AlwaCrop('watermelon', 'Watermelon', 'شووتی', '🍉'),
  AlwaCrop('sunflower', 'Sunflower', 'گوڵەبەڕۆژە', '🌻'),
  AlwaCrop('pomegranate', 'Pomegranate', 'هەنار', '🌱'),
  AlwaCrop('okra', 'Okra', 'بامیە', '🫛'),
  AlwaCrop('apple', 'Apple', 'سێو', '🍎'),
];

/// The crop for [code]; an unknown code keeps its code as the name.
AlwaCrop alwaCrop(String code) => kAlwaCrops.firstWhere(
  (c) => c.code == code,
  orElse: () => AlwaCrop(code, code, code, '🌱'),
);

/// Most open listings one phone may have (`too_many_listings`).
const kAlwaMaxOpen = 20;

/// A listing stays open 1 to 14 days; the app sends 14 unless the farmer
/// picks fewer.
const kAlwaMaxDays = 14;

double? _d(Object? v) => (v as num?)?.toDouble();

/// Kilometres between two points (haversine), for the demo server and for
/// the distance when the server does not send one.
double alwaKm(double lat1, double lon1, double lat2, double lon2) {
  const r = 6371.0;
  double rad(double d) => d * math.pi / 180;
  final dLat = rad(lat2 - lat1);
  final dLon = rad(lon2 - lon1);
  final a =
      math.pow(math.sin(dLat / 2), 2) +
      math.cos(rad(lat1)) *
          math.cos(rad(lat2)) *
          math.pow(math.sin(dLon / 2), 2);
  return 2 * r * math.asin(math.sqrt(a));
}

/// One crop on sale (AlwaListingResponse plus BACKEND.md 2.14).
class AlwaListing {
  const AlwaListing({
    required this.id,
    required this.crop,
    required this.quantityKg,
    required this.priceIqdPerKg,
    required this.status,
    required this.createdAt,
    required this.closesAt,
    this.lat,
    this.lon,
    this.sellerPhone,
    this.distanceKm,
    this.soldAt,
  });

  final String id;
  final String crop;
  final double quantityKg;
  final double priceIqdPerKg;

  /// `open`, `sold`, `closed` or `cancelled`.
  final String status;

  /// When it was posted. The server's list rows leave it out (only one
  /// listing by id has it), so it can be unknown; never guessed.
  final DateTime? createdAt;
  final DateTime closesAt;

  /// Where the crop is. Server: not built yet (BACKEND.md 2.14 #2).
  final double? lat;
  final double? lon;

  /// The seller's phone, shown from the start. Server: not built yet.
  final String? sellerPhone;

  /// From the phone, when the list was asked with a point. Server: not built yet.
  final double? distanceKm;

  /// When the seller marked it sold. Server: not built yet.
  final DateTime? soldAt;

  bool get isOpen => status == 'open';
  bool get hasPoint => lat != null && lon != null;

  /// The distance to ([lat], [lon]): the server's when it sent one,
  /// else worked out from the listing's point; null when neither is known.
  double? kmFrom(double? lat, double? lon) {
    if (distanceKm != null) return distanceKm;
    if (lat == null || lon == null || !hasPoint) return null;
    return alwaKm(lat, lon, this.lat!, this.lon!);
  }

  factory AlwaListing.fromJson(Map<String, dynamic> j) => AlwaListing(
    id: '${j['id']}',
    crop: j['crop'] as String? ?? '',
    quantityKg: _d(j['quantity_kg']) ?? 0,
    priceIqdPerKg: _d(j['asking_price_iqd_per_kg']) ?? 0,
    status: j['status'] as String? ?? 'open',
    createdAt: DateTime.tryParse(j['created_at'] as String? ?? '')?.toLocal(),
    closesAt: DateTime.parse(j['closes_at'] as String).toLocal(),
    lat: _d(j['lat']),
    lon: _d(j['lon']),
    sellerPhone: _phoneOf(j),
    distanceKm: _d(j['distance_km']),
    soldAt: DateTime.tryParse(j['sold_at'] as String? ?? '')?.toLocal(),
  );
}

/// The seller's phone: `seller_phone` once the server sends it (BACKEND.md
/// 2.14 #3); until then the app carries it in `seller_name`, which the test
/// server already shows to buyers, so only a phone-shaped name counts.
String? _phoneOf(Map<String, dynamic> j) {
  for (final k in ['seller_phone', 'seller_name']) {
    final v = (j[k] as String?)?.trim() ?? '';
    if (k == 'seller_phone' && v.isNotEmpty) return v;
    if (RegExp(r'^\+?[0-9 ]{10,16}$').hasMatch(v)) return v;
  }
  return null;
}

/// What the Sell screen sends (POST /alwa/listings, BACKEND.md 2.14 #1).
class NewAlwaListing {
  const NewAlwaListing({
    required this.crop,
    required this.quantityKg,
    required this.priceIqdPerKg,
    required this.lat,
    required this.lon,
    this.days = kAlwaMaxDays,
    this.sellerPhone,
  });

  /// The seller's sign-in phone, shown to buyers so they can call.
  final String? sellerPhone;
  final String crop;
  final double quantityKg;
  final double priceIqdPerKg;
  final double lat;
  final double lon;

  /// Closes after this many days, 1 to 14.
  final int days;

  Map<String, dynamic> toJson(DateTime now) => {
    'crop': crop,
    // Whole numbers: the server's quantity_kg and price are integers and
    // refuse 4000.0.
    'quantity_kg': quantityKg.round(),
    'asking_price_iqd_per_kg': priceIqdPerKg.round(),
    'closes_at':
        '${now.add(Duration(days: days)).toUtc().toIso8601String().split('.').first}Z',
    'lat': lat,
    'lon': lon,
  };
}

/// An alwa (AlwaMarketResponse). The point is BACKEND.md 2.14 #5, not on
/// the server yet.
class AlwaMarket {
  const AlwaMarket({
    required this.slug,
    required this.nameEn,
    this.nameKu,
    this.lat,
    this.lon,
  });
  final String slug;
  final String nameEn;
  final String? nameKu;
  final double? lat;
  final double? lon;

  factory AlwaMarket.fromJson(Map<String, dynamic> j) => AlwaMarket(
    slug: j['slug'] as String? ?? '',
    nameEn: j['name_en'] as String? ?? j['slug'] as String? ?? '',
    nameKu: j['name_ku'] as String?,
    lat: _d(j['lat']),
    lon: _d(j['lon']),
  );

  /// The market nearest ([lat], [lon]) when the markets have points, else
  /// the first one. Null when there are none.
  static AlwaMarket? pick(List<AlwaMarket> all, double? lat, double? lon) {
    if (all.isEmpty) return null;
    final placed = all.where((m) => m.lat != null && m.lon != null).toList();
    if (lat == null || lon == null || placed.isEmpty) return all.first;
    placed.sort(
      (a, b) => alwaKm(
        lat,
        lon,
        a.lat!,
        a.lon!,
      ).compareTo(alwaKm(lat, lon, b.lat!, b.lon!)),
    );
    return placed.first;
  }
}

/// One crop's price on the board (AlwaPriceResponse).
class AlwaPrice {
  const AlwaPrice({
    required this.crop,
    required this.priceIqdPerKg,
    this.changePct7d,
    this.fixed = false,
  });
  final String crop;
  final double priceIqdPerKg;

  /// Change against 7 days ago; null when there is nothing to compare.
  final double? changePct7d;

  /// A government (fixed) price, like wheat.
  final bool fixed;

  factory AlwaPrice.fromJson(Map<String, dynamic> j) => AlwaPrice(
    crop: j['crop'] as String? ?? '',
    priceIqdPerKg: _d(j['price_iqd_per_kg']) ?? 0,
    changePct7d: _d(j['change_pct_7d']),
    fixed: j['fixed'] == true,
  );
}

/// "Today at the alwa": what Ministry staff typed in for one market.
class AlwaPriceBoard {
  const AlwaPriceBoard({
    required this.market,
    required this.day,
    required this.prices,
    required this.nearest,
  });
  final AlwaMarket market;

  /// The day the prices are for; null when staff typed nothing.
  final DateTime? day;
  final List<AlwaPrice> prices;

  /// True when the market was picked by distance from the phone.
  final bool nearest;

  AlwaPrice? priceOf(String crop) {
    for (final p in prices) {
      if (p.crop == crop) return p;
    }
    return null;
  }

  factory AlwaPriceBoard.fromJson(
    Map<String, dynamic> j,
    AlwaMarket market, {
    required bool nearest,
  }) => AlwaPriceBoard(
    market: market,
    day: DateTime.tryParse(j['day'] as String? ?? ''),
    prices: [
      for (final p in j['prices'] as List? ?? const [])
        AlwaPrice.fromJson(p as Map<String, dynamic>),
    ],
    nearest: nearest,
  );
}
