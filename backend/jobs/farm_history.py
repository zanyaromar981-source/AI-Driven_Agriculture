#!/usr/bin/env python3
"""Farm history: ten years of monthly values for every farm.

Runs every 10 minutes. One run reads from the backend which months of which
metric each farm already has, fetches what is missing from the outside
sources, turns it into months, and pushes each metric to the backend.

- A farm with no history (a new farm) gets the last 120 full months.
- Once a day, on the first run after 02:00 UTC, every farm is brought up to
  last month, and its last two stored months are sent again, because the
  sources revise their newest values.
- A run with nothing missing makes one call to the backend and none to any
  outside service.

The three sources, and what a value is (the README says more):
- weather: ERA5 and ERA5-Land reanalysis through Open-Meteo. Rain and
  reference evapotranspiration come from ERA5 (cells of about 25 km), air
  temperature and soil moisture from ERA5-Land (about 9 km). A model of the
  weather, not a gauge on the farm.
- greenness: MODIS NDVI (MOD13Q1), one 250 m pixel at the farm's centre,
  through the ORNL DAAC subset service. Sixteen-day values flagged as cloud
  or snow are dropped.
- groundwater: NASA GRACE-DA weekly percentile maps, cells of about 25 km.
  Not a well depth. Needs the `rasterio` package; without it this source is
  left out and the rest still runs.

A farm is smaller than every one of these grids. Each series says so in its
`source` text.

Needs only the Python standard library, plus `rasterio` for groundwater.

Environment:
  FARM_DOCTOR_API       base address, default http://localhost:8790/v1
                        (API_URL is read when this one is not set)
  INGEST__SERVICE_KEY   the backend's ingest key, required
                        (SERVICE_KEY is read when this one is not set)
  FARM_HISTORY_CACHE    folder for the lock, the state and the groundwater
                        maps (about 0.5 GB), default ./cache/farm_history
  FARM_HISTORY_SOURCES  which sources to run, default weather,greenness.
                        Groundwater is off until it is switched on here:
                        weather,greenness,groundwater

Run with --dry-run to fetch and log without pushing or remembering anything.
Run with --point LAT,LON to try the sources for one place, without a backend.
"""

import calendar
import datetime as dt
import fcntl
import json
import os
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
API = (os.environ.get("FARM_DOCTOR_API") or os.environ.get("API_URL") or "http://localhost:8790/v1").rstrip("/")
KEY = os.environ.get("INGEST__SERVICE_KEY") or os.environ.get("SERVICE_KEY") or ""
CACHE = Path(os.environ.get("FARM_HISTORY_CACHE", HERE / "cache" / "farm_history"))
SOURCES = [name.strip() for name in os.environ.get("FARM_HISTORY_SOURCES", "weather,greenness").split(",")]

ARCHIVE = "https://archive-api.open-meteo.com/v1/archive"
MODIS = "https://modis.ornl.gov/rst/api/v1/MOD13Q1"
GRACE = "https://nasagrace.unl.edu/globaldata"

HISTORY_MONTHS = 120
# A month of daily values is kept when at least this share of its days has one.
MIN_SHARE_OF_DAYS = 0.9
# A month of weekly groundwater maps (four or five) is kept with at least this many.
MIN_WEEKS = 3
# The ORNL service returns at most this many 16-day dates per request.
MODIS_DATES_PER_CALL = 10
MODIS_NDVI = "250m_16_days_NDVI"
MODIS_RELIABILITY = "250m_16_days_pixel_reliability"
MODIS_SCALE = 0.0001
# Pixel reliability: 0 good, 1 marginal, 2 snow or ice, 3 cloudy, -1 no data.
MODIS_USABLE = (0, 1)

# Free services: one call at a time, a pause after each, and patience when
# they ask for it.
PAUSE_S = 1.0
TRIES = 5
# A source that failed for a farm is left alone this long.
RETRY_AFTER_S = 3600
# A run stops starting new work after this long; the next run carries on.
# The timer's unit allows 60 minutes.
RUN_BUDGET_S = 45 * 60
REFRESH_HOUR_UTC = 2
AGENT = "farm-doctor-farm-history"

# What each weather metric is in the archive, how its days become a month
# ("sum" or "mean"), and how many decimals are kept.
WEATHER = {
    "rain_mm": ("precipitation_sum", "sum", 1),
    "temp_max_c": ("temperature_2m_max", "mean", 2),
    "temp_min_c": ("temperature_2m_min", "mean", 2),
    "et0_mm": ("et0_fao_evapotranspiration", "sum", 1),
    "soil_moisture": ("soil_moisture_0_to_7cm_mean", "mean", 3),
}
METRICS = {
    "weather": list(WEATHER),
    "greenness": ["greenness"],
    "groundwater": ["groundwater_pct"],
}
UNITS = {
    "rain_mm": "mm",
    "temp_max_c": "°C",
    "temp_min_c": "°C",
    "et0_mm": "mm",
    "soil_moisture": "m3/m3",
    "greenness": "ndvi",
    "groundwater_pct": "percentile",
}
# The backend keeps at most 200 characters.
SOURCE_TEXT = {
    "rain_mm": "ERA5 reanalysis via Open-Meteo, cells of about 25 km: monthly total of daily rain and snow at the farm centre. A weather model, not a gauge; a farm is far smaller than one cell.",
    "temp_max_c": "ERA5-Land reanalysis via Open-Meteo, cells of about 9 km: monthly mean of each day's highest air temperature at 2 m. A weather model; a farm is far smaller than one cell.",
    "temp_min_c": "ERA5-Land reanalysis via Open-Meteo, cells of about 9 km: monthly mean of each day's lowest air temperature at 2 m. A weather model; a farm is far smaller than one cell.",
    "et0_mm": "Monthly total of daily FAO-56 reference evapotranspiration (well-watered grass), worked out by Open-Meteo from ERA5 reanalysis weather, cells of about 25 km. Not measured at the farm.",
    "soil_moisture": "ERA5-Land reanalysis via Open-Meteo, cells of about 9 km: monthly mean water content of the top 7 cm of soil. A model value for the wider area, not a probe in this farm's soil.",
    "greenness": "NASA MODIS Terra NDVI (MOD13Q1) via ORNL DAAC, one 250 m pixel at the farm centre: monthly mean of 16-day values not flagged cloud or snow. A small farm shares the pixel with its neighbours.",
    "groundwater_pct": "NASA GRACE-DA weekly groundwater storage percentile (50 is normal for the season), model cells of about 25 km: monthly mean. Not a well depth and not measured at the farm.",
}
assert all(len(text) <= 200 for text in SOURCE_TEXT.values()), "a source text is over 200 characters"


def log(message):
    print(f"{dt.datetime.now(dt.timezone.utc):%Y-%m-%d %H:%M:%S}Z {message}", flush=True)


# Months are (year, month) pairs; they travel as "YYYY-MM".


def month_text(month):
    return f"{month[0]:04d}-{month[1]:02d}"


def month_of(text):
    return int(text[:4]), int(text[5:7])


def month_back(month, count):
    index = month[0] * 12 + month[1] - 1 - count
    return index // 12, index % 12 + 1


def days_in(month):
    return calendar.monthrange(*month)[1]


def first_day(month):
    return dt.date(month[0], month[1], 1)


def last_day(month):
    return dt.date(month[0], month[1], days_in(month))


class Failed(Exception):
    """An outside service or the backend did not give what was asked."""


class BackendDown(Failed):
    """The backend did not answer, or failed on its side: the same push is
    worth sending again later."""


class OutOfTime(Exception):
    """The run has used its time; what is left waits for the next run."""


def fetch(url, timeout, headers=None):
    """GET with a timeout, patient with 429 and 5xx. Returns the body."""
    request = urllib.request.Request(url, headers={"user-agent": AGENT, **(headers or {})})
    wait = 2.0
    for attempt in range(1, TRIES + 1):
        try:
            with urllib.request.urlopen(request, timeout=timeout) as response:
                body = response.read()
            time.sleep(PAUSE_S)
            return body
        except urllib.error.HTTPError as error:
            if error.code != 429 and error.code < 500:
                raise Failed(f"{error.code} from {urllib.parse.urlsplit(url).netloc}: {error.read()[:200]!r}")
            asked = error.headers.get("Retry-After", "")
            problem, wait = f"{error.code}", max(wait, float(asked)) if asked.isdigit() else wait
        except OSError as error:  # refused, reset, timed out
            problem = str(error)
        if attempt == TRIES:
            raise Failed(f"{urllib.parse.urlsplit(url).netloc} gave up after {TRIES} tries: {problem}")
        log(f"{urllib.parse.urlsplit(url).netloc}: {problem}, waiting {wait:.0f} s (try {attempt} of {TRIES})")
        time.sleep(wait)
        wait = min(wait * 2, 120)


def backend(method, path, body=None):
    data = json.dumps(body).encode() if body is not None else None
    headers = {"x-service-key": KEY, "content-type": "application/json"}
    request = urllib.request.Request(f"{API}{path}", data=data, method=method, headers=headers)
    try:
        with urllib.request.urlopen(request, timeout=60) as response:
            raw = response.read()
            return json.loads(raw) if raw else {}
    except urllib.error.HTTPError as error:
        kind = BackendDown if error.code >= 500 else Failed
        raise kind(f"backend refused with {error.code}: {error.read()[:200]!r}")
    except OSError as error:
        raise BackendDown(f"backend unreachable: {error}")


# Weather: Open-Meteo archive.


def months_from_days(days, values, how, decimals):
    """Daily values to months. A month with too few days is left out; a
    total over a month with a few days missing is scaled to the whole month."""
    per_month = {}
    for day, value in zip(days, values):
        if value is not None:
            per_month.setdefault(month_of(day), []).append(value)
    months = {}
    for month, found in per_month.items():
        if len(found) < MIN_SHARE_OF_DAYS * days_in(month):
            continue
        mean = sum(found) / len(found)
        months[month] = round(mean * days_in(month) if how == "sum" else mean, decimals)
    return months


def fetch_weather(lat, lon, start, end):
    """Monthly values of the five weather metrics from `start` to `end`.

    `era5_seamless` is asked for by name so the series does not change model
    half way: without it the archive mixes in a forecast model from 2017 on.
    It serves rain and evapotranspiration from ERA5 and temperature and soil
    moisture from ERA5-Land, about five days behind today."""
    query = urllib.parse.urlencode(
        {
            "latitude": f"{lat:.5f}",
            "longitude": f"{lon:.5f}",
            "start_date": first_day(start).isoformat(),
            "end_date": last_day(end).isoformat(),
            "daily": ",".join(variable for variable, _, _ in WEATHER.values()),
            "models": "era5_seamless",
            "timezone": "UTC",
        }
    )
    answer = json.loads(fetch(f"{ARCHIVE}?{query}", timeout=120))
    daily = answer.get("daily")
    if not daily or "time" not in daily:
        raise Failed(f"weather archive answered without days: {str(answer)[:200]}")
    return {
        metric: months_from_days(daily["time"], daily.get(variable) or [], how, decimals)
        for metric, (variable, how, decimals) in WEATHER.items()
    }


# Greenness: MODIS NDVI from the ORNL DAAC subset service.


def modis_dates(lat, lon):
    """Every 16-day period the product has, as (code, first day)."""
    answer = json.loads(fetch(f"{MODIS}/dates?latitude={lat:.5f}&longitude={lon:.5f}", 90, {"accept": "application/json"}))
    return [(entry["modis_date"], dt.date.fromisoformat(entry["calendar_date"])) for entry in answer["dates"]]


def modis_band(lat, lon, band, first, last):
    query = urllib.parse.urlencode(
        {
            "latitude": f"{lat:.5f}",
            "longitude": f"{lon:.5f}",
            "band": band,
            "startDate": first,
            "endDate": last,
            "kmAboveBelow": 0,
            "kmLeftRight": 0,
        }
    )
    answer = json.loads(fetch(f"{MODIS}/subset?{query}", 120, {"accept": "application/json"}))
    return {entry["modis_date"]: entry["data"][0] for entry in answer.get("subset", []) if entry.get("data")}


def fetch_greenness(lat, lon, start, end, dates):
    """Monthly NDVI from `start` to `end`: the mean of the 16-day values of
    the pixel at the point that are not cloud, snow or missing. A 16-day
    period belongs to the month its middle day falls in. A month with no
    usable value is left out."""
    wanted = [(code, day) for code, day in dates if start <= month_of((day + dt.timedelta(days=8)).isoformat()) <= end]
    per_month = {}
    for index in range(0, len(wanted), MODIS_DATES_PER_CALL):
        chunk = wanted[index : index + MODIS_DATES_PER_CALL]
        ndvi = modis_band(lat, lon, MODIS_NDVI, chunk[0][0], chunk[-1][0])
        reliability = modis_band(lat, lon, MODIS_RELIABILITY, chunk[0][0], chunk[-1][0])
        for code, day in chunk:
            raw = ndvi.get(code)
            if raw is None or reliability.get(code) not in MODIS_USABLE or not -2000 <= raw <= 10000:
                continue
            month = month_of((day + dt.timedelta(days=8)).isoformat())
            per_month.setdefault(month, []).append(raw * MODIS_SCALE)
    return {month: round(sum(found) / len(found), 4) for month, found in per_month.items()}


# Groundwater: NASA GRACE-DA weekly maps, kept in the cache folder.


def have_rasterio():
    try:
        import rasterio  # noqa: F401
    except ImportError:
        return False
    return True


def grace_weeks(start, end):
    """The days NASA has a weekly folder for, inside the months asked."""
    listing = fetch(f"{GRACE}/", timeout=120).decode("utf-8", "replace")
    days = sorted({dt.datetime.strptime(name, "%Y%m%d").date() for name in re.findall(r"/globaldata/(\d{8})/", listing)})
    return [day for day in days if start <= (day.year, day.month) <= end]


def grace_map(day, deadline):
    """The path of the week's groundwater map, downloading it if the cache
    does not have it. None when NASA has no map for that week."""
    folder = CACHE / "grace"
    name = f"gws_perc_025deg_GL_{day:%Y%m%d}.tif"
    path, missing = folder / name, folder / f"{name}.missing"
    if path.exists():
        return path
    if missing.exists() and time.time() - missing.stat().st_mtime < 7 * 86400:
        return None
    if time.monotonic() > deadline:
        raise OutOfTime
    folder.mkdir(parents=True, exist_ok=True)
    try:
        body = fetch(f"{GRACE}/{day:%Y%m%d}/{name}", timeout=180)
    except Failed as error:
        if "404" not in str(error):
            raise
        missing.touch()
        return None
    temporary = folder / f"{name}.tmp"
    temporary.write_bytes(body)
    temporary.replace(path)
    return path


def read_points(path, points):
    """The map's percentile at each (lat, lon), or None where it has none."""
    import rasterio

    with rasterio.open(path) as raster:
        values = []
        for sample in raster.sample([(lon, lat) for lat, lon in points]):
            value = float(sample[0])
            missing = value != value or (raster.nodata is not None and value == raster.nodata)
            values.append(None if missing or not 0 <= value <= 100 else value)
        return values


def fetch_groundwater(places, start, end, deadline):
    """Monthly groundwater percentile for every place at once, so each map
    is opened once. Returns one {month: value} per place, or None when the
    maps could not all be fetched in the time this run has left."""
    weeks = grace_weeks(start, end)
    paths, fetched = [], 0
    for day in weeks:
        cached = (CACHE / "grace" / f"gws_perc_025deg_GL_{day:%Y%m%d}.tif").exists()
        try:
            path = grace_map(day, deadline)
        except OutOfTime:
            log(f"groundwater: {len(paths)} of {len(weeks)} weekly maps are in the cache, the next run carries on")
            return None
        fetched += 0 if cached or path is None else 1
        if path is not None:
            paths.append((day, path))
    if fetched:
        log(f"groundwater: {fetched} weekly maps downloaded, {len(paths)} of {len(weeks)} weeks have a map")

    per_place = [{} for _ in places]
    for day, path in paths:
        for found, value in zip(per_place, read_points(path, places)):
            if value is not None:
                found.setdefault((day.year, day.month), []).append(value)
    return [
        {month: round(sum(found) / len(found), 1) for month, found in months.items() if len(found) >= MIN_WEEKS}
        for months in per_place
    ]


# What to do for one farm and one source.


def refresh_key(now):
    """Names the day whose refresh a run belongs to: it changes at 02:00 UTC."""
    return (now - dt.timedelta(hours=REFRESH_HOUR_UTC)).date().isoformat()


def plan(farm, source, state, now, last_month):
    """The first month to fetch for this farm from this source, or None when
    there is nothing to do in this run."""
    key = f"{farm['farm_id']}:{source}"
    today = refresh_key(now)
    oldest = month_back(last_month, HISTORY_MONTHS - 1)
    stored = farm.get("metrics", {})
    redo = state["redo"].get(key)
    if redo and now.isoformat() < redo["after"]:
        return None
    # A metric the source had nothing for is not asked for again the same day.
    absent = [m for m in METRICS[source] if m not in stored and state["empty"].get(f"{farm['farm_id']}:{m}") != today]
    if absent:
        return oldest
    if redo:
        return max(month_of(redo["start"]), oldest)
    if state["done"].get(key) == today:
        return None
    # The daily refresh: from the last stored month, and at least the last
    # two months again, because the newest values get revised.
    last_stored = [month_of(stored[m]["last_month"]) for m in METRICS[source] if m in stored]
    return max(min(last_stored + [month_back(last_month, 1)]), oldest)


def push(farm_id, metric, months, start, dry_run):
    """Pushes one metric's months. Returns False if the backend refused."""
    points = [{"month": month_text(m), "value": v} for m, v in sorted(months.items()) if m >= start]
    if not points:
        log(f"farm {farm_id} {metric}: the source has no usable month from {month_text(start)}, nothing pushed")
        return True
    span = f"{len(points)} months {points[0]['month']} to {points[-1]['month']}"
    if dry_run:
        log(f"farm {farm_id} {metric}: {span} fetched (dry run, not pushed); last value {points[-1]['value']}")
        return True
    body = {
        "unit": UNITS[metric],
        "source": SOURCE_TEXT[metric],
        "as_of": dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "points": points,
    }
    try:
        backend("PUT", f"/ingest/farms/{farm_id}/history/{metric}", body)
    except BackendDown as error:
        # The months are kept and pushed by the next run, so a restart of
        # our own backend does not cost the outside service a second fetch.
        path = CACHE / "pending" / f"{farm_id}_{metric}.json"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(body))
        log(f"farm {farm_id} {metric}: FAILED, {error}; {span} kept for the next run")
        return False
    except Failed as error:
        log(f"farm {farm_id} {metric}: FAILED, {error}")
        return False
    log(f"farm {farm_id} {metric}: {span} pushed")
    return True


def push_pending(farms, state, now):
    """Pushes the months an earlier run fetched but could not deliver.
    Returns how many pushes went through and how many still fail."""
    alive = {str(farm["farm_id"]) for farm in farms}
    pushed, failed = 0, 0
    for path in sorted((CACHE / "pending").glob("*.json")):
        farm_id, metric = path.stem.split("_", 1)
        source = next((name for name, metrics in METRICS.items() if metric in metrics), None)
        if farm_id not in alive or source is None:
            path.unlink()
            continue
        body = json.loads(path.read_text())
        try:
            backend("PUT", f"/ingest/farms/{farm_id}/history/{metric}", body)
        except BackendDown as error:
            log(f"farm {farm_id} {metric}: FAILED again, {error}; kept for the next run")
            failed += 1
            continue
        except Failed as error:
            log(f"farm {farm_id} {metric}: FAILED, {error}; the kept months are dropped")
            path.unlink()
            failed += 1
            continue
        path.unlink()
        pushed += 1
        log(f"farm {farm_id} {metric}: {len(body['points'])} months fetched by an earlier run pushed")
        if not list((CACHE / "pending").glob(f"{farm_id}_*.json")):
            state["redo"].pop(f"{farm_id}:{source}", None)
            state["done"][f"{farm_id}:{source}"] = refresh_key(now)
    return pushed, failed


def settle(state, farm_id, source, results, start, now, ok):
    """Remembers how a farm's source went, so the next run neither repeats
    finished work nor hammers a service that is failing."""
    key = f"{farm_id}:{source}"
    if not ok:
        # Counted from this moment, not from the start of a run that may
        # have been going for most of an hour.
        after = (dt.datetime.now(dt.timezone.utc) + dt.timedelta(seconds=RETRY_AFTER_S)).isoformat()
        state["redo"][key] = {"after": after, "start": month_text(start)}
        return
    state["redo"].pop(key, None)
    state["done"][key] = refresh_key(now)
    for metric in METRICS[source]:
        if not results.get(metric):
            state["empty"][f"{farm_id}:{metric}"] = refresh_key(now)
        else:
            state["empty"].pop(f"{farm_id}:{metric}", None)


def load_state():
    path = CACHE / "state.json"
    state = json.loads(path.read_text()) if path.exists() else {}
    return {name: state.get(name, {}) for name in ("done", "redo", "empty")}


def save_state(state, farm_ids):
    """Writes the state, forgetting farms that no longer exist."""
    alive = {str(farm_id) for farm_id in farm_ids}
    kept = {name: {k: v for k, v in entries.items() if k.split(":")[0] in alive} for name, entries in state.items()}
    CACHE.mkdir(parents=True, exist_ok=True)
    temporary = CACHE / "state.json.tmp"
    temporary.write_text(json.dumps(kept, indent=1, sort_keys=True))
    temporary.replace(CACHE / "state.json")


def run(farms, state, now, dry_run):
    """Does one run's work. Returns how many farm sources failed."""
    deadline = time.monotonic() + RUN_BUDGET_S
    this_month = (now.year, now.month)
    last_month = month_back(this_month, 1)
    failed, dates = 0, None

    def out_of_time(what):
        if time.monotonic() > deadline:
            log(f"{what}: this run has used its {RUN_BUDGET_S // 60} minutes, the next run carries on")
            return True
        return False

    def finish(farm, source, results, start, ok):
        nonlocal failed
        ok = ok and all([push(farm["farm_id"], metric, months, start, dry_run) for metric, months in results.items()])
        failed += 0 if ok else 1
        if not dry_run:
            settle(state, farm["farm_id"], source, results, start, now, ok)
            save_state(state, [f["farm_id"] for f in farms])

    # Weather first: one call gives a new farm five of its seven series.
    for farm in farms if "weather" in SOURCES else []:
        start = plan(farm, "weather", state, now, last_month)
        if start is None or out_of_time("weather"):
            continue
        try:
            finish(farm, "weather", fetch_weather(farm["lat"], farm["lon"], start, last_month), start, True)
        except (Failed, KeyError, ValueError) as error:
            log(f"farm {farm['farm_id']} weather ({', '.join(METRICS['weather'])}): FAILED, {error}")
            finish(farm, "weather", {}, start, False)

    for farm in farms if "greenness" in SOURCES else []:
        start = plan(farm, "greenness", state, now, last_month)
        if start is None or out_of_time("greenness"):
            continue
        try:
            dates = dates or modis_dates(farm["lat"], farm["lon"])
            months = fetch_greenness(farm["lat"], farm["lon"], start, last_month, dates)
            finish(farm, "greenness", {"greenness": months}, start, True)
        except (Failed, KeyError, ValueError, IndexError) as error:
            log(f"farm {farm['farm_id']} greenness: FAILED, {error}")
            finish(farm, "greenness", {}, start, False)

    # Groundwater last and for all farms together: the weekly maps are
    # shared, so they are downloaded once and each is opened once.
    if "groundwater" in SOURCES:
        due = [(farm, plan(farm, "groundwater", state, now, last_month)) for farm in farms]
        due = [(farm, start) for farm, start in due if start is not None]
        if due and not have_rasterio():
            log(f"groundwater_pct: skipped for {len(due)} farms, the rasterio package is not installed (see the README)")
        elif due and not out_of_time("groundwater"):
            first = min(start for _, start in due)
            try:
                found = fetch_groundwater([(farm["lat"], farm["lon"]) for farm, _ in due], first, last_month, deadline)
            except (Failed, OSError, ValueError) as error:  # rasterio's errors are OSErrors
                log(f"groundwater_pct for {len(due)} farms: FAILED, {error}")
                found = [False] * len(due)
            for (farm, start), months in zip(due, found or []):
                if months is False:
                    finish(farm, "groundwater", {}, start, False)
                else:
                    finish(farm, "groundwater", {"groundwater_pct": months}, start, True)

    return failed


def main():
    dry_run = "--dry-run" in sys.argv or "--point" in sys.argv
    now = dt.datetime.now(dt.timezone.utc)

    if "--point" in sys.argv:
        lat, lon = (float(part) for part in sys.argv[sys.argv.index("--point") + 1].split(","))
        farms = [{"farm_id": "point", "lat": lat, "lon": lon, "metrics": {}}]
        state = {"done": {}, "redo": {}, "empty": {}}
        sys.exit(1 if run(farms, state, now, dry_run) else 0)

    if not KEY:
        sys.exit("INGEST__SERVICE_KEY is not set")

    # Two runs at once would fetch and push the same months twice.
    CACHE.mkdir(parents=True, exist_ok=True)
    lock = open(CACHE / "farm_history.lock", "w")
    try:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError:
        log("another run is still going: nothing done")
        return

    state = {"done": {}, "redo": {}, "empty": {}} if dry_run else load_state()
    kept_failed = 0
    try:
        farms = backend("GET", "/ingest/farms/history/coverage").get("farms", [])
        if not dry_run and (CACHE / "pending").is_dir():
            pushed, kept_failed = push_pending(farms, state, now)
            if pushed:
                # What was just pushed is no longer missing.
                save_state(state, [farm["farm_id"] for farm in farms])
                farms = backend("GET", "/ingest/farms/history/coverage").get("farms", [])
    except Failed as error:
        log(f"coverage: FAILED, {error}")
        sys.exit(1)
    failed = run(farms, state, now, dry_run) + kept_failed
    if failed:
        log(f"done: {len(farms)} farms, {failed} farm sources FAILED and are tried again in an hour")
        sys.exit(1)
    log(f"done: {len(farms)} farms, nothing failed")


if __name__ == "__main__":
    main()
