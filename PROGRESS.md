# PROGRESS

Short tracker of what is done, in progress and next. Updated with every change and committed. The detailed log with reasons and numbers is `STATUS.md`.

Last update: 2026-10-08 17:21

## Done
- Scope: Farm Doctor, 5 AIs + one Claude doctor, no long-range forecasts (`Scope_and_Build_Plan_FINAL.md`)
- Research: 96,941-paper library, 13,793 PDFs, analysis reports in `reports/`
- Data feeds tested: 17 free keyless feeds (`reports/Data_Feeds_Tested.md`)
- Backend test harness `farm_doctor/`: Field Eye (Sentinel-2), Weather Planner, Season Check, Neighbour Watch, Dam Watch, Doctor; `build_now.py` writes `web/now.json` for all 16 zones
- Field boundaries from space: SAM tested, good on plains (`farm_doctor/field_boundaries/`)
- Web: time-machine app `web/index.html`; Ministry dashboard template `web/dashboard.html` (map-first, no data wired); sign-in mock `web/app_signin.html`; 50 logos `web/logos.html`
- Design in Pencil, one file `design/jutyar_app.pen`: page Sign in (phone, code, my farms), page Add farm (walk-and-tap corners, paint the 10 m grid with crop emojis, farm ready)
- Repo on GitHub with CLAUDE.md rules
- Scope file corrected to the Flutter app; BACKEND.md carries the alert rules and the screen decisions
- `BACKEND.md` v1: the frontend-to-backend contract (phone account, 10 m UTM cell grid, crop codes, endpoints for OTP, farms, status, plan, doctor, reports, push, region; section 7 = data flow and tables)

## In progress
- Farmer app design in Pencil, 4 jobs: (1) Farm page: Home + cell card DONE, awaiting review, (2) Ask / Reading / Answer, (3) More page: Report + Alerts, (4) Settings + Grain Sun logo
- Ministry dashboard: wire `web/now.json` into the template (paused until the template is settled)

## Decisions taken (2026-10-08 planning)
- Tab bar; Home = all farms stacked; cell tap = small card + cells / crops / farm toggle; colours + numbers; Ask = text + photos; own reports only; field edge = always walk; logo 36 Grain Sun; English placeholders on new screens

## Decisions pending (user)
- none

## Next
- Design the farm home screen (my field from space, this week's plan, ask the Doctor)
- Wire the dashboard to `now.json`
- Gemini API key (`GEMINI_API_KEY`) in `farm_doctor/.env` so the Doctor can answer

## Waiting on the backend
- Confirm or edit `BACKEND.md` sections 1 and 2; answer section 6

## Needs from the user
- Gemini API key from Google AI Studio (decided: Gemini is the Doctor)
- Sorani speaker to check the Doctor's text
- Organizer confirmation that pre-built work is allowed
