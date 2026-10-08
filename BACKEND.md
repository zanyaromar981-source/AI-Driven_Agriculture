# BACKEND.md: what the frontend needs from the backend

This file is the contract between the farmer app / dashboards (frontend) and the backend. The frontend writes here what it sends and what it expects back. If the backend needs something from the frontend, it writes `FRONTEND.md`, and the frontend follows that file strictly. Both files live at the repo root and are committed with every change.

Status: v2, 2026-10-08 20:48. Section 0 says exactly what the app still needs, checked against `backend/` at a0ade90 and `FRONTEND.md` v2. Backend: confirm or edit each section; mark changes with your date.

## 0. What the app still needs (read this first)

Checked 2026-10-08 20:48 against `backend/` at commit a0ade90 and `FRONTEND.md` v2. The app was also run on Android against a stand-in server with the backend's exact shapes over plain `http://`: sign in, My farms and opening a farm worked; Home then stopped at the missing status call (row 5).

### 0.1 Calls the app makes, in the order a farmer hits them

| # | Call | Backend today | Still needed |
|---|---|---|---|
| 1 | Sign in: `POST /v1/auth/otp/send`, `/v1/auth/otp/verify` (2.1) | built; for the demo, one fixed code (`AUTH__FIXED_SIGN_IN_CODE`) | Nothing for the demo. An SMS provider before real farmers (open point 2). |
| 2 | My farms: `GET /v1/farms` (2.2) | built | Nothing. |
| 3 | Save a farm: `POST /v1/farms` (2.2) | built, `Idempotency-Key` honoured, area from the outline | Cells and crop areas in 0.2 (sizes only, not blocking). |
| 4 | Open a farm: `GET /v1/farms/{id}` (2.2) | built | `inside_pct` on cells, 0.2 (not blocking). |
| 5 | Farm from space: `GET /v1/farms/{id}/status` (2.3) | not built | **This blocks Home.** Home loads a farm and its status together; today the `404` makes every farm show "Could not load this farm (not_found)". Fastest fix: the stub answer in 2.3 until the satellite job exists. |
| 6 | This week: `GET /v1/farms/{id}/plan` (2.4) | not built | Home still opens without it ("Weather forecast not available right now"). Then serve the stored plan (2.4). |

Not needed yet, because their screens are not built: Ask the Doctor (2.5), reports (2.6), alerts and devices (2.7), `DELETE /v1/account`.

### 0.2 Sizes: cells and crop areas (not blocking, but numbers disagree until fixed)

| Topic | Backend today | The app needs | Why |
|---|---|---|---|
| Farm `cells` | cells whose centre is inside the outline | every 10 m cell the outline touches, each with `inside_pct` (0 to 100) | The app sends every touched cell (the edge ones come back in `dropped_cells`, which is harmless). Home adds up `inside_pct` for the weak and measured m²; without it, edge cells count as whole. |
| `crops[].dunam` | painted cells / 25 | the sum of its cells' inside areas | Crops then add up to `area_dunam`. Today the crop list and the farm total disagree by the edge cells, up to 30 to 50% on plots under half a dunam (user decision 2026-10-08, section 1, Area). |

The app's clipping code can be ported: `cellsTouching` in `app/lib/geo.dart` returns each touched cell with its m² inside.

### 0.3 Answers to FRONTEND.md section 7

1. `PUT /v1/farms/{id}/cells` changing only the listed cells: fine. The app sends the whole painting on create, and a future repaint screen will send every cell.
2. 50,000 cells (2,000 dunam) per farm: fine. The app refuses outlines over 1,000 dunam (2,500,000 m²) before sending.
3. `GET`/`PUT /v1/me`: the Settings screen is not built yet. Its design shows the language, the phone number, the number of farms, two notification switches and "Delete my account and farms". So `phone` and `lang` are enough now. `name` can stay empty: the app never asks for a name ("No password, no name"). The switches belong to `POST /v1/devices` (2.7), the number of farms comes from `GET /v1/farms`, and delete is `DELETE /v1/account`.

### 0.4 Reaching the server

- Build the app with `--dart-define=API_URL=http://<server>:3000/v1`. From the Android emulator the Mac is `10.0.2.2`; a phone on the same Wi-Fi uses the Mac's address on that network.
- Plain `http://` works: tested on Android 2026-10-08 (Flutter's own network code is not held back by Android's cleartext rule). Use `https://` before real farmers.

## 1. Shared definitions (both sides must use exactly these)

| Thing | Definition |
|---|---|
| Phone | E.164 string, Iraqi mobile: `"+9647501234567"` (no spaces). The phone is the account; there is no name or password. |
| Farm | one outline (polygon) + a grid of cells + a name. One phone can own many farms. |
| Point | GPS corner tapped by the farmer: `{"lat": 36.0312, "lon": 44.6021, "acc_m": 6, "t": "2026-10-08T14:03:11Z"}` (WGS84 decimal degrees, accuracy in metres, UTC time). |
| Cell grid | 10 m squares aligned to the Sentinel-2 pixel grid: **UTM zone 38N (EPSG:32638)**, cell = `{"e": floor(easting/10), "n": floor(northing/10)}`. One cell = one satellite pixel. The frontend computes `e`,`n` from lat/lon with proj4 (EPSG:4326 → EPSG:32638); the backend validates and may correct. |
| Crop codes | `wheat`, `barley`, `tomato`, `cucumber`, `potato`, `onion`, `watermelon`, `grape`, `olive`, `sunflower`, `chickpea`, `empty`. Lower-case ASCII; the app maps them to emojis and Sorani labels. New codes only by editing this list on both sides. |
| Area | dunam (1 dunam = 2,500 m²). **Farm area = the exact area inside the outline** (shoelace in UTM metres), not a cell count. Each cell carries `inside_pct` (0 to 100, how much of it lies inside the outline); crop areas = sum of their cells' inside areas, so crops add up to the farm area. The backend computes areas; the app shows the same numbers before saving. (Changed 2026-10-08: counting whole cells was up to 30% off on small plots.) |
| Dates | ISO 8601. Timestamps are UTC and end in `Z` (`2026-10-08T17:01:35Z`); days are plain `YYYY-MM-DD`. |
| Language | every human-readable string the backend returns comes in both `"ku"` (Sorani, Arabic script) and `"en"`. The app shows `ku` by default. |
| Condition levels | `normal`, `watch`, `alarm`, `none` (no data yet). Season labels: `too_early`, `normal`, `dry`, `drought`, `wet`. |
| Confidence | `sure`, `likely`, `unsure`. |
| Numbers | the backend never rounds away precision needed for display; the app rounds. Percentages are 0–100 integers unless stated. |
| Admin units (added 2026-10-08) | 4 governorates (`Duhok`, `Erbil`, `Sulaymaniyah`, `Halabja`), 33 KRG districts, 78 sub-districts, as in `web/map_demo/kri_map_data.js`. Shapes are Iraq CSO 2019 sub-districts regrouped the KRG way (Akre, Shekhan, Bardarash in Duhok; Soran, Khalifan, Chuman, Sidakan, Mergasor, Harir, Pirmam, Taqtaq, Qushtapa as Erbil districts; Shahrazur in Sulaymaniyah). Kifri and Khanaqin are context only. Use the English `en` names as keys, `ku` for display. |

## 2. Endpoints the app needs

All paths start with `/v1` (FRONTEND.md). `Authorization: Bearer <token>` on everything after OTP verify. JSON in, JSON out, UTF-8, snake_case.

### 2.0 How the app connects (built 2026-10-08)
- The server address is set when the app is built: `flutter build apk --release --dart-define=API_URL=http://<host>:3000/v1`. The paths below are relative to it (`<API_URL>/farms`). Without `API_URL` the app runs on its built-in demo server (`app/lib/api/fake_api.dart`), which answers with exactly these shapes.
- Headers the app sends: `Accept: application/json` always; `Content-Type: application/json; charset=utf-8` with a body; `Authorization: Bearer <token>` after sign-in; `Idempotency-Key` on `POST /v1/farms`.
- Errors: JSON `{"error": "<code>", ...}` with the HTTP status from section 4. Extra fields (`field`, `retry_after_s`, `source`) reach the app. A non-JSON error page still works; the app then takes the code from the status.
- `401` on any call after sign-in signs the farmer out (back to the phone screen). `POST /v1/auth/otp/verify` answering `401 bad_code` does not sign anyone out.
- Waits: 10 s to connect, 20 s for an answer. No answer counts as offline: the app shows its saved copy and keeps unsent farms. So `status` and `plan` must answer from stored data (section 7C) and never fetch satellites or weather during the call.
- Upload queue (unsent farms): on `401`, `408`, `429`, `5xx` or no answer it keeps the farm and tries again; on any other `4xx` it drops the farm and tells the farmer. (Changed 2026-10-08: `429` used to drop the farm.) Send `422` only for a farm that can never be accepted, and `503` for "try later".
- Plain `http://` works on Android (tested 2026-10-08); use `https://` before real farmers (open point 5).
- Test without the phone: `app/test/http_api_test.dart` runs the app's real client against a small local server written from this file.

### 2.1 Sign in (needed first)
- `POST /v1/auth/otp/send`, no token, body `{"phone": "+9647501234567", "lang": "ku"}` → `200 {"sent": true, "retry_after_s": 60}`. Same answer whether the number is new or known. Asking again too soon: `429 {"error": "rate_limited", "retry_after_s": ...}`.
- `POST /v1/auth/otp/verify`, no token, body `{"phone": "+9647501234567", "code": "123456"}` → `200 {"token": "...", "farms_count": 2}`, or `401 {"error": "bad_code"}`. Codes: 6 digits, 10 minutes, 5 tries, one use. A number seen for the first time becomes an account here.
- Demo: one fixed code for every phone (`AUTH__FIXED_SIGN_IN_CODE`). Turn it off before real farmers.
- The app reads: `token` (required; kept as it is and sent as `Bearer`), `farms_count` (optional), `sent` (optional, `true` if missing), `retry_after_s` (optional, 60 if missing). On `bad_code` it shows "Wrong code, try again".

### 2.2 My farms

**FarmSummary** (one item of `GET /v1/farms`):

| Field | Type | Required | The app uses it for |
|---|---|---|---|
| `id` | string | yes | opening the farm (`GET /v1/farms/{id}`) |
| `name` | string, 1 to 100 characters | yes | the farm card and the Home title |
| `area_dunam` | number, not rounded, area inside the outline | yes | the card and the Farm view, shown in m² (× 2,500) |
| `crops` | `[{"crop", "dunam"}]`, largest first, no `empty` | no (empty list) | the card's main crop, the Crops view |
| `status` | `normal`, `watch`, `alarm`, `none` | no (`none`) | the pill on the card and on Home |
| `last_picture` | `YYYY-MM-DD` or `null` | no (`null` = "waiting for satellite") | the card |
| `centroid` | `{"lat", "lon"}` | no | not shown today |

**Farm** = FarmSummary plus:

| Field | Type | Required | The app uses it for |
|---|---|---|---|
| `outline` | `[{"lat", "lon"}]`, 3 or more, as walked | yes | drawing the farm on the map |
| `cells` | `[{"e", "n", "crop", "inside_pct", "greenness_pct", "level"}]` | yes (`e`, `n`, `crop`); `inside_pct` 100 if missing; `greenness_pct` and `level` `null` until readings exist | drawing, the Crops view, the weak and measured m² |
| `picture_date` | `YYYY-MM-DD` or `null` | no | not shown today |

- `GET /v1/farms` → `200 {"farms": [FarmSummary]}`.
- `GET /v1/farms/{id}` → `200 {"farm": Farm}`, or `404` when the farm does not exist or belongs to another phone.
- `POST /v1/farms` with `Idempotency-Key`, body:
  ```json
  {"name": "کێڵگەی سەرەوە",
   "points": [{"lat": 36.0312, "lon": 44.6021, "acc_m": 6, "t": "2026-10-08T14:03:11Z"}, ...],
   "cells": [{"e": 46415, "n": 398748, "crop": "wheat"}, ...],
   "created_offline_at": "2026-10-08T14:10:00Z"}
  ```
  → `201 {"farm": Farm, "dropped_cells": [{"e", "n"}]}`. The same key again answers `201` with the farm made the first time.
  - `points`: 3 to 50 dots in walking order; the polygon closes itself (tapped corners, or a walked track the app trimmed to at most 50). The app always sends `acc_m` and `t`; `acc_m: 0` means the dot was placed by hand on the map (test mode only).
  - `cells`: every cell the outline touches, unpainted ones as `"empty"` (0.2). A sent cell the farm does not keep comes back in `dropped_cells`; that is not an error.
  - `422` with `bad_polygon`, `farm_too_large` or `too_many_farms` (FRONTEND.md section 5).
  - Example check (pyproj): `{"lat": 36.0312, "lon": 44.6021}` is easting 464152.26, northing 3987482.22, so cell `{"e": 46415, "n": 398748}`.
  - Edge cells with a small `inside_pct` are mixed pixels: the backend may skip them for greenness.
- `PUT /v1/farms/{id}/cells` and `DELETE /v1/farms/{id}`: as in FRONTEND.md; the app does not call them yet.

### 2.3 My field from space (Field Eye)
- `GET /v1/farms/{id}/status` → `200 Status`, or `404` like 2.2. Answer from stored readings; never fetch a satellite during the call.
- **Stub, until the satellite job exists** (enough for Home to open the farm):
  ```json
  {"picture_date": null, "next_picture_expected": "2026-10-10",
   "greenness_pct_of_normal": null, "weak_where": null, "cells": [],
   "crops": [{"crop": "wheat", "dunam": 96, "greenness_pct_of_normal": null, "level": "none"}]}
  ```
- **Full answer** once readings exist:
  ```json
  {"picture_date": "2026-10-05", "next_picture_expected": "2026-10-10",
   "greenness_pct_of_normal": 101, "weak_where": "north-east",
   "cells": [{"e": 46415, "n": 398748, "greenness_pct": 78, "level": "watch", "since": "2026-09-25"}],
   "crops": [{"crop": "tomato", "dunam": 16, "greenness_pct_of_normal": 81, "level": "watch"}],
   "cloud_pct": 0, "pct_of_neighbours": 85, "surface": "growing", "weak_share_pct": 3,
   "history": [{"year": 2025, "greenness": 0.079}]}
  ```

| Field | The app uses it for |
|---|---|
| `picture_date` | "From space 5 Oct"; `null` = "Waiting for the first satellite picture" |
| `next_picture_expected` | "next picture 10 Oct" |
| `greenness_pct_of_normal` | the Farm view's big number (whole farm, 100 = normal) |
| `weak_where` | the weak line ("north-east corner"); words: `north`, `middle`, `south` + `west`, `centre`, `east`, joined by `-` |
| `cells[]`: `e`, `n`, `greenness_pct`, `level`, `since` | cell colours, the cell card, the weak m². `greenness_pct` = this cell vs its own normal this week (100 = normal), `null` when cloudy or not sown. `since` = first day of the current level. |
| `crops[]`: `crop`, `dunam`, `greenness_pct_of_normal`, `level` | the Crops view and crop card; `null` % = "not sown yet" |
| `cloud_pct`, `pct_of_neighbours`, `surface`, `weak_share_pct`, `history` | not shown today (the Doctor will use them) |

### 2.4 This week's plan (Weather Planner)
- `GET /v1/farms/{id}/plan` → `200 Plan`. Until it exists any error is fine (Home still opens); `503 {"error": "upstream_down", "source": "weather"}` says it best. Answer from the stored plan (every 6 hours, section 7B); never call the weather service during the call. No forecasts beyond 10 days, ever. Thresholds come from `reports/Farm_Advice_Research.md`.
  ```json
  {"from": "2026-10-08", "days": 10,
   "rain_mm": [0, 0, 2.1, 14.2, 0, 0, 0, 0, 0, 0],
   "tmin": [9, 8, 7, -3, 6, 7, 8, 9, 9, 10], "tmax": [24, 23, 20, 15, 19, 22, 24, 25, 25, 26],
   "alerts": [{"type": "frost", "day": "2026-10-11", "value": -3, "level": "alarm", "ku": "...", "en": "Frost -3 °C Sun night"}],
   "decisions": [{"code": "frost_check", "ku": "...", "en": "Hard frost on Sun. Check the heads 7 to 10 days after."}],
   "source": "Open-Meteo (ECMWF/GraphCast family)", "issued": "2026-10-08T06:00:00Z"}
  ```

| Field | Required | The app uses it for |
|---|---|---|
| `from` (`YYYY-MM-DD`) | yes | the first day chip |
| `issued` (UTC with `Z`) | yes | "issued 8 Oct 09:00" |
| `rain_mm` (one number or `null` per day, up to 10) | yes | one day chip per item; rain of 1 mm or more is shown |
| `alerts[]`: `type`, `day`, `level`, `value`, `ku`, `en` | no | a red or amber dot on that day (`alarm` or `watch`) |
| `decisions[]`: `code`, `ku`, `en` | no | the list under the chips, one icon per `code`; empty = "Nothing to act on in the next 10 days" |
| `source` | no | the source line |
| `tmin`, `tmax`, `days` | no | not shown today |

- Alert `type`: `frost`, `heat`, `heavy_rain`, `dry_spell`, `rust_weather`, `sunn_pest`, `dust`, `spray_window`, `sowing_rain`, `urea_rain`. Decision `code`: `sow_wait`, `sow_go`, `urea_go`, `urea_hold`, `spray_ok`, `check_rust`, `count_sunn_pest`, `frost_check`, `heat_check`, `dust_delay`.
- The rules with their numbers are in `farm_doctor/weather_planner.py`; the app's demo server has the same rules producing exactly this JSON (`planFromWeather` in `app/lib/api/fake_api.dart`), handy as a reference.

**2.5 to 2.8 are for later**: their screens are not built yet. Their paths also start with `/v1`.

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
- Outbox (built 2026-10-08): `POST /farms` is first written to a file on the phone, then sent; with no internet it stays there and is retried every 30 seconds and when the app comes back to the front. The border being marked is also saved on the phone after every dot, and the sign-in token is kept, so the app opens and works in the field with no signal. "No internet" = the request never reached the server; any other 4xx answer (not 401, 408 or 429) removes the farm from the outbox (it will not succeed on retry).

## 4. Errors
Shape (FRONTEND.md section 5): `{"error": "<code>", "detail": "<English text>"}`, plus `field` or `retry_after_s` when they apply. What the app does:

| Status | Codes | The app |
|---|---|---|
| `400` | `bad_request` | shows an error with the code; a queued farm is dropped |
| `401` | `unauthorized` | signs the farmer out (back to the phone screen); a queued farm is kept |
| `401` | `bad_code` (sign-in only) | shows "Wrong code, try again"; nobody is signed out |
| `404` | `not_found` | Home shows "Could not load this farm" |
| `408`, `429`, `5xx` | `rate_limited` (with `retry_after_s`), `upstream_down` (with `source`), `server_error` | a queued farm is kept and sent again. Home shows "Could not load this farm" for the farm or status, and the last saved plan (or "Weather forecast not available right now") for the plan. The saved copy of everything is shown only when there is no answer at all. |
| `422` | `invalid`, `bad_polygon`, `farm_too_large`, `too_many_farms` | a queued farm is dropped and the farmer is told |

The only code the app branches on today is `bad_code`; the others are shown as they are and will get Sorani messages later.

## 5. What already exists on the backend side (today)
| Call | Where | State |
|---|---|---|
| 2.1 sign in | `backend/` (Rust, axum, Postgres) | built; demo uses one fixed code; no SMS provider yet |
| 2.2 farms: list, create, open, repaint, delete | `backend/` | built; cells and crop areas still by the centre rule (0.2) |
| profile `GET`/`PUT /v1/me` | `backend/` | built (not used by the app yet) |
| 2.3 status | `farm_doctor/field_eye.py` `measure(lon, lat, date)` | Python only: one 1 km square around a point, about 16 s; needs the per-cell version and a stored daily job. Decided 2026-10-08 (user, Arya): the Python AIs push results into `backend/` through `/v1/ingest` (guarded by `X-Service-Key`) and `backend/` serves them. The app-facing `GET /v1/farms/{id}/status` is not built yet. |
| 2.4 plan | `farm_doctor/weather_planner.py` `plan(lon, lat)` | Python only; decisions are English sentences, need the codes, `ku` and the alert list. Same route in through `/v1/ingest`; `GET /v1/farms/{id}/plan` not built yet. |
| season label | `farm_doctor/season_check.py` | Python only, 16 zones |
| 2.5 doctor | `farm_doctor/doctor.py` | Python only; needs `GEMINI_API_KEY` |
| 2.6 nearby reports | `farm_doctor/neighbour_watch.py` | test data only |
| 2.8 region | `farm_doctor/build_now.py` → `web/now.json` | works, 61 s for 16 zones |
| 2.7 push, `DELETE /v1/account` | nothing yet | later |

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
`POST /farms/{id}/ask`: the server gathers the stored numbers for that farm (status, plan, season, nearby reports, dams), calls the Doctor's model (2.10) with the rulebook, saves the case in `cases`, returns the answer. The app shows "reading the field" meanwhile.

**E. Offline**
The app keeps a local copy of its farms, the last status and plan, and shows them with their date when there is no signal; it syncs when back online (Idempotency-Key on POSTs).

**Tables (minimum):** `users (phone, created_at)`, `devices (user, push_token, platform, lang)`, `farms (id, user, name, outline, area_dunam, created_at)`, `cells (id, farm, e, n, crop)`, `cell_readings (cell, date, greenness_pct, level, cloud_pct)`, `plans (farm, issued, json)`, `reports (id, farm, cell, type, note, photo, lat, lon, t)`, `cases (id, farm, question, inputs_json, answer_json, t)`, `otp_codes (phone, code_hash, expires, tries)`.

## 8. Open points (answer here)
1. Field edge shortcut: offer the satellite-detected edge (SAM) as a suggestion, always walk, or walk then tidy. (User decision pending.)
2. SMS provider for Iraqi numbers (Twilio, local SMS gateway) and cost. Until then the demo uses one fixed code (2.1).
3. Where the backend runs for the demo (laptop, DigitalOcean droplet) and the base URL.
4. Voice: Google Chirp 2 `ckb-IQ` or type-only for the demo.
5. Server address for the demo: plain `http://` works with no app change (tested on Android 2026-10-08), so a laptop on the venue Wi-Fi is fine; `https://` before real farmers.
