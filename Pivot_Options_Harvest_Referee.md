# Making the project unbeatable: pivot options (2026-10-07)

## The problem we found (news research)
Every spring Baghdad caps how much Kurdish wheat it buys. Kurdish farmers' income depends on it.
| year | Baghdad's quota for Kurdistan | notes |
|---|---|---|
| 2024 | 700,000 t | |
| 2025 | 400,000 t (raised from 292,000 after pressure) | drought: ~2 million dunams of rainfed land failed; Koya alone lost 36,000 dunams / ~$12M; 850,000 IQD/t → ~$229M paid in installments; split Sulaimani+Halabja 158k, Erbil 122k, Duhok 118k |
| 2026 | 290,000 t | KRG Ministry of Agriculture: "projected yield of more than 1.5 million tons", calls the cap unacceptable; estimates in the news come from "farmer assessment" ("15–20 times the seed") and ministry observation; no survey or satellite number cited |
Sources: Rudaw 2025-04-04, 2025-06-09, 2025-10-14, 2026-04-25; Channel8; PUKmedia (links in chat answer).
Existing tools: FAO ASIS (global drought stress map, 1 km, no harvest forecast for Kurdistan, not validated locally, not in Kurdish); iMMAP Iraq agromet bulletins (2017). Nothing local, validated, or in Sorani found.

## What our data can already prove for this
- Satellite spring greenness vs **official KRSO wheat yield**: r = +0.78 (Kurdistan, 24 harvests 2000–2023); Erbil +0.84.
- **Spring 2026 was the greenest spring in 27 years of satellite records** for 7 of 16 zones (avg 131% of normal; Duhok, Zakho, Akre, Koya, Makhmour, Garmiyan = #1). Greener than 2019 (128%) and 2020 (127%), when official production was 2.4 M t and 2.9 M t. → strong independent evidence against a 290,000 t cap.
- Tonnes estimate per region (`backtest/harvest_tonnes_test2.py`): median error 20% at mid-April vs 32% for "same as last 3 years". Rough, because we do not measure planted area yet.
- Spring 2025 replay (season hidden): end of March risk Koya 63%, Duhok 63%, Zakho 67% (all really bad); Chamchamal missed (21%); false alarms Qaradagh 75%, Bazian 73%.
- Earlier results still hold: monthly warning 0.77 (Dec) → 0.89 (mid-Apr); La Niña → 5 of 7 bad wheat harvests; 2026/27 strong El Niño + 6 of 7 forecast models → wet winter.

## Options
| | idea | strength | weakness |
|---|---|---|---|
| A | **Harvest referee**: independent satellite evidence of each governorate's wheat harvest for the quota + payment debate | concrete money problem (hundreds of millions $), government user, 2026 headline number ready, validated vs official stats | tonnes only ±20% until we map planted area (Sentinel-2 / Earth Engine in the build); politically sensitive → frame as neutral evidence |
| B | **Drought early warning** (current plan, sharpened): monthly risk map Dec–Apr + La Niña alarm | strongest proof (hidden-season tests), farmer + ministry value | sounds like FAO ASIS; less "wow" |
| C | **Both as one season story** ("Xerman" / خەرمان = harvest, name to check with native speakers): Sept outlook → Dec–Apr warnings → Apr–May harvest estimate for the quota | covers the whole season, every part backed by a number, live 2026/27 prediction | more to build in 48 h → harvest page must reuse the same pipeline |
Recommendation: **C, opened with A's story** ("Baghdad agreed to buy 290,000 t. Erbil says 1.5 million. Nobody measured. We did, from space.").

## What changes in the build (if chosen)
1. Add a "Harvest" page: per governorate, this spring's greenness vs 27 years, rank, closest past years with official tonnes, estimated range.
2. Optional, biggest new job: planted-area map from Sentinel-2 in Earth Engine → tonnes error should drop (not tested).
3. Keep the monthly warning + El Niño/La Niña outlook as the "AI predicts" part.

## Round 2: "find better stuff" (2026-10-07, measuring instead of predicting)
Researched problems where the satellite MEASURES the present (certain) instead of forecasting:
1. **Wheat verification (harvest referee, upgraded).** Parliament report (Apr 2020, Al Jazeera): imported/smuggled wheat sold to state silos at the local subsidized price; merchants sell under farmers' names; silo staff forged transport papers; 752 t "eaten by birds" in Najaf. Baghdad caps Kurdish purchases (700k → 400k → 290k t) while Erbil claims 1.5 M t. → Satellite shows which land really grew wheat and how good the season was = evidence both Erbil and Baghdad can trust. Region/governorate level ready (r = 0.78 with official yields, 2026 = greenest spring in 27 years); field level needs Sentinel-2 (Earth Engine), untested.
2. **Water from space (Dukan/Darbandikhan).** Dukan 24% full in June 2025 (20-year low), hydro power halted, Slemani drinking water at risk; news already used satellite images (−56% lake area 2019→2025). **Our MODIS reservoir download does NOT measure the lake correctly** (tested: 2025 shows more water than full 2019; values 10–90 km² vs ~270 km² real) → would need Landsat/Sentinel-2 water index or JRC surface water in Earth Engine.
3. **Farmland eaten by cities.** Decades of farmland expropriation around Erbil/Slemani (TU Dortmund study; ~1,000 dunams seized near Erbil airport in 2021). Very visual time-lapse; politically sensitive; weak AI part; needs Earth Engine.
Dropped: harvest fires (2,513 dunams burned in Erbil 2024; Duhok $100k+) → fire season is June–July, nothing live to show in October.
Recommendation: 1.

## Round 3: functionality that shocks AND is useful (2026-10-07, proposal, waiting for go)
Checked today: Sentinel-2 10 m scenes are free without any account (AWS Earth Search STAC; Koya has a 3%-cloud scene from 2026-10-04). Open-Meteo 16-day forecast shows the first autumn rain Oct 12–17 (Chamchamal 24 mm, Zakho 38, Soran 40, Penjwen 53; Erbil 7, Makhmour 8) = live during hackathon week.
1. **"Send your location → your land from space"** (Telegram bot + web, Kurdish): 10 m photo of the farmer's land this week, its spring greenness every year 2000–2026 (best/worst years), 16-day rain outlook, season outlook (El Niño), and from December the monthly risk. Example Koya: best springs 2026 160%, 2020 150%, 2023 148%; worst 2008 54%, 2009 57%, 2025 61%.
2. **Alerts**: "first soaking rain is coming / has fallen in your area" (now), monthly risk alerts (Dec–Apr), mid-April harvest-or-graze check.
3. **Time-machine map** (web): every spring 2000–2026, droughts turn red; replay of 2007/08 (Dec 31 2007 all zones red → harvest −67%).
Dependencies: Telegram bot token (user creates via BotFather), Claude API key (cost to confirm), fast per-point 26-year history needs Earth Engine (fallback: pre-compute demo villages via NASA ORNL). Unknown: which messenger farmers use most.
