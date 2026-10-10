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

/// One thing the Marketplace sells (GET /products, FRONTEND.md 5).
class Product {
  const Product(this.code, this.group, this.unit, this.nameEn, [this.nameKu]);
  final String code;

  /// One of [kProductGroups].
  final String group;

  /// `kg`, `tray_30` (a tray of 30 eggs), `litre` or `head` (one animal).
  final String unit;
  final String nameEn;
  final String? nameKu;

  factory Product.fromJson(Map<String, dynamic> j) => Product(
    j['code'] as String? ?? '',
    j['group'] as String? ?? 'crops',
    j['unit'] as String? ?? 'kg',
    j['name_en'] as String? ?? j['code'] as String? ?? '',
    j['name_ku'] as String?,
  );
}

/// The product groups, in the order of the chips.
const kProductGroups = [
  'crops',
  'fish_meat_eggs',
  'honey_dairy',
  'animals',
  'nuts_dried',
];

/// The most of one unit a listing may hold (FRONTEND.md 5); the least is 1.
const kUnitMax = {
  'kg': 1000000,
  'tray_30': 10000,
  'litre': 100000,
  'head': 1000,
};

/// The 30 products the server lists today, as it lists them.
const kProducts = <Product>[
  Product('wheat', 'crops', 'kg', 'Wheat', 'گەنم'),
  Product('barley', 'crops', 'kg', 'Barley', 'جۆ'),
  Product('tomato', 'crops', 'kg', 'Tomato', 'تەماتە'),
  Product('cucumber', 'crops', 'kg', 'Cucumber', 'خەیار'),
  Product('potato', 'crops', 'kg', 'Potato', 'پەتاتە'),
  Product('onion', 'crops', 'kg', 'Onion', 'پیاز'),
  Product('watermelon', 'crops', 'kg', 'Watermelon', 'شووتی'),
  Product('grape', 'crops', 'kg', 'Grape', 'ترێ'),
  Product('olive', 'crops', 'kg', 'Olive', 'زەیتوون'),
  Product('sunflower', 'crops', 'kg', 'Sunflower', 'گوڵەبەڕۆژە'),
  Product('chickpea', 'crops', 'kg', 'Chickpea', 'نۆک'),
  Product('pomegranate', 'crops', 'kg', 'Pomegranate'),
  Product('okra', 'crops', 'kg', 'Okra'),
  Product('eggplant', 'crops', 'kg', 'Eggplant'),
  Product('pepper', 'crops', 'kg', 'Pepper'),
  Product('apple', 'crops', 'kg', 'Apple'),
  Product('fish', 'fish_meat_eggs', 'kg', 'Fish', 'ماسی'),
  Product('chicken', 'fish_meat_eggs', 'kg', 'Chicken', 'مریشک'),
  Product(
    'eggs',
    'fish_meat_eggs',
    'tray_30',
    'Eggs (tray of 30)',
    'هێلکە (تەبەقەی ٣٠)',
  ),
  Product('honey', 'honey_dairy', 'kg', 'Honey', 'هەنگوین'),
  Product('milk', 'honey_dairy', 'litre', 'Milk', 'شیر'),
  Product('yogurt', 'honey_dairy', 'kg', 'Yogurt', 'ماست'),
  Product('cheese', 'honey_dairy', 'kg', 'Cheese', 'پەنیر'),
  Product('sheep', 'animals', 'head', 'Sheep', 'مەڕ'),
  Product('goat', 'animals', 'head', 'Goat', 'بزن'),
  Product('cow', 'animals', 'head', 'Cow', 'مانگا'),
  Product('walnut', 'nuts_dried', 'kg', 'Walnuts', 'گوێز'),
  Product('almond', 'nuts_dried', 'kg', 'Almonds', 'بادەم'),
  Product('raisin', 'nuts_dried', 'kg', 'Raisins', 'مێوژ'),
  Product('dried_fig', 'nuts_dried', 'kg', 'Dried figs', 'هەنجیری وشک'),
];

/// The products the screens know: [kProducts] until the server's list
/// (GET /products) has been read, then that.
List<Product> alwaProducts = kProducts;

/// The product for [code], or null when the app has never heard of it.
Product? alwaProduct(String code) {
  for (final p in [...alwaProducts, ...kProducts]) {
    if (p.code == code) return p;
  }
  return null;
}

/// A picture for each product that is not a crop, then one per group.
const _productEmoji = {
  'fish': '🐟',
  'chicken': '🐔',
  'eggs': '🥚',
  'honey': '🍯',
  'milk': '🥛',
  'yogurt': '🥣',
  'cheese': '🧀',
  'sheep': '🐑',
  'goat': '🐐',
  'cow': '🐄',
  'walnut': '🌰',
  'almond': '🥜',
  'raisin': '🍇',
  'dried_fig': '🍪',
};

/// The picture of a product group (chips, and a product without its own).
const kGroupEmoji = {
  'crops': '🌱',
  'fish_meat_eggs': '🐟',
  'honey_dairy': '🍯',
  'animals': '🐑',
  'nuts_dried': '🌰',
};

/// The crop or product for [code], with its names and picture; an unknown
/// code keeps its code as the name.
AlwaCrop alwaCrop(String code) {
  for (final c in kAlwaCrops) {
    if (c.code == code) return c;
  }
  final p = alwaProduct(code);
  if (p == null) return AlwaCrop(code, code, code, '🌱');
  return AlwaCrop(
    code,
    p.nameEn,
    p.nameKu ?? p.nameEn,
    _productEmoji[code] ?? kGroupEmoji[p.group] ?? '🌱',
  );
}

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
    required this.product,
    required this.group,
    required this.unit,
    required this.quantity,
    required this.priceIqd,
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

  /// The product's code (a crop code for a crop).
  final String product;
  final String group;

  /// `kg`, `tray_30`, `litre` or `head`.
  final String unit;

  /// How many, in [unit].
  final double quantity;

  /// The asking price for one [unit].
  final double priceIqd;

  /// The old names, from when a listing was only a crop in kg.
  String get crop => product;
  double get quantityKg => quantity;
  double get priceIqdPerKg => priceIqd;

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
    // The new fields; the old kg ones when the server has not sent them.
    product: _code(j),
    group: j['group'] as String? ?? alwaProduct(_code(j))?.group ?? 'crops',
    unit: j['unit'] as String? ?? alwaProduct(_code(j))?.unit ?? 'kg',
    quantity: _d(j['quantity']) ?? _d(j['quantity_kg']) ?? 0,
    priceIqd:
        _d(j['asking_price_iqd']) ?? _d(j['asking_price_iqd_per_kg']) ?? 0,
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

String _code(Map<String, dynamic> j) =>
    j['product'] as String? ?? j['crop'] as String? ?? '';

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
    required this.product,
    required this.quantity,
    required this.priceIqd,
    required this.lat,
    required this.lon,
    this.days = kAlwaMaxDays,
    this.sellerPhone,
  });

  /// The seller's sign-in phone, shown to buyers so they can call.
  final String? sellerPhone;
  final String product;

  /// How many, in the product's unit; the price is for one unit.
  final double quantity;
  final double priceIqd;
  final double lat;
  final double lon;

  /// Closes after this many days, 1 to 14.
  final int days;

  Map<String, dynamic> toJson(DateTime now) => {
    'product': product,
    // Whole numbers: the server's quantity and price are integers and
    // refuse 4000.0.
    'quantity': quantity.round(),
    'asking_price_iqd': priceIqd.round(),
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

/// A person who does farm work for pay (FRONTEND.md 5B): farmers call them.
class Worker {
  const Worker({
    required this.id,
    required this.name,
    required this.costIqd,
    this.costPer = 'day',
    this.phone,
    this.note,
    this.zoneSlug,
    this.lat,
    this.lon,
    this.distanceKm,
    this.available = true,
    this.createdAt,
    this.updatedAt,
  });
  final String id;
  final String name;
  final String? phone;
  final int costIqd;

  /// `day` or `hour`.
  final String costPer;

  /// What work they do.
  final String? note;
  final String? zoneSlug;
  final double? lat;
  final double? lon;

  /// Only when the list was asked with a point.
  final double? distanceKm;

  /// False when the worker paused the card (only their own card says so).
  final bool available;
  final DateTime? createdAt;
  final DateTime? updatedAt;

  factory Worker.fromJson(Map<String, dynamic> j) => Worker(
    id: '${j['id']}',
    name: j['name'] as String? ?? '',
    phone: j['phone'] as String?,
    costIqd: (j['cost_iqd'] as num?)?.round() ?? 0,
    costPer: j['cost_per'] as String? ?? 'day',
    note: j['note'] as String?,
    zoneSlug: j['zone_slug'] as String?,
    lat: _d(j['lat']),
    lon: _d(j['lon']),
    distanceKm: _d(j['distance_km']),
    available: j['available'] != false,
    createdAt: DateTime.tryParse(j['created_at'] as String? ?? '')?.toLocal(),
    updatedAt: DateTime.tryParse(j['updated_at'] as String? ?? '')?.toLocal(),
  );
}
