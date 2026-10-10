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
  if (clay == null && normal == null) return const [];
  final cls = clay == null ? null : soilClass(clay, sand);
  final groups = <CropGroup>[];

  if (normal != null) {
    final rainfed = <String>[];
    if (normal >= 300) rainfed.add('wheat');
    if (normal >= 200) rainfed.add('barley');
    if (normal >= 300) rainfed.add(cls == 'heavy' ? 'chickpea?' : 'chickpea');
    if (rainfed.isNotEmpty) {
      final soilWords = switch (cls) {
        null =>
          'about ${normal.round()} mm of rain. Your soil is not read yet, so this may change',
        'heavy' => 'heavy clay and about ${normal.round()} mm of rain',
        _ => '$cls soil and about ${normal.round()} mm of rain',
      };
      groups.add(
        CropGroup('fits', rainfed, 'Winter crops that handle $soilWords.'),
      );
    }
  } else if (cls != null) {
    groups.add(
      CropGroup(
        'fits',
        ['wheat', 'barley', if (cls != 'heavy') 'chickpea'],
        'From the soil only: winter crops that handle ${cls == 'heavy' ? 'heavy clay' : '$cls soil'}. The rain here is not read yet, so this may change.',
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

/// Crops a whole district is known for, recommended to every farm in it
/// (user, 2026-10-10: "for all Akre district recommend rice"). Facts from
/// local reporting (Kurdistan24, Rudaw): about 13,000 dunams of rice a year.
List<CropGroup> districtCrops(String? zoneSlug) => switch (zoneSlug) {
  'akre' => const [
    CropGroup(
      'local',
      ['rice'],
      "Akre district is the Kurdistan Region's best-known rice area (Sadri rice). "
          'It is planted in flooded fields in April and May and harvested from '
          'mid-October, so it needs a summer stream or canal.',
    ),
  ],
  _ => const [],
};

/// One crop grown around the farm (BACKEND.md 2.17, topic `crops_grown`).
class GrownCrop {
  const GrownCrop(this.code, this.ha, this.irrigatedPct);
  final String code;
  final double ha;
  final double? irrigatedPct;
  bool get irrigated => (irrigatedPct ?? 0) >= 50;
}

/// The top crops grown within about 15 km, biggest first. Empty when the
/// topic is not in yet.
List<GrownCrop> grownInArea(FarmInsights f, {int top = 5}) {
  final t = f.topic('crops_grown');
  if (t == null) return const [];
  final out = <GrownCrop>[];
  for (final x in t.measures) {
    final ha = x.value;
    if (!x.code.endsWith('_ha_15km') || ha == null) continue;
    final code = x.code.substring(0, x.code.length - '_ha_15km'.length);
    out.add(GrownCrop(code, ha, t.m('${code}_irrigated_pct')));
  }
  out.sort((a, b) => b.ha.compareTo(a.ha));
  return out.take(top).toList();
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
  final (hardCount, _) = hardFrost(f.topic('weather'));
  if (hardCount > 0) {
    out.add(
      'Hard spring frost came $hardCount ${hardCount == 1 ? 'time' : 'times'} since 1981. '
      'Ask your agricultural office which wheat variety heads late enough for this area.',
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
