# Farm AI ideas check (2026-10-08)

The user pasted three AI-written lists of farm problems and AI fixes for Kurdistan. Six research agents checked each claim against 2020–2026 sources. Verdict scale: **real & big**, **real but small**, **weak evidence**. "Fit" means fit with our wheat drought alarm (government planners, rain-fed wheat, weather + satellite data).

## Summary

| # | Idea | Problem real? | AI fix honest in 48 h? | Fit | Decision |
|---|---|---|---|---|---|
| 1 | **Wheat rust / sunn pest risk from weather** (reframed from "phone scan") | Real & big: rust in Raparin and Kirkuk, Apr 2024; yellow rust costs 10–70% on weak varieties | Yes: weather rules (Morocco stripe-rust thresholds, Turkish sunn-pest degree-days), free hourly forecasts | **Strong**: same users, crop and data; matters in WET years | **Add** |
| 2 | Yield before harvest | Real | Already built and tested: mid-April score 0.89, tonnes ±20–25% | Already in | **Have it** |
| 3 | Sorani voice/text advisor | Need real: 215 extension staff for ~50,000 wheat farmers | Only as a channel for our alerts, with no doses | Medium | **Use as delivery** |
| 4 | Irrigation timing (groundwater) | Real & big: Erbil basin −75 m (2005–24) | Yes (FAO-56 water balance), but no local proof | Medium (irrigated farms, farmer users) | Later |
| 5 | Late spring frost | Real but weakly documented | Map possible, but valley forecasts run ~3 °C too warm | Medium | Later |
| 6 | Phone photo pest diagnosis | Problem real; fix weak (31–65% right in real fields) | No local images | — | Drop |
| 7 | Fertilizer variable-rate maps | Weak: farmers likely under-use (prices up 2.4×) | No ground truth; no variable-rate spreaders | Breaks the no-chemicals rule | Drop |
| 8 | Price crash forecasts | Real & big (tomatoes 100 IQD/kg, 2025) | No: policy and smuggling drive prices, and there are no daily wholesale data | Weak | Drop |
| 9 | Wheat grading at silos | Real but medium (Makhmour 2024: 40% rejected, bribes alleged) | Camera grading works in Turkey (91–99.9%), but can't read moisture and has no legal standing | Poor | Drop |
| 10 | Orchard ripeness by phone | Real crop (80,000 t pomegranate); the bottleneck is markets | Phone sugar estimates aren't reliable | Poor | Drop |
| 11 | Weed spot-spraying | Weeds real (mostly grasses) | Needs hardware; grass weeds in wheat are the hard case; farmers use backpack sprayers | Poor | Drop |
| 12 | Hornet detection at hives | Beekeeping medium (~16k keepers); hornet losses undocumented | Detection works (VespAI), but a "repellent alarm" is unproven and needs hardware | Poor | Drop |
| 13 | Diesel pump fuel theft | Weak: no farm evidence; grid improving | Industrial data only | Poor | Drop |

## Facts worth using in the pitch (verified)
- Upstream water can't be relied on. Iran's Nowsud tunnel (47–48 km, since Aug 2020) can redirect the Sirwan in drought seasons. Darbandikhan got 0.9 bcm in 2021 vs a 4.7 bcm average, and inflow from Iran was "almost zero" in Oct 2022 ([MEE](https://www.middleeasteye.net/news/iraq-kurdistan-water-catastrophe-iran-rivers), [Rudaw](https://rudaw.net/english/middleeast/iraq/100420221)). Rain stays the only input we can count on.
- Iraq: +2 °C and −9% rain by 2050 ([Atlantic Council](https://www.atlanticcouncil.org/programs/middle-east-programs/rafik-hariri-center-for-the-middle-east/mena-futures-lab/macromena/climate-profile-iraq/)).
- DPM Qubad Talabani, 30 Mar 2026, at the Sulaymaniyah Agriculture Directorate: asked for a "comprehensive roadmap" for agriculture ([PUKmedia](https://pukmedia.com/EN/Details/80948)).
- Bazian valley: 12,000–14,000 plastic tunnel houses, up from zero in 2007. The Dutch-funded "Bazian Valley Fresh" brand launched Jul 2026; pesticide residues in 5 samples were below EU limits ([RVO 2024](https://www.agroberichtenbuitenland.nl/actueel/nieuws/2024/09/18/value-chain-development-in-kurdish-region-of-iraq), [RVO 2026](https://www.agroberichtenbuitenland.nl/actueel/nieuws/2026/07/31/iraq-new-dutch-backed-vegetable-brand-for-kurdistan-region)). These are irrigated vegetables, not wheat.
- Extension gap: about 215 extension workers and engineers across KRI ([UHD journal](https://journals.uhd.edu.iq/index.php/uhdjst/article/view/1201)); about 50,000 wheat farmers on 3.3 M dunams (2026).
- Wheat purchases from KRI: 700k t (2024) → 400k t (2025) → ~290k t (2026) ([Rudaw](https://www.rudaw.net/english/kurdistan/250420261)). Government price 700,000 IQD/t in plan, 500,000 outside.
- Wet 2026: Sulaimani groundwater rose 2–16 m in April 2026, the first rise in 6 years ([Rudaw](https://rudaw.net/sorani/kurdistan/010420265)). Don't say "drought now".

## Do NOT use
- "Baghdad AI irrigation: +35% yield, −36% water": real paper (AL-Rubaye, *Agribusiness*, Feb 2026), but one season on irrigated wheat in Baghdad.
- "93–96% soil prediction": a Pakistan/US rice prototype tested on datasets.
- "Diyala PhD, 85% disease accuracy": not found.
- "Over 60% herbicide saved": corn and soybean figures, not wheat with wild oats.
- "Fuel theft on farm pumps": no evidence in KRI.

## Details per idea (key sources)

**1. Rust and sunn pest.**
- Raparin rust, Apr 2024: about 180,000 dunams of wheat in the area. Local officials blamed planting time and seed, not late detection ([Rudaw](https://rudaw.net/english/kurdistan/04042024)).
- Kirkuk leaf rust, Apr 2024: the directorate ran a spraying campaign ([KirkukNow](https://kirkuknow.com/en/fromPeople/673)).
- Sunn pest, Koya 2021: farmers know the pest. The problems are spray money and unsprayed neighbouring fields ([Rudaw](https://rudaw.net/english/lifestyle/25032021)).
- Weather rules to reuse:
  - Morocco stripe rust: humidity >90%, 8–16 °C, ≥4 h without rain ([USQ](https://research.usq.edu.au/item/q5q78/weather-based-predictive-modeling-of-wheat-stripe-rust-infection-in-morocco)).
  - Turkey sunn pest: base temperature 13.3 °C, 84 degree-days to first nymphs ([TUBITAK](https://journals.tubitak.gov.tr/agriculture/vol40/iss4/11)).
  - Ethiopia runs 7-day rust forecasts ([DOAJ](https://doaj.org/article/37f1d413f134418f8de21d666978ee3d)).
- Not validated for KRI.

**3. Advisor.**
- Phone advice raises yield by about 4% on average.
- No yield results exist yet for LLM chatbots.
- Harm case: a chatbot's spray plan killed ~10 ha of sesame in China (Jul 2026) ([AsiaE](https://view.asiae.co.kr/en/article/2026081809013307789)).
- 28.5% of farmers in Duhok, Nineveh and Basra use banned products ([KirkukNow](https://kirkuknow.com/en/news/71519)).
- Sorani speech-to-text: Google Chirp. Badini speech recognition gets 55% of words wrong, so it's not usable.

**4. Irrigation.**
- Erbil groundwater falls ~1.24 m/yr.
- Sulaimani has ~15,000 illegal wells.
- Elsewhere, advice saved water: IRRISAT Italy 25–30%, Chameleon sensors in Africa ~50%.
- In KRI, only FAO's WaPOR monitoring exists (Shamamuk); there is no farmer advice service.

**5. Frost.**
- Feb 2025 Duhok cold snap damaged blooming walnut and quince; no numbers given.
- The 2026 orchard losses were from rain and hail.
- KRI orchards have no frost protection equipment.
- In Korea, 62.8% of alert users said the alerts helped.

**8. Prices.**
- Duhok potatoes: 150 vs 400–450 IQD/kg when Iranian imports arrive (Apr 2024).
- Sep 2026: tomato farm-gate price fell 40%, followed by an import ban.
- WFP monthly retail prices exist (HDX), but they hide farm-gate crashes.

**9. Silos.**
- 2020 grade prices: about $466 / $391 / $308 per tonne; bribes reported ([Al Jazeera](https://www.aljazeera.com/economy/2020/6/10/rampant-corruption-scorches-iraqs-grain-farmers)).
- Turkish Grain Board camera test on 4,119 samples ([journal](https://www.agriculturejournal.org/volume14number1/determination-of-besatz-in-cereals-using-physical-analysis-instrument-based-on-imaging-and-artificial-neural-network-technology/)).

**10–13.**
- Pomegranate peel colour doesn't track ripeness.
- John Deere See & Spray results are from corn and soybean.
- VespAI hornet detection: ≥0.99 precision.
- Grid: 85% of households had 24 h power by Apr 2026.
