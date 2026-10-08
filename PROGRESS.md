# PROGRESS

Short tracker of what is done, in progress and next. Updated with every change and committed. The detailed log with reasons and numbers is `STATUS.md`.

Last update: 2026-10-08 16:53
Previous: 2026-10-08 16:48

## Done
- Scope: Farm Doctor, 5 AIs + one Claude doctor, no long-range forecasts (`Scope_and_Build_Plan_FINAL.md`)
- Research: 96,941-paper library, 13,793 PDFs, analysis reports in `reports/`
- Data feeds tested: 17 free keyless feeds (`reports/Data_Feeds_Tested.md`)
- Backend test harness `farm_doctor/`: Field Eye (Sentinel-2), Weather Planner, Season Check, Neighbour Watch, Dam Watch, Doctor; `build_now.py` writes `web/now.json` for all 16 zones
- Field boundaries from space: SAM tested, good on plains (`farm_doctor/field_boundaries/`)
- Web: time-machine app `web/index.html`; Ministry dashboard template `web/dashboard.html` (map-first, no data wired); sign-in mock `web/app_signin.html`; 50 logos `web/logos.html`
- Design in Pencil, one file `design/jutyar_app.pen`: page Sign in (phone, code, my farms), page Add farm (walk-and-tap corners, paint the 10 m grid with crop emojis, farm ready)
- Repo on GitHub with CLAUDE.md rules
- `BACKEND.md` v1: the frontend-to-backend contract (phone account, 10 m UTM cell grid, crop codes, endpoints for OTP, farms, status, plan, doctor, reports, push, region; section 7 = data flow and tables)

## In progress
- Farmer app design in Pencil (next screens after Add farm)
- Ministry dashboard: wire `web/now.json` into the template (paused until the template is settled)

## Decisions pending (user)
- Field edge: 1 suggest the satellite edge, 2 always walk the corners, 3 walk then tidy
- App-to-backend split: app collects GPS points and painted cells, backend builds the farm and attaches satellite values (proposed)
- Logo pick (top 10 on the logos page)

## Next
- Design the farm home screen (my field from space, this week's plan, ask the Doctor)
- Wire the dashboard to `now.json`
- Claude API key in `farm_doctor/.env` so the Doctor can answer

## Waiting on the backend
- Confirm or edit `BACKEND.md` sections 1 and 2; answer section 6

## Needs from the user
- Claude API key
- Sorani speaker to check the Doctor's text
- Organizer confirmation that pre-built work is allowed
