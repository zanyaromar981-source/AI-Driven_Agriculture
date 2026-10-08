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
