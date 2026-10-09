# Data jobs

Scripts that compute numbers and push them to the backend's `/v1/ingest` routes. The backend only stores and serves.

## region_runner.py

Rain and greenness against normal for each of the 33 districts, every 12 hours.

- **Measured:** `rain_pct_of_normal`, the rain of the last 365 days at the district's centre against the same window in the 10 years before. Source: Open-Meteo archive (ERA5 reanalysis, cells of about 9 to 25 km). It is a district-scale figure, not a field-scale one.
- **Derived, not measured:** `dryness` is the rain figure on the dashboard's 0 to 100 scale, `100 - rain% / 2`. Normal rain is 50. Soil moisture and greenness are not in it.
- **Measured:** `greenness_pct_vs_normal`, from satellite (see "Greenness" below). Empty when there is no usable picture.
- **Not computed:** water need, nitrogen hold, best crops. The backend only says that water need is "0 to 100, higher is more", with no rule for what a number means, so nothing honest can be put on that scale yet; and no data here can rank crops.

The first run fetches eleven years of daily rain, two districts a minute, so it takes about 20 minutes. Later runs fetch only the last 40 days and take seconds. The rain is kept in `cache/rain.json`.

Run by hand (`--dry-run` fetches and works out everything but pushes nothing, and needs no key):

```sh
INGEST__SERVICE_KEY=... FARM_DOCTOR_API=http://localhost:8790/v1 python3 region_runner.py
```

On the server it runs from a systemd timer (`farm-doctor-region-runner.timer`) at 00:15 and 12:15 UTC. See its last run with `journalctl -u farm-doctor-region-runner -n 50`.

### Past months: `--backfill-months N`

The dashboard compares a month with the same month one year earlier and in up to five earlier years, and it can only compare what is stored. A normal run stores the current month alone. Run once with `--backfill-months 36` and the job also pushes a reading for each of the 36 whole months before this one, each worked out exactly as today's is, as it stood on that month's last day: the 365 days ending then against the ten years before them.

- It needs older rain than the daily run keeps (eleven years before the oldest month), and fetches the missing years once: about 5 minutes on a server that already has `cache/rain.json`, about 40 minutes from nothing (one district a minute).
- A month is skipped, and logged, when fewer than 8 of its 10 earlier years have rain. The reading's `source` names the number of years really used.
- The weather archive allows about 10,000 weighted calls a day from one address, and 33 districts times fourteen years is more than that. If it refuses, the job stops with the archive's own words; run it again later and it carries on from the cache.
- Running it again pushes the same numbers again (the route is an upsert), so it is safe to repeat. The timer never backfills.

### Greenness (`ndvi.py`)

- **Source:** MODIS NDVI, product MOD13Q1 (Terra, 250 m pixels, one picture per 16 days), through the free ORNL DAAC subset service (`modis.ornl.gov/rst/api/v1`, no key).
- **Where:** a square of about 19 km around the district's centre, keeping only the pixels inside the district's outline (`district_shapes.json`). Not the whole district.
- **Which land:** all of it. Fields, rangeland, forest and towns in the square all count. **Cropland cannot be told from rangeland here**, so this is how green the land looks, not how the crops are doing.
- **Water:** pixels that read as open water (NDVI below zero) are dropped. Without that, a lake fuller than in other years showed as land that lost its green: Ranya's square is half Dukan lake and read 27% below normal after a wet year. Some of that is left: the shore that is under water this year is missing from this year's mean and present in the dry years'.
- **Clouds and snow:** pixels the product marks as cloud, snow or ice (pixel reliability 2 or 3) or leaves empty are dropped. A picture with fewer than half of its pixels left is not used, and the reading then has no greenness. In a cloudy or snowy winter the pixels that remain may be the lower, warmer ones.
- **The figure:** mean NDVI of the picture against the mean of the same 16 days in each of the 10 years before, in percent (`0` is normal, `-20` is a fifth less green, as the backend defines it). At least 5 of the 10 years must be usable; the `source` names the number used and the day the picture starts.
- **Which picture:** the latest one with half of its 16 days before the reading's day. A picture reaches the service two to four weeks after its 16 days began, so today's reading shows the land of two to six weeks ago. Older than 48 days and it is not used.
- **Cost:** the service takes 3 to 8 seconds a call, one call at a time (four at once were slower in total). The newest picture with its ten earlier years is 22 calls a district: about 40 minutes for the 33 districts the first time. After that a run costs one call, plus 66 calls (about 8 minutes) once every 16 days when a new picture arrives, plus the earlier years of that picture if they are not cached yet. Everything fetched is kept in `cache/ndvi.json` and never asked for again.
- **Time limit:** the job stops asking after `GREENNESS_MINUTES` (20 by default) and the next run carries on, so on a fresh server the greenness fills in over the first few runs. Readings go out without greenness until then. A service that is down never stops the rain.
- **Past months:** with `--backfill-months` the job also wants the pictures of those months and their ten earlier years, about 13 years of pictures: some 2,000 calls. Measured on 9 Oct 2026: about 12 minutes a district, so 6 to 7 hours for all 33, and that evening the service stopped answering for hours. Do it once by hand with a long limit (below); it can be stopped and started again, and months without greenness are filled by running the backfill again.

Fill the cache alone, without the backend, and print each district's NDVI:

```sh
GREENNESS_MINUTES=480 python3 ndvi.py --months 36    # all districts, today and 36 months back
python3 ndvi.py sulaymaniyah                         # one district, today
```

The `source` of a reading is one of:

```
Open-Meteo ERA5, district centre: 365-day rain vs 10-yr normal; dryness=100-rain%/2; to 2026-10-08
ERA5 rain 365d to 2026-10-08 vs 10y; dryness=100-rain%/2; MODIS NDVI all land from 2026-09-14 vs 10y
```

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
- **Grouping:** detections within about 2 km on the same day are pushed as one fire, with the time of the latest detection.
- **Wind:** `wind_kmh` and `wind_direction` are the wind 10 m above the ground at the fire's place in the hour of its latest detection, from Open-Meteo's weather model (one call per 50 fires). It is a model value for a cell of several km, not a measurement at the fire. The direction is where the wind blows **towards** (the way the fire is driven), so a wind from the north-east is pushed as `sw`. Kept in `cache/fire_wind.json`, so a fire is asked about once; if the weather service does not answer, the fire is pushed without wind.
- **Farms nearby:** `farms_within_5km` is the number of farms whose centre lies within 5 km of the fire, from `GET /v1/ingest/farms`. `0` means the farms were read and none is near; empty means they could not be read.
- **Sent empty:** `area_ha` (a detection is a hot pixel, not an outline, and no area is made up from the pixel size) and `farmers_alerted` (nobody is alerted yet).
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

Job names: `dryness` (region runner), `fires`, `groundwater`, `briefs`, `dams`, `farm_analysis`. The job `plans` (`farm_plan.py`) is the one exception: it reports itself, see its section. If the backend cannot be reached the job still runs and keeps its own exit code. A job run by hand without the wrapper is not shown on the status page.

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
## farm_history.py

Every 10 minutes: ten years of monthly values for every farm, pushed to `PUT /v1/ingest/farms/{id}/history/{metric}` and read by the app from `GET /v1/farms/{id}/history`.

One run asks the backend what each farm already has (`GET /v1/ingest/farms/history/coverage`) and fetches only what is missing:

- **A new farm** (a metric with nothing stored): the last 120 full months.
- **Once a day**, on the first run after 02:00 UTC: every farm from its last stored month to last month, at least the last two months again, because the sources revise their newest values. Months not sent stay as they are.
- **Nothing missing:** one call to the backend, no call to any outside service.

| Metric | Unit | Source | Cell | A month is |
|---|---|---|---|---|
| `rain_mm` | mm | ERA5 reanalysis, through Open-Meteo | about 25 km | the total of the days |
| `et0_mm` | mm | FAO-56 reference evapotranspiration, worked out by Open-Meteo from ERA5 | about 25 km | the total of the days |
| `temp_max_c`, `temp_min_c` | °C | ERA5-Land reanalysis, through Open-Meteo | about 9 km | the mean of each day's highest or lowest |
| `soil_moisture` | m3/m3 | ERA5-Land, top 7 cm of soil (`soil_moisture_0_to_7cm_mean`) | about 9 km | the mean of the days |
| `greenness` | NDVI | NASA MODIS Terra, MOD13Q1, through the ORNL DAAC subset service | one 250 m pixel | the mean of its usable 16-day values |
| `groundwater_pct` | percentile | NASA GRACE-DA weekly maps | about 25 km | the mean of its weekly maps |

Read this before trusting a number:

- **A farm is smaller than every one of these cells.** Two farms a few kilometres apart get the same weather and the same groundwater figure. Greenness is the only one near field scale, and a farm under about 6 hectares still shares its pixel with its neighbours. Only the pixel at the farm's centre is read, not the whole outline.
- **The weather is a model, not a gauge.** ERA5 is a reanalysis: the weather of the past as a model rebuilds it from observations. In the mountains its rain can be well off a local station. The job asks Open-Meteo for `models=era5_seamless` by name, so the ten years come from one model; without it the archive switches to a forecast model from 2017 on, and the series would jump. Open-Meteo's `era5_land` alone has no rain and no evapotranspiration, which is why those two come from ERA5 at 25 km.
- **A month with too little data is not stored.** A month of daily values needs at least 90% of its days; a total over a month with a few days missing is scaled up to the whole month (mean day times days in the month). The reanalysis is about five days behind, so the month just ended can be made from 28 days at first and is replaced by the daily refresh.
- **Greenness:** a 16-day value is dropped when its pixel reliability says snow or ice (2), cloud (3) or no data (-1); good (0) and marginal (1) are kept. A 16-day period belongs to the month its middle day falls in. A month with no usable value is simply absent, so winter months can be missing.
- **Groundwater is not a well depth.** It is a percentile against the same time of year in 1948 to 2012 (50 is normal), from a model fed with satellite gravity readings. It cannot see local pumping. A month needs at least 3 of its 4 or 5 weekly maps.
- **Year figures and the normal** are made by the backend from the stored months, not by this job.

How long it takes, measured on 9 Oct 2026 for two farms near Sulaymaniyah: weather is one call and about 2 seconds per farm. Greenness for a new farm is 47 calls to ORNL (10 dates a call is the service's limit, two bands, one call for the list of dates); the service answered in 5 to 40 seconds a call, so one farm took 6 minutes and the other 20. Groundwater reads NASA's weekly archive folders (`nasagrace.unl.edu/globaldata/YYYYMMDD/`): the first time it downloads 520 maps of about 1 MB each into `cache/farm_history/grace/` (0.5 GB), about 4.5 seconds a map, 40 minutes in all, so it is spread over two runs; after that a new farm is read from the cache in seconds and a week adds one map. The daily refresh costs one weather call and two ORNL calls per farm, so beyond a few hundred farms the greenness refresh will need batching.

Being polite: one call at a time, a second's pause after each, a timeout on every call, and up to five tries with a growing wait on 429 and 5xx. A source that fails for a farm is left alone for an hour. Months that were fetched while the backend was down are kept in `cache/farm_history/pending/` and pushed by the next run, without asking the source again. One farm failing does not stop the others, and the run then exits with 1. A run stops starting new work after 45 minutes and the next run carries on. Two runs cannot overlap: the job holds `cache/farm_history/farm_history.lock`. What was done today is kept in `cache/farm_history/state.json`; deleting it only makes the next run do the daily refresh again.

To fetch a series again, clear it on the dashboard (`DELETE /v1/dashboard/farms/{id}/history/{metric}`): the next run sees it missing.

```sh
INGEST__SERVICE_KEY=... FARM_DOCTOR_API=http://localhost:8790/v1 python3 farm_history.py --dry-run
python3 farm_history.py --point 35.56,45.43        # try the sources for one place, no backend
FARM_HISTORY_SOURCES=weather python3 farm_history.py   # leave a source out
```

Groundwater is off by default, because its first fill is slow: switch it on with `FARM_HISTORY_SOURCES=weather,greenness,groundwater`. It needs `rasterio` (the same venv as `groundwater_runner.py`); without it the job logs one line and runs the other two sources. On the server it runs from `farm-doctor-farm-history.timer`. See its last runs with `journalctl -u farm-doctor-farm-history -n 100`.

## farm_plan.py

"This week's plan" in the farmer app (`GET /v1/farms/{id}/plan`): the next 10 days of weather at each farm, turned into farm work. Source: Open-Meteo forecast (free, no key; daily rain, lowest and highest temperature, and hourly temperature, humidity, rain and wind), the Open-Meteo air-quality forecast (PM10, 5 days) and, for sunn pest, the Open-Meteo archive (daily mean temperature since 1 January).

- **Measured by nobody:** every number is a forecast for one model cell of several km, not a reading at the farm. `rain_mm`, `tmin` and `tmax` are pushed as the weather service gave them; a day it gave no number for stays `null`.
- **Derived:** the alerts and decisions. They are the rules of `farm_doctor/weather_planner.py`, in the shape and with the English sentences of `planFromWeather` in the app (`app/lib/api/fake_api.dart`): sowing rain (October to December), urea before rain (January to March), spray windows, frost and hard frost, heat (April and May), rust weather, sunn pest, dust.
- **Sorani:** the app has no Sorani sentence for any of these yet, so `ku` is the English sentence. Nobody here writes Sorani for farmers without a speaker checking it.
- **Crops are not looked at.** Like `weather_planner.py` the job assumes winter wheat: a farm with only vegetables still gets the sowing, urea and rust lines.
- **Not in the plan:** soil moisture, evapotranspiration and river flow, which `weather_planner.py` reports but turns into no decision; and the alert types `heavy_rain`, `dry_spell`, `spray_window` and `sunn_pest`, which no rule in either reference raises.
- **Never more than 10 days.** The backend refuses an eleventh.

**The numbers come from the dashboard's Rules page.** Each run starts with `GET /v1/ingest/rules?used_by=weather_planner`; a rule that is missing, or a backend that does not answer that call, falls back to the number built into the job.

| Rule | What it changes |
|---|---|
| `frost_c` | a night at or below it gets a `frost` alert |
| `hard_frost_c` | a night at or below it makes the alert an `alarm` and adds `frost_check` |
| `heat_c` | April and May: a day at or above it gets a `heat` alert and `heat_check` |
| `heavy_rain_mm` | January to March: the first day with this much rain gets `urea_rain` and `urea_go`; none means `urea_hold` |
| `sowing_rain_mm` | October to December: three days with this much rain together give `sowing_rain` and `sow_go`; none means `sow_wait` |
| `rust_weather_hours` | this many cool wet hours make rust weather "high" (an `alarm`); "some" starts at 8 hours, built in |
| `spray_window_hours` | how many good daytime hours in a row make a day a spray day (`spray_ok`) |
| `sunn_pest_degree_days` | from this sum of degree-days `count_sunn_pest` is shown, until 223 (built in) |
| `dust_pm10` | a PM10 peak at or above it gives a `dust` alert and `dust_delay` |

**How it runs.** The timer fires every 10 minutes. The job asks `GET /v1/ingest/farms/plans/coverage` and works only on farms whose plan is missing or was issued more than 6 hours ago, so a new farm has its plan within minutes and a run with nothing to do costs that one call. A lock file (`farm_plan.lock` in the cache folder) makes a run leave at once while another is still going. Farm centres are grouped on a grid of 0.05 degrees (about 5 km); one forecast is fetched per group, at the middle of the farms in it, 40 groups per Open-Meteo call. In the mountains two farms of one group can differ by hundreds of metres in height, so their real nights differ by more than the one forecast says. The degree-days already counted are kept per group in `farm_plan_degree_days.json`, so the archive is asked only for the days since, and not at all once a group is past 223 for the year.

One farm that fails does not stop the others. The last line is `done: N pushed, N fresh, N failed`, and the job exits 1 if any farm failed; those farms are still due and are tried again 10 minutes later.

**Reporting.** A wrapper report every 10 minutes would bury the run history in runs that did nothing, so this job is not started through `report_run.py`. It reports itself under the job `plans`: every run that pushed or failed, and one quiet run every 6 hours so a healthy job with no farms due is not shown as late.

```sh
INGEST__SERVICE_KEY=... FARM_DOCTOR_API=http://localhost:8790/v1 python3 farm_plan.py
python3 farm_plan.py --dry-run             # fetches and prints the plans, pushes and reports nothing
python3 farm_plan.py --force               # every farm, however fresh its plan is (after a rule change)
python3 farm_plan.py --keep-raw /tmp/raw   # also saves the raw forecasts, to check a plan by hand
```

On the server it runs from `farm-doctor-farm-plan.timer`. See its last runs with `journalctl -u farm-doctor-farm-plan -n 50`.

## Push notifications: `push_sender.py`

Sends red alerts (level `alarm`) to farmers' phones through Firebase Cloud Messaging. Every 5 minutes it asks the backend which alerts are waiting (`GET /v1/ingest/alerts/unpushed`; the backend applies "only alarms" and "at most one push per farm per day"), sends each to the phones of the farm's owner, and marks it pushed. A phone whose token Firebase reports dead is removed.

Setup, once:

1. In the Firebase console of the app's project: Project settings, Service accounts, "Generate new private key". That downloads a JSON file.
2. Put the file on the server as `/opt/farm-doctor/fcm-service-account.json` (`chmod 600`), or anywhere and set `FCM__SERVICE_ACCOUNT_FILE` in `deploy/.env`. Never commit it.
3. `systemctl enable --now farm-doctor-push-sender.timer`.

There is no "API key" to paste: Firebase switched the old server key off in 2024, and the file above is what replaces it. Without the file the job logs that nothing was sent and exits cleanly. The job needs the `openssl` program (it signs the sign-in request with it) and no Python package. `--dry-run` prints what would be sent.

The app must register each phone with `POST /v1/devices` (FRONTEND.md) and include the Firebase messaging library with the project's `google-services.json`.


## farm_analysis.py

Fills the app's "Field history" screen (`GET /v1/farms/{id}/insights`). Every 10 minutes it asks the backend for the farm list (one call) and works only on farms that miss a topic or whose topic is past its refresh age. Each topic is pushed as soon as it is ready. Standard library only. `groundwater` stays with `groundwater_runner.py`.

| Topic | From | Refreshed |
|---|---|---|
| `rain` | ERA5 daily rain since 1981, Open-Meteo archive (`era5_seamless`, whose rain is ERA5 at about 25 km) | daily |
| `weather` | the same download's daily minimum and maximum air temperature (ERA5-Land, about 9 km) | daily |
| `soil` | SoilGrids 2.0 at 250 m, and heights from Open-Meteo's elevation API (Copernicus DEM 90 m) | stored once |
| `greenness` | Sentinel-2 (10 m, springs from 2016) and Landsat (30 m, springs 1984 to 2015) through Microsoft Planetary Computer | newest picture daily, seasons weekly |
| `dryness` | derived from the rain and the greenness, plus the fires the backend stores | daily |

**How each figure is made**

- **Rain.** Season = 1 October to 31 May. Normal = the mean of the 30 seasons ending in 1991 to 2020 (as `evidence/past_seasons/season_rain.py`). Drought = a season under 80% of normal. Trend = straight line through all complete seasons. "So far" = rain since 1 October against the mean of the same days in the normal seasons; pushed only between October and May.
- **Weather.** A frost night = minimum under 0 C; a season runs July to June. Usual last spring frost and first autumn frost = the median day over the normal seasons, given only when frost came in at least 20 of the 30. **Hard spring frost = minimum of -2 C or colder between 15 March and 15 May. This line is ours:** the laptop analysis did not leave its definition. On the test field it gives 4 seasons, two of them (1991/92, 2011/12) also in the fixture's list for another field. Spring heat = days of 31 C or more in April and May.
- **Soil.** Clay, sand, organic carbon and pH are the mean of SoilGrids' 0-5, 5-15 and 15-30 cm layers, weighted by thickness. Height is the centre point; slope is the mean of Horn's slope at the 9 inner points of a 5x5 grid of heights 90 m apart. It is a modelled world map, pushed as `unsure`.
- **Greenness, the field.** The ingest listing gives a farm's centre and area but no outline, so the field is read as **a square box around the centre with the farm's area**. A long or crooked field shares the box with its neighbours; the `source` says "box at centre". A picture counts when 60% of the box is clear (Sentinel-2 scene classes 4 to 7; Landsat's clear bit with no cloud, shadow or snow). NDVI is the mean of the clear pixels (`farm_doctor/field_eye.py`).
- **Greenness, now.** The newest Sentinel-2 picture of the last 30 days with a clear box, against the mean of the clearest picture within 20 days of the same date in each earlier year (at least 3 years). `now_behind_share_pct` = pixels under 70% of the field's median, when that median is over 0.15; `now_behind_dx_m` / `dy_m` = where they sit from the centre, east and north. `now_stage` is a rule of thumb, ours: under 0.2 NDVI is 0 (bare), or 4 (after the season) from June to September; 0.2 to 0.35 is 1 (coming up) from October to February, 2 from March to May, 4 from June to September; 0.35 to 0.6 is 2; above is 3.
- **Greenness, the seasons.** Spring peak = the highest picture of February to May (Sentinel-2: the clearest pass of every 10 days; Landsat: the 8 clearest scenes). A season counts with two clear spring pictures, one of them between 1 March and 15 May; `seasons_measured` is that count. **Landsat reads lower than Sentinel-2** (test field: median peak 0.55 against 0.77), so each season is compared with the median of its own satellite and Landsat peaks are scaled to the Sentinel-2 level by the ratio of the medians. That hides any real change between the two periods, so **`trend_peak_ndvi_per_decade` is not pushed**. `weak_share_pct` = 10 m pixels whose own spring peak is under 70% of the field's median pixel in at least 8 of 10 Sentinel-2 seasons that had a crop (median 0.3 or more); needs 3 such seasons. A season that is over is read once and kept.
- **Dryness.** `rain_green_r` = correlation of season rain (% of normal) with the spring peak; drought under 80%, wet from 115% (`farm_doctor/season_check.py`). Summer green = the lowest July-August picture still at 0.25 or more (`planted_area_test.py`). Not pushed under 5 seasons that have both rain and a peak.

**Left out, on purpose**

- `fire_detections` (the fixture's count since 2000): the job can read only `GET /v1/fires`, which answers the last 7 days. It pushes `fire_detections_7d` instead, under its own code, so the app does not say "no fire was seen" about 26 years on the strength of one week. The count is of the backend's stored fires (grouped, flares removed), within 1 km.
- `summer_surface_c_normal` (Landsat ground temperature): not built.
- MODIS: not used.

**Limits to know**

- **Open-Meteo counts the 45-year download as about a thousand calls and allows ten thousand a day** (as we read its rules; not measured). So the archive is asked at the centre rounded to 0.05 degrees (about 5 km: farms of one village share one download, inside one ERA5 cell in most cases), the 45 years are fetched once per place, and later runs ask only for the last days. More than about 8 new places in a day will be refused with 429; the job then tries those again later.
- **SoilGrids is often slow or down.** It gets one quick try (25 s), so it never holds up the other topics, and one patient try at the end of the run. If both fail, height and slope are pushed alone and the soil is asked again with the back-off below.
- A topic that fails is tried again after 10 minutes, then 20, 40, up to 6 hours. One farm or topic failing does not stop the rest; the run exits 1.
- A run stops starting satellite work after 8 minutes; the next run goes on. A lock file stops two runs at once.
- Everything is kept in `cache/farm_analysis/` (`FARM_ANALYSIS_CACHE`): `state.json` (when each topic was last done; the backend only knows a topic's `as_of` day), the rain series, the soil, heights and the finished seasons. **If the folder is lost,** every farm is analysed again from nothing (and the archive limit above bites), but soil is not pushed again for farms that have it.
- It reports to the job status page as `farm_analysis` itself, and only for runs that pushed or failed; the timer does not use `report_run.py`.

```sh
python3 farm_analysis.py --dry-run --at 35.36,45.70,13   # one place (lat, lon, dunam), no backend, pushes nothing
INGEST__SERVICE_KEY=... FARM_DOCTOR_API=http://localhost:8790/v1 python3 farm_analysis.py
python3 farm_analysis.py --farm 12 --force                # one farm again, ignoring the refresh ages
```

First run, 9 Oct 2026, a 3.2 ha rain-fed field in the Sharazur plain (35.360 N, 45.700 E, 545 m): rain and weather after 2 s, soil after 5 s, the newest picture after 13 s, the 39 seasons after 106 s (446 pictures read, 14 searches), dryness right after. Normal rain 786 mm, 10 droughts since 1981/82; 34 frost nights; clay 36%, pH 7.4; spring peak 0.79, bare in every one of 41 summers; newest picture 4 Oct, NDVI 0.12 against 0.12 usual. A daily refresh took 2 s (1 archive call, 1 search), 7 s when a new picture had come (11 searches, 11 pictures); a quiet run is one backend call.

On the server it runs from `farm-doctor-farm-analysis.timer` every 10 minutes. See its last runs with `journalctl -u farm-doctor-farm-analysis -n 50`.

## farm_alerts.py

Alerts for each farm, every 30 minutes, made only from what the backend already stores. It writes through `PUT /v1/ingest/farms/{id}/alerts/{key}`; the app reads them at `GET /v1/farms/{id}/alerts` and `GET /v1/alerts`.

- **Fire near a farm:** each fire of the last 24 hours (`GET /v1/fires?hours=24`, fires marked out are skipped) within 5 km of a farm's centre (`GET /v1/ingest/farms`) gives that farm one alert, key `fire:<fire id>`: `alarm` within 2 km, `watch` beyond, always confidence `unsure`. The text says "satellite fire detection" with the distance, the direction and the time, and that it is unchecked. It never says there is a fire.
- **Plan alerts:** `alerts_from_plan` turns the `alerts` of a farm's weekly plan into alerts with key `<type>:<day>`, confidence `likely`, the plan's own Sorani and English, and the plan's matching decision as the action (the alert's own text when there is none). **Not live yet:** `fetch_plan` reads `GET /v1/ingest/farms/{id}/plan`, which does not exist yet; a 404 means "no plan", so this part starts by itself when the route does.
- **Running again** replaces the wording of an alert and never adds a second one, unticks one the farmer ticked, or marks one as not pushed. A lock file (`FARM_ALERTS_LOCK`) keeps two runs from overlapping.
- **Limits:** the distance is from the centre of the farm, so a large farm can be nearer the fire than the text says. A satellite detection can be a gas flare or a controlled burn. Fire alerts carry the English text in the Sorani fields until a native speaker has translated the two sentences in `fire_alert`.
- **No pushes yet:** nothing sends these to a phone. `GET /v1/ingest/alerts/unpushed` and `POST /v1/ingest/alerts/{id}/pushed` are there for a push sender, which needs a Firebase key the team does not have yet.
- It is not started through `report_run.py`: the status table has no such job.

```sh
python3 farm_alerts.py --dry-run
python3 -m unittest test_farm_alerts -v
```
