import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/api/api.dart';
import 'package:jutyar/api/fake_api.dart';
import 'package:jutyar/app_scope.dart';
import 'package:jutyar/l10n/strings.dart';
import 'package:jutyar/screens/alwa/alwa_home_screen.dart';
import 'package:jutyar/screens/alwa/alwa_my_listings_screen.dart';
import 'package:jutyar/screens/alwa/alwa_sell_screen.dart';
import 'package:jutyar/screens/alwa/alwa_widgets.dart';

/// The phone stands in the centre of Sulaymaniyah, where the demo sellers are.
const _here = GpsFix(35.5617, 45.4329, 8);

/// The demo server with scripted failures and an empty market.
class _Market extends FakeApi {
  _Market({this.empty = false, this.fail, this.tooMany = false});
  final bool empty;
  final ApiException? fail;
  final bool tooMany;

  @override
  Future<List<AlwaListing>> alwaListings({double? lat, double? lon}) async {
    if (fail != null) throw fail!;
    return empty ? [] : super.alwaListings(lat: lat, lon: lon);
  }

  @override
  Future<AlwaPriceBoard?> alwaPriceBoard({double? lat, double? lon}) async {
    if (fail != null) throw fail!;
    if (!empty) return super.alwaPriceBoard(lat: lat, lon: lon);
    return const AlwaPriceBoard(
      market: AlwaMarket(slug: 'sulaymaniyah', nameEn: 'Sulaymaniyah'),
      day: null,
      prices: [],
      nearest: false,
    );
  }

  @override
  Future<AlwaListing> createAlwaListing(
    NewAlwaListing listing, {
    String? idempotencyKey,
  }) async {
    if (tooMany) throw ApiException(409, 'too_many_listings');
    return super.createAlwaListing(listing, idempotencyKey: idempotencyKey);
  }
}

void main() {
  late FakeApi api;

  setUp(() {
    AlwaGps.last = null;
    AlwaGps.locate = ({bool ask = true}) async => _here;
    AlwaMiniMap.showTiles = false;
  });

  /// A tall phone so the long Alwa pages fit, and a signed-in demo farmer.
  Future<void> signIn(
    WidgetTester t, {
    FakeApi? with_,
    String phone = '+9647501234567',
  }) async {
    t.view.physicalSize = const Size(1080, 4800);
    t.view.devicePixelRatio = 2.625;
    addTearDown(t.view.reset);
    api = (with_ ?? FakeApi())..assumeOnline = true;
    await t.runAsync(() => api.verifyOtp(phone: phone, code: '123456'));
  }

  Widget app(Widget home) => AppScope(
    ku: false,
    s: const S(false),
    api: api,
    setKu: (_) {},
    child: MaterialApp(home: Scaffold(body: home)),
  );

  /// Long enough for the demo server's answers (0.6 s each).
  Future<void> settle(WidgetTester t) async {
    for (var i = 0; i < 4; i++) {
      await t.pump(const Duration(seconds: 1));
    }
  }

  Finder rich(String s) => find.textContaining(s, findRichText: true);

  testWidgets('browse: prices and crops for sale, nearest first', (t) async {
    await signIn(t);
    await t.pumpWidget(app(const AlwaHomeScreen()));
    await settle(t);

    expect(find.text('Alwa market'), findsOneWidget);
    expect(find.text('TODAY AT THE ALWA'), findsOneWidget);
    expect(rich('Sulaymaniyah ·'), findsOneWidget);
    expect(find.text('gov. price'), findsOneWidget);
    expect(find.text('+7%'), findsOneWidget);
    expect(find.text('FOR SALE NEAR YOU'), findsOneWidget);
    expect(find.text('nearest first'), findsOneWidget);
    // The nearest seller first: tomato, 2 km away, 700 IQD/kg.
    expect(find.text('2 km'), findsOneWidget);
    expect(rich('· 4,000 kg'), findsOneWidget);
    expect(find.text('Closes in 18 h'), findsOneWidget);
    // 14 sellers: ten show, then "Show 4 more".
    expect(find.text('Show 4 more'), findsOneWidget);
    await t.ensureVisible(find.text('Show 4 more'));
    await t.pump();
    await t.tap(find.text('Show 4 more'));
    await t.pump();
    expect(find.text('Show 4 more'), findsNothing);
  });

  testWidgets('a crop filter shows only that crop', (t) async {
    await signIn(t);
    await t.pumpWidget(app(const AlwaHomeScreen()));
    await settle(t);
    final chip = find.text('Watermelon').first;
    await t.ensureVisible(chip);
    await t.pump();
    await t.tap(chip);
    await t.pump();
    expect(rich('· 8,000 kg'), findsOneWidget);
    expect(rich('· 4,000 kg'), findsNothing);
  });

  testWidgets('a listing shows crop, kg, price, phone, posted, closes', (
    t,
  ) async {
    await signIn(t);
    await t.pumpWidget(app(const AlwaHomeScreen()));
    await settle(t);
    await t.tap(rich('· 8,000 kg'));
    await settle(t);

    expect(find.text('Watermelon'), findsOneWidget);
    expect(find.text('250'), findsOneWidget);
    expect(find.text('IQD/kg asking'), findsOneWidget);
    expect(find.text('8,000 kg · 2,000,000 IQD for all of it'), findsOneWidget);
    expect(find.text('9 km from you'), findsOneWidget);
    expect(find.text('Quantity'), findsOneWidget);
    expect(find.text('8,000 kg'), findsOneWidget);
    expect(find.text('Posted'), findsOneWidget);
    expect(find.text('Closes'), findsOneWidget);
    expect(rich('9 days left'), findsOneWidget);
    expect(find.text('+964 771 987 6543'), findsOneWidget);
    expect(find.text('Copy the number to call'), findsOneWidget);
    expect(find.text('open'), findsOneWidget);
  });

  testWidgets('sell: refuses empty fields, then puts the crop on sale', (
    t,
  ) async {
    await signIn(t);
    await t.pumpWidget(app(const AlwaSellScreen()));
    await settle(t);

    expect(find.text('Your current location'), findsOneWidget);
    expect(find.text('GPS ±8 m'), findsOneWidget);
    expect(find.text('+964 750 123 4567'), findsOneWidget);
    expect(rich('You have 2 open.'), findsOneWidget);

    await t.tap(find.text('Put on sale'));
    await t.pump();
    expect(find.text('Pick a crop.'), findsOneWidget);
    expect(find.text('Type how many kg you sell.'), findsOneWidget);
    expect(find.text('Type your price per kg.'), findsOneWidget);
    await t.pump(const Duration(seconds: 3));

    await t.tap(find.text('Tomato'));
    TextField field(String key) => t.widget<TextField>(
      find.descendant(
        of: find.byKey(ValueKey(key)),
        matching: find.byType(TextField),
      ),
    );
    await t.enterText(
      find.descendant(
        of: find.byKey(const ValueKey('alwa-kg')),
        matching: find.byType(TextField),
      ),
      '4000',
    );
    await t.enterText(
      find.descendant(
        of: find.byKey(const ValueKey('alwa-price')),
        matching: find.byType(TextField),
      ),
      '700',
    );
    await t.pump();
    expect(field('alwa-kg').controller!.text, '4,000');
    expect(find.text('Pick a crop.'), findsNothing);

    // One day shorter than the default 14.
    expect(find.text('14'), findsOneWidget);
    await t.tap(find.bySemanticsLabel('One day less'));
    await t.pump();
    expect(find.text('13'), findsOneWidget);

    await t.tap(find.text('Put on sale'));
    await settle(t);
    expect(
      find.byType(AlwaSellScreen),
      findsNothing,
      reason: 'closed on success',
    );

    final mine = (await t.runAsync(() => api.myAlwaListings()))!;
    final made = mine.firstWhere(
      (l) => l.crop == 'tomato' && l.isOpen && int.parse(l.id) >= 900,
    );
    expect(made.quantityKg, 4000);
    expect(made.priceIqdPerKg, 700);
    expect(made.lat, _here.lat);
    expect(made.closesAt.difference(made.createdAt!).inDays, 13);
  });

  testWidgets('sell: 20 open listings shows the too-many banner', (t) async {
    await signIn(t, with_: _Market(tooMany: true));
    await t.pumpWidget(app(const AlwaSellScreen()));
    await settle(t);
    await t.tap(find.text('Wheat'));
    await t.enterText(find.byType(TextField).at(0), '1000');
    await t.enterText(find.byType(TextField).at(1), '800');
    await t.tap(find.text('Put on sale'));
    await settle(t);
    expect(find.text('You have 20 open listings'), findsOneWidget);
    expect(find.text('Open My listings'), findsOneWidget);
  });

  testWidgets('my listings: open, sold and closed, and mark as sold', (
    t,
  ) async {
    await signIn(t);
    await t.pumpWidget(app(const AlwaMyListingsScreen()));
    await settle(t);

    expect(find.text('My listings'), findsOneWidget);
    expect(find.text('OPEN · 2 OF 20'), findsOneWidget);
    expect(find.text('Tomato · 4,000 kg'), findsOneWidget);
    expect(find.text('Cucumber · 1,500 kg'), findsOneWidget);
    expect(find.text('Closes in 18 h'), findsOneWidget);
    expect(find.text('SOLD'), findsOneWidget);
    expect(find.text('Potato · 6,000 kg'), findsOneWidget);
    expect(rich('You marked it sold on'), findsOneWidget);
    expect(find.text('CLOSED'), findsOneWidget);
    expect(find.text('Barley · 5,000 kg'), findsOneWidget);
    expect(rich('Closed by itself on'), findsOneWidget);
    expect(find.text('Sell another crop'), findsOneWidget);

    await t.tap(find.text('Mark as sold').first);
    await settle(t);
    expect(find.text('OPEN · 1 OF 20'), findsOneWidget);

    // Delete asks first.
    await t.tap(find.text('Delete').first);
    await t.pumpAndSettle();
    expect(find.text('Delete this listing?'), findsOneWidget);
    await t.tap(find.text('Delete').last);
    await settle(t);
    expect(find.text('OPEN · 1 OF 20'), findsNothing);
  });

  testWidgets('empty market: calm empty states', (t) async {
    await signIn(t, with_: _Market(empty: true));
    await t.pumpWidget(app(const AlwaHomeScreen()));
    await settle(t);
    expect(find.text('No prices posted today'), findsOneWidget);
    expect(find.text('No crops for sale near you yet'), findsOneWidget);
  });

  testWidgets('my listings empty for a new phone', (t) async {
    await signIn(t, phone: '+9647501234560');
    await t.pumpWidget(app(const AlwaMyListingsScreen()));
    await settle(t);
    expect(find.text('You have nothing on sale yet'), findsOneWidget);
    expect(find.text('Sell a crop'), findsOneWidget);
  });

  testWidgets('no internet: says so with Try again', (t) async {
    await signIn(t, with_: _Market(fail: ApiException(0, 'offline')));
    await t.pumpWidget(app(const AlwaHomeScreen()));
    await settle(t);
    expect(find.text('No internet'), findsNWidgets(2));
    expect(find.text('Try again'), findsNWidgets(2));
  });

  testWidgets('location off: banner, crops still listed, no distance', (
    t,
  ) async {
    AlwaGps.locate = ({bool ask = true}) async =>
        throw const GpsProblem(GpsIssue.serviceOff);
    await signIn(t);
    await t.pumpWidget(app(const AlwaHomeScreen()));
    await settle(t);
    expect(find.text('Location is off'), findsOneWidget);
    expect(find.text('Turn on location'), findsOneWidget);
    expect(find.text('nearest first'), findsNothing);
    expect(find.text('2 km'), findsNothing);
    expect(rich('· 4,000 kg'), findsOneWidget);
  });

  test('time left and numbers read like the design', () {
    final now = DateTime(2026, 10, 9, 10);
    expect(timeLeft(DateTime(2026, 10, 21, 8, 10), now).text, '12 days left');
    expect(
      timeLeft(now.add(const Duration(hours: 18)), now).text,
      'Closes in 18 h',
    );
    expect(fmtInt(2800000), '2,800,000');
    expect(fmtKm(0.42), '420 m');
    expect(fmtKm(2.3), '2 km');
    expect(fmtPhone('+9647704128890'), '+964 770 412 8890');
    expect(fmtDayTime(DateTime(2026, 10, 7, 8, 10)), 'Wed 7 Oct, 08:10');
  });
}
