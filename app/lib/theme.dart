import 'package:flutter/material.dart';

/// Colour tokens, copied from the Pencil file design/jutyar_app.pen.
class JColors {
  static const bg = Color(0xFFF2F4EF);
  static const card = Color(0xFFFFFFFF);
  static const accent = Color(0xFF1E7A5A);
  static const accentSoft = Color(0xFFE3F0EA);
  static const gold = Color(0xFFB98A2B);
  static const goldSoft = Color(0xFFF6EDD9);
  static const ink = Color(0xFF162019);
  static const muted = Color(0xFF5E6E64);
  static const faint = Color(0xFF8A978F);
  static const placeholder = Color(0xFF9AA79F);
  static const line = Color(0xFFDDE3DA);
  static const cardLine = Color(0xFFE7EBE4);
  static const toggleBg = Color(0xFFE6EAE2);
  static const levelNormal = Color(0xFF2E8B5B);
  static const levelNormalSoft = Color(0xFFD9EEDF);
  static const levelWatch = Color(0xFFE3A11F);
  static const levelWatchSoft = Color(0xFFFBEBD3);
  static const levelAlarm = Color(0xFFB23A2E);
  static const levelAlarmSoft = Color(0xFFF6DCD8);
  static const levelNone = Color(0xFFC7CDC4);
  static const levelNoneSoft = Color(0xFFEEF0EC);
  static const rayGold = Color(0xFF8C6414);
  static const sunCentre = Color(0xFFA6761C);
}

const kFontKu = 'NotoSansArabic';
const kFontLat = 'Manrope';

/// Text in the current language: Sorani uses Noto Sans Arabic, English uses Manrope.
TextStyle jText(
  bool ku, {
  required double size,
  FontWeight weight = FontWeight.w400,
  Color color = JColors.ink,
  double? height,
}) => TextStyle(
  fontFamily: ku ? kFontKu : kFontLat,
  fontFamilyFallback: const [kFontLat, kFontKu],
  fontSize: size,
  fontWeight: weight,
  color: color,
  height: height,
);

/// Latin text (numbers, brand, phone numbers) in every language.
TextStyle latText({
  required double size,
  FontWeight weight = FontWeight.w500,
  Color color = JColors.ink,
  double? height,
  double? letterSpacing,
}) => TextStyle(
  fontFamily: kFontLat,
  fontFamilyFallback: const [kFontKu],
  fontSize: size,
  fontWeight: weight,
  color: color,
  height: height,
  letterSpacing: letterSpacing,
);

ThemeData jutyarTheme() => ThemeData(
  useMaterial3: true,
  scaffoldBackgroundColor: JColors.bg,
  colorScheme: ColorScheme.fromSeed(
    seedColor: JColors.accent,
    surface: JColors.bg,
  ),
  fontFamily: kFontLat,
  textSelectionTheme: const TextSelectionThemeData(
    cursorColor: JColors.accent,
    selectionHandleColor: JColors.accent,
  ),
);
