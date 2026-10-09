# Data jobs

Scripts that compute numbers and push them to the backend's `/v1/ingest` routes. The backend only stores and serves.

## region_runner.py

Rain against normal for each of the 33 districts, every 12 hours.

- **Measured:** `rain_pct_of_normal`, the rain of the last 365 days at the district's centre against the same window in the 10 years before. Source: Open-Meteo archive (ERA5 reanalysis, cells of about 9 to 25 km). It is a district-scale figure, not a field-scale one.
- **Derived, not measured:** `dryness` is the rain figure on the dashboard's 0 to 100 scale, `100 - rain% / 2`. Normal rain is 50. Soil moisture and greenness are not in it yet.
- **Not computed:** greenness, water need, nitrogen hold, best crops.

The first run fetches eleven years of daily rain, two districts a minute, so it takes about 20 minutes. Later runs fetch only the last 40 days and take seconds. The rain is kept in `cache/rain.json`.

Run by hand:

```sh
INGEST__SERVICE_KEY=... FARM_DOCTOR_API=http://localhost:8790/v1 python3 region_runner.py
```

On the server it runs from a systemd timer (`farm-doctor-region-runner.timer`) at 00:15 and 12:15 UTC. See its last run with `journalctl -u farm-doctor-region-runner -n 50`.

## daily_brief.py

Every night at midnight Baghdad time (21:00 UTC) an AI agent reads the day's stored values, searches the web a little, and writes a short brief for the whole region and for each district that has farms, in Sorani and English. The script pushes the briefs to the backend, and also tells it which district each farm lies in (the district whose centre is nearest: a rough rule, no boundaries are used).

- **The agent, for now:** the Codex command-line tool, run without a terminal (`codex --search exec ...`). It must be installed and signed in on the server (`codex login status`). Moving to an API later means changing one function, `run_agent`.
- **What it may use:** only the numbers the script gives it, plus what it finds online. Anything from the web must be listed under `sources` with the page it came from; if it finds nothing it has to say so.
- **What it may not do:** invent numbers, or give pesticide or fertiliser doses or product names.
- **What the script checks:** the answer's shape, the backend's length limits (long text is cut), that every source is a real `http` address, and that every district named exists. It does not check that the agent's sentences are true: a person should read the briefs before they are trusted.

Try it without asking the agent or pushing anything:

```sh
INGEST__SERVICE_KEY=... FARM_DOCTOR_API=http://localhost:8790/v1 python3 daily_brief.py --dry-run
```

On the server it runs from `farm-doctor-daily-brief.timer`. See its last run with `journalctl -u farm-doctor-daily-brief -n 50`.
