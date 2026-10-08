# Farm Doctor — SmartSuli AI Challenge 2026

AI for Kurdistan Region farming, built for the **SmartSuli AI Challenge** (The Foundation, Slemani).
Build days: 8–9 October 2026 at the Foundation Hub, Culture Factory. Demo and judging: 10 October 2026.
Judging rule from the organizers: "what works, not presentation polish".

## One sentence

Five AIs watch Kurdistan's farmland from space, from the weather, from farmers' photos and from farmers' reports, and one AI doctor (Claude) turns all of it into plain Sorani advice for farmers and a live picture for the Ministry of Agriculture and Water Resources.

## What it is

| Part | For whom | What it does |
|---|---|---|
| **Ministry dashboard** (web, no login) | Ministry planners, extension officers, public | Kurdistan → 5 areas → 16 zones. Per zone: season label (normal / dry / drought), field condition from space, 10-day weather alerts, outbreak pins, dam levels, river flow, dust. The Doctor writes the weekly Sorani brief. |
| **Farmer app** ("Jutyar", working name) | Farmers and herders | Phone-number sign-in, pick your farm (tap once, SAM draws the boundary), see it from space, this week's plan, ask the Doctor with voice, text or photos in Sorani, report a problem on the map. Designed on the web first, Flutter later. |
| **Time machine** (proof) | Judges, Ministry | Replays the 2007/08 drought and the 2015/16 wet year as they were measured from space and from rain, month by month. |

### The 5 AIs (each returns numbers and labels only)

| # | AI | Input (all free, keyless) | Output |
|---|---|---|---|
| 1 | **Field Eye** | Sentinel-2 (10 m) now, MODIS 250 m as 25-year history | field greenness vs its own normal and vs 5 km neighbours; sudden-drop flag; weak-patch share |
| 2 | **Season Check** | ERA5 / CHIRPS rain, greenness, soil moisture vs 2001–2025 normal | zone label: normal / dry / drought, with an honesty tag |
| 3 | **Weather Planner** | Open-Meteo 10-day forecast, air quality, GloFAS river flow | frost / heat nights, sowing rain (≥20 mm), urea rain (≥12 mm), spray windows, rust hours (<18 °C + wet leaves), sunn-pest degree-days, dust |
| 4 | **Plant Doctor / photo triage** | 3–6 farmer photos + short KRI disease notes | top-3 causes, confidence, photo-quality flag |
| 5 | **Neighbour Watch** | geotagged farmer reports | similar reports within 20 km in the last 14 days (test data until the app collects real ones) |
| + | **Dam Watch** | Sentinel-2 / Landsat lake area | Dukan and Darbandikhan area and % of full |

### The Doctor (Claude)

Gets the five outputs as one JSON plus a short rulebook. Answers in Sorani and English: most likely cause, how sure, what to do now, what it cannot tell, and which input drove each conclusion. Hard rules: **no pesticide or fertilizer doses**, says "unsure, see an officer" when inputs conflict, answers only from the rulebook and checked notes, logs every case.

## Current state (8 October 2026, 16:00)

- Scope frozen in `Scope_and_Build_Plan_FINAL.md`. Agile build started: **Sprint 1 = Ministry dashboard "the region now"**, Sprint 2 = farmer view.
- `farm_doctor/` test harness works end to end for one field (Field Eye live on Sentinel-2, Weather Planner, Season Check, Neighbour Watch, Dam Watch, Doctor via the Claude API). `build_now.py` runs the 5 AIs for all 16 zones and writes `web/now.json`.
- `web/index.html` = first web app (time machine, hex map, "From space" layer, Ask Claude drawer). `web/dashboard.html` = new Ministry map-first template (data wiring paused). `web/app_signin.html` = farmer app sign-in and "My farms" window.
- Field boundaries from space tested: SAM draws fields well on the plains, misses about half on hills (`farm_doctor/field_boundaries/`).
- Running log of every decision: **`STATUS.md`** (read the end first).

### Open items

1. Claude API key for the Doctor (`farm_doctor/.env`, never committed).
2. Sorani speech-to-text key (Google Chirp 2, `ckb-IQ`) or type-only for the demo.
3. A native Sorani speaker to score the Doctor's answers.
4. Pixxel (hyperspectral 5 m) and Hydrosat (thermal 70 m) prices or trial access for "which part of the farm is sick". Neither contacted yet.
5. Confirm with the organizers that pre-built work is allowed (user says yes, preparing in advance is allowed; judges want live AI).

## How to run

Everything is Python 3 standard library plus `urllib`, unless noted. Data feeds are free and keyless.

**Ministry web app (local):**
```bash
cd web
# index.html is a page fragment (it is published as a claude.ai artifact); wrap it once for local viewing:
{ printf '<!doctype html><html><head><meta charset=utf8><meta name=viewport content="width=device-width,initial-scale=1"></head><body>'; cat index.html; printf '</body></html>'; } > _preview.html
python3 -m http.server 8765
# open http://127.0.0.1:8765/_preview.html  (also /dashboard.html, /app_signin.html, /usecase_diagram.html)
```

**Farm Doctor for one field:**
```bash
cd farm_doctor
echo 'ANTHROPIC_API_KEY=sk-ant-...' > .env        # git-ignored
python3 -I run_case.py 44.60 36.03 2025-04-01 "why is my wheat yellow?" photo.jpg
python3 -I build_now.py                          # all 16 zones → ../web/now.json
```
Replay dates use the ERA5 archive as a stand-in for the forecast. Model: `FARM_DOCTOR_MODEL` env, default `claude-sonnet-5-5` (about $0.03 per analysis).

**Rebuild the web data:**
```bash
python3 -I web/build_data.py                     # zone numbers from the hidden-winter tests → web/data.js
python3 -I web/build_data.py --refresh-forecast  # 16-day rain forecast per zone (run on demo morning)
python3 -I web/build_data.py --refresh-oni       # latest NOAA El Niño value
python3 -I web/build_greenness.py                # MODIS greenness grid → web/green.js (needs backtest_data/modis_grid, see below)
```

**Field boundary test** (`farm_doctor/field_boundaries/`): needs a venv with numpy, scipy, scikit-image, pillow, torch, segment-anything, opencv-python-headless, plus Meta's SAM ViT-B checkpoint (375 MB, not in git). 2–3 minutes per 3 km window on a laptop CPU.

## What is NOT in git (and how to get it back)

| Excluded | Size | Rebuild |
|---|---|---|
| `evidence/past_seasons/backtest_data/` (MODIS, CHIRPS, Open-Meteo, reservoirs, fires, planted area, 592-date greenness grid) | 5.3 GB | `evidence/past_seasons/backtest/fetch_all.py` (1,609 jobs) and `fetch_greenness_grid.py` (4.7 GB); small inputs such as ENSO, boundaries, climate indices and the KRSO harvest CSV **are** kept |
| `research_library/pdfs/`, `metadata/`, `library.sqlite` | 25 GB | `research_library/harvest_papers.py` → `build_library.py` → `download_pdfs.py` (see its README) |
| `farm_doctor/.env`, `farm_doctor/cases/` | – | API key and per-field case logs stay local |
| `web/_preview.html`, `*.log`, `__pycache__/` | – | generated |

## Folder map

| Path | What |
|---|---|
| `STATUS.md` | running log of decisions, tests and results, newest at the end |
| `Scope_and_Build_Plan_FINAL.md` | the frozen scope, hour-by-hour plan, demo script, OUT list |
| `Use_Case_Diagram.md`, `web/usecase_diagram.html` | actors and use cases (Mermaid text and SVG) |
| `Agriculture_Note_Learning.md` | the team's farming notes (sowing, urea, rust, sunn pest) |
| `farm_doctor/` | the 5 AIs + Doctor + runner (see its README) |
| `web/` | Ministry app, dashboard template, farmer sign-in, data builders |
| `reports/` | research write-ups: what AI can do for KRI agriculture (96,941 papers), farm-advice rules with numbers, what KRI farmers asked for 2023–26, 17 data feeds tested, 13 farm-AI ideas checked, AI-in-agriculture investigation |
| `evidence/past_seasons/` | 26-season backtest: `BACKTEST_RESULTS.md` (all investigations), `DATA_INVENTORY.md`, `backtest/` scripts and small outputs |
| `research_library/` | scripts and README for the paper library |
| `research_notes/` | raw notes from the research agents |
| `evidence/*.md`, `Kurdistan_Problems_List.md`, `SmartSuli_10_Ideas_Report.md`, `Idea_Rega_Business_Journey.md` | idea-selection phase (September 2026), kept for the record |
| `Scope_Wheat_Drought_Alarm.md`, `Build_Plan_Satellite_Monitor.md`, `Pivot_Options_Harvest_Referee.md`, `Test_Plan_Forecast_and_Farmers.md` | **outdated** earlier scopes, superseded by the FINAL plan |

## What we can honestly claim (and what we cannot)

- Field Eye: 1,089 of 1,089 pixels match NASA's own MODIS service; 88% agreement with zone outcomes over 26 seasons.
- Season Check: 2007/08 drought and 2015/16 wet year replay correctly; Dukan 2025 measured at −56%, matching the AFP report.
- Weather Planner rules come from ICARDA / Montana / Hartfield trials with numbers (`reports/Farm_Advice_Research.md`).
- Photo triage and the Doctor: **no KRI validation yet**. Plan: 100 local photos scored with the plant-protection office after the hackathon.
- Long-range drought prediction was researched for three days and then **taken out of the product** (keep it to now + 10 days). The honest numbers are in `evidence/past_seasons/BACKTEST_RESULTS.md`: by 31 March a rain-based alarm catches 6 of the last 7 droughts; pre-season prediction has no skill beyond the El Niño sign.
- Only 0.07% of the 96,941 papers in the library test real-world impact; lab accuracy of 99% becomes about 72% in the field. The pitch says so.

## How this idea was chosen

September: 10 ideas for Kurdistan society problems, then 130 problems listed, then Rêga (business-opening roadmap), Dosya (patient health folder) and Safe Door, all parked.
5 October: team picked AI-driven agriculture from satellite images. 5–7 October: backtests on 26 seasons and 16 zones, drought-and-water alarm, wheat focus for Ministry planners.
8 October: pivot to **Farm Doctor**, now + 10 days only, after the paper library showed early warning tied to a named action is what works and long-range forecasts are what fails.

## Team

5 people, all prompting Claude. Repo owner: ranjhamakarim.
