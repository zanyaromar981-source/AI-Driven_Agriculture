import '../api/api.dart';
import '../geo.dart' show fmtM2;

/// All user-facing text. Sorani (ku) is the default, English second.
/// Sorani copy still needs a native speaker's check.
class S {
  const S(this.ku);
  final bool ku;

  String get code => ku ? 'ku' : 'en';
  String t(String k, String e) => ku ? k : e;

  String get back => t('گەڕانەوە', 'Back');
  String get h1 => t('ژمارەی مۆبایلەکەت بنووسە', 'Enter your phone number');
  String get lead1 => t(
    'هیچ وشەی نهێنی و ناو نییە. کێڵگەکانت بە ئەم ژمارەیە دەبەسترێنەوە.',
    'No password, no name. Your farms are tied to this number.',
  );
  String get tel => t('ژمارەی مۆبایل', 'Mobile number');
  String get hintStart => t('کۆدێکی ٦ ژمارەیی بە', 'We send a 6-digit code by');
  String get hintEnd => t('بۆت دێت', '');
  String get send => t('کۆد بنێرە', 'Send code');
  String get noteTitle =>
      t('یەک ژمارە، هەموو کێڵگەکانت', 'One number, all your farms');
  String get noteBody => t(
    'ئەگەر زیاتر لە یەک کێڵگەت هەیە، هەموویان لەسەر هەمان ژمارە دەبن.',
    'If you have more than one farm, they all sit under the same number.',
  );
  String get h2 => t('کۆدەکە بنووسە', 'Enter the code');
  String get lead2 => t('کۆدێکمان نارد بۆ', 'We sent a code to');
  String get noCode => t('کۆدەکەت پێ نەگەیشت؟', "Didn't get the code?");
  String get resend => t('دووبارە بنێرە', 'Resend');
  String get verify => t('پشتڕاستکردنەوە', 'Verify');
  String get wrongCode => t('کۆدەکە هەڵەیە', 'Wrong code, try again');
  String get h3 => t('کێڵگەکانت', 'My farms');
  String get lead3 => t('بەستراو بە', 'Linked to');
  String get tagline => t('یارمەتیدەری کشتوکاڵی', 'farm advisor');
  String get add => t('کێڵگەیەکی نوێ زیاد بکە', 'Add a new farm');
  String get empty1 => t('هێشتا کێڵگەت نییە', 'No farms yet');
  String get empty2 => t(
    'لەسەر نەخشەکە کێڵگەکەت دیاری بکە. ماهوارە باقییەکەی دەکات.',
    'Mark your field on the map. The satellite does the rest.',
  );
  String get sent => t('کۆد نێردرا', 'Code sent');
  String get welcome => t('بەخێربێیت', 'Welcome');
  String get dunam => t('دۆنم', 'dunam');
  String get last => t('دوا وێنەی ماهوارە', 'last satellite picture');
  String get opening => t('دەچیتە', 'Opening');
  String get error => t('هەڵەیەک ڕوویدا', 'Something went wrong');

  // Add farm: walk the corners
  String get cornersTitle =>
      t('سنووری کێڵگەکەت دیاری بکە', 'Mark your field edge');
  String get cornersSub => t(
    'بڕۆ بۆ هەر گۆشەیەک و دوگمەکە دابگرە',
    'Walk to each corner and press the button',
  );
  String get dot => t('خاڵ', 'Dot');
  String get undo => t('لابردن', 'Undo');
  String get closeShape => t('شێوەکە دابخە', 'Close the shape');
  String get offlineHint =>
      t('بەبێ ئینتەرنێت کار دەکات', 'Works without internet');
  String get testTap =>
      t('تاقیکردنەوە: لەسەر نەخشە دابگرە', 'Test: tap the map to add a corner');
  String get waitingGps => t('چاوەڕێی GPS بکە', 'Waiting for GPS');
  String get gpsOff =>
      t('GPS کوژاوەتەوە. دابگرە بۆ هەڵکردن', 'GPS is off. Tap to turn it on');
  String get gpsDenied => t(
    'ڕێگە بە GPS نەدراوە. دابگرە بۆ ڕێگەدان',
    'No GPS permission. Tap to allow',
  );

  // Map styles
  String get mapStyle => t('شێوەی نەخشە', 'Map style');
  String styleName(String code) => switch (code) {
    'satellite' => t('ماهوارە', 'Satellite'),
    'map' => t('نەخشە', 'Map'),
    _ => t('بەرزی و نزمی', 'Terrain'),
  };
  String get metres => t('م', 'm');
  String get needThree =>
      t('لانیکەم ٣ گۆشە پێویستە', 'At least 3 corners are needed');
  String get crosses => t(
    'سنوورەکە خۆی دەبڕێت. گۆشەکان بە ڕیز دابنێ',
    'The edge crosses itself. Add the corners in walking order.',
  );
  String get tooBig => t(
    'کێڵگەکە لە 2,500,000 م² گەورەترە. گۆشەکان بپشکنە',
    'The field is over 2,500,000 m². Check the corners.',
  );
  String corners(int n) => t('$n گۆشە', '$n corners');

  // Walk mode
  String get modeTap => t('گۆشە بە گۆشە', 'Tap corners');
  String get modeWalk => t('بە ڕۆیشتن', 'Walk the edge');
  String get walkSub => t(
    'دەستپێک دابگرە و بە درێژایی سنوورەکەدا بڕۆ. خاڵەکان خۆیان دادەنرێن.',
    'Press Start and walk along the edge. The dots are placed for you.',
  );
  String get start => t('دەستپێک', 'Start');
  String get stop => t('وەستان', 'Stop');
  String walked(int m, int dots) => t('$m م · $dots خاڵ', '$m m · $dots dots');
  String get shapeClosed =>
      t('گەیشتیتەوە سەرەتا. شێوەکە داخرا', 'Back at the start. Shape closed.');
  // Offline
  String get savedOffline => t(
    'لەسەر مۆبایلەکە پاشەکەوت کرا. کە ئینتەرنێت هەبوو دەنێردرێت',
    'Saved on the phone. It uploads when there is internet.',
  );
  String get waitingUpload => t('چاوەڕێی ناردن', 'waiting to upload');
  String offlineList(String when) =>
      t('ئینتەرنێت نییە. کێڵگەکان وەک $when', 'No internet. Farms as of $when');
  String get draftBack => t(
    'سنوورە تەواونەکراوەکەت گەڕایەوە',
    'Your unfinished field edge is back',
  );
  String uploaded(int n) =>
      t('$n کێڵگە نێردرا', n == 1 ? '1 farm uploaded' : '$n farms uploaded');
  String rejected(String name) =>
      t('سێرڤەر "$name"ی وەرنەگرت', 'The server refused "$name"');
  String get noInternet => t('ئینتەرنێت نییە', 'No internet');

  String get walkFirst => t('سەرەتا وەستان دابگرە', 'Press Stop first');

  // Add farm: paint the crops
  String get paintTitle => t('ڕووەکەکان دیاری بکە', 'Mark the crops');
  String get paintSub =>
      t('پەنجەت بەسەر خانەکاندا ڕابکێشە', 'Drag your finger over the cells');
  String get sheetTitle => t('ئەم خانانە چین؟', 'What are these cells?');
  String get selectAll => t('هەمووی', 'Select all');
  String get selectFirst => t('سەرەتا خانەکان هەڵبژێرە', 'Select cells first');
  String get cells => t('خانە', 'cells');
  String get wholeFarm => t('هەموو کێڵگە', 'Whole farm');
  String get next => t('دواتر', 'Next');

  // Add farm: farm ready
  String get cropTypes => t('جۆر ڕووەک', 'crop types');
  String get satNote => t(
    'ماهوارە هەر ٥ ڕۆژ جارێک هەر خانەیەک دەبینێت',
    'The satellite sees every cell once every 5 days',
  );
  String get changeCrops => t('گۆڕینی ڕووەکەکان', 'Change crops');
  String get save => t('پاشەکەوت بکە', 'Save');
  String get saved => t('کێڵگەکە پاشەکەوت کرا', 'Farm saved');
  String get newFarmName => t('کێڵگەی نوێ', 'New farm');
  String get farmName => t('ناوی کێڵگە', 'Farm name');
  String get ok => t('باشە', 'OK');
  String get cancel => t('پاشگەزبوونەوە', 'Cancel');

  String crop(String? c) => switch (c) {
    'wheat' => t('گەنم', 'Wheat'),
    'barley' => t('جۆ', 'Barley'),
    'tomato' => t('تەماتە', 'Tomato'),
    'cucumber' => t('خەیار', 'Cucumber'),
    'potato' => t('پەتاتە', 'Potato'),
    'onion' => t('پیاز', 'Onion'),
    'watermelon' => t('شووتی', 'Watermelon'),
    'grape' => t('ترێ', 'Grape'),
    'olive' => t('زەیتوون', 'Olive'),
    'sunflower' => t('گوڵەبەڕۆژە', 'Sunflower'),
    'chickpea' => t('نۆک', 'Chickpea'),
    'empty' => t('بەتاڵ', 'Empty'),
    null => '',
    _ => c,
  };

  /// [hasPicture] = false means a brand-new farm the satellite has not seen yet.
  String status(FarmStatus st, {bool hasPicture = true}) => switch (st) {
    FarmStatus.normal => t('باش', 'looking good'),
    FarmStatus.watch => t('چاودێری', 'watch'),
    FarmStatus.alarm => t('مەترسی', 'alarm'),
    FarmStatus.none when !hasPicture => t(
      'چاوەڕێی ماهوارە',
      'waiting for satellite',
    ),
    FarmStatus.none => t('هێشتا نەچێنراوە', 'not sown yet'),
  };

  // Farm Home. English for now (decision 2026-10-08, BACKEND.md 2.9);
  // Sorani comes after a native speaker's check.
  String get tabHome => 'Home';
  String get tabAlerts => 'Alerts';
  String get tabAsk => 'Ask the Doctor';
  String get tabSettings => 'Settings';
  String get notBuilt => 'Not built yet';
  String get viewCells => 'Cells';
  String get viewCrops => 'Crops';
  String get viewFarm => 'Farm';
  String fromSpace(String date, String? next) => next == null
      ? 'From space $date'
      : 'From space $date · next picture $next';
  String waitingFirst(String? next) => next == null
      ? 'Waiting for the first satellite picture'
      : 'Waiting for the first satellite picture · expected $next';
  String levelName(FarmStatus l) => switch (l) {
    FarmStatus.normal => 'Normal',
    FarmStatus.watch => 'Watch',
    FarmStatus.alarm => 'Alarm',
    FarmStatus.none => 'No data',
  };

  /// "north-east" -> "north-east corner", "middle-east" -> "east side".
  String place(String where) {
    final [ns, ew] = where.contains('-') ? where.split('-') : [where, 'centre'];
    if (ns == 'middle' && ew == 'centre') return 'middle';
    if (ns == 'middle') return '$ew side';
    if (ew == 'centre') return '$ns side';
    return '$where corner';
  }

  /// Weak area of the measured area, in m² (user 2026-10-08: metres, not dunam or cells).
  String weakLine(double weakM2, double measuredM2, String? where) =>
      weakM2.round() == 0
      ? 'No weak spots · ${fmtM2(measuredM2)} m² measured'
      : '${fmtM2(weakM2)} of ${fmtM2(measuredM2)} m² weak${where == null ? '' : ' · ${place(where)}'}';
  String get notMeasured => 'Not measured yet';
  String get thisWeek => 'THIS WEEK';
  String pctOfNormal(int p) => '$p% of normal';
  String cellName(String label) => 'Cell $label';
  String sinceLine(String date, int days) {
    final w = days ~/ 7;
    final span = w >= 1
        ? (w == 1 ? '1 week' : '$w weeks')
        : (days == 1 ? '1 day' : '$days days');
    return 'Since $date · $span';
  }

  String compareLine(int? neighbours, int? whole) => [
    if (neighbours != null) 'Neighbours $neighbours%',
    if (whole != null) 'whole farm $whole%',
  ].join(' · ');
  String get askSpot => 'Ask about this spot';
  String get reportHere => 'Report here';
  String get noReading => 'No satellite reading for this cell yet';
  String measuredOn(String part, String all) => 'measured on $part of $all m²';
  String offlineCopy(String when) => 'No internet. Showing the copy from $when';
  String get weatherDown => 'Weather forecast not available right now';
  String forecastSource(String src, String when) =>
      'Forecast: $src · issued $when';
  String get nothingToDo => 'Nothing to act on in the next 10 days';
  String get loadFailed => 'Could not load this farm';
  String get retry => 'Try again';
  String get notSownYet => 'not sown yet';
  String get wholeFarmLabel => 'Whole farm';
  String get farmingAssistant => 'Farming assistant';

  // Area in square metres (user, 2026-10-08: "use meters, don't use donum").
  String get m2 => t('م²', 'm²');
  String get selectedArea => t('هەڵبژێردراو', 'Selected');
  String get oneSquare => t('یەک خانە', 'One square');
  String get insideFarm => t('لەناو کێڵگەکەدا', 'inside the farm');
}
