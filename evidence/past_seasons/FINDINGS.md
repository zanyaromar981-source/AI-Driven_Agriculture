# Past seasons investigation (2026-10-05)

Data: Open-Meteo historical (ERA5), daily rain 1990–2026, 8 points: Slemani city, Garmiyan/Kalar, Chamchamal, Bazian, Ranya, Penjwen, Sharazur, Qaradagh. Season = Oct–May. Scripts: `season_rain.py`, `monthly_warning.py`. Table: `season_rain_summary.json`.

## The data matches known history
- Bad seasons (bottom 25%): 1998/99, 1999/00, 2000/01, 2007/08, 2008/09, 2016/17, 2020/21, 2021/22, 2024/25 → matches the known droughts
- Wettest: 2018/19 (150%), 2025/26 (155%) → matches the known wet years
- Normal season (1991–2020): ~674 mm Oct–May, ~217 mm Oct–Dec (8-zone average)
- 2024/25: 381 mm = 57% of normal

## How early would a RAIN-ONLY warning have caught bad seasons?
| Warning made at | Bad seasons caught | False alarms | 2024/25 |
|---|---|---|---|
| End of December | 4 of 9 | 5 | 72% of normal → MISSED |
| End of January | 5 of 9 | 4 | 50% → WARNED |
| End of February | 7 of 9 | 2 | 50% → WARNED |
| End of March | 8 of 9 | 1 | 49% → WARNED |

## Meaning
- A December forecast from rain alone is weak (coin-flip level). The 2025 drought did NOT show yet in Oct–Dec 2024 (72%; similar starts in 2017/18, 2019/20 and 2022/23 ended normal).
- By the end of January 2025 the warning is clear, and by the end of February it's reliable (7/9). That's 3–4 months before the harvest.
- Pitch: "a warning that gets sharper every month," not a "December crystal ball."
- Still to test: whether adding soil moisture + satellite greenness makes the December warning better (the real backtest, Test 1).

## News + station check (web research, 2026-10-05)
- **Best ground-truth source:** KRSO monthly rainfall at Sulaimani station 2012–2024: https://krso.gov.krd/content/upload/1/root/داتاى-كه‌ش-و-هه‌وا-20-2-2020-ئينگليزى2.pdf (normal ≈ 717 mm Oct–May, ≈ 255 mm Oct–Dec)
- **Bad years confirmed:** 1999–2001, 2007–09, 2011/12 (planting failed, replanted late Jan), 2020/21, 2021/22, 2024/25. **Good years:** 2018/19, 2019/20, 2025/26.
- **Damage size:** in bad years KRI wheat fell 45–50% (1.4 → 0.75 → 0.4 Mt for 2020–2022). In 2025, ~2M dunams of rainfed land failed and ~0.8 Mt was lost (Rudaw 090620251, 040420251; Peregraf 9102).
- **2024/25 start:** Sulaimani Oct–Dec 2024 = 180 mm (vs ~255 normal): October 0.8, November 135, December 43.5. Erbil had only 45.8 mm and Duhok 34 mm by Jan 11 → **the 2025 drought hit Erbil and Duhok much harder than Slemani**.
- **Spring decides:** 2017/18 and 2022/23 started dry and ended normal. 2020/21 started normal and failed because March–April were dry. This matches our rain test: the warning must update monthly through April.
- **Data gap to note:** ERA5 (our grid data) and the station sometimes disagree (2017/18: ERA5 90% of normal vs. station 837 mm ≈ 117%). Use the KRSO station data to check the model.
- **Sources:** FAO GIEWS Iraq archive https://cb.apps.fao.org/country/IRQ/pdf_archive/IRQ_Archive.pdf; Rudaw 110120261; Peregraf 10416.

## Whole Kurdistan Region (16 zones, added 2026-10-05)
Added: Erbil, Duhok, Zakho, Soran, Koya, Halabja, Akre, Makhmour (file `rain_daily_erbil_duhok.json`).
- 2024/25 by end of DECEMBER: Erbil 40%, Chamchamal 47%, Makhmour 53%, Soran 54% of normal → **these zones already showed the drought in December**, while Penjwen (97%) and Halabja (110%) looked fine.
- Region-wide warning: Dec 5/9 caught (2025 missed at 69%); Jan 6/9 (2025 warned); Mar 8/9 with 1 false alarm.
- Meaning: warn **per zone**, not region-wide. Erbil and the dry plains get the warning 1 month earlier.
