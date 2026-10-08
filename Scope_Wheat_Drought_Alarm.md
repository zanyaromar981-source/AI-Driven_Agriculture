> **SUPERSEDED (2026-10-08 11:10): replaced by `Scope_and_Build_Plan_FINAL.md` (Farm Doctor, no long-range forecasts).**

# Scope: Kurdistan wheat drought alarm (FINAL, 2026-10-08)

Replaces `Build_Plan_Satellite_Monitor.md` (Oct 5). Every number below comes from a hidden-year test in `evidence/past_seasons/BACKTEST_RESULTS.md`.

## In one sentence
From October to March, our system tells the Ministry of Agriculture & Water Resources each month how this year's **wheat** harvest is heading, for all of Kurdistan, each governorate and each zone, **months before the harvest**.

## Who and why
- **User:** government planners (Ministry of Agriculture & Water Resources). Not farmers: wheat is planted Oct–Dec, before any honest alarm can fire.
- **Crop:** rain-fed wheat, which lives on the winter that comes *after* planting.
- **Problem:** a drought hits about 1 winter in 3 (7 of the last 20). In drought years wheat per donum fell 23–54% in 4 of 6 cases (KRSO); 2021 had 1 million tonnes less wheat than 2020.
- **What planners do with it:** start the drought plan early: wheat stocks/imports, farmer support, budget.

## The map: 3 levels (zoom in)
| Level | Areas | Light comes from |
|---|---|---|
| Kurdistan Region | 1 | the rain alarm (below) |
| Main areas | Duhok, Erbil, Slemani, Halabja, Garmiyan (KRSO publishes wheat for all 5) | the light most of its zones show |
| Zones | 16 tested zones: Duhok, Zakho, Akre · Erbil, Makhmour, Soran, Koya · Slemani, Chamchamal, Bazian, Ranya, Penjwen, Sharazur, Qaradagh · Halabja · Garmiyan | crop risk model per zone |

The official 20-district map is NOT used: no weather data per district, and Baghdad's map leaves Akre, Shekhan and Kifri outside the KRI governorates.

## The calendar: what the planner sees, and how good it is
| When | Signal | Light | Proof (hidden-year tests) |
|---|---|---|---|
| Early Oct | El Niño / La Niña → winter forecast | Wet / Normal / Dry badge | Wet called 7 times, right 5; Dry 7, right 4; Normal 22, right 7 (36 winters). Wet and dry calls never hit the opposite extreme |
| ~10 Dec | rain since 1 Oct + 7 world seasonal forecasts | 🟡 **watch** | 9 of 10 droughts flagged, 5 false watches in 35 winters (cut-off learned with each winter hidden) |
| 31 Jan | rain since 1 Oct < **52%** of normal | 🔴 **early alarm** | 4 of 10 droughts, 1 false alarm (incl. 2024/25) |
| 28 Feb | rain since 1 Oct < **69%** of normal | 🔴 alarm | 6 of 10, 1 false alarm |
| 31 Mar | rain since 1 Oct < **81%** of normal | 🔴 alarm | 9 of 10, 2 false alarms |
| Dec–Mar monthly | zone crop risk (Team model: weather + satellite greenness + El Niño) | 🟢/🟡/🔴 per zone | AUC Dec 0.76 → Mar 0.85; March: 66% of bad zones caught at 18% false alarms |

Last 20 winters: **6 of 7 droughts caught, 3 by 31 January**, 2 false alarms (both in March).

**What 🔴 means for wheat:** "expect wheat per donum about 25–55% below normal" (4 of 6 past droughts; 2 were milder).

## The AI
1. **Prediction (we trained it):** "Team" model = logistic regression + random forest, trained on 26 seasons × 16 zones. Re-run in seconds with fresh data. No deep learning (too few years; tested, no gain).
2. **Explanation (no training):** Claude (Opus 5.5 for the demo, about $0.06 per question) with tool calls into our data: rain so far, zone risk, look-alike past years. Answers in Sorani and English.

## Screens (with the animations)
1. **Map (home):** Kurdistan → main area → zone, flying in and out; lights fade in; numbers count up; date slider Oct → Mar; El Niño banner.
2. **Time machine:** pick a past winter (2007/08 drought, 2015/16 wet, 2020/21, 2024/25) → **Play**: days run Oct → Mar, rain bar fills, map colours shift, the watch/alarm pops on its real date. The real harvest stays hidden → **Reveal**: cards flip (✓/✗), official wheat tonnes count down (2007/08: 654,601 → 216,072 t).
3. **Ask Claude:** click any zone or point → Claude explains the light in Sorani and names the past winters that looked like this one.
4. **This winter, live (2026/27):** sealed October call: strong El Niño (+2.16) → "drought unlikely". It updates monthly from December.

Greenness layer: the 250 m map of the whole region (592 dates, 2000–Jul 2026) animates under screens 1–2. It shows measured data, not a prediction.

## OUT (not built)
Tonnes forecast · farmer app / Flutter · Telegram · summer dam water forecast · wells and springs · fires · farmland loss · wheat-quota count · barley · district-level predictions · zone calls before the season · fertilizer/chemical advice.

## Pitch
- **Line:** "We can't see a drought before the rain season starts. Nobody can. We catch it in mid-winter, months before the harvest fails: 6 of the last 7 droughts, half of them by January."
- **Named action rule, "the 31 January rule":** if rain since October is below about half of normal on 31 January, the Ministry starts its drought plan that week.
- **Demo order:** live 2026/27 call → time machine 2007/08 with Reveal → 2015/16 (stays green: no false alarm) → Ask Claude in Sorani.

## Web app (built 2026-10-08)
`web/index.html` + `web/data.js` (built by `web/build_data.py`), preview with `web/preview.sh`. Published privately: https://claude.ai/artifact/RgVJfQhF8Es9MM9Cqg4kvT

## Data status
- Ready: weather 16 zones 1990–2026, greenness 16 zones and the whole-region grid, El Niño, 7 seasonal-forecast archives, KRSO wheat 1970–2023, boundaries.
- Refresh during the event: NOAA El Niño update (Thu Oct 8), latest rain on demo morning.

## Open items
1. Claude API key (you create it).
2. A native Sorani speaker checks Claude's text before the pitch.
3. Sharazur: the official district map puts our Sharazur zone in Halabja district, so the web app shows it under Halabja.
4. Optional, for the pitch: one real voice from the Ministry or the KRSO.
