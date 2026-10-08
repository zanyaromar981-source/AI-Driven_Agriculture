# PROGRESS

Short tracker of what is done, in progress and next. Updated with every change and committed. The detailed log with reasons and numbers is `STATUS.md`.

Last update: 2026-10-09 02:16

## Done
- Scope: Farm Doctor, 5 AIs + one Claude doctor, no long-range forecasts (`Scope_and_Build_Plan_FINAL.md`)
- Research: 96,941-paper library, 13,793 PDFs, analysis reports in `reports/`
- Data feeds tested: 17 free keyless feeds (`reports/Data_Feeds_Tested.md`)
- Backend test harness `farm_doctor/`: Field Eye (Sentinel-2), Weather Planner, Season Check, Neighbour Watch, Dam Watch, Doctor; `build_now.py` writes `web/now.json` for all 16 zones
- Field boundaries from space: SAM tested, good on plains (`farm_doctor/field_boundaries/`)
- Web: time-machine app `web/index.html`; Ministry dashboard template `web/dashboard.html` (map-first, no data wired); sign-in mock `web/app_signin.html`; 50 logos `web/logos.html`
- Design in Pencil, one file `design/jutyar_app.pen`, 14 screens: Sign in (phone, code, my farms), Add farm (corners, paint the grid, farm ready), Farm (home with stacked farms, cell card, crop view, farm view, ask, reading, answer), More (report, alerts, settings); Grain Sun logo
- Control Room website in Pen, `design/web/jutyar_control_room.pen`: 48 frames measured from the live page (motion spec, opening sequence, 19 sections, 10 states), rebuilt by `design/web/build_web_pen.py`
- Dashboard design in Pencil, `design/dashboard/jutyar_dashboard.pen`, 23 frames: Ministry dashboard by district (33 KRG districts on real borders, the map is one reusable component also saved as `kri_map_component.pen`, latitude / longitude rulers, zoom into Chamchamal's real sub-districts), Water, The Doctor, Fire alerts, Compare years, Sorani version, farmer phone view, Alwa market (dashboard + phone), and the Control Room (8 screens, protected mode: farmers and farms, crop register, send an alert, inbox, rules, Alwa control, data health, officers and history; builder `build_control_room.py`). Sample numbers only
- Flutter app `app/` (Jutyar): Sign in (phone, code, my farms) and Add farm (walk the corners with GPS, paint crops on the 10 m grid, farm ready, save; map styles Satellite with Kurdish place names, Map, Terrain; walk mode; exact area; works offline with upload later) working on a fake server with BACKEND.md shapes; 8 tests; runs on the user's phone
- Flutter: each farm opens on its own screen; Edit (move border dots, live m², crops kept) and Delete farm, both work offline
- Flutter Home (open a farm): a tap in My farms opens Home with all farms stacked, scrolled to that farm. Per farm: Cells / Crops / Farm views drawn from the outline (works offline), cell card on tap, weak-cell line counted by area, This week from the live Open-Meteo forecast with the Weather Planner rules, last copy kept on the phone. Tab bar in place (only Home works). English labels for now
- App icon: Grain Sun on the cream tile, Android (adaptive) and iOS, replaces the default Flutter logo
- Pitch deck in Pencil, `design/pitch_deck.pen`, 8 slides, v2 after user review (cover, problem and gap, app design, app features, calculations, water map of Dukan and Darbandikhan on the team zone map with real numbers, live demo, summary), name Khor (خۆر), olive and ochre palette, photo-led
- Control Room web demo `web/admin_demo/` (not connected, sample data), in the Jutyar app style with the Grain Sun preloader and 16 load and interaction animations: 19 sections to run the whole app and database from the web (overview, approvals, farmers and farms, officers, crop register, region data, alerts, inbox, the Doctor, Alwa, rules, app control, notifications and SMS, texts, data jobs, database, security and privacy, history, system settings); roles, two-officer rule and protected mode work in the page
- Map demo `web/map_demo/` (Leaflet, real map tiles): 4 governorates, 33 KRG districts, 78 sub-districts from the CSO 2019 KML regrouped by the KRG maps of Duhok, Erbil and Halabja; 66 towns at exact OSM points; live lat/lon to 6 decimals, DMS, UTM 38S; click to pin with elevation and an Open in Google Maps link; latitude / longitude rulers and grid down to 1 second; search in English, Sorani, coordinates or OpenStreetMap; GPS locate
- Flutter app is ready for the real server: build with `--dart-define=API_URL=https://...` and it uses the server (BACKEND.md 2.0); without it, the demo server. Refused token signs out; no answer = offline (saved copies, upload queue keeps farms). Tested against a local test server
- BACKEND.md v2: section 0 lists exactly what the app still needs from `backend/` (checked against a0ade90 and FRONTEND.md v2), answers FRONTEND.md section 7; plain http tested and works on Android
- Repo on GitHub with CLAUDE.md rules
- Scope file corrected to the Flutter app; BACKEND.md carries the alert rules and the screen decisions
- `BACKEND.md` v1: the frontend-to-backend contract (phone account, 10 m UTM cell grid, crop codes, endpoints for OTP, farms, status, plan, doctor, reports, push, region; section 7 = data flow and tables)
- Backend in Rust + axum, `backend/` (clean architecture + vertical slices): slices `farmers` (sign in with phone and code, profile) and `farms` (list, create from walked corners and painted cells, get, repaint, delete, repeat-safe upload); answers in the BACKEND.md shapes so the app's `HttpApi` works unchanged; 138 tests, checked against a real Postgres; `FRONTEND.md` v3 says what is built. Also slices for the dashboard (`zones` with 33 districts, `dams`, `outlooks`, `water`, `fires`), the Alwa market, per-farm insights and farm status: 43 routes, 625 tests, critic pass done

## In progress
- Backend: hosted on a test server with a 12-hourly rain-against-normal job for the 33 districts; next are role-based dashboard routes, more data jobs (dams, fires, soil), touched cells with `inside_pct`, weekly plan
- Flutter app, one screen at a time: next is Ask the Doctor (the tab bar is in place)
- Review of the full farmer app design in Pencil (all 4 jobs done)
- Ministry dashboard: wire `web/now.json` into the template (paused until the template is settled)

## Decisions taken (2026-10-08 planning)
- Tab bar; Home = one farm per screen (changed 2026-10-08 evening, user); cell tap = small card + cells / crops / farm toggle; colours + numbers; Ask = text + photos; own reports only; field edge = always walk; logo 36 Grain Sun; English placeholders on new screens

## Decisions pending (user)
- App name under the icon: Jutyar (now) or Khor / خۆر (pitch deck)?
- Farm size still not accurate on the phone: which number, and the real size?
- Map source: user dislikes the current map (asked for Google, then Leaflet). Leaflet map demo built in `web/map_demo/` for review

## Next
- Flutter: Ask the Doctor, Report, Alerts, Settings (Home done; its Ask and Report buttons say "not built yet")
- Flutter: Home labels to Sorani after a native speaker check
- Flutter: set `kTestMode` to false before any release
- Pitch deck: swap the AI satellite picture on slide 5 for a real Sentinel-2 capture, add team names, refresh the lake numbers on demo morning
- Wire the dashboard to `now.json`
- Map demo: Sorani names for 5 sub-districts, Bamo sub-district shape, Erbil's Khabat / Bnaslawa / Barhka / Ainkawa district shapes
- Gemini API key (`GEMINI_API_KEY`) in `farm_doctor/.env` so the Doctor can answer

## Waiting on the backend (BACKEND.md section 0)
- Control Room (BACKEND.md 2.11): officer sign-in with roles and 2-step, `/v1/admin` routes for farms (protected mode), crop register, alerts with second-officer approval, inbox, rules, Alwa prices, jobs, officers, and an insert-only audit log
- `GET /v1/farms/{id}/status`: blocks Home today (every farm shows "Could not load this farm"); the stub in BACKEND.md 2.3 is enough for now
- `GET /v1/farms/{id}/plan` (Home opens without it)
- Farm cells with `inside_pct` and crop areas from them, so sizes match the app (not blocking)
- `PUT /v1/farms/{id}` to edit a farm's border, crops and name (needed by the edit screen in progress)

## Needs from the user
- Gemini API key from Google AI Studio (decided: Gemini is the Doctor)
- Sorani speaker to check the Doctor's text
- Organizer confirmation that pre-built work is allowed
