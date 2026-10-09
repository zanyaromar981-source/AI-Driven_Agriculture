# Farm Doctor — test harness (2026-10-08)

One field in, advice out. Every AI returns numbers and labels only; the Doctor (Claude) turns them into Sorani and English advice with provenance.

| File | AI | Source |
|---|---|---|
| `field_eye.py` | Field Eye | Sentinel-2 via Planetary Computer (keyless); greenness now vs own 3-year normal vs 5 km neighbours; weak-patch share and where |
| `weather_planner.py` | Weather Planner | Open-Meteo 10-day forecast + air quality + GloFAS river flow; rules: sowing rain, urea rain, spray windows, frost, heat, rust hours, sunn-pest degree-days, dust |
| `season_check.py` | Season Check | rain since 1 Oct vs normal (web/data.js), honest alarm lines, El Niño note, zone crop light |
| `neighbour_watch.py` + `reports.json` | Neighbour Watch | farmers' pinned reports (TEST DATA until the app collects real ones) |
| `dam_watch.py` | Dam Watch | latest measured Dukan / Darbandikhan areas |
| `doctor.py` | The Doctor | Gemini by default (Claude optional), plain HTTPS, rulebook, JSON answer: likely / confidence / why / actions / cannot_tell / refer / sorani / english |
| `run_case.py` | runner | runs everything for one field and date, logs to `cases/` |
| `doctor_service.py` | Doctor service | local web service for the backend (127.0.0.1:8090, `DOCTOR_PORT`): `POST /ask` with the farm, its 20-year history, question and photos; runs Field Eye, Weather Planner, Season Check and Dam Watch at once (about 15 s), asks the Doctor, answers in the app's shape; 503 `doctor_not_ready` without a key. Neighbour Watch is left out on purpose: its reports are test data |

Run: `python3 -I run_case.py <lon> <lat> <YYYY-MM-DD> ["question"] [photo.jpg ...]`
Keys: create `farm_doctor/.env` with `GEMINI_API_KEY=...` (Google AI Studio) and optionally `ANTHROPIC_API_KEY=...` (git-ignored, never printed). Settings: `FARM_DOCTOR_PROVIDER=gemini|claude` (default gemini), `FARM_DOCTOR_MODEL` (default gemini-2.5-flash or claude-sonnet-5-5).
Replay dates use the ERA5 archive as a stand-in for the forecast (historical forecasts are not free).
