# FINAL scope and build plan — "Farm Doctor" (2026-10-08)

Replaces `Scope_Wheat_Drought_Alarm.md`. Decisions behind it: no long-range forecasts (now + 10 days only); software only; helpful for the public; everything backed by `reports/What_AI_Can_Do_For_KRI_Agriculture.md` (analysis of 96,941 papers) and `reports/Farm_Advice_Research.md`.

## One sentence
Five AIs watch Kurdistan's farmland from space, from the weather, from farmers' photos and from farmers' reports, and one AI doctor turns all of it into plain Sorani advice for farmers and a live picture for the Ministry.

## Who it helps
- **Farmers** (about 50,000 wheat farmers, about 215 extension staff): what to do this week, what a sick plant probably is, proof when a field is damaged.
- **The Ministry of Agriculture and water managers**: the state of every district now, where outbreaks start, how full the dams are.
- **The public**: dam levels, drought status, farmland lost to cities, in Sorani.

## The 5 AIs (each returns numbers and labels only)

| # | AI | Input (free) | Output to the doctor | Evidence |
|---|---|---|---|---|
| 1 | **Field eye** | Sentinel-2 / MODIS, 25-year history (built) | field greenness vs its own normal and vs neighbours; sudden-drop flag; "weak every year" flag | strong (FAO ASIS method) |
| 2 | **Season check** | rain (ERA5/CHIRPS), greenness, soil moisture vs 2001–2025 normal (built) | district label: normal / dry / drought, with "reliable from mid-season" tag | strong; our 2008 and 2016 replays |
| 3 | **10-day weather** | Open-Meteo forecast | per area: frost/heat nights, heavy rain, dry spell, rust weather (<18 °C + wet leaves), sunn-pest window (degree-days), spray window, "sow on ≥20 mm", "urea before ≥12 mm" | medium; rules from ICARDA/Hartfield/Montana trials |
| 4 | **Photo triage** | 3–6 farmer photos + short KRI disease notes | top-3 causes, confidence, photo-quality flag | medium; 65–95% on real farmer photos with notes |
| 5 | **Outbreak map** | geotagged farmer reports and questions | similar reports within 20 km in the last 14 days | medium-weak |
| + | **Dam watch** | Sentinel-2 (built) | Dukan / Darbandikhan area and % of full | strong |

## The doctor (Claude, Sorani + English)
Gets the five outputs as one JSON plus a short rulebook. Answers: most likely cause, how sure, **what to do now** (timing, non-chemical steps, "call the extension officer / vet"), what it cannot tell, and **which input drove each conclusion**. Hard rules: no pesticide or fertilizer doses; says "unsure, see an officer" when inputs conflict or confidence is low; answers only from the rulebook and checked notes; logs every case.

## Screens
1. **Telegram bot (farmers):** send a location pin, a voice note or text in Sorani, or photos → the doctor's answer plus the field's satellite picture. Weekly "this week on your farm" message per village.
2. **Web map (Ministry and public):** Kurdistan → 5 areas → 16 zones: season label, field condition, 10-day alerts, outbreak pins, dams, farmland-loss history. Built on the existing app (hex map, time machine, Sorani).
3. **Time machine (proof):** replay 2008 (drought) and 2016 (wet) as they were measured; "From space" layer. The forecast game is removed.

## Demo (3 minutes)
1. A judge drops a pin on any field → its satellite picture and condition appear with a Sorani explanation. (20 s)
2. A farmer voice note in Sorani plus 3 photos of a sick leaf → the doctor answers: likely rust, 70% sure, cool wet nights this week, 2 similar reports 8 km away, "check the flag leaf, call the plant-protection office, no spraying before confirmation". (40 s)
3. **The disagreement case:** photos say pest, weather says no, no reports nearby → the doctor says "unsure, see an officer". (20 s)
4. Ministry map: Raparin lights up with farmer reports next to the satellite browning. (20 s)
5. Time machine 2008: the region turns brown from space; the Ministry's weekly brief writes itself. (30 s)
6. Honesty slide: lab 99% vs field 72%; only 0.07% of 97k papers test real impact; our validation plan. (20 s)

## OUT
Long-range forecasts (season, El Niño, climate), the Predict/Reveal game, drought alarms as predictions, Dukan June forecast, yield in tonnes, pesticide/fertilizer doses, food-safety photo checks, price forecasts, credit/insurance, soil maps, fraud flags, Flutter app, any hardware.

## Build plan (Oct 8–9, team of 5)

| When | Backend | Frontend | Data/AI | Testing | Presentation |
|---|---|---|---|---|---|
| **Day 1 morning** | FastAPI skeleton; tool endpoints: field_eye, season_check, weather10, dam_watch (wrap existing scripts) | strip forecast game from web app; new "Now" home: season label + 10-day alerts | rulebook v1 (weather rules with numbers); KRI disease notes (rust, sunn pest, septoria, aphids, weeds) | test set: 20 field pins with known answers (2008 brown, 2016 green, 2025 Dukan) | storyline + honesty slide |
| **Day 1 afternoon** | Telegram bot: pin → field_eye + doctor; photos → photo_triage | outbreak pins layer; Ministry area panel | doctor prompt: JSON in, Sorani out, provenance, refusal rules | 10 Sorani questions to the doctor, scored by a native speaker | record the 2008 time-machine clip |
| **Day 1 night** | report logging; voice note → Sorani speech-to-text (Google Chirp) | polish animations; phone layout | photo test on Halabja dataset + 20 web photos | the disagreement case works end to end | rehearse demo 1 |
| **Day 2 morning** | weekly village message job; dam_watch live refresh | "From space" layer in the Now view | calibrate confidence wording; weekly brief generator | full demo run ×3, fix breaks | slides final, 3-min timing |
| **Day 2 afternoon** | freeze; backups; offline fallback JSON | freeze | validation numbers on the honesty slide | final checklist | rehearse ×3 with the judges' questions |

Owner of each script today: `web/build_data.py`, `web/build_greenness.py`, `evidence/past_seasons/backtest/dam_water_test.py` (lake area), `fetch_openmeteo.py` (weather).

## Open items (need you)
1. Claude API key.
2. Telegram bot token (BotFather, 2 minutes).
3. Google Cloud key for Sorani speech-to-text (or type-only for the demo).
4. A native Sorani speaker to score the doctor's answers.
5. Ask the organizers whether pre-built work is allowed.

## Validation we can honestly claim
- Field eye: 1089/1089 pixels match NASA's own service; 88% agreement with zone outcomes on 26 seasons.
- Season check: 2008 and 2016 replays, Dukan 2025 (−56%, matches AFP).
- Photo triage and doctor: none yet in KRI. Plan: 100 local photos scored with the plant-protection office after the hackathon.
