# BACKEND.md: what the frontend needs from the backend

This file is the contract between the farmer app / dashboards (frontend) and the backend. The frontend writes here what it sends and what it expects back. If the backend needs something from the frontend, it writes `FRONTEND.md`, and the frontend follows that file strictly. Both files live at the repo root and are committed with every change.

Status: v1, proposed by the frontend on 2026-10-08. Backend: confirm or edit each section; mark changes with your date.

## 1. Shared definitions (both sides must use exactly these)

| Thing | Definition |
|---|---|
| Phone | E.164 string, Iraqi mobile: `"+9647501234567"` (no spaces). The phone is the account; there is no name or password. |
| Farm | one outline (polygon) + a grid of cells + a name. One phone can own many farms. |
| Point | GPS corner tapped by the farmer: `{"lat": 36.0312, "lon": 44.6021, "acc_m": 6, "t": "2026-10-08T14:03:11Z"}` (WGS84 decimal degrees, accuracy in metres, UTC time). |
| Cell grid | 10 m squares aligned to the Sentinel-2 pixel grid: **UTM zone 38N (EPSG:32638)**, cell = `{"e": floor(easting/10), "n": floor(northing/10)}`. One cell = one satellite pixel. The frontend computes `e`,`n` from lat/lon with proj4 (EPSG:4326 → EPSG:32638); the backend validates and may correct. |
| Crop codes | `wheat`, `barley`, `tomato`, `cucumber`, `potato`, `onion`, `watermelon`, `grape`, `olive`, `sunflower`, `chickpea`, `empty`. Lower-case ASCII; the app maps them to emojis and Sorani labels. New codes only by editing this list on both sides. |
| Area | dunam (1 dunam = 2,500 m²). **Farm area = the exact area inside the outline** (shoelace in UTM metres), not a cell count. Each cell carries `inside_pct` (0 to 100, how much of it lies inside the outline); crop areas = sum of their cells' inside areas, so crops add up to the farm area. The backend computes areas; the app shows the same numbers before saving. (Changed 2026-10-08: counting whole cells was up to 30% off on small plots.) |
| Dates | ISO 8601, UTC for timestamps (`...Z`), plain `YYYY-MM-DD` for days. |
| Language | every human-readable string the backend returns comes in both `"ku"` (Sorani, Arabic script) and `"en"`. The app shows `ku` by default. |
| Condition levels | `normal`, `watch`, `alarm`, `none` (no data yet). Season labels: `too_early`, `normal`, `dry`, `drought`, `wet`. |
| Confidence | `sure`, `likely`, `unsure`. |
| Numbers | the backend never rounds away precision needed for display; the app rounds. Percentages are 0–100 integers unless stated. |
| Admin units (added 2026-10-08) | 4 governorates (`Duhok`, `Erbil`, `Sulaymaniyah`, `Halabja`), 33 KRG districts, 78 sub-districts, as in `web/map_demo/kri_map_data.js`. Shapes are Iraq CSO 2019 sub-districts regrouped the KRG way (Akre, Shekhan, Bardarash in Duhok; Soran, Khalifan, Chuman, Sidakan, Mergasor, Harir, Pirmam, Taqtaq, Qushtapa as Erbil districts; Shahrazur in Sulaymaniyah). Kifri and Khanaqin are context only. Use the English `en` names as keys, `ku` for display. |

## 2. Endpoints the app needs

Base URL and auth: `Authorization: Bearer <token>` on everything after OTP verify. JSON in, JSON out, UTF-8.

### 2.1 Sign in (OTP)
- `POST /auth/otp/send` body `{"phone": "+9647501234567", "lang": "ku"}` → `200 {"sent": true, "retry_after_s": 59}`. Rate-limit per phone; same response whether the number is new or known.
- `POST /auth/otp/verify` body `{"phone": "+9647501234567", "code": "123456"}` → `200 {"token": "...", "farms_count": 2}` or `401 {"error": "bad_code"}`. Codes expire in 10 minutes, 5 tries.

### 2.2 My farms
- `GET /farms` → `200 {"farms": [FarmSummary]}`
  FarmSummary: `{"id": "f_01HX...", "name": "کێڵگەی سەرەوە", "area_dunam": 120, "crops": [{"crop":"wheat","dunam":96},{"crop":"tomato","dunam":16}], "status": "normal|watch|alarm|none", "last_picture": "2026-10-05", "centroid": {"lat":36.03,"lon":44.60}}`
- `GET /farms/{id}` → `200 {"farm": Farm}`, or `404 not_found` when the farm is not this phone's. Added 2026-10-08: Home needs the outline and cells to draw each farm.
- `POST /farms` body:
  ```json
  {"name": "کێڵگەی سەرەوە",
   "points": [Point, Point, Point, ...],            // 3 to 50 dots in walking order, polygon closes itself (tapped corners, or a walked track the app trimmed to at most 50)
   "cells": [{"e": 46415, "n": 398748, "crop": "wheat"}, ...],    // every cell inside the outline with its crop ("empty" if not painted)
   "created_offline_at": "2026-10-08T14:10:00Z"}
  ```
  → `201 {"farm": Farm, "dropped_cells": [{"e","n","crop"}...]}` (`dropped_cells` = the sent cells that fell outside the outline, usually empty). Backend rules: snap points to the grid, reject if the polygon self-intersects (`422 {"error":"bad_polygon"}`), drop cells outside the outline. Farm cells = every 10 m cell that overlaps the outline (edge cells are cut along the border and carry `inside_pct`); the app sends all of them, unpainted ones as `"empty"`. Edge cells with a small `inside_pct` are mixed pixels: the backend may skip them for greenness.
  - Example check (2026-10-08, pyproj): point `{"lat": 36.0312, "lon": 44.6021}` is easting 464152.26, northing 3987482.22, so cell `{"e": 46415, "n": 398748}`. The earlier example `e: 462337` was wrong.
  - `area_dunam` and each `crops[].dunam` are decimals (two places are enough); the app formats them. Farm `cells[]` items also carry `"inside_pct"`.
  - `acc_m: 0` on a point means it was placed by hand on the map (test mode only), not by GPS.
  Farm: FarmSummary + `{"outline": [{"lat","lon"}...], "cells": [{"e","n","crop","greenness_pct": 0–200|null, "level": "normal|watch|alarm|none"}], "picture_date": "2026-10-05"|null}`
- `PUT /farms/{id}/cells` body `{"cells": [{"e","n","crop"}...]}` → `200 {"farm": Farm}` (repaint crops).
- `DELETE /farms/{id}` → `204`.

### 2.3 My field from space (Field Eye)
- `GET /farms/{id}/status` → `200 {"picture_date": "2026-10-05", "cloud_pct": 0, "greenness_pct_of_normal": 101, "pct_of_neighbours": 85, "surface": "bare|sparse|growing|dense", "weak_share_pct": 0, "weak_where": "north-east"|null, "cells": [{"e","n","greenness_pct","level","since": "2026-09-25"|null}...], "history": [{"year": 2025, "greenness": 0.079}, ...], "next_picture_expected": "2026-10-10"}`
  Rule: `greenness_pct` is the cell vs its own normal for this week (100 = normal). `null` when cloudy. The backend fetches Sentinel-2; the app never calls satellites.
  App side (2026-10-08): Home also reads `crops` (2.9) and counts cells by area, `sum(inside_pct) / 100`, so an edge cell half inside counts as half and the count matches the dunams (25 cells = 1 dunam).

### 2.4 This week's plan (Weather Planner)
- `GET /farms/{id}/plan` → `200 {"from": "2026-10-08", "days": 10, "rain_mm": [0,0,2.1,...], "tmin": [...], "tmax": [...], "alerts": [{"type": "frost|heat|heavy_rain|dry_spell|rust_weather|sunn_pest|dust|spray_window|sowing_rain|urea_rain", "day": "2026-10-12", "value": -2.1, "level": "watch|alarm", "ku": "...", "en": "..."}], "decisions": [{"code": "sow_wait|sow_go|urea_go|urea_hold|spray_ok|check_rust|count_sunn_pest|frost_check|heat_check|dust_delay", "ku": "...", "en": "..."}], "source": "Open-Meteo (ECMWF/GraphCast family)", "issued": "2026-10-08T06:00:00Z"}`
  Rule: no forecasts beyond 10 days, ever. Thresholds come from `reports/Farm_Advice_Research.md`.

### 2.5 Ask the Doctor
- `POST /farms/{id}/ask` multipart: `question` (text, optional), `voice` (DEFERRED: not in v1, decided 2026-10-08; field reserved), `photos[]` (jpeg, 1–6, ≤ 4 MB each, optional), `cell` (optional `{"e","n"}` the farmer tapped), `lang` (`ku|en`).
  → `200 {"likely": "...", "confidence": "sure|likely|unsure", "why": ["field_eye -> ...", "weather -> ..."], "actions_this_week": ["..."] (max 3), "cannot_tell": ["..."], "refer_to_officer": true|false, "ku": "...", "en": "...", "transcript": "..." (if voice), "case_id": "c_..."}`
  Rules (hard): no pesticide or fertilizer doses, no product names; only numbers from the AIs; `unsure` + `refer_to_officer: true` when inputs conflict. Response time target ≤ 25 s; the app shows "reading the field" meanwhile.

### 2.6 Reports (Neighbour Watch)
- `POST /reports` body `{"farm_id": "f_...", "cell": {"e","n"}|null, "type": "yellow_stripes|insects|wilting|flood|hail|fire|animal_disease|other", "note": "...", "photo_id": "..."|null, "lat", "lon", "t"}` → `201 {"report_id": "r_..."}`
- `GET /reports/nearby?lat=&lon=&km=20&days=14` → `200 {"count": 3, "by_type": {"yellow_stripes": 2}, "closest": [{"type","km","days_ago"}]}`. Used by the Doctor and the Ministry only: **farmers do not see other farmers' reports** (decided 2026-10-08). Never return another farmer's phone or exact location; round to 1 km.
- `GET /reports/mine` → `200 {"reports": [{"report_id","farm_id","cell","type","note","t","status": "sent|seen_by_officer"}]}`: the farmer's own reports list.

### 2.7 Alerts (push)
- `POST /devices` body `{"push_token": "...", "platform": "android|ios", "lang": "ku"}` → `204`.
- Push rules (decided 2026-10-08): at most **one push per farm per day**; only level `alarm` (red) is pushed, `watch` is shown in the app only; every push says what to do and how sure: `{"farm_id","alert_id","type","day","level":"alarm","confidence":"sure|likely|unsure","ku","en","action_ku","action_en"}`. The weekly plan goes out Sunday 06:00 local as one message.
- `GET /farms/{id}/alerts?days=30` → `200 {"alerts": [{"alert_id","type","day","level","confidence","ku","en","action_ku","action_en","pushed": true|false,"done": true|false}]}`; `POST /alerts/{id}/done` → `204`.
- `POST /devices` also takes `"notify": {"red_alerts": true, "weekly_plan": true}`.
- `DELETE /account` → `204` (removes the phone, farms, reports and cases).

### 2.8 Dashboard (Ministry, no login)
- `GET /region/now` → the structure of `web/now.json` (zones with field_eye, weather, season, neighbours; dams; summary; brief). Keep that shape; the dashboard already reads it.
- Map demo (`web/map_demo/`, 2026-10-08) needs no backend: boundaries and towns are a static file. It calls two free public services from the browser: Nominatim (OpenStreetMap search, max 1 request per second, only when the user types 3+ letters) and Open-Meteo elevation (one call per dropped pin). If the dashboard later needs per-district numbers, key them by the admin unit `en` names above.

### 2.9 App decisions that affect the backend (2026-10-08)
- Field edge: the farmer **always walks the corners**; no satellite edge suggestion in the app flow (SAM stays a backend tool for the Ministry map).
- Home shows all farms stacked: `GET /farms` must return every farm with enough to draw the grid summary (`status`, `last_picture`, `crops`), and `GET /farms/{id}/status` is called per farm on open.
- Opening a farm (built 2026-10-08): a tap in My farms opens Home at that farm. Home calls `GET /farms/{id}`, `/status` and `/plan` for every farm and keeps the last copy of each on the phone, shown with its date when offline. Home shows the `en` texts until the Sorani check.
- Cell tap views: cell, crop plot, whole farm. The backend adds per-crop summaries to 2.3: `"crops": [{"crop","dunam","greenness_pct_of_normal","level"}]` and the whole-farm `greenness_pct_of_normal` (already there).
- Labels: new screens are English for now; Sorani comes later, but the backend keeps returning both `ku` and `en`.

### 2.10 The Doctor's model (decided 2026-10-08)
- Provider: **Gemini** (Google AI Studio key `GEMINI_API_KEY`, default model `gemini-2.5-flash`), chosen for cost (about 5–10x cheaper per answer). The call is one swappable function (`farm_doctor/doctor.py`: `FARM_DOCTOR_PROVIDER=gemini|claude`, `FARM_DOCTOR_MODEL`), so a head-to-head test against Claude in Sorani (20 questions, scored by a native speaker) can be run before any real rollout. The rulebook, the JSON answer shape and the no-doses rule are identical for both.

## 3. Offline rules (frontend side, so the backend knows what to expect)
- The app collects points and painted cells with no internet and stores them locally. It POSTs the farm when online; `created_offline_at` carries the real time. Expect bursts of old farms.
- The app caches the last `status`, `plan` and farms list; it shows the cached copy with its date when offline. The backend sets `Cache-Control: max-age` honestly (status: 1 day; plan: 6 hours).
- Idempotency: the app sends `Idempotency-Key` headers on POSTs; repeat keys must not create duplicates. A repeated key returns the farm made the first time (the app retries uploads that lost their answer).
- Outbox (built 2026-10-08): `POST /farms` is first written to a file on the phone, then sent; with no internet it stays there and is retried every 30 seconds and when the app comes back to the front. The border being marked is also saved on the phone after every dot, and the sign-in token is kept, so the app opens and works in the field with no signal. "No internet" = the request never reached the server; any 4xx answer removes the farm from the outbox (it will not succeed on retry).

## 4. Errors
`400 bad_request`, `401 unauthorized`, `404 not_found`, `422 {error, field}` for validation, `429 rate_limited {retry_after_s}`, `503 upstream_down {source: "sentinel|weather|claude"}` when a feed is down (the app then shows the cached copy and the reason).

## 5. What already exists on the backend side (today)
| Endpoint | Module in `farm_doctor/` | State |
|---|---|---|
| 2.3 status | `field_eye.measure(lon, lat, date)` | works, ~16 s per field; needs the polygon/cell version |
| 2.4 plan | `weather_planner.plan(lon, lat)` | works; decisions are English strings, need codes + ku |
| season label | `season_check.check(lon, lat, date)` | works for the 16 zones; per-farm = nearest zone |
| 2.6 nearby | `neighbour_watch.nearby(lon, lat, date)` | works on test data |
| dams | `dam_watch.latest(date)` | works from the CSV |
| 2.5 doctor | `doctor.ask(inputs, question, photos)` | works, needs `ANTHROPIC_API_KEY`; voice not yet |
| 2.8 region | `build_now.py` → `web/now.json` | works, 61 s for 16 zones |
| 2.1 OTP, 2.2 farms, 2.7 push | nothing yet | to build |

## 7. How the data flows (app, server, database)

**A. The farmer creates a farm** (app → server → database)
1. The app collects GPS corners and painted cells; this works offline.
2. The app sends one `POST /farms` with points and cells (section 2.2).
3. The server validates the polygon, snaps cells to the 10 m grid, and saves `farms` + `cells` (one row per cell with its crop).
4. The server answers with the farm; if a satellite picture is already cached for that area, cell values come back at once.

**B. The server keeps every farm fresh** (background jobs, no app involved)
- Daily `satellite_job`: for each farm, look for a new Sentinel-2 picture; compute greenness per cell vs that cell's normal; store rows in `cell_readings (cell_id, date, greenness_pct, level, cloud_pct)`.
- Every 6 hours `weather_job`: run the Weather Planner per farm centroid; store `plans (farm_id, issued, days[], alerts[], decisions[])`; push alerts of level `alarm`.
- Weekly `brief_job`: the Doctor writes the Ministry brief and the per-village farmer message.
Heavy work happens once here, never when the farmer opens the app.

**C. The farmer opens the app** (app → server → database → app)
1. `GET /farms/{id}/status` reads the latest `cell_readings`; no satellite call; answers in milliseconds.
2. The app paints each cell from `level` and shows `picture_date`.
3. `GET /farms/{id}/plan` reads the stored plan.

**D. Ask the Doctor** (the only slow call, 10–25 s)
`POST /farms/{id}/ask`: the server gathers the stored numbers for that farm (status, plan, season, nearby reports, dams), calls Claude with the rulebook, saves the case in `cases`, returns the answer. The app shows "reading the field" meanwhile.

**E. Offline**
The app keeps a local copy of its farms, the last status and plan, and shows them with their date when there is no signal; it syncs when back online (Idempotency-Key on POSTs).

**Tables (minimum):** `users (phone, created_at)`, `devices (user, push_token, platform, lang)`, `farms (id, user, name, outline, area_dunam, created_at)`, `cells (id, farm, e, n, crop)`, `cell_readings (cell, date, greenness_pct, level, cloud_pct)`, `plans (farm, issued, json)`, `reports (id, farm, cell, type, note, photo, lat, lon, t)`, `cases (id, farm, question, inputs_json, answer_json, t)`, `otp_codes (phone, code_hash, expires, tries)`.

## 8. Open points (answer here)
1. Field edge shortcut: offer the satellite-detected edge (SAM) as a suggestion, always walk, or walk then tidy. (User decision pending.)
2. OTP provider for Iraqi numbers (Twilio, local SMS gateway) and cost.
3. Where the backend runs for the demo (laptop, DigitalOcean droplet) and the base URL.
4. Voice: Google Chirp 2 `ckb-IQ` or type-only for the demo.
