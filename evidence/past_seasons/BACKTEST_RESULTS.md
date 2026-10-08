# Backtest results: would the AI have warned before the bad harvests? (2026-10-06)

Zones: Slemani, Makhmour, Erbil, Chamchamal, Duhok, Garmiyan, Sharazur, Soran (8 of 16; cut on user request to shorten the satellite download). Seasons 1999/00–2024/25 (26). Rows per cutoff: 208 zone-seasons.

**Inputs known at the cutoff date:** rain since 1 Oct vs. the zone's 1991–2020 normal, soil moisture 0–100 cm (7-day mean) vs. normal, mean temperature anomaly, ET0 vs. normal, last season's Oct–May rain vs. normal (all ERA5 via Open-Meteo), and from February the current MODIS greenness vs. the zone median.
**Outcome ("bad" zone-season):** MODIS 250 m NDVI on cropland-like pixels in an 8 km box per zone (pixels whose median spring peak is 0.30–0.75). Main definition = mean NDVI 22 Mar–9 May (grain filling) in the zone's bottom 25% of 2000–2025. Second definition = spring peak NDVI (6 Mar–9 May), bottom 25%.
**Model:** logistic regression, features standardised, L2. Leave-one-season-out: all zones of a season are held out together. "Warned" = probability ≥ 0.5 (also shown at 0.35). Baselines: always-normal, same-as-last-year, rain < 75% of normal.

> **CORRECTION 2026-10-07:** the February and March numbers in this first backtest are too good. The "satellite greenness now" input used composites that are not available by the cutoff (Feb used the Feb 18–Mar 5 composite; Mar used the Mar 22–Apr 6 composite, which is also part of the outcome). Honest numbers, 16 zones, with El Niño added: see **"Deep investigation"** at the end. March sensitive = 66% caught / 18% false alarms (not 71% / 16%); March cautious = 52% / 8%, 81% right overall.

## Season level (what the pitch says)
A season counts as bad when at least half of the zones were bad. Caught = the model warned (≥ half of zones ≥ threshold).

| Warning made at | Outcome = late-spring greenness | | Outcome = spring peak | |
|---|---|---|---|---|
| | caught (thr 0.5) | false alarms | caught (thr 0.5) | false alarms |
| End of Dec | 3/6 (thr 0.35: 5/6) | 2/20 (0.35: 3/20) | 3/6 (0.35: 6/6) | 1/20 (0.35: 4/20) |
| End of Jan | 2/6 (thr 0.35: 4/6) | 1/20 (0.35: 7/20) | 2/6 (0.35: 5/6) | 0/20 (0.35: 5/20) |
| End of Feb | 3/6 (thr 0.35: 5/6) | 1/20 (0.35: 2/20) | 3/6 (0.35: 5/6) | 1/20 (0.35: 3/20) |
| End of Mar | 4/6 (thr 0.35: 6/6) | 1/20 (0.35: 4/20) | 4/6 (0.35: 5/6) | 1/20 (0.35: 3/20) |

**Plain-language accuracy (late-spring outcome, threshold 0.5):**

- End of Dec: 21 of 26 seasons judged right (81%). Per zone: 29% of bad zone-seasons caught, 9% false alarms, balanced accuracy 60%.
- End of Jan: 21 of 26 seasons judged right (81%). Per zone: 20% of bad zone-seasons caught, 8% false alarms, balanced accuracy 56%.
- End of Feb: 22 of 26 seasons judged right (85%). Per zone: 45% of bad zone-seasons caught, 9% false alarms, balanced accuracy 68%.
- End of Mar: 23 of 26 seasons judged right (88%). Per zone: 52% of bad zone-seasons caught, 6% false alarms, balanced accuracy 73%.

### Season by season (late-spring outcome). B = bad season; number = model's mean risk at that date

| Season | Bad? | Dec | Jan | Feb | Mar | Zones bad |
|---|---|---|---|---|---|---|
| 1999/00 | **BAD** | **0.62** | 0.44 | **0.60** | **0.70** | Slemani, Makhmour, Erbil, Chamchamal, Garmiyan, Sharazur, Soran |
| 2000/01 |  | 0.31 | 0.41 | **0.46** | 0.29 | Erbil |
| 2001/02 |  | 0.27 | 0.24 | 0.25 | 0.18 | Sharazur |
| 2002/03 |  | 0.11 | 0.15 | 0.12 | 0.09 | Sharazur |
| 2003/04 |  | 0.10 | 0.06 | 0.06 | 0.14 | Sharazur |
| 2004/05 |  | 0.16 | 0.13 | 0.13 | 0.14 | Sharazur, Soran |
| 2005/06 |  | **0.53** | 0.35 | 0.29 | 0.41 | Sharazur |
| 2006/07 |  | 0.13 | 0.25 | 0.17 | 0.19 | Garmiyan |
| 2007/08 | **BAD** | **0.54** | **0.56** | **0.65** | **0.83** | Slemani, Makhmour, Erbil, Chamchamal, Duhok, Garmiyan, Sharazur, Soran |
| 2008/09 | **BAD** | **0.51** | **0.64** | **0.67** | **0.69** | Makhmour, Erbil, Chamchamal, Garmiyan |
| 2009/10 |  | 0.07 | 0.12 | 0.09 | 0.08 | Makhmour |
| 2010/11 | **BAD** | 0.44 | 0.29 | 0.34 | 0.33 | Makhmour, Erbil, Chamchamal, Duhok, Garmiyan, Soran |
| 2011/12 | **BAD** | 0.34 | 0.28 | 0.39 | **0.51** | Slemani, Makhmour, Erbil, Chamchamal, Duhok, Garmiyan, Soran |
| 2012/13 |  | 0.08 | 0.08 | 0.06 | 0.09 | Slemani |
| 2013/14 |  | 0.22 | 0.25 | 0.28 | 0.15 | Soran |
| 2014/15 |  | 0.04 | 0.08 | 0.05 | 0.05 | – |
| 2015/16 |  | 0.02 | 0.03 | 0.03 | 0.03 | – |
| 2016/17 |  | 0.26 | 0.39 | 0.52 | 0.30 | – |
| 2017/18 |  | **0.47** | **0.45** | 0.25 | 0.32 | Erbil, Duhok |
| 2018/19 |  | 0.01 | 0.02 | 0.02 | 0.01 | – |
| 2019/20 |  | 0.17 | 0.12 | 0.09 | 0.04 | – |
| 2020/21 |  | 0.26 | 0.37 | 0.27 | 0.31 | Slemani, Duhok |
| 2021/22 | **BAD** | 0.39 | 0.34 | 0.41 | 0.41 | Slemani, Makhmour, Chamchamal, Duhok, Garmiyan, Soran |
| 2022/23 |  | 0.36 | 0.40 | 0.35 | 0.16 | – |
| 2023/24 |  | 0.11 | 0.08 | 0.03 | 0.03 | – |
| 2024/25 |  | 0.27 | 0.34 | 0.33 | **0.45** | Slemani, Chamchamal, Duhok |

## Zone level (every zone-season separately, late-spring outcome)

| Warning made at | Recall (bad zone-seasons caught) | False-alarm rate | Precision | Accuracy | Always-normal acc. | Same-as-last-year recall / FAR | Rain<75% rule recall / FAR |
|---|---|---|---|---|---|---|---|
| End of Dec | 29% (thr 0.35: 64%) | 9% (0.35: 16%) | 55% | 75% | 73% | 43% / 21% | 86% / 33% |
| End of Jan | 20% (thr 0.35: 54%) | 8% (0.35: 24%) | 48% | 73% | 73% | 43% / 21% | 71% / 32% |
| End of Feb | 45% (thr 0.35: 62%) | 9% (0.35: 17%) | 66% | 79% | 73% | 43% / 21% | 71% / 28% |
| End of Mar | 52% (thr 0.35: 71%) | 6% (0.35: 16%) | 76% | 83% | 73% | 43% / 21% | 73% / 14% |

Model weights (standardised; negative = more of this lowers the risk):

- Dec: rain_pct -0.61, soil_pct -0.34, temp_anom -0.35, et0_pct +0.64, last_rain_pct -0.34
- Jan: rain_pct -0.42, soil_pct -0.16, temp_anom -0.42, et0_pct +0.71, last_rain_pct -0.37
- Feb: rain_pct -0.41, soil_pct -0.18, temp_anom -0.21, et0_pct +0.46, last_rain_pct -0.27, green_now -0.85
- Mar: rain_pct -0.51, soil_pct -0.23, temp_anom -0.30, et0_pct +0.43, last_rain_pct -0.21, green_now -1.12

## The 2025 test: trained on 1999/00–2023/24 only, shown data up to each date

Risk per zone (* = the zone really had a bad 2025 by the late-spring outcome):

| Zone | Rain Oct–Dec | Dec | Jan | Feb | Mar | Bad in 2025? |
|---|---|---|---|---|---|---|
| Slemani | 60% | 0.41 | **0.51** | 0.30 | **0.57** | **yes** |
| Makhmour | 44% | 0.25 | 0.30 | 0.32 | 0.31 | no |
| Erbil | 38% | 0.32 | 0.32 | 0.43 | **0.58** | no |
| Chamchamal | 47% | 0.16 | 0.21 | 0.21 | 0.30 | **yes** |
| Duhok | 46% | 0.36 | 0.40 | **0.62** | **0.80** | **yes** |
| Garmiyan | 97% | 0.31 | 0.37 | 0.12 | 0.11 | no |
| Sharazur | 120% | 0.14 | 0.21 | 0.12 | 0.16 | no |
| Soran | 50% | 0.23 | 0.42 | **0.50** | **0.77** | no |

## Sanity check on 2025/26 (the wet year, no label yet): mean risk per date

- Dec: mean 0.18, max 0.31
- Jan: mean 0.20, max 0.32
- Feb: –
- Mar: –

## Zone boxes

- Slemani: 1087 cropland-like pixels of 1089; median late-spring NDVI 0.432, bottom-25% cut 0.389
- Makhmour: 547 cropland-like pixels of 1089; median late-spring NDVI 0.285, bottom-25% cut 0.229
- Erbil: 1073 cropland-like pixels of 1089; median late-spring NDVI 0.487, bottom-25% cut 0.441
- Chamchamal: 1020 cropland-like pixels of 1089; median late-spring NDVI 0.361, bottom-25% cut 0.301
- Duhok: 1032 cropland-like pixels of 1089; median late-spring NDVI 0.433, bottom-25% cut 0.391
- Garmiyan: 651 cropland-like pixels of 1089; median late-spring NDVI 0.278, bottom-25% cut 0.247
- Sharazur: 900 cropland-like pixels of 1089; median late-spring NDVI 0.553, bottom-25% cut 0.518
- Soran: 1000 cropland-like pixels of 1089; median late-spring NDVI 0.447, bottom-25% cut 0.408

## Pre-season test: El Niño / La Niña known in September (added 2026-10-06, `backtest/enso_test.py`, `backtest/oni_boost.py`)
Source: NOAA ONI (Jul–Aug–Sep value, public early October). Outcome A = 8-zone season rain 1991/92–2025/26 (35 seasons, 9 bad). Correlation ONI(Sep) vs season rain: **+0.59**.

| September state | Seasons | Season rain (% of normal) | Droughts |
|---|---|---|---|
| El Niño (ONI ≥ +0.5) | 7 | mean 122%, **never below 101%** | **0 of 7** |
| Neutral | 21 | mean 99%, 48–155% | 6 of 21 (29%) |
| La Niña (ONI ≤ −0.5) | 7 | mean 74%, **never above 99%** | 3 of 7 (43%) |

- Rule "warn if La Niña": catches 3/9 droughts, 4 false alarms in 26 → weak as a warning. The strong side is the **all-clear**: no El Niño winter was a drought in 35 years.
- Zone-level pre-season model (ONI + last season + Sept soil), satellite outcome: 55% caught / 23% false alarms at best → not usable as a zone warning on its own.
- **Adding ONI to the monthly model helps:** Dec 64→70% caught, Jan 54→62%, Feb 62→68%, Mar 71→75% (sensitive setting, false alarms unchanged).
- **2026/27 outlook:** ONI JAS 2026 = +2.16, strong El Niño (strongest since 2015/16). Data says: wet-or-normal winter, drought unlikely. This is a live, checkable prediction for the Oct 10 pitch.
- Caveat: 7 seasons per phase is a small sample. Neutral years hold most droughts (2020/21, 2021/22, 2024/25), so "no El Niño" is not "safe".

### Hidden-winter test of the September call (`backtest/enso_hide_test.py`, output in `backtest/enso_hide_test_output.txt`)
Each of the 35 winters 1991/92–2025/26 hidden in turn; the call uses only the September Pacific state (NOAA ONI JAS) and the other winters.
- Calls made: **14 of 35** (7 El Niño → "wet or normal, no drought"; 7 La Niña → "dry-leaning, below normal"). 21 neutral winters → no call.
- **Direction right: 14 of 14.** Strong version right (El Niño → above normal / La Niña → drought): 10 of 14.
- Rain % predicted from ONI alone: average error 21 points vs 25 points for always guessing "normal" → the exact amount is not predictable, the direction is.
- The limit: 6 of the 9 droughts happened in neutral winters (incl. 2008/09, 2020/21, 2021/22, 2024/25) → the September call says nothing about them; the December warning takes over.
- 2026/27: ONI +2.16 → El Niño call "wet or normal winter"; the two comparable strong El Niños (1997/98, 2015/16) gave 129% and 142% of normal rain.

## Other pre-season signals tested (added 2026-10-07, `backtest/signals_test.py`, output `backtest/signals_test_output.txt`)
Question: is there anything else like El Niño/La Niña that is known before the season and predicts our winter rain? Same bar as before: 35 real seasons 1991/92–2025/26, region rain % of normal, bad = bottom 25%. 12 signals tested, each as its Jul–Sep state (Oct for snow). Data: `backtest_data/climate_indices/` (NOAA PSL, NOAA CPC, Rutgers Snow Lab).

| signal (known by end of Sept) | r vs season rain | verdict |
|---|---|---|
| **ENSO (El Niño/La Niña, ONI)** | **+0.59** | passes even the strict bar (0.46 for 12 tests). Only real signal. |
| Southern Oscillation Index | −0.57 | same phenomenon measured by air pressure; adds nothing on top of ONI |
| Atlantic Multidecadal (AMO) | −0.47 | looks good but is a slow 30-year swing (16 cool seasons avg 109%, 16 warm avg 87%); drops to −0.40 after removing the trend; ENSO alone on the same 32 seasons gives r=+0.68, ENSO+AMO cross-validated gives +0.65 → no gain. NOAA stopped publishing it in Jan 2023. Not used. |
| Pacific Decadal (PDO) | +0.32 | weak, overlaps with ENSO; no gain |
| Indian Ocean Dipole | +0.09 (Oct–Dec: +0.22) | nothing for the full season |
| Tropical N. Atlantic SST, QBO, Sep NAO, Sep AO, Sep EA/WR, Sep Scandinavia, Oct Eurasian snow | −0.27 … +0.12 | nothing. ENSO+QBO happened to score 9/9 caught, 5/26 false alarms in cross-validation, but QBO alone has no signal (−0.17) → treated as luck of testing 12 things. |

Same-winter atmosphere (Dec–Feb mean) vs season rain, for explanation only (cannot be known in advance): NAO −0.06, Arctic Osc. −0.09, EA/West Russia +0.33, Scandinavia +0.02. The NAO, which drives Europe's winters, does **not** drive ours.

**How early is the ENSO signal usable?** (ONI 3-month window → season rain)

| ONI window | known by | r | El Niño seasons → droughts | La Niña seasons → droughts (below normal) |
|---|---|---|---|---|
| MAM | early June | +0.21 | 7 → 3 | 6 → 4 (6) |
| AMJ | early July | +0.44 | 6 → 1 | 5 → 3 (5) |
| **MJJ** | **early August** | **+0.60** | **7 → 0** | 5 → 3 (5) |
| JAS | early October | +0.59 | 7 → 0 | 7 → 3 (7) |
| ASO | early November | +0.60 | 9 → 0 | 9 → 5 (9) |
| SON | early December | +0.61 | 12 → 0 | 12 → 7 (11) |

→ The call can be made in **early August**, two months earlier than the September version, with the same reliability. Before July the spring state of the Pacific says nothing (the known "spring barrier").
Strong El Niño seasons (ONI JAS ≥ +1.0): 1997/98 +1.8 → 129%, 2015/16 +1.7 → 142%, 2023/24 +1.2 → 110%. 2026: MJJ +1.39, JAS +2.16 (stronger than both 1997 and 2015 at this point).

## Caveats
- 8 zones, 26 seasons: small. Seasons are the real sample size (26), zones within a season move together.
- ERA5 is a weather model, not gauges; MODIS 250 m mixes fields, roads and villages. Farmland points were picked by hand near each town.
- The outcome is greenness, not yield or money. Greenness tracks the crop well in rainfed wheat/barley; it says nothing about irrigated summer crops.
- Logistic regression at threshold 0.5 is cautious; the risk number itself is what the app shows.


## Deep investigation: what to combine for the best prediction (2026-10-07)
Scripts: `backtest/deep_investigation.py` (+ `_part2.py`, `_part3.py`); outputs in `backtest/out_deep/` (`report.txt`, `report_part2.txt`, `report_part3.txt`, `summary.json`, `oof_predictions.csv`, `features_all.json`).
Setup: **all 16 zones**, seasons 1999/00–2024/25, leave-one-season-out (all zones of a season hidden together), same outcome as above. Fixed leak: satellite input now uses only composites really published by the cutoff (Feb: Feb 2–17; Mar: Feb 2–Mar 21).
Score: AUC = out of 100 pairs (one bad zone-season, one good), how often the model gives the bad one the higher risk (50 = coin flip). Bootstrap: 1000 reshuffles of seasons, "beats" ≥ 90% = probably real.

**Inputs tested** (on top of the current model: rain so far, soil moisture, temperature, ET0, last season's rain, satellite greenness from Feb):
| input | result |
|---|---|
| El Niño index (latest ONI at the cutoff) | **helps**: Jan 0.72→0.76 (beats 93%), Feb 0.76→0.78 (91%); pre-season it is the only thing that works (0.71) |
| rain timing (first soaking rain, rainy days, longest dry spell, last 45 days) | nothing (Dec 0.76→0.76, Jan 0.72→0.71) |
| region-wide rain + last spring's greenness | nothing or worse |
| 11 other climate indices (separate test above) | nothing |
| last year + September soil (pre-season) | useless (0.44) |

**AI models tested** (all inputs): logistic regression (LR), stronger-ridge LR, Random Forest (tree AI, 3 seeds, stable), look-alike years (k-nearest seasons, region-wide and same-zone), and averages.
- Look-alike years are the **weakest** predictor (same-zone: 0.62–0.75). Use them to *explain* ("this winter looks like 2007/08"), not to score.
- Random Forest beats LR in Jan/Feb (0.78/0.80), ties in Dec/Mar.
- **Final = "Team": average of LR+El Niño and Random Forest.** Beats the current model in 98% of reshuffles in Jan and Feb; ties in Dec and Mar.

**Final numbers, fixed thresholds (what the product would really do):**
| month | AUC | sensitive (risk ≥35%): caught / false alarms | cautious (≥50%): caught / false alarms / right overall |
|---|---|---|---|
| Pre-season (El Niño only) | 0.71 | 32% / 9% | not usable (6% caught) → give the direction only |
| Dec | 0.77 | 67% / 22% | 29% / 9% / 75% |
| Jan | 0.79 | 62% / 17% | 27% / 7% / 75% |
| Feb | 0.81 | 65% / 25% | 39% / 4% / 81% |
| Mar | 0.85 | 66% / 18% | 52% / 8% / 81% |
"Always say normal" is right 73% of the time. Pre-season and Dec rows are LR+El Niño (Team is not better there).

**Other findings**
- **Satellite beats perfect weather:** even knowing the real weather up to Apr 30 gives only AUC 0.79; weather + satellite at Mar 31 gives 0.84–0.85. The greenness picture carries information no rain forecast can.
- **What the model learned:** Dec: rain so far + El Niño + heat/ET0. Jan: El Niño is the strongest input. Feb: El Niño ≈ greenness. Mar: greenness dominates.
- **Per zone (Mar):** best Garmiyan 0.98, Chamchamal 0.96, Erbil 0.95, Koya 0.95; worst Ranya 0.58, Sharazur 0.74, Penjwen 0.74 (wetter mountain zones, Ranya next to Dukan lake / irrigation).
- **Erbil + Makhmour irrigation:** removing them makes results slightly worse (0.84→0.82), they are predicted well (0.93) → keep them; the rainfed mask is still nice-to-have, not urgent.
- **Season ranking (Mar):** the 5 bad seasons rank 1, 2, 7, 8, 12 of 26 by mean risk; 2024/25 ranked 5th.
- **2026/27 pre-season:** model gives ~2% chance of a bad spring, but ONI +2.16 is stronger than anything in training (max 2015/16 +1.73) → say "under 10%". The two strongest El Niño seasons in the satellite era, 2015/16 and 2023/24, had 0 of 16 zones bad.

**Still not tested:** satellite greenness in Dec/Jan (download queue, `modis_full/*_autumn|jan`); official seasonal forecast models (separate check running); soil type (SoilGrids paused).


## Investigation 2: what else can raise accuracy? (2026-10-07 afternoon)
Scripts: `backtest/improve_test.py`, `future_value_test.py`, `field_mask_test.py`, `miss_analysis.py`; reports in `backtest/out_deep/*_report*.txt`. Reference = LR + El Niño (AUC Dec 0.77, Jan 0.76, Feb 0.78, Mar 0.84).

| tested | result |
|---|---|
| 8 new weather inputs (frost days, warm days, snow share, shallow soil 0–7/7–28 cm, air dryness VPD, sunshine, rain hours; ERA5, 12 zones) | nothing (−0.03 … +0.00); all together makes it worse |
| satellite compared with the same date in other years (anomaly) | +0.00 … +0.01 (Mar latest-image anomaly beats ref in 88% of reshuffles: borderline) |
| greening speed Feb→Mar | +0.01 (66%) |
| zone dryness × rain (dry zones react more) | nothing |
| predict harvest size (regression) instead of bad/not-bad | AUC same; Mar catch@15%FA 67–71% vs 59% (62–69%) → small, not proven |
| cleaner field mask (only pixels that dry down in late May and are not green in Feb) | nothing; 99% of outcomes identical (8 km box average is robust) |
| **perfect knowledge of ALL future weather until Apr 30** (upper limit of any forecast) | Dec 0.77→0.82, Jan 0.76→0.82, Feb 0.78→0.85, Mar 0.84→0.86. Next month only: ≤ +0.02 |

**Where the errors are** (sensitive threshold 35%):
| month | severe failures (zone's worst 10%) caught | mild bad (10–25%) caught | false alarms in slightly-below-average seasons (25–50%) | false alarms in clearly good seasons (top half) |
|---|---|---|---|---|
| Dec | 75% | 61% | 33% | 17% |
| Jan | 75% | 50% | 35% | 22% |
| Feb | 81% | 48% | 39% | 19% |
| Mar | 79% | 53% | 39% | 11% |

**Conclusions**
1. Local weather is used up: no extra weather variable adds anything.
2. Even a perfect weather forecast adds at most 5–7 AUC points. Real seasonal forecasts can only add part of that.
3. Most errors are borderline seasons right next to the "bad" line; real disasters are caught ~8 of 10 times. Pitch line: "it catches 8 of 10 severe failures; the misses are near-normal years".
4. Remaining levers still being tested: Dec/Jan satellite pictures (download), CHIRPS satellite rain + NOAA crop-health index 1981+ (helper), official seasonal forecast models (helper).


## Investigation 3: satellite rain, NOAA crop health, and REAL official harvests (2026-10-07)
Data fetched by helper (see DATA_INVENTORY): CHIRPS v2 monthly rain 1981–Sep 2026 per zone (`backtest_data/chirps/`), NOAA STAR cropland VHI/VCI/TCI weekly per province 1982–2026 (`backtest_data/vhi/`), **KRSO official wheat & barley area/production/yield per governorate 1970–2023** (`backtest_data/harvest_stats/`, source: KRSO "Summary of agricultural crop data 1969–2023", April 2025). Script `backtest/new_sources_test.py`, report `backtest/out_deep/new_sources_report.txt`.

**CHIRPS rain and NOAA crop health as extra inputs:** no proven gain on any month (best: "average of both rains" Dec 79% of reshuffles, VCI/TCI Mar 69%). Not needed for the build.

**Does our satellite outcome match the official harvest?** (yield with technology trend removed)
| | wheat | barley |
|---|---|---|
| 16-zone mean late-spring greenness vs **Kurdistan Region** yield, 24 harvests 2000–2023 | **r = +0.78** | r = +0.66 |
| per governorate, 2000–2023 | Erbil +0.84, Duhok +0.62, Sulaymaniyah +0.27 | Erbil +0.74, Duhok +0.34, Sulaymaniyah −0.02 |
→ The satellite outcome is a real harvest signal (strongest for Erbil/Duhok wheat). Sulaymaniyah's official numbers barely follow the satellite; from 2019 the yields look like rounded estimates (700, 750).

**El Niño / La Niña vs 43 years of official wheat harvests (Kurdistan Region, 1981–2023):**
| state before sowing (ONI Jul–Sep) | harvests | bad (bottom 25%) | average yield vs trend |
|---|---|---|---|
| strong El Niño (≥ +1.0) | 3 (1987/88, 1997/98, 2015/16) | **0** | **125%** (103–140%) |
| weak/moderate El Niño | 5 | 2 | 90% |
| neutral | 28 | 4 | 110% |
| La Niña (≤ −0.5) | 7 | **5** | **65%** |
Barley: strong El Niño 0/3 bad (avg 101%); La Niña 5/7 bad (avg 63%).
→ La Niña = strong warning (5 of 7 bad wheat harvests). Weak El Niño is NOT a guarantee (1991/92, 2004/05, 2009/10 were 71–75%). Strong El Niño: all 3 good. 2026/27 is ONI +2.16 = strongest on record at this time of year.

**Predicting a bad wheat harvest per governorate** (3 governorates, seasons hidden one by one, 1991–2023): El Niño alone pre-season AUC 0.77; weather + El Niño Dec 0.76, Mar 0.75. On 1999–2022 only, AUC drops to 0.61–0.68 and adding MODIS greenness gives no gain (0.59–0.65): governorate harvest numbers are coarse (whole governorate, irrigated + rainfed mixed, estimates) and 24 seasons is a small test. Barley: no skill (≈0.5) → do not promise barley forecasts.


## Investigation 4: official seasonal forecast models (2026-10-07)
Helper downloaded 7 models (CanSIPS-IC4, NCEP-CFSv2, GFDL-SPEAR, NASA-GEOSS2S, COLA-CCSM4, COLA-CESM1, ECMWF-SEAS5) via IRI's keyless OPeNDAP endpoint, starts Sep/Oct/Nov/Dec 1991–2026, box 35–37°N 43–46°E (`backtest_data/seasonal_models/`, `backtest/fetch_seasonal_models.py`, `backtest/seasonal_models_test.py`, output `backtest/seasonal_models_test_output.txt`). Crop test: `backtest/seasonal_models_crop_test.py` → `out_deep/seasonal_models_crop_report.txt`.

**Rain forecast skill vs El Niño alone** (35 seasons, ERA5 16-zone rain):
- October forecast of the winter (Oct–May): models do NOT beat El Niño and add ~0 on top (they mostly repeat the El Niño signal; r with ONI +0.6…+0.85). Drought flagging Oct–May: ONI 7/9, CanSIPS 9/9, GFDL 8/9, model mean 7/9.
- **December start → Dec–Feb rain: real extra skill** (ONI r +0.41; CanSIPS .68, ECMWF .64, NASA .62; ONI+model cross-validated +0.23 to +0.28), stable when dropping years and in both halves of the record.
- 2026/27 (October 2026 start): 6 of 7 models put Oct–Dec in their wettest third (CanSIPS 126%, CFSv2 174%, GFDL 130%, NASA 111%, CCSM4 98%, CESM1 140%, ECMWF 145% of own normal). Agrees with the El Niño call: wet winter.

**But does it improve the crop warning?** Adding the December forecast (Jan–Mar or Dec–Feb rain; all 7 models or best 3) to the Dec and Jan warnings: greenness outcome AUC 0.74–0.77 vs 0.77/0.76 reference; official wheat harvest 0.72–0.77 vs 0.76/0.74 → **no gain** (best: 61% of reshuffles). Same lesson as the perfect-forecast test: better rain forecasts barely move the crop warning.
→ Use the models only as a public "second opinion" on the winter rain outlook (e.g. "6 of 7 international models agree: wet winter"), not as a model input.


## Investigation 5: winter satellite pictures + mid-April (2026-10-07)
- **January pictures** (composites Jan 1–16, Jan 17–Feb 1; `backtest/winter_green_test.py`, `out_deep/winter_green_report_janonly.txt`): Jan warning 0.76 → 0.77 (82% of reshuffles, not proven); Feb no change. **December pictures** (Nov 17–Dec 18, `out_deep/winter_green_report.txt`): Dec warning 0.77 → 0.76–0.77, no gain (5–14% of reshuffles). Jan with Dec+Jan pictures: best +0.01 (crop emergence, 88%), not proven. → Winter satellite pictures do not help; the satellite only starts to matter in February.
- **Mid-April check (Apr 15, graze-or-harvest decision)** (`backtest/april_test.py`, `out_deep/april_report.txt`): AUC **0.89**; sensitive 71% caught / 15% false alarms; cautious 53% / 5%, 83% right. This is where the first backtest's "March" numbers really belong: the Mar 22–Apr 6 picture is published around Apr 10–15. It already sees the start of the outcome window → call it a confirmation, not a forecast.

## Overall answer: what raises accuracy? (all investigations, 2026-10-07)
Tested ~30 extra inputs and model types against 26 hidden seasons (16 zones) plus 43 years of official KRSO harvests. Proven gains: **El Niño index** (Jan/Feb, pre-season) and the **Team model** (simple regression + Random Forest, Jan/Feb). Everything else (11 climate indices, rain timing, 8 extra weather variables, CHIRPS rain, NOAA crop health, field mask, zone effects, harvest-size regression, 7 seasonal forecast models, January pictures) adds ≤ 1 point or nothing.
Why: even a perfect weather forecast to Apr 30 adds only 5–7 points; the misses are borderline seasons; severe failures are already caught ~8 of 10.
Accuracy rises with the date, not with more data sources: Pre-season 0.71–0.77 → Dec 0.77 → Jan 0.79 → Feb 0.81 → Mar 0.85 → mid-Apr 0.89.


## Investigation 6: other satellite uses (2026-10-07, keyless Planetary Computer / FIRMS)
### 6a. Planted area + single-field check (Sentinel-2 10 m) — PARTLY works
Script `backtest/planted_area_test.py` (subcommands fetch/analyze/profile/fieldcheck); data `backtest_data/planted_area/`; outputs `backtest/out_planted_area/` (sown_fraction.csv, krso_comparison.txt, field_profiles.csv, classmap_*_2018-2026.png).
Method: 3 windows 5×5 km (Erbil plain, Koya plain, Sumel plain), per pixel max spring NDVI (Mar 5–May 5) and summer NDVI (Jul 15–Aug 31), SCL cloud mask; summer ≥0.25 = summer-green, else spring ≥0.45 = winter crop, else fallow/bare.
| harvest year | Erbil plain | Koya plain | Sumel plain |
|---|---|---|---|
| 2018 | 0.55 | 0.37 | 0.23 |
| 2019 wet | 0.73 | 0.64 | 0.77 |
| 2020 wet | 0.65 | 0.47 | 0.68 |
| 2021 dry | 0.46 | 0.27 | 0.25 |
| 2022 dry | 0.37 | 0.24 | 0.10 |
| 2023 | 0.56 | 0.61 | 0.57 |
| 2024 | 0.67 | 0.61 | 0.65 |
| 2025 dry | 0.54 | **0.02** | **0.12** |
| 2026 wet | 0.59 | 0.61 | 0.71 |
- Shows where a crop actually GREW (field-shaped map, every year). Koya plain 2025: 98% of land with no crop canopy (news: Koya lost 36,000 dunams). Erbil window partly irrigated (centre pivots) → drought hits less.
- Does NOT measure SOWN area: a sown-but-failed field looks like an unsown one. Vs KRSO area (2018–2023, n=6): negative r (−0.16 … −0.81) because KRSO area did not drop in drought years (2021 KRI total 5.03 M donum = highest) → KRSO area behaves like registered/sown area. Vs KRSO production: r = +0.48 … +0.70.
- Do NOT multiply KRSO-calibrated yield by this area (double-counts drought).
- Single field: full-season NDVI profile (14 dates) in **3.0–3.6 s** (occasional 20 s calls) → clear signatures: winter crop (rise Feb–Apr to ~0.97, dry by June), bare fallow (flat 0.1), irrigated summer crop, orchard/double crop. Cannot prove "sown but failed"; no ground truth → no accuracy number.
- Caveats: Planetary Computer reflectance offset inconsistent on some 2022+ scenes (handled by a check); no cropland mask (rangeland can pass 0.45 in wet years) → add ESA WorldCover; `/item/crop/*.npy` 404, use `/item/bbox/*.npy`.

### 6b. Crop fires (NASA FIRMS, keyless yearly CSVs + MCD64A1 burned area) — PARTLY works
Script `backtest/fires_test.py`; data `backtest_data/fires/`; outputs `backtest/out_fires/` (fires_by_governorate_year.csv, burned_area_mcd64a1_may_jul.csv, burned_area_cropland_split.csv).
May–Jul VIIRS detections, vegetation / on cropland (ESA WorldCover 2021; 430 year-round hotspots such as flares removed):
| year | Dohuk | Erbil | Sulaymaniyah | Kirkuk | Ninawa |
|---|---|---|---|---|---|
| 2018 | 119/9 | 455/120 | 343/95 | 151/128 | 93/61 |
| **2019** | 582/58 | **1070/548** | 783/238 | **3216/2137** | **5388/4791** |
| 2020 | 600/87 | 957/646 | 945/424 | 1967/1212 | 4159/3408 |
| 2021 | 1401/26 | 500/64 | 571/96 | 339/238 | 331/52 |
| 2022 | 52/7 | 83/28 | 39/15 | 215/180 | 70/39 |
| 2023 | 324/61 | 403/155 | 709/401 | 665/545 | 229/168 |
| 2024 | 513/133 | 495/310 | 842/422 | 972/635 | 1356/1169 |
(full 2012–2024 in CSV; NOAA-20 and MODIS agree)
- 2019 fire wave clearly visible (Kirkuk/Ninawa peak, 2–5× normal; MCD64A1 burned area 2019 = highest since 2001 for Ninawa 3,611 km², Kirkuk 949 km²). Dohuk 2021 spike = mountain fires (only 26 on cropland) → cropland mask essential.
- 2024 Erbil: 3rd highest of 13 on cropland; burned area 53 km² on cropland ≈ 8× the 2,513 dunams in the news (satellite includes stubble burning, 500 m pixels) → use for year patterns, not damage figures.
- Not "every fire": VIIRS passes ~13:30 and ~01:30; 375 m points. Free 7-day files are hours old (not minutes). 2025/2026 yearly files need a FIRMS MAP_KEY (not signed up).
- **Live-demo surprise:** cropland fires peak in Sep–Oct (stubble burning before sowing): Oct 2024 = 1,340 cropland detections in the 3 KRI governorates vs 584 in June. Last 7 days (Sep 30–Oct 6 2026): Erbil 170 (150 on cropland), Dohuk 57, Sulaymaniyah 42, peak Oct 4 → real fires can be shown live during the hackathon (residue burns, not harvest losses).

### 6c. Farmland built over by cities (Impact Observatory 10 m annual land cover 2017–2023) — PARTLY works
Script `backtest/farmland_loss_test.py`; data `backtest_data/farmland_loss/`; outputs `backtest/out_farmland_loss/` (farmland_loss_by_city_year.csv, farmland_loss_summary.json, worldcover_crosscheck.csv).
Rings: Erbil 25 km, Slemani 20 km, Duhok 20 km. Pixel-tracked 2017→2023:
| | Erbil | Slemani | Duhok |
|---|---|---|---|
| net built growth | +80.6 km² (+27%) | +74.7 km² (+43%) | +43.9 km² (+43%) |
| stable new built | 71.1 km² | 63.9 km² | 38.3 km² |
| farmland lost, low (was crops before) | 42.0 km² (16,800 dunams) | 29.5 km² (11,800) | 17.2 km² (6,900) |
| farmland lost, high (crops + rangeland) | 60.3 km² | 60.4 km² | 36.4 km² |
All three: ~89–157 km² = 35,000–63,000 dunams in 6 years.
- Built trend solid (85–86% of built pixels have clean histories); single years ±10 km² noise (2022 dip).
- Crop class swings with rainfall (100–350 km²/yr crops↔rangeland flips) → never present year-to-year crop area as "loss".
- Dataset's "built" ≈ 2× ESA WorldCover; vs published studies it likely overstates growth 2–3× (Erbil ~13 vs ~5 km²/yr; Duhok ~7 vs ~2.75 km²/yr) → present as a range, labelled "satellite estimate, likely high".
- Ankawa 2021 seizure (~1,000 dunams) not located/verified. Ignore the jtuh.org "325% lost" claim.
- Speed: ring-year statistics at 512 px 0.7–1.4 s (built within 0.6% of full res) → live-demo-able; change map must be precomputed (~50 s).

### 6d. Dam water from space (Sentinel-2 20 m + Landsat 30 m, NDWI > 0) — WORKS
Script `backtest/dam_water_test.py`; outputs `backtest/dam_water_areas.csv`, `dam_water_areas_window_medians.json`, `dam_water_test_output.txt`; cache `backtest_data/dam_water/` (939 API responses).
| year | Dukan late May/Jun km² | Dukan Sep/Oct | Darbandikhan late May/Jun | Darbandikhan Sep/Oct |
|---|---|---|---|---|
| 2008 (Landsat) | 132.5 | – | 39.2 | – |
| 2019 | 271.9 | 225.2 | 102.9 | 54.8 |
| 2021 | 182.6 | 159.7 | 40.2 | 35.8 |
| 2024 | 208.3 | 183.7 | 107.4 | 77.0 |
| **2025** | **112.8** | **82.7** | 66.1 | 47.3 |
| **2026** | **272.9** | **249.5** | 109.8 | 71.4 |
(all years 2017–2026 in the JSON)
- Reproduces AFP's "−56% 2019→2025" (we get 55–59%) and the 2026 refill (Dukan back to full 2019 size; Sep 2026 = 3× Sep 2025). Official full areas: Dukan 270 km² (ours 271–276), Darbandikhan 113 km² (ours 103–110). Sentinel-2 vs Landsat same day: 2% apart.
- Volume from area is rough (±0.3–1 bcm): June 2025 ≈ 1.9 bcm (reported 1.6–1.8); Dukan curve from Rashid 2023.
- 9–43 cloud-free dates/year; one lake-date measured live in 3–7 s (retry occasional 504s); use median of 2–3 dates (haze).

### 6e. Summer dam forecast (can winter data predict the summer lake?) — PARTLY (Dukan June only)
Scripts `backtest/dam_history_landsat.py` (Landsat lake areas 1988–2016 → `dam_history_landsat.csv`), `backtest/dam_forecast_test.py` (output `dam_forecast_test_output.txt`). Catchment rain: CHIRPS monthly at 6 points per catchment incl. Iranian side (`backtest_data/catchment_rain/`). Inputs: last October's lake area (from space), ONI Jul–Sep, catchment rain Oct→cutoff. Leave-one-year-out, 30–32 years 1989–2026.
| target | baseline error | forecast error | low summers flagged (bottom 25%) | false alarms |
|---|---|---|---|---|
| **Dukan June**, end of January | 35 km² | 28 km² (10% of full) | **8/9** | 5/23 |
| Dukan June, end of March | 35 | 26 | 7/9 | 6/23 |
| Dukan October (end of summer) | 35 | 27–31 | 0–3/9 | 1–4/23 |
| Darbandikhan June | 25 | 22–23 | 0/8 | 2–3/22 |
| Darbandikhan October | 12 | 10 | 3–4/8 | 4–7/22 |
- 2025 crisis: Dukan June forecast fell 197 (Nov) → 175 (Jan) → 168 (Mar) km², true 113 → right direction ("below normal"), but far too mild. 2008: Jan forecast 141 vs true 133 (good).
- Darbandikhan and late-summer levels: little or no skill (upstream dams in Iran, dam operations).
- 2027 (from Oct 2026 lake + ONI +2.16, extrapolated): Dukan June ≈ 264 km² (98% full); Darbandikhan June ≈ full.
→ Use: live measurement = strong; forecast = "direction from January" for Dukan only.

## Investigation 7: why 2024/25 was missed, and a separate rain-drought alarm (added 2026-10-07 night, `backtest/drought_alarm_test.py`, output `backtest/drought_alarm_test_output.txt`)
**Why the crop model missed 2024/25:** its inputs were extreme by 31 Jan 2025 (rain 50% of normal = 3rd driest of 27, soil and rainy days driest ever), but the crop risk stayed average (27–29% until Feb) because (1) 2023/24 was very wet (126% rain, 122% greenness) and the model learned a wet year softens the next, (2) El Niño was neutral (strongest weight in the Jan model, −0.63), and (3) the crops really held up: Feb greenness 109% of normal, final 93%, only 6/16 zones bad. Rain has a small weight in the crop model (−0.21 per SD).
**Crop model variants** (drop last-year inputs, add region rain): none beats the current Team (reshuffle wins 0.01–0.33). 2024/25 moves only from rank 13 → 11 of 26. → keep the crop model as is; it was not wrong about the crops.
**Rain-drought alarm (new, whole region, 36 winters 1990/91–2025/26, drought = Oct–May rain ≤ 80% of normal, 10 droughts):** once rain has fallen, the plain "rain since 1 Oct vs normal" beats every model; adding El Niño after December makes it worse (2024/25 rank 4 → 11 in Jan).

| Cutoff | Rain-so-far rule AUC | Droughts caught with 0 / 2 / 4 false alarms (of 26 normal winters) | 2024/25 rank of 36 |
|---|---|---|---|
| 30 Sep (El Niño only) | 0.87 | 0 / 3 / 8 of 10 | 19 (invisible) |
| 31 Dec | 0.82 | 1 / 3 / 4 | 12 |
| 31 Jan | 0.89 | 4 / 5 / 8 | **4** |
| 28 Feb | 0.94 | 6 / 6 / 9 | 4 |
| 31 Mar | 0.97 | 6 / 9 / 10 | 3 |

Candidate named rules (thresholds read off all 36 winters, so real-world will be slightly worse):
- **31 Jan: rain since 1 Oct < 55% of normal → early drought alarm.** History: 4 droughts (1998/99, 2007/08, 2008/09, 2024/25 at 50%), 1 false alarm (1990/91, 52%).
- **28 Feb: rain since 1 Oct < 67% of normal → drought alarm.** History: 6 of 10 droughts (incl. 2024/25 at 53%), **0 false alarms in 36 years**.

**Honest version of the rain rule** (`backtest/drought_rule_honest.py`, output `drought_rule_honest_output.txt`): for each winter the cut-off is learned from the other 35 winters only (largest cut-off with 0 false alarms there), then applied to the hidden winter.

| Date | Cut-off learned | Hidden-winter result (10 droughts, 26 normal winters) |
|---|---|---|
| 31 Dec | 30–38% | 1 caught, 1 false alarm → too early, don't use |
| **31 Jan** | 52–61% | **4 caught (1998/99, 2007/08, 2008/09, 2024/25), 1 false alarm (1990/91)** |
| **28 Feb** | 69% | **6 caught (+1999/00, 2020/21), 1 false alarm (1990/91)** |
| **31 Mar** | 67–81% (≤1 false alarm learned: 81–82%) | 6 caught / 1 false; or 9 caught / 2 false (2010/11, 2016/17) |

**Rule for the build (cut-offs learned on all 36 winters):** rain since 1 Oct, mean of the 16 zones, % of 1991–2020 normal → **31 Jan < 52% = early drought alarm; 28 Feb < 69% = drought alarm** (both: no false alarm in history); **31 Mar < 81% = drought alarm** (learned allowing 1 false alarm, because it lifts catches from 6 to 9). Pitch numbers, all from the hidden-winter test: Jan 4/10 caught with 1 false alarm, Feb 6/10 with 1, Mar 9/10 with 2, in 36 winters.

**Dukan / Darbandikhan June forecast + lake size at end of winter** (`backtest/dam_forecast_march_test.py`, output `dam_forecast_march_test_output.txt`; March lake = max of clear Landsat measurements 1 Feb–31 Mar, scene cloud < 20%, `backtest/dam_history_march.csv`; 28 Dukan / 20 Darbandikhan years got a clear picture):

| Lake (years) | Inputs at 31 Mar | Avg error | Low summers caught | False alarms | 2025 (real 113 km²) |
|---|---|---|---|---|---|
| Dukan (22) | now: last Oct lake + El Niño + rain | 26 km² | 6/6 | 5 | 164 |
| Dukan (22) | **March lake + rain** | **24 km²** | 5/6 | **3** | **144** |
| Darbandikhan (16) | now | 24 km² | 1/4 | 2 | – |
| Darbandikhan (16) | **March lake only** | **16 km²** | 2/4 | **0** | – |

- Correction to the earlier chat answer: the old forecast did put 2025 inside the "low summer" zone (168 ≤ 165 + 10%), but it said 168 km² when the real June area was 113. The March lake brings it to 144: closer, still 31 km² too high.
- Dukan's 2025 inputs were not extreme (catchment rain Oct–Mar 72% of normal, last Oct lake 184 km²), yet June fell to the lowest in 32 years. The rest of the drop is something our data does not see (not verified: less snow, upstream use in Iran, more release downstream).
- Small samples (22 and 16 years) → use the March lake as a live input in the product (measurable in seconds), but don't claim a big accuracy gain for Dukan. For Darbandikhan it is the best input we have.

## Investigation 8: Mediterranean and Persian Gulf sea temperature (added 2026-10-08, `backtest/sea_temp_test.py`, output `backtest/sea_temp_test_output.txt`)
NOAA ERSST v5 monthly area means (East Mediterranean 30–36E 31–36N, whole Mediterranean 0–36E 30–41N, Persian Gulf 48–56E 24–30N), 1981 → Sep 2026, via IRI data.tsv (keyless). Outcome: 36 winters, 16-zone Oct–May rain, 10 droughts (≤ 80%).
- **Nothing beats or adds to El Niño.** Best single hint: warm East Mediterranean in Jul–Sep (trend removed) → slightly drier winter, r −0.33 (not significant after correcting for 12 tests; alone in the hidden-winter test r +0.19). Whole Mediterranean r ≈ 0; Persian Gulf r −0.2 (alone −0.01).
- El Niño alone (Oct): r +0.48, 8/10 droughts caught, 6 false alarms, AUC 0.85. + East Med: 9/10, 7 false, AUC 0.84. + Gulf: 7/10, 7 false. Same picture with the Sep–Nov values (known early Dec).
- → Not used. Total pre-season signals tested now 14; only El Niño works.

## Investigation 9: a December "watch" (added 2026-10-08, `backtest/dec_alarm_test.py`, output `backtest/dec_alarm_test_output.txt`)
Inputs on ~10 Dec: rain since 1 Oct up to 30 Nov (16 zones) + the 7 world seasonal forecasts started 1 Dec for Dec–Feb (6 NMME + ECMWF SEAS5, issued ~5–10 Dec; bias and scale learned without the hidden winter) + El Niño. Outcome: same 10 droughts, 35 winters 1991/92–2025/26, each hidden in turn.

| Date | Inputs | r with season rain | Drought AUC | Caught with 0 / 1 / 2 / 4 false alarms |
|---|---|---|---|---|
| ~10 Nov | rain Oct + El Niño + forecasts | +0.47 | 0.80 | 0 / 0 / 5 / 6 |
| ~10 Dec | rain Oct–Nov only | +0.41 | 0.68 | 0 / 0 / 0 / 3 |
| ~10 Dec | world forecasts only | +0.54 | 0.80 | 1 / 2 / 3 / 4 |
| **~10 Dec** | **rain Oct–Nov + world forecasts** | **+0.66** | **0.85** | 1 / 1 / 2 / **9** |
| 31 Dec | rain Oct–Dec only (reference) | +0.76 | 0.83 | 1 / 1 / 3 / 6 |

- → December works as a **watch (yellow), not an alarm**: flagging the driest-looking winters catches **9 of 10 droughts with 4 false watches in 35 years** when the cut-off is read off all winters; with the cut-off also learned with each winter hidden (as the web app does, `web/build_data.py`): **9 of 10 droughts, 5 false watches** (1996/97, 2005/06, 2010/11, 2017/18, 2022/23). Too many false alarms to call it red.
- 2024/25 on 10 Dec: predicted 83% of normal (real 59%) → inside the watch, not red.
- Needs the Dec forecasts live (IRI keyless, ~10 Dec); for replays the archived forecasts are on disk (`backtest_data/seasonal_models/`).

## Investigation 10: each drought on 31 December (added 2026-10-08, `backtest/dec31_test.py`, output `backtest/dec31_test_output.txt`)
Signals on 31 Dec: rain since 1 Oct, the 10 Dec watch, rain + 7 world forecasts (their Jan–Feb part, Dec start), zone crop lights. Cut-offs learned with each winter hidden; 10 droughts in 35 winters.
- **Sure (red) on 31 Dec: 3 of 10** (1998/99, 2007/08, 2011/12) with 1 false alarm (1995/96), using rain + world forecasts.
- Allowing 4–5 false alarms: rain alone 7/10, rain + forecasts 6/10. The 10 Dec watch was already on for 9/10.
- **Not visible on 31 Dec: 2000/01, 2013/14, 2021/22, 2024/25** — rain was 70–101% of normal by 31 Dec; the drought came from a dry January–March.
- Zone crop lights on 31 Dec: 1999/00 16 red zones, 2007/08 7, 2008/09 6, all others 0.
