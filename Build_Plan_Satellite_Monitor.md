> **OUTDATED (2026-10-08): replaced by `Scope_Wheat_Drought_Alarm.md`.** Kept for history only.

# SmartSuli AI Challenge: satellite farm & water monitor (build plan)

> **UPDATE 2026-10-05: scope is the whole KURDISTAN REGION, not only Slemani.** 16 zones incl. Erbil/Duhok. See STATUS.md + evidence/past_seasons/FINDINGS.md.

## Context
Hackathon: SmartSuli AI Challenge, build Oct 8–9 2026 at Culture Factory Slemani, pitch Oct 10. The team picked "AI-driven agriculture using satellites." Scope we agreed on in chat: **an AI satellite monitor for Slemani's farmland and water**. It shows how much water the dams hold, which farm zones are drying out, and which crops fit each zone, and it explains everything in Sorani for water/agriculture administrators.
Team: 5 people, all building by prompting Claude. Roles: backend, frontend, testing, presentation, and 1 undecided.
Project folder: `/Users/ranjhamakarim/Documents/SmartSuli_AI_Challenge/` (STATUS.md is already updated with Parts 1–3).

## What the research changed (read first)
1. **2026 was a WET year.** Darbandikhan overflowed in March 2026, Dukan came within ~1 m of full, and Kurdistan held 9+ bcm by May. 2025 was the worst drought in about 20 years: Dukan was at ~24% (1.6 of 7 bcm) in June 2025 and ~2M dunams of crops failed. → Pitch it as an **early warning for the next drought**, and replay 2025 in the demo as the proof case. Don't claim "Slemani is drying right now"; the judges will know the dams are full.
2. **October = bare fields.** Winter wheat/barley is sown Oct–Dec and harvested May–Jun. Live greenness (NDVI/NDMI) in October shows bare soil, not crop stress. → For "now" use soil moisture + rain + ET0. Use greenness for the **last growing season (Mar–May)**.
3. **Dukan doesn't clearly irrigate Slemani farms** (couldn't confirm). It mainly supplies drinking water to Sulaymaniyah, Ranya and Kirkuk. **Darbandikhan → Garmiyan** is confirmed: ~95% of Garmiyan farms are irrigated from the Sirwan. → Use Garmiyan as the irrigation story, and keep Dukan for reservoir watch.
4. **The SoilGrids API is paused.** → Use HWSD v2 (FAO soil map, in Google Earth Engine) or download a Slemani tile before Oct 8.
5. **Rules are unknown** (team size, judging, whether we can prepare data/code before the event). The "previous edition was buses" claim is unconfirmed; the only SmartSuli hackathon found was about healthcare. → Ask the organizers today.
6. **Sharazur may now be in Halabja governorate** (not Sulaymaniyah). Check the zone list against the OCHA boundaries.

## System parts (final)
| # | Part | Data | How |
|---|---|---|---|
| 1 | Reservoir watch | Sentinel-2 MNDWI (+ Sentinel-1 radar on cloudy months) | Monthly water area inside Dukan and Darbandikhan polygons → volume estimate from anchors: Dukan 7.0 bcm @ 270 km², Darbandikhan 3.0 bcm @ 113 km² (labelled "estimate") |
| 2 | Dryness ranking | Sentinel-2 NDVI/NDMI, Landsat 8/9 surface temperature, CHIRPS rain, Open-Meteo soil moisture + ET0 | Per zone: z-score vs the same period in past years (2017→), average the z-scores into a dryness score, rank zones. Also VHI (<40 = drought). Mask to cropland (ESA WorldCover class 40) |
| 3 | Honest numbers | from Part 2 | "Drier than N of the last M years", "X% of cropland stressed", "soil moisture Y% below normal" |
| 4 | Crop fit | Zone rain/water history + HWSD v2 soil + FAO-56 crop water table | Rules: the zone's usual water vs. each crop's seasonal mm → fits / risky / doesn't fit |
| 5 | Sorani explanations | Parts 1–4 numbers | Claude API turns the numbers into 2–3 Sorani sentences with fixed farming terms. A native speaker checks them |
| 6 | Fire alerts (maybe) | NASA FIRMS (free key, ~3 h delay) + Sentinel-2 dNBR | Last only if time allows |

Zones (draft): Garmiyan/Kalar, Chamchamal, Bazian, Ranya/Bitwen, Penjwen, Qaradagh, Sharazur (check governorate), Dukan area.

## Tech stack (simplest that works)
- **Data:** Google Earth Engine Python API (one platform for everything; free Community tier). Copernicus Sentinel Hub is the backup.
- **Precompute:** a Python script exports per-zone time series to JSON/CSV, so the demo doesn't depend on live Earth Engine calls.
- **Backend:** Python FastAPI. It serves the JSON, computes rankings and crop fit, and calls the Claude API for Sorani text (cached).
- **Frontend:** React + Vite + Leaflet map. Zones are coloured by dryness, with a reservoir chart, a zone detail panel, and RTL Sorani text.
- **Live extra:** Open-Meteo (no key) for current soil moisture/rain on the zone panel.

## Team roles
| Person | Job |
|---|---|
| Undecided → **Data person** | Earth Engine scripts, exports per-zone data, checks the numbers make sense |
| Backend | FastAPI, ranking + crop fit + Claude/Sorani endpoint |
| Frontend | Map, charts, Sorani RTL UI |
| Testing | Checks numbers against known facts (Dukan ~24% in June 2025, full in spring 2026), tests every demo click path, keeps the offline fallback ready |
| Presentation | Pitch story (2025 drought → early warning), slides, demo script, organizer Q&A |

## Timeline
**Before Oct 8 (only if the rules allow preparing; ask first)**
1. Today: DM the organizers: can we prepare data/accounts before? Judging criteria? Team size?
2. Today: everyone creates a Google Earth Engine noncommercial project and tests a login (approval can take time).
3. Oct 6–7: download boundaries (OCHA Iraq admin3), reservoir polygons, HWSD tile; get a NASA FIRMS key; get a Claude API key (cost confirmed with the user first).
4. Oct 6–7: call 1 farmer or someone at the agriculture/water directorate. Ask how Garmiyan water is split and what 2025 looked like (gives the pitch real evidence).

**Build (48 h)**
- 0–6 h: Earth Engine exports for Parts 1 + 2 (2017–2026). Skeleton backend + map.
- 6–18 h: ranking + honest numbers; reservoir chart; zones coloured on the map.
- 18–30 h: crop fit; Sorani explanations; zone detail panel.
- 30–40 h: testing against known facts, offline fallback, fire alerts only if ahead.
- 40–48 h: demo script rehearsal, slides, freeze code.

## Verification
- Reservoir: the Dukan curve must show the 2025 low (~20–25%) and near-full in spring 2026. If not, the method is wrong.
- Dryness: spring 2025 must rank much drier than spring 2026 in every zone.
- Crop fit: rice/high-water crops should show "doesn't fit" in dry zones; wheat/barley should fit rainfed zones in normal years.
- Sorani: a native speaker reads every generated sentence before the pitch.
- Demo: runs from precomputed JSON with Wi-Fi off.

## Open questions (for organizers / team)
- Can data and code be prepared before Oct 8?
- Judging criteria, and do they want a live AI model (not only rules)?
- Who in Slemani decides Garmiyan water allocation? (Is there a real user to quote?)

## After approval
Save this research + plan to `/Users/ranjhamakarim/Documents/SmartSuli_AI_Challenge/Build_Plan_Satellite_Monitor.md`, update STATUS.md, and update memory.
