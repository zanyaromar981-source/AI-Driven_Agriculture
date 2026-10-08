import '../../api/api.dart';
import 'history_text.dart';

/// A crop group on the "What fits this field" card.
class CropGroup {
  const CropGroup(this.kind, this.crops, this.why);

  /// fits, irrigation or poor.
  final String kind;

  /// Crop codes from BACKEND.md; a trailing "?" marks a doubtful fit.
  final List<String> crops;
  final String why;
}

/// Which common crops usually suit this field, as a rule of thumb from
/// FAO EcoCrop-style crop needs (soil texture, pH, season rain, spring heat).
/// Not checked against salinity, soil depth or the farmer's water source.
List<CropGroup> cropFit(FarmInsights f) {
  final soil = f.topic('soil');
  final rain = f.topic('rain');
  final weather = f.topic('weather');
  final clay = soil?.m('clay_pct_topsoil');
  final sand = soil?.m('sand_pct_topsoil');
  final ph = soil?.m('ph_topsoil');
  final normal = rain?.m('normal_mm_oct_may');
  final heat = weather?.m('spring_heat_days_normal');
  if (soil == null || rain == null || clay == null || normal == null) {
    return const [];
  }
  final cls = soilClass(clay, sand);
  final groups = <CropGroup>[];

  final rainfed = <String>[];
  if (normal >= 300) rainfed.add('wheat');
  if (normal >= 200) rainfed.add('barley');
  if (normal >= 300) rainfed.add(cls == 'heavy' ? 'chickpea?' : 'chickpea');
  if (rainfed.isNotEmpty) {
    groups.add(
      CropGroup(
        'fits',
        rainfed,
        'Winter crops that handle ${cls == 'heavy' ? 'heavy clay' : '$cls soil'} and about ${normal.round()} mm of rain.',
      ),
    );
  }

  final irrigated = ['tomato', 'cucumber'];
  final lightLovers = ['potato', 'onion', 'watermelon'];
  if (cls == 'light') irrigated.addAll(lightLovers);
  groups.add(
    CropGroup(
      'irrigation',
      irrigated,
      heat != null && heat >= 10
          ? 'April and May bring about ${heat.round()} days over 31 °C, and summers are dry.'
          : 'Summers are dry: these need water after the rain stops.',
    ),
  );

  if (cls == 'heavy') {
    groups.add(
      const CropGroup('poor', [
        'potato',
        'onion',
        'watermelon',
      ], 'They grow best in light, sandy soil; heavy clay holds them back.'),
    );
  }
  if (ph != null && (ph < 5.5 || ph > 8.5)) {
    groups.add(
      CropGroup(
        'poor',
        const [],
        'The soil pH (${ph.toStringAsFixed(1)}) is outside what most crops like: test it first.',
      ),
    );
  }
  return groups;
}

/// Short suggestions from this field's own history. No doses, no promises.
List<String> suggestions(FarmInsights f) {
  final out = <String>[];
  final share = drySeasonShare(f.topic('rain'));
  final droughts = f.topic('rain')?.m('drought_seasons');
  if (share != null && droughts != null) {
    out.add(
      'About $share was dry here (${droughts.round()} since 1981). If you choose between wheat and barley, barley copes better with a dry year.',
    );
  }
  final hard = hardFrostSeasons(f.topic('weather'));
  if (hard.isNotEmpty) {
    out.add(
      'Hard spring frost came ${hard.length} times since 1981. Ask your agricultural office which wheat variety heads late enough for this area.',
    );
  }
  final heat = f.topic('weather')?.m('spring_heat_days_normal');
  if (heat != null && heat >= 10) {
    out.add(
      'Plan water before planting tomato or cucumber: without irrigation the hot April and May will stop them.',
    );
  }
  return out;
}
