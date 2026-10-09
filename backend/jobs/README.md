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

## fires_runner.py

Satellite fire detections inside the 33 districts, every 3 hours, with gas flares removed. Source: NASA FIRMS files for three VIIRS satellites and MODIS (the last 7 days, no key).

- **Inside a district:** tested against the district outlines in `district_shapes.json` (made from `web/map_demo/kri_map_data.js`). Detections outside every district are dropped.
- **Flares:** a flare burns at one spot day and night for weeks; a crop fire moves and is over in hours. The job remembers each spot of about 1 km and the days it was hot (`cache/fire_cells.json`, 30 days). A spot hot on 3 or more days, at night on at least 2 of them, counts as a flare, with the cells touching it. Its detections are not pushed. Every run logs how many were removed.
- **What this gets wrong:** a real fire that burns in one place for 3 days and nights is hidden from its third day; a new flare shows as a fire for its first two days; on the first run the memory holds only 7 days, so some flares get through.
- **Grouping:** detections within about 2 km on the same day are pushed as one fire, with the time of the latest detection and the number of farms within 5 km.
- **Other limits:** a detection reaches NASA's files about 3 hours after the satellite passed; a short fire between passes is never seen; burned area is not measured.

First dry run, 9 Oct 2026: 3,048 detections in the box, 1,063 inside the districts, 284 of those removed as flares, 128 fires in the last 48 hours, most in Makhmur, Sumel, Qushtapa, Shekhan and Erbil. Whether those 128 are all real fires has not been checked by a person.

```sh
python3 fires_runner.py --dry-run
```

## groundwater_runner.py

Once a day: for each farm, how wet the ground is around it, deep and shallow, against its own history. Source: NASA's weekly GRACE-DA maps of groundwater storage, root-zone soil moisture and surface soil moisture, as percentiles (50 is normal for the time of year; 10 means only 10% of past years were this dry). Pushed as the farm's `groundwater` topic with confidence `unsure`.

- **It is not a well depth and not measured at the farm.** One value covers a square of about 25 km, so every farm in that square gets the same number. It is a model fed with satellite gravity readings.
- **It cannot see local pumping.** A village whose wells are falling looks the same as its neighbours.
- **Nothing better exists for free.** No source gives groundwater depth at farm scale in Iraq.
- New farms get their value on the next daily run. NASA changes the maps once a week.

First read, 9 Oct 2026 (maps of 6 Oct), at the 33 district centres: groundwater between the 4th and 18th percentile everywhere (mean 11), while root-zone soil moisture sits between the 46th and 95th. So after a wet year the top of the soil is wet, but the model's deep storage is still low.

It needs one package that the other jobs do not:

```sh
python3 -m venv /opt/farm-doctor/venv && /opt/farm-doctor/venv/bin/pip install rasterio
/opt/farm-doctor/venv/bin/python groundwater_runner.py --dry-run
```

## Reporting runs: `report_run.py`

The dashboard's job status page (`GET /v1/dashboard/jobs`) shows what each job reported. A job does not report itself: the timer starts it through `report_run.py`, which tells the backend when the run started, runs the job, and tells the backend how it ended (ok when the job exited 0, the count from the job's `done: N ...` line, and its last line as the message).

```sh
python3 report_run.py dryness /usr/bin/python3 region_runner.py
```

Job names: `dryness` (region runner), `fires`, `groundwater`, `briefs`, `dams`, `farm_analysis`. If the backend cannot be reached the job still runs and keeps its own exit code. A job run by hand without the wrapper is not shown on the status page.

## dams_runner.py

Once a day: the water area of Dukan and Darbandikhan lakes, measured on Sentinel-2 images. It is the method tested in `evidence/past_seasons/backtest/dam_water_test.py`, ported without changing a threshold: Sentinel-2 L2A through Microsoft Planetary Computer (free, no key; its statistics service counts the pixels, no image is downloaded), one rectangle per lake read at 20 m, water where NDWI `(B03-B08)/(B03+B08)` is above 0.

- **Which passes:** the last 30 days, Dukan from relative orbit 135 only and Darbandikhan from orbit 92 only, when every tile reports under 20% cloud. A pass is pushed only when it is clear: cloud and cloud shadow under 0.5% of the rectangle and every piece at least 99.5% inside the swath. Every pass that is not clear is logged with its numbers.
- **Measured:** `lake_area_km2`.
- **Derived, and not what its name says:** `pct_full` is the lake area as a share of the full lake area (Dukan 270 km2, Darbandikhan 113 km2, the official full areas quoted in `evidence/past_seasons/BACKTEST_RESULTS.md` section 6d; an area just above them is sent as 100). It is **not** the stored volume as a share of capacity. A lake loses volume faster than area, so the figure reads too high when the lake is low: in June 2025 Dukan was reported at 24% of its volume while its area was 42% of full. Every reading's `source` says "pct_full = lake AREA / full N km2, not volume".
- **Not computed:** `volume_bn_m3` and `farm_supply_bn_m3` stay empty. No tested area-to-volume curve exists for these lakes: Global Water Watch and DAHITI hold none (`research_notes/Data_Sources_Reservoirs.md`), the Dukan curve tried in the backtest was off by 0.3 to 1 billion m3 and is not kept in the repo, and nothing was tried for Darbandikhan.
- **A stored day is left alone,** so a run repeats safely and a correction made by staff on the dashboard is not written over. The job reads the stored days from `GET /v1/dams/{slug}/history`.
- **Limits:** a lake is passed every 2 to 5 days and winter passes are often cloudy, so readings can be weeks apart. A pass reaches Planetary Computer hours to two days after it was flown. Haze moves a reading by a few km2.
- **When the satellite service does not answer:** each request is tried 4 times; a lake is left after 2 passes that could not be read. Nothing is pushed for what could not be measured, the last line counts the failures, the job exits 1, and the next day's run tries the same passes again.

`--backfill` pushes the tested history in `dam_history.csv`: the 111 clear rows of `dam_water_areas.csv` (2008 to 2026; Landsat 5 and 8 at 30 m for 2008, 2013 and 2019, Sentinel-2 from 2017), 110 readings because one day was measured by both satellites and the Sentinel-2 one is kept. It measures nothing new. Unlike the daily run it replaces what is stored for those days. Run it once on a new server:

```sh
INGEST__SERVICE_KEY=... FARM_DOCTOR_API=http://localhost:8790/v1 python3 dams_runner.py --backfill
python3 dams_runner.py --dry-run     # measures for real, pushes nothing
```

First run, 9 Oct 2026: 110 readings backfilled, then 13 passes found, 6 new clear ones pushed, 6 already in the history (the new measurements equal the backtest's to the last digit), 1 not clear. It took 83 seconds; a run with nothing new takes about 10. Newest: Dukan 249 km2 on 27 Sep 2026 (83 km2 a year before), Darbandikhan 68 km2 on 4 Oct 2026 (47 km2 a year before).

On the server it runs from `farm-doctor-dams-runner.timer` at 13:20 UTC. See its last run with `journalctl -u farm-doctor-dams-runner -n 50`.
