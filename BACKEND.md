# BACKEND.md: what the frontend needs from the backend

This file is the contract between the farmer app / dashboards (frontend) and the backend. The frontend writes here what it sends and what it expects back. If the backend needs something from the frontend, it writes `FRONTEND.md`, and the frontend follows that file strictly. Both files live at the repo root and are committed with every change.

Status: v3, 2026-10-09 16:20. Section 0 says exactly what the app still needs, checked against `FRONTEND.md` v4 (commit 13ba149). Backend: confirm or edit each section; mark changes with your date. Added 2026-10-08 21:46: section 2.11, the Control Room (officer sign-in and `/v1/admin` routes for dashboard screens 11 to 18).

## 0. What the app still needs (read this first)

Checked 2026-10-09 16:20 against `FRONTEND.md` v4 with its section 14 (commit 13ba149). The app follows FRONTEND.md v4 since 2026-10-09 14:41 and is built against the test server `http://95.217.14.92:8790/v1`. The size points of the old 0.2 (cells with `inside_pct`, crop areas from inside areas) are done on the backend side: thank you.

### 0.1 Calls the app makes, in the order a farmer hits them

| # | Call | Backend today | What the app needs from the backend |
|---|---|---|---|
| 1 | Sign in: `POST /v1/auth/otp/send`, `/v1/auth/otp/verify` (2.1) | built, real codes through OTPIQ on the test server | Nothing. |
| 2 | My farms, save, open, edit, delete: `/v1/farms` (2.2) | built, edit with `PUT /v1/farms/{id}` | Nothing. |
| 3 | Farm from space: `GET /v1/farms/{id}/status` (2.3) | placeholder, every measured field `null` | **Blocks the main picture on Home.** See 0.2, answer to FRONTEND.md 13.1. |
| 4 | This week: `GET /v1/farms/{id}/plan` (2.4) | not built (`404`) | **The most wanted route for farmers.** The 10-day plan from the forecast with the Weather Planner rules in 2.4. The app shows "10-day plan coming soon" until it exists. It goes through the server only: the app does not call Open-Meteo itself. |
| 5 | Ask the Doctor: `POST /v1/farms/{id}/ask` (2.5) | route built; on the test server nothing runs behind it, so every question answers `502 doctor_failed` (FRONTEND.md v5, 6) | **The centre button of the app. See 2.15:** the server gathers the farm's data, gives it with the photos to Codex (already signed in there) and returns the checked answer. No Gemini key needed. |
| 6 | Insights: `GET /v1/farms/{id}/insights` | built, groundwater filled daily | Nothing for the route. Other topics wait for the per-farm analysis job (FRONTEND.md 13.4). |
| 7 | Daily brief: `GET /v1/farms/{id}/brief` | built | Nothing. The app card is next. |
| 8 | Profile: `GET`/`PUT /v1/me` | built | Nothing. The Settings screen is next. |
| 9 | Alwa market, farmer routes (FRONTEND.md 5) | built, with offers, grade, pickup, market and hidden phones | **Simpler Alwa (user decision 2026-10-09): see 2.14.** GPS point on each listing, phones shown, nearest first, mark as sold, markets with a point. |

Needed next, because their screens are being built now:
- `DELETE /v1/account` for Settings ("Delete my account and farms").
- Alerts list and `POST /v1/devices` for push (2.7).
- Reports (2.6).

### 0.2 Answers to FRONTEND.md section 13

1. Farm status: answer pending (user decision, 2026-10-09).
2. Alwa: the app has no offers (user decision 2026-10-09). Buyers call the seller, whose phone is shown from the start. See 2.14.
3. Protected mode: for the website team (2.12).
4. Per-farm analysis job: answer pending.
5. Missing from `backend/API.md`: nothing found for the app so far.

### 0.3 Reaching the server

- The app is built with `--dart-define=API_URL=http://95.217.14.92:8790/v1`. Plain `http` is allowed in the app for that address only (`app/android/app/src/main/res/xml/network_security_config.xml`).
- The app no longer uses the Codespace server. The three test farms made there from the user's phone are not on the test server.

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
- `PUT /v1/farms/{id}` (added 2026-10-08 21:10, for editing a farm): body as `POST /v1/farms` (`name`, `points`, `cells`, `created_offline_at`); replaces the outline, cells and name; same validation (`bad_polygon`, `farm_too_large`); `Idempotency-Key` honoured; `404` for another phone's farm. → `200 {"farm": Farm, "dropped_cells": [{"e", "n"}]}`.
- `DELETE /v1/farms/{id}` (used since 2026-10-08 21:33): `204`, or `404` when the farm is gone or belongs to another phone. The app counts `404` as already deleted.
- Edits and deletes made without internet wait in the phone's queue like new farms (section 3); deletes are sent first, then new farms and edits, oldest first.
- `PUT /v1/farms/{id}/cells`: as in FRONTEND.md; the app does not call it (edits use `PUT /v1/farms/{id}`).

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
- What the app does (2026-10-09): sends `question` (trimmed, max 1000 characters), `lang`, `cell` as JSON text when the farmer asked from a tapped square, and each photo as a repeated `photos` part (JPEG, made 1600 px and 80% quality on the phone, so well under 4 MB; PNG only if the gallery gives one). It waits up to 120 s. It reads `likely`, `confidence`, `why` (shown as "Input: conclusion", with an icon chosen from the input's name), `actions_this_week` (first 3), `cannot_tell`, `refer_to_officer` (shows "show this to the office"), `ku` and `en` (a Sorani / English switch). It does not use `case_id` or `transcript` yet. Error codes it words for the farmer: `doctor_not_ready` ("not switched on yet"), `bad_photo`, `empty_question`, `404`, offline; anything else is "could not answer this time".
- Backend, 2026-10-09 12:11: built as `POST /v1/farms/{id}/ask`, exactly as in `FRONTEND.md` section 13. Differences from the line above: photos may be JPEG or PNG (set each part's Content-Type), 0 to 6; the answer adds `inputs_used`; no `case_id` yet (nothing is stored; needed later for the Control Room inbox) and no `transcript` (voice deferred); `502 doctor_failed` or `503 doctor_not_ready` when the Doctor service cannot answer. The backend waits up to 90 s for the Doctor, so this call needs a longer answer timeout than the 20 s of 2.0.

### 2.6 Reports (Neighbour Watch)
- `POST /reports` body `{"farm_id": "f_...", "cell": {"e","n"}|null, "type": "yellow_stripes|insects|wilting|flood|hail|fire|animal_disease|other", "note": "...", "photo_id": "..."|null, "lat", "lon", "t"}` → `201 {"report_id": "r_..."}`
- `GET /reports/nearby?lat=&lon=&km=20&days=14` → `200 {"count": 3, "by_type": {"yellow_stripes": 2}, "closest": [{"type","km","days_ago"}]}`. Used by the Doctor and the Ministry only: **farmers do not see other farmers' reports** (decided 2026-10-08). Never return another farmer's phone or exact location; round to 1 km.
- `GET /reports/mine` → `200 {"reports": [{"report_id","farm_id","cell","type","note","t","status": "sent|seen_by_officer"}]}`: the farmer's own reports list.
- What the app does now (2026-10-09), until these routes exist: the Report screen (opened from a square's card) sends `POST /v1/messages` (FRONTEND.md 4) with `kind` `report`, `farm_id`, up to 4 `photos`, and `text` = `"<type>, square <label>: <note>"` (for example `"Yellow stripes, square E12: since Monday"`), with an `Idempotency-Key`. "My reports" is `GET /v1/messages/mine` filtered to `kind` `report`: `new` shows as "Sent", any other state as "Seen by officer". When `POST /reports` is built, the app moves to it and sends `type` and `cell` as fields.

### 2.7 Alerts (push)
- `POST /devices` body `{"push_token": "...", "platform": "android|ios", "lang": "ku"}` → `204`.
- Push rules (decided 2026-10-08): at most **one push per farm per day**; only level `alarm` (red) is pushed, `watch` is shown in the app only; every push says what to do and how sure: `{"farm_id","alert_id","type","day","level":"alarm","confidence":"sure|likely|unsure","ku","en","action_ku","action_en"}`. The weekly plan goes out Sunday 06:00 local as one message.
- `GET /farms/{id}/alerts?days=30` → `200 {"alerts": [{"alert_id","type","day","level","confidence","ku","en","action_ku","action_en","pushed": true|false,"done": true|false}]}`; `POST /alerts/{id}/done` → `204`.
- `POST /devices` also takes `"notify": {"red_alerts": true, "weekly_plan": true}`.
- `DELETE /account` → `204` (removes the phone, farms, reports and cases).
- What the app does now (2026-10-09): the Alerts tab calls `GET /v1/farms/{id}/alerts?days=30` for the open farm and groups them by day (today, yesterday, last week); a `404` shows "Alerts are coming soon". The done tick is kept on the phone until `POST /alerts/{id}/done` exists. Settings keeps the two notification switches on the phone until `POST /devices` takes `notify`, and "Delete my account and farms" calls `DELETE /v1/account`; a `404` says nothing was deleted and to call the office.

### 2.8 Dashboard (Ministry, no login)
- `GET /region/now` → the structure of `web/now.json` (zones with field_eye, weather, season, neighbours; dams; summary; brief). Keep that shape; the dashboard already reads it.
- Map demo (`web/map_demo/`, 2026-10-08) needs no backend: boundaries and towns are a static file. It calls two free public services from the browser: Nominatim (OpenStreetMap search, max 1 request per second, only when the user types 3+ letters) and Open-Meteo elevation (one call per dropped pin). If the dashboard later needs per-district numbers, key them by the admin unit `en` names above.

### 2.9 App decisions that affect the backend (2026-10-08)
- Field edge: the farmer **always walks the corners**; no satellite edge suggestion in the app flow (SAM stays a backend tool for the Ministry map).
- One farm per screen (user, 2026-10-08; replaces "Home shows all farms stacked"): `GET /v1/farms` still needs `status`, `last_picture` and `crops` for the My farms cards.
- Opening a farm: a tap in My farms opens that farm alone. The app calls `GET /v1/farms/{id}`, `/status` and `/plan` for that farm only, and keeps the last copy on the phone, shown with its date when offline. The farm screen shows the `en` texts until the Sorani check.
- Each farm's screen has Edit (border, crops and name, through `PUT /v1/farms/{id}`) and Delete (`DELETE /v1/farms/{id}`). Both work without internet and are sent later from the phone's queue.
- Cell tap views: cell, crop plot, whole farm. The backend adds per-crop summaries to 2.3: `"crops": [{"crop","dunam","greenness_pct_of_normal","level"}]` and the whole-farm `greenness_pct_of_normal` (already there).
- Labels: new screens are English for now; Sorani comes later, but the backend keeps returning both `ku` and `en`.

### 2.10 The Doctor's model (decided 2026-10-08)
- Provider: **Gemini** (Google AI Studio key `GEMINI_API_KEY`, default model `gemini-2.5-flash`), chosen for cost (about 5–10x cheaper per answer). The call is one swappable function (`farm_doctor/doctor.py`: `FARM_DOCTOR_PROVIDER=gemini|claude`, `FARM_DOCTOR_MODEL`), so a head-to-head test against Claude in Sorani (20 questions, scored by a native speaker) can be run before any real rollout. The rulebook, the JSON answer shape and the no-doses rule are identical for both.

### 2.11 Control Room: the Ministry runs the app from the dashboard (added 2026-10-08)

**Replaced by 2.12 where they differ (user, 2026-10-09):** staff sign in with email and password, roles are made by staff from permissions, no second-officer approvals, no audit page, no protected mode on the website.

Design: `design/dashboard/jutyar_dashboard.pen`, screens 11 to 18 (builder `design/dashboard/build_control_room.py`). Not built on the backend yet. Every path is under `/v1/admin`, with an **officer** token (not a farmer token).

**Privacy: Protected mode (user decision 2026-10-08).** Officers see a farmer's phone as `+964 750 ••• 4567` and a farm only at 1 km (its sub-district and a 1 km rounded centre), **unless** that farm has an open report or a Doctor case with `refer_to_officer: true`. Then the exact outline, cells and the shared point open for officers of that area, and close again when the report or case is closed. Seeing the full phone needs a typed reason. Every look and every change is written to the history, which nobody can edit or delete (kept 5 years).

**Roles.** `viewer`: totals and maps only, no farms, no phones. `district_officer`: their own governorates or districts: farms (protected), inbox, draft alerts, show a phone with a reason. `admin`: everything, plus rules, prices and officers. Sending an alert and changing a rule always need a **second officer** to approve. An officer without 2-step sign-in cannot open farms.

| Screen | Calls |
|---|---|
| Officer sign-in | `POST /v1/admin/auth/otp/send`, `/verify` (phone must be on the officer list) + second step (TOTP) → officer token with `role` and `areas` |
| 11 Farmers and farms | `GET /admin/farms?gov=&district=&sub=&crop=&level=&synced_since=&q=&page=` → `{"total", "farms": [{"id", "name", "phone_masked", "district", "sub_district", "area_dunam", "main_crop", "level", "last_sync", "access": "1km|open|blocked", "open_reason": {"report_id"|"case_id"}}]}`. `GET /admin/farms/{id}`: the outline and cells only when `access` is `open`, otherwise `centre_1km`. `POST /admin/farmers/{phone_id}/reveal {"reason"}` → the full phone. `POST /admin/farmers/{id}/block`, `/unblock`, `DELETE /admin/farmers/{id}` (on the farmer's request, same effect as `DELETE /v1/account`). Officers never change a farm's outline or crops. |
| 12 Crop register | `GET /admin/crops?by=district|sub_district&gov=` → per unit `{"unit", "farms", "dunam", "crops": [{"crop", "dunam"}], "week_change_dunam"}`, from painted cells (inside areas, 0.2). Registered farms only, not a census. |
| 13 Send an alert | `POST /admin/alerts` (draft) `{"area": {"districts": [], "sub_districts": []} or {"polygon"}, "type", "day", "level", "confidence", "ku", "en", "action_ku", "action_en", "crops": []}`. `GET /admin/alerts/{id}/reach` → `{"farms", "farmers", "already_pushed_today", "no_push_token"}`. `POST /admin/alerts/{id}/approve` (a second officer) sends it within the push rules of 2.7: one push per farm per day, so farms already pushed get it at 06:00 the next day. `POST /admin/alerts/{id}/stop`. |
| 14 Inbox | `GET /admin/inbox?kind=report|case&state=&area=` (reports of 2.6 and Doctor cases with `refer_to_officer`), `POST /admin/inbox/{id}/assign {"officer_id"}`, `/seen` (the farmer's report becomes `seen_by_officer`), `/reply {"ku", "en"}` (shown to the farmer in the app), `/close` (the farm goes back to protected). |
| 15 Rules | `GET /admin/rules` (every threshold with its value, source, version), `POST /admin/rules/{code}/changes {"value", "reason"}`, `POST /admin/rule-changes/{id}/approve` or `/reject`. The jobs read the rule values from here; the old value is kept. Today's values: `farm_doctor/weather_planner.py` and the dryness bands in `backend/src/features/zones/domain/enums.rs`. |
| 16 Alwa control | `PUT /admin/alwa/markets/{slug}/prices/{crop}/{day} {"price_iqd_per_kg", "fixed"}` then `POST /admin/alwa/prices/publish`; `POST /admin/alwa/listings/{id}/pause`, `DELETE /admin/alwa/listings/{id}`; `GET /admin/alwa/flags` (price far from the market price, repeats, disputes). |
| 17 Data health | `GET /admin/jobs` → per job `{"name", "every", "last_run", "ok", "summary", "last_14_days": ["ok"|"late"|"failed"|null]}`, `POST /admin/jobs/{name}/run`; `GET /admin/server` (requests, 5xx, p95, database); `GET`/`POST`/`DELETE /admin/service-keys`; `GET /admin/config` shows a warning while `AUTH__FIXED_SIGN_IN_CODE` is set. App versions need the app to send `X-App-Version` on every call (frontend will add it). |
| 18 Officers and history | `GET`/`POST /admin/officers`, `PUT /admin/officers/{id} {"role", "areas"}`, `DELETE /admin/officers/{id}`; `GET /admin/audit?kind=&officer=&from=&to=` (read only, exportable as CSV). |

New tables: `officers (id, phone, name, role, areas, totp_secret, created_at, disabled_at)`, `audit_log (id, at, officer_id or job, action, target_kind, target_id, reason, before_json, after_json)` (insert only), `alerts (id, draft_by, approved_by, area_json, type, day, level, confidence, texts_json, state, sent_at)`, `rule_values (code, value_json, version, source, changed_by, approved_by, at)`, `inbox_actions (item_id, officer_id, action, note, at)`, `farmer_blocks (farmer_id, by, reason, at)`.

- Edit until `PUT /v1/farms/{id}` exists (decided 2026-10-09, user option A): the app saves an edited farm as `POST /v1/farms` (new id, same name, new outline and crops) and then `DELETE /v1/farms/{old id}`. The new farm gets the full 20-year analysis again; the old id disappears. When the backend adds PUT, the app switches back to one call.

### 2.12 The website: what the backend must add (for Arya, 2026-10-09)

**What the website is.** One site in two parts: a public **View** page (map, news bar, dams, fires, compare years, Alwa prices, no login) and the **Admin** part for staff (sign in with `/v1/dashboard/auth/login`, then roles and permissions decide what each person sees). It is designed first in `design/web/jutyar_website.pen`, then coded in `web/control_room/`. **The site keeps no data of its own**: every number comes from the server, and nothing is sample data anymore. Kurdish (Sorani) first, English second.

Decisions (user, 2026-10-09): no Texts page; no approvals; many roles made by staff (the existing `roles` and `staff` slices are right); the news bar and the alerts list are built by the site from routes that already exist; the Doctor is being worked on separately.

The list is in the order the site needs it. Each new resource is also a permission resource (`<resource>:<action>`) added to `GET /v1/dashboard/permissions`; the `Owner` role gets it automatically.

#### A. Needed before the site can run

**A1. CORS.** The site will be hosted on its own address, so the API must allow it. Today `OPTIONS` answers `405` and no `Access-Control-*` header is sent.
- Allowed origins from a setting, for example `HTTP__CORS_ORIGINS=https://jutyar.example,http://127.0.0.1:5173` (comma list; no `*`, because requests carry a token).
- Methods `GET, POST, PUT, DELETE, OPTIONS`; request headers `Authorization, Content-Type, If-None-Match, Idempotency-Key`; exposed headers `ETag, X-Api-Version`; preflight `204` with `Access-Control-Max-Age: 600`.
- In axum: `tower-http` feature `cors`, a `CorsLayer` on the router.

**A2. Cache versions:** section 2.13.

**A3. A place on every farm.** Farms have only `centroid`, so reports by governorate and district cannot be made.
- Add `governorate`, `zone_slug`, `sub_zone_slug` to farms, computed from the centroid when a farm is created or edited (point in the 72 sub-district shapes already in the database); `null` if outside.
- Return them in `FarmSummaryResponse`, `DashboardFarmSummaryResponse` and `DashboardFarmResponse`.
- Filters on `GET /v1/dashboard/farms`: `governorate`, `zone`, `sub_zone`, `crop`, `q` (farm name or owner phone), next to `owner_phone`, `page`, `rows_per_page`; and `sort=created_at|area_dunam|name`, `order=asc|desc`.

**A4. Totals for reports and charts.** The site must not download every farm to add them up.
- `GET /v1/dashboard/stats/farms?governorate=&zone=&crop=` (`farms:read`) answers `{"as_of", "totals": {"farmers", "farms", "dunam"}, "by_governorate": [...], "by_zone": [...], "by_sub_zone": [...], "by_crop": [{"crop", "dunam", "farms", "farmers"}]}`. Each `by_...` area row is `{"slug", "name_en", "name_ku", "farmers", "farms", "dunam", "crops": [{"crop", "dunam", "farms"}]}`. A farmer is counted once per area where they have a farm.
- `GET /v1/stats/farms` (no login, for the View page): only `totals`, `by_governorate`, `by_zone`, `by_crop`; no names, no phones. Switched off by `public_farm_totals: false` (B4).

**A5. Farmer details for the support letter.** The letter names the farmer and their place.
- Add to farmers: `gender` (`male|female|null`), `birth_year` (number or null), `village` (text or null), `governorate`, `zone_slug`, `sub_zone_slug` (home place, may be null), `notes` (staff only), `blocked` (bool).
- `PUT /v1/dashboard/farmers/{id}` accepts them. `GET /v1/dashboard/farmers` filters `q` (name or phone), `governorate`, `zone`, `blocked`, with `page`, `rows_per_page`, `sort`.
- `blocked: true` signs the farmer out at once and refuses sign-in: `403 {"error": "blocked"}` on `/v1/auth/otp/verify`.
- `POST /v1/dashboard/farmers/{id}/letters` (`farmers:read`) with `{"purpose", "lang": "ku|en"}` answers everything the letter prints in one call: the farmer, their farms with place and crops, totals, and a stored letter number `JTY-<yyyymm>-<farmer id>-<n>` (table `letters (id, number, farmer_id, staff_id, purpose, lang, created_at)`). `GET /v1/dashboard/letters/{number}` checks a letter later.

**A6. Staff details.** Add `phone` and `job_title` to staff (`POST` and `PUT /v1/dashboard/staff`), and `PUT /v1/dashboard/me` so a person changes their own name, phone and password (`{"name", "phone", "current_password", "new_password"}`). Forgot password needs an email sender: later. Until then an Owner resets passwords (already built).

#### B. New resources

**B1. Crops** (resource `crops`). Today the crop list is a fixed enum; staff need to add and rename crops.
- Table `crops (code primary key, name_en, name_ku, color, category, season, yield_kg_per_dunam, active, sort_order, created_at, updated_at)`. `category`: `cereal|vegetable|fruit|legume|oil|fodder|other`. `season`: `winter|summer|perennial`. Seeded with today's codes.
- `GET /v1/crops` (no login: the app and the View page read names and colours here); `GET` and `POST /v1/dashboard/crops`; `PUT` and `DELETE /v1/dashboard/crops/{code}`.
- Farms, cells and Alwa listings check crop codes against this table (only active crops for new data). Deleting a crop in use answers `409 {"error": "crop_in_use"}`; the site then offers to switch it off (`active: false`).
- `code` matches `^[a-z_]{2,24}$` and never changes.

**B2. Rules** (resource `rules`). Every number the jobs use to decide a warning or a colour, changeable without a release.
- Tables `rules (code primary key, grp, name_en, name_ku, meaning_en, meaning_ku, value, unit, min_value, max_value, default_value, used_by, updated_by, updated_at)` and `rule_changes (id, code, old_value, new_value, reason, staff_id, at)` (insert only).
- Seeded from today's numbers: weather planner in `farm_doctor/weather_planner.py` (frost 0 °C, hard frost -2 °C, heat 31 °C, heavy rain 12 mm, sowing rain 20 mm in 3 days, rust weather 24 h, spray window 6 h, sunn pest 84 degree-days, dust PM10 150); dryness bands in `backend/src/features/zones/domain/enums.rs`; Field Eye (watch below 85% of normal greenness, alarm below 70%, skip pictures over 30% cloud).
- `GET /v1/dashboard/rules`; `PUT /v1/dashboard/rules/{code}` with `{"value", "reason"}` (reason required, 3 to 500 characters; outside min and max answers `422 bad_range`); `GET /v1/dashboard/rules/{code}/history`; `POST /v1/dashboard/rules/{code}/reset` (back to `default_value`, also logged).
- The jobs and the band calculation read the values from this table (once per run). `used_by` is `weather_planner`, `dryness` or `field_eye`, so the site can say where each number applies.

**B3. Inbox: messages from farmers** (resource `messages`).
- Farmer app: `POST /v1/messages` with `{"kind": "question|report|complaint|request|other", "text", "farm_id?"}` and photos like Ask the Doctor (multipart, 0 to 4, JPEG or PNG, 4 MB each); `GET /v1/messages/mine` with the replies.
- Dashboard: `GET /v1/dashboard/messages?state=&kind=&governorate=&zone=&q=&page=&rows_per_page=` (newest first, with the farmer's name, phone, farm name and place); `GET /v1/dashboard/messages/{id}` (with photo URLs); `PUT /v1/dashboard/messages/{id}` with `{"state": "new|read|replied|closed"}`; `POST /v1/dashboard/messages/{id}/reply` with `{"text_ku", "text_en?"}` (state becomes `replied`, keeps `replied_by` and `replied_at`; the farmer sees it in the app).
- `GET /v1/dashboard/messages/counts` answers `{"new", "read", "replied", "closed"}` for the menu badge.
- Later, Doctor cases with `refer_to_officer: true` land here as `kind: "doctor"`, once cases are stored.

**B4. App control** (resource `app`). What the farmer app reads at start, set from the site.
- One row `app_config`: `latest_version`, `min_version`, `update_message_ku`, `update_message_en`, `maintenance` (bool), `maintenance_message_ku`, `maintenance_message_en`, `maintenance_from`, `maintenance_until` (UTC or null), `announcement_on`, `announcement_ku`, `announcement_en`, `features` (switches `add_farm`, `walk_mode`, `satellite`, `doctor`, `reports`, `alwa`, `plan`, `push`), `limits` (`farms_per_phone`, `max_farm_dunam`, `min_corners`, `max_corners`, `gps_meters`), `help_phone`, `public_farm_totals` (bool, used by A4).
- `GET /v1/app/config` (no login; the app reads it at start, cached by version); `GET` and `PUT /v1/dashboard/app/config`.
- The app sends `X-App-Version: 1.0.3` on every call. The server keeps `app_versions_seen (farmer_id, version, last_seen)`; `GET /v1/dashboard/app/versions` answers `[{"version", "farmers", "share"}]` for the last 30 days. A version below `min_version` answers `426 {"error": "update_required"}`.

**B5. Data job status** (resource `jobs`, read only). Whether the automatic jobs ran on time.
- Table `job_runs (id, job, started_at, finished_at, ok, rows, message)`. Each job reports itself with the service key: `PUT /v1/ingest/jobs/{job}/runs/{started_at}` with `{"finished_at", "ok", "rows", "message"}`.
- Table `jobs (job primary key, name_en, name_ku, every_hours)`: `dryness` 12, `fires` 3, `groundwater` 24, `dams` 24, `briefs` 24, `farm_analysis` on demand.
- `GET /v1/dashboard/jobs` answers per job `{"job", "name_en", "name_ku", "every_hours", "last_run", "last_ok", "next_due", "state": "ok|late|failed|never", "last_14_days": ["ok"|"late"|"failed"|null], "message"}`. `late` means no finished run within `every_hours` × 1.5. No run button and no keys on the site.

#### C. What the site builds itself (no new route)

- **News bar:** from `GET /v1/fires?hours=24`, `GET /v1/dams`, `GET /v1/region/overview` (driest districts), `GET /v1/briefs/latest?scope=region` (headline) and the Alwa price board. It appears only once this data is in the cache.
- **Alerts:** a read-only list of fire detections, dryness band changes and the brief's `watch` and `alarm` points, from the same routes. Sending messages to farmers waits for push (2.7).
- **Crop register report, government report, support letter:** from A4 and A5, printed by the browser.

#### D. Permissions after this section

Today: `zones, dams, outlooks, water, fires, alwa, farmers, farms, insights, briefs, staff, roles`. New: `crops, rules, messages, app, jobs`. The site hides a menu item when the person has no `read` on its resource and hides each button whose action they do not hold. The server still checks every call.

### 2.13 Caching and cache invalidation (frontend and backend together)

Goal: the site opens from its cache at once, shows skeletons only for what is missing, and downloads a topic again **only when the server says it changed**.

**Backend**
1. Table `data_versions (topic primary key, version bigint not null, changed_at timestamptz)`. Topics: `zones, sub_zones, dams, fires, outlooks, water, alwa_prices, alwa_listings, farmers, farms, crops, rules, briefs, messages, app_config, jobs, staff_roles`.
2. Every write to a topic (dashboard, ingest, farmer app) runs `UPDATE data_versions SET version = version + 1, changed_at = now() WHERE topic = $1` **in the same transaction** as the write. A job that writes many rows bumps once, at the end of its run.
3. `GET /v1/versions` (no login) answers `{"api": "1.4.0", "versions": {"zones": 41, "dams": 7, "fires": 1290}, "server_time"}` with the public topics only. `GET /v1/dashboard/versions` (any staff) adds the private ones (`farmers, farms, messages, jobs, staff_roles`).
4. Every `GET` answers `ETag: W/"<topic>-<version>-<short hash of path and query>"` and `Cache-Control: no-cache` (staff routes add `private`). A request with a matching `If-None-Match` answers `304` with no body. A route that reads several topics uses the highest of their versions.
5. Header `X-Api-Version` on every answer; change it whenever an answer's shape changes.

**Frontend**
1. On start: draw the page at once from the cache in IndexedDB, then call `/v1/versions` once (and `/v1/dashboard/versions` when signed in).
2. For each topic whose server version is higher than the cached one: refetch only that topic's routes that are on screen or cached, with `If-None-Match`; a `304` keeps the cached copy. Topics that did not change are not called.
3. While the site is open: ask `/v1/versions` again every 60 s and whenever the tab comes back into view, so a new fire run shows within a minute.
4. After the site itself writes (for example edits a farmer), it refetches that topic at once.
5. The whole cache is wiped when `X-Api-Version` changes or when the site's own `CACHE_SCHEMA` number changes (bumped by the frontend when it changes how it stores data). Private topics are wiped on sign-out and are stored per staff id, so two people on one computer never see each other's data.
6. Until `/v1/versions` exists: time limits per topic (fires 10 min; Alwa 15 min; zones, dams, briefs 1 h; crops, rules, app config 6 h; staff lists 5 min), plus `ETag` when the server sends one.

**Loading (user decision 2026-10-09):** the branded intro (the Grain Sun drawing itself, 00 to 100) plays only on the first visit, when the cache is empty, and its counter follows the real calls. Later visits open straight from the cache; anything not cached yet shows a skeleton shaped like its content. The news bar stays hidden until its data is cached.

### 2.14 Alwa market in the app: simple listings (user decision, 2026-10-09)

The farmer app uses a simpler Alwa than the one built (FRONTEND.md 5). A listing is only: crop, quantity, asking price, where it is (the phone's GPS), the seller's phone, when it was posted and when it closes. Buyers call the seller directly. There are no offers in the app.

Keep as built: the 16 crop codes, `quantity_kg`, `asking_price_iqd_per_kg`, `closes_at` (at most 14 days; the app sends 14 days unless the farmer picks fewer), at most 20 open listings per phone (`too_many_listings`), cancel with `DELETE /v1/alwa/listings/{id}`, the price board (`/v1/alwa/markets/{slug}/prices`) with its empty state.

What the app needs changed or added:

| # | Change | Why |
|---|---|---|
| 1 | `POST /v1/alwa/listings` takes `lat` and `lon` (the phone's GPS, WGS84). `market`, `pickup` and `grade` become optional; the app does not send them. The backend may fill `zone_slug` from the point. | No market or area picker: the place is where the farmer stands. |
| 2 | Every listing answer (`GET /v1/alwa/listings`, `/{id}`, `/mine`) carries `lat`, `lon` and `seller_phone`, visible to anyone signed in. | User decision: farmers sign in with their phone, so phones are shown from the start. |
| 3 | `GET /v1/alwa/listings?lat=&lon=&crop=` sorts open listings by distance from that point and adds `distance_km` to each. | Buyers see the nearest first. |
| 4 | `POST /v1/alwa/listings/{id}/sold` (seller only): marks the listing sold without an offer. | Today a listing is sold only by accepting an offer; the app has no offers. |
| 5 | Each market (`AlwaMarketResponse`) carries `lat` and `lon`. | The app shows the price board of the market nearest the phone. |

Not used by the app: the offer and accept routes, `/v1/alwa/offers/mine`, `/v1/alwa/deals`, `grade`, `pickup`, `buyer_kind`. They can stay for the website.

- What the app does now (2026-10-09 18:00), until rows 1 to 3 are built: it sends `market` (the first market) and `pickup: "farm"` so the post is not refused, sends `quantity_kg` and the price as whole numbers, and puts the seller's sign-in phone in `seller_name` (any text up to 60 characters, already shown to buyers). It reads `seller_phone` first, else a phone-shaped `seller_name`. List rows have no `created_at` today, so the posted date is left out there. "Call the seller" opens the phone app with the number typed in. When `seller_phone` exists, the app stops writing the phone into `seller_name`.

### 2.15 Ask the Doctor on the test server: Codex reads the photos and the farm's data (user decision, 2026-10-09)

The app's centre button (Ask the Doctor) already sends `POST /v1/farms/{id}/ask` exactly as FRONTEND.md 6 says and already shows the `200` answer. On the test server every question answers `502 doctor_failed` because nothing runs behind the route. To make the button work, the server must do this for every question:

1. **Gather the farm's data** (about 15 s, all at once): the farm (centre, exact area, crops, outline, the tapped cell if any), what `/insights` knows about it, the newest Sentinel-2 reading of the field (Field Eye), the 10-day forecast with the Weather Planner rules (2.4), the season so far (Season Check) and the dam levels (Dam Watch). All of this is fetched by the server; the app calls no outside service.
2. **Ask Codex**, the `codex exec` program that is already signed in on the test server for the nightly brief: the question, `lang`, every photo attached as an image (`codex exec --image <file>`), the gathered data, and the Doctor's rulebook (`RULEBOOK` in `farm_doctor/doctor.py`: answer only in the JSON below, never a pesticide or fertilizer dose, say `unsure` and refer to an officer when unsure). No Gemini key is needed: this replaces Gemini (2.10) on the test server.
3. **Check the answer and return it** in the shape the app reads (FRONTEND.md 6): `{"likely", "confidence": "sure|likely|unsure", "why": [...], "actions_this_week": [...] (at most 3), "cannot_tell": [...], "refer_to_officer", "ku", "en", "inputs_used": [...]}`. Anything that is not this shape becomes `502 doctor_failed`, as today.
4. **Answer within 90 s.** The app waits 120 s. Two farmers asking at the same time must both get an answer (run Codex calls side by side or one after the other, both fit in 90 s for the demo).

The fastest way, already tested on a Mac: run `farm_doctor/doctor_service.py` next to the backend on the test server (`DOCTOR_URL=http://127.0.0.1:8090`). It already does steps 1 and 3 and today calls Gemini or Claude in step 2 (`FARM_DOCTOR_PROVIDER`). A `codex` provider that runs `codex exec` with the photos is the only missing piece; it lives in `farm_doctor/` and the app side can add it. The other way is for the backend to call `codex exec` itself, as `backend/jobs/daily_brief.py` does.

## 3. Offline rules (frontend side, so the backend knows what to expect)
- The app collects points and painted cells with no internet and stores them locally. It POSTs the farm when online; `created_offline_at` carries the real time. Expect bursts of old farms.
- The app keeps the last farms list and, per farm, the last farm, `status` and `plan`. On opening it shows that copy at once, asks the server, and swaps in the fresh answer; if the server fails or there is no internet, the copy stays on screen with its date (since 2026-10-08 21:44). So every open still makes the normal calls. The backend sets `Cache-Control: max-age` honestly (status: 1 day; plan: 6 hours).
- Idempotency: the app sends `Idempotency-Key` headers on POSTs; repeat keys must not create duplicates. A repeated key returns the farm made the first time (the app retries uploads that lost their answer).
- Outbox (built 2026-10-08): `POST /farms` is first written to a file on the phone, then sent; with no internet it stays there and is retried every 30 seconds and when the app comes back to the front. The border being marked is also saved on the phone after every dot, and the sign-in token is kept, so the app opens and works in the field with no signal. "No internet" = the request never reached the server; any other 4xx answer (not 401, 408 or 429) removes the farm from the outbox (it will not succeed on retry). Since 2026-10-08 21:33 the same queue also holds edits (`PUT /v1/farms/{id}`) and deletes (`DELETE /v1/farms/{id}`); deletes go first, and a `404` on a delete counts as done.

## 4. Errors
Shape (FRONTEND.md section 5): `{"error": "<code>", "detail": "<English text>"}`, plus `field` or `retry_after_s` when they apply. What the app does:

| Status | Codes | The app |
|---|---|---|
| `400` | `bad_request` | shows an error with the code; a queued farm is dropped |
| `401` | `unauthorized` | signs the farmer out (back to the phone screen); a queued farm is kept |
| `401` | `bad_code` (sign-in only) | shows "Wrong code, try again"; nobody is signed out |
| `404` | `not_found` | the farm screen keeps its saved copy (with its date) if it has one, else shows "Could not load this farm"; a delete counts as done |
| `408`, `429`, `5xx` | `rate_limited` (with `retry_after_s`), `upstream_down` (with `source`), `server_error` | a queued farm is kept and sent again. The farm screen keeps its saved copy (with its date) if it has one, else shows "Could not load this farm". The plan falls back to the last saved plan, or "Weather forecast not available right now". |
| `422` | `invalid`, `bad_polygon`, `farm_too_large`, `too_many_farms` | a queued farm is dropped and the farmer is told |

The only code the app branches on today is `bad_code`; the others are shown as they are and will get Sorani messages later.

## 5. What already exists on the backend side (today)
| Call | Where | State |
|---|---|---|
| 2.1 sign in | `backend/` (Rust, axum, Postgres) | built; demo uses one fixed code; no SMS provider yet |
| 2.2 farms: list, create, open, repaint, delete | `backend/` | built; cells and crop areas still by the centre rule (0.2) |
| 2.2 edit a farm `PUT /v1/farms/{id}` | `backend/` | not built |
| per-farm insights `GET /v1/farms/{id}/insights` | `backend/` (commit 81faa15) | built (topics such as water, soil, rain). Not the same as 2.3: Home calls `/status` and does not read `/insights` yet. |
| profile `GET`/`PUT /v1/me` | `backend/` | built (not used by the app yet) |
| 2.3 status | `farm_doctor/field_eye.py` `measure(lon, lat, date)` | Python only: one 1 km square around a point, about 16 s; needs the per-cell version and a stored daily job. Decided 2026-10-08 (user, Arya): the Python AIs push results into `backend/` through `/v1/ingest` (guarded by `X-Service-Key`) and `backend/` serves them. The app-facing `GET /v1/farms/{id}/status` is not built yet. |
| 2.4 plan | `farm_doctor/weather_planner.py` `plan(lon, lat)` | Python only; decisions are English sentences, need the codes, `ku` and the alert list. Same route in through `/v1/ingest`; `GET /v1/farms/{id}/plan` not built yet. |
| season label | `farm_doctor/season_check.py` | Python only, 16 zones |
| 2.5 doctor | `backend/` route, `farm_doctor/` service | `POST /v1/farms/{id}/ask` built 2026-10-09: the backend passes the question, the farm and its insights to the local Doctor service at `DOCTOR_URL` and returns its checked answer. The service needs `GEMINI_API_KEY`. No cases stored yet. |
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
