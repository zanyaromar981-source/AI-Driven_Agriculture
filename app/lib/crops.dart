import 'package:flutter/material.dart';

/// The 12 crop codes from BACKEND.md section 1, in the order of the design's picker.
class Crop {
  const Crop(this.code, this.emoji, this.color);
  final String code;
  final String emoji;
  final Color color;
}

const kCrops = <Crop>[
  Crop('wheat', '🌾', Color(0xFFE0B13A)),
  Crop('barley', '🌿', Color(0xFFC8B560)),
  Crop('tomato', '🍅', Color(0xFFD9483B)),
  Crop('cucumber', '🥒', Color(0xFF6DB352)),
  Crop('potato', '🥔', Color(0xFFA9784A)),
  Crop('onion', '🧅', Color(0xFFB46FA8)),
  Crop('watermelon', '🍉', Color(0xFFEF7C8E)),
  Crop('grape', '🍇', Color(0xFF7E57C2)),
  Crop('olive', '🫒', Color(0xFF7D8B3A)),
  Crop('sunflower', '🌻', Color(0xFFF5C518)),
  Crop('chickpea', '🫘', Color(0xFFD9B88A)),
  Crop('empty', '⬜', Color(0x33FFFFFF)),
];

/// Crops grown in the area (BACKEND.md 2.17), shown on the advice card.
/// Kept out of [kCrops] so the paint picker stays at 12.
const kAreaCrops = <Crop>[
  Crop('rice', '🍚', Color(0xFFE8E2C8)),
  Crop('fig', '🟣', Color(0xFF8E5A8C)),
  Crop('sumac', '🍂', Color(0xFFB0413E)),
  Crop('pistachio', '🥜', Color(0xFF93C572)),
];

Crop cropOf(String code) => [
  ...kCrops,
  ...kAreaCrops,
].firstWhere((c) => c.code == code, orElse: () => kCrops.last);
