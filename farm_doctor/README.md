# Farm Doctor — test harness (2026-10-08)

One field in, advice out. Every AI returns numbers and labels only; the Doctor (Claude) turns them into Sorani and English advice with provenance.

| File | AI | Source |
|---|---|---|
| `field_eye.py` | Field Eye | Sentinel-2 via Planetary Computer (keyless); greenness now vs own 3-year normal vs 5 km neighbours; weak-patch share and where |
| `weather_planner.py` | Weather Planner | Open-Meteo 10-day forecast + air quality + GloFAS river flow; rules: sowing rain, urea rain, spray windows, frost, heat, rust hours, sunn-pest degree-days, dust |
| `season_check.py` | Season Check | rain since 1 Oct vs normal (web/data.js), honest alarm lines, El Niño note, zone crop light |
| `neighbour_watch.py` + `reports.json` | Neighbour Watch | farmers' pinned reports (TEST DATA until the app collects real ones) |
| `dam_watch.py` | Dam Watch | latest measured Dukan / Darbandikhan areas |
| `doctor.py` | The Doctor | Claude API (plain HTTPS), rulebook, JSON answer: likely / confidence / why / actions / cannot_tell / refer / sorani / english |
| `run_case.py` | runner | runs everything for one field and date, logs to `cases/` |

Run: `python3 -I run_case.py <lon> <lat> <YYYY-MM-DD> ["question"] [photo.jpg ...]`
Key: create `farm_doctor/.env` with one line `ANTHROPIC_API_KEY=sk-ant-...` (git-ignored, never printed). Model: `FARM_DOCTOR_MODEL` env (default claude-sonnet-5-5).
Replay dates use the ERA5 archive as a stand-in for the forecast (historical forecasts are not free).
