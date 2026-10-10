#!/usr/bin/env python3
"""Farm analysis: fills the farmer app's "Field history" screen, farm by farm.

Runs every 10 minutes from a timer. One call to the backend lists every farm
and the topics it already has. The job works only on farms that miss a topic
or whose topic is older than its refresh age, and pushes each topic as soon
as it is ready (the app shows them arriving one by one):

  rain       ERA5 daily rain since 1981 (Open-Meteo archive, era5_seamless):
             October-May totals, the 1991-2020 normal, droughts, trend, this
             season so far. Refreshed daily.
  weather    Frost and heat from the same download's daily minimum and
             maximum air temperature (ERA5-Land). Refreshed daily.
  soil       SoilGrids 250 m (clay, sand, organic carbon, pH of 0-30 cm) and
             height and slope from the Copernicus 90 m height model
             (Open-Meteo elevation). Stored once.
  greenness  NDVI of the field from Sentinel-2 (10 m) and, before 2016,
             Landsat (30 m), read through Microsoft Planetary Computer: the
             newest clear picture against the same weeks of earlier years
             (daily), and the spring peak of every season (weekly).
  dryness    Derived: season rain against the spring peak, summer greenness,
             and the fires the backend stores near the farm. Daily.

`groundwater` belongs to groundwater_runner.py. Not computed: the trend of the
spring peak, the summer ground temperature, and fires before the last week
(see README.md for why).

Read this before trusting the numbers (the full method is in README.md):
- Rain and temperature are reanalysis values for a cell of 9 to 25 km, asked
  at the farm centre rounded to 0.05 degrees. They are not a gauge at the farm.
- Soil is a world map modelled at 250 m, not a test of the field.
- Greenness is read in a square box around the farm centre with the farm's
  area, because the ingest listing gives no outline. A long or crooked field
  shares its box with its neighbours.
- A number that could not be computed is left out. Nothing is guessed.

Needs only the Python standard library.

Environment:
  FARM_DOCTOR_API       base address, default http://localhost:8790/v1
  INGEST__SERVICE_KEY   the backend's ingest key (required)
  FARM_ANALYSIS_CACHE   folder for downloads, results and the lock,
                        default ./cache/farm_analysis

Options:
  --dry-run             compute what is due and print it, push nothing
  --at LAT,LON[,DUNAM]  with --dry-run: analyse this place, no backend needed
  --farm ID             only this farm
  --force               ignore the refresh ages (still uses the caches)
"""

import array
import ast
import concurrent.futures as cf
import datetime as dt
import fcntl
import json
import math
import os
import statistics as st
import struct
import sys
import threading
import time
import urllib.error
import urllib.parse
import urllib.request
import zlib
from pathlib import Path

HERE = Path(__file__).resolve().parent
API = os.environ.get("FARM_DOCTOR_API", "http://localhost:8790/v1").rstrip("/")
KEY = os.environ.get("INGEST__SERVICE_KEY", "")
CACHE = Path(os.environ.get("FARM_ANALYSIS_CACHE", HERE / "cache" / "farm_analysis"))

ARCHIVE = "https://archive-api.open-meteo.com/v1/archive"
ELEVATION = "https://api.open-meteo.com/v1/elevation"
SOILGRIDS = "https://rest.isric.org/soilgrids/v2.0/properties/query"
# The same SoilGrids 2.0 maps through ISRIC's map service (WCS). On 2026-10-10 the query
# service above answered every point with empty values, even outside Iraq, while this
# one gave real pixels (Akre 39% clay, pH 7.2).
SOILGRIDS_MAPS = "https://maps.isric.org/mapserv"
STAC = "https://planetarycomputer.microsoft.com/api/stac/v1/search"
DATA = "https://planetarycomputer.microsoft.com/api/data/v1"
UA = (
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 "
    "(KHTML, like Gecko) Chrome/128.0 Safari/537.36"
)

# The backend's limits (src/features/insights/domain).
SOURCE_LIMIT, SUMMARY_LIMIT, LABEL_LIMIT, UNIT_LIMIT, MEASURES_LIMIT = 120, 300, 60, 20, 20

# Refresh ages. A little under a day so a 10-minute timer does not drift.
DAILY_S = 23 * 3600
WEEKLY_S = 7 * 24 * 3600
# A run stops starting satellite work after this long; the next run goes on.
BUDGET_S = 8 * 60
# A topic that failed is tried again after 10 minutes, then 20, 40, up to 6 hours.
RETRY_FIRST_S, RETRY_MAX_S = 600, 6 * 3600

# ---- rain and weather (methods of evidence/past_seasons/season_rain.py and
# farm_doctor/season_check.py)
FIRST_YEAR = 1981
NORMAL_SEASONS = range(1990, 2020)  # start years: seasons 1990/91 to 2019/20, ending 1991 to 2020
DROUGHT_BELOW_PCT = 80  # the fixture's and the dashboard's line
WET_FROM_PCT = 115  # season_check.py calls a season "wet" from 115%
FROST_BELOW_C = 0.0
HARD_FROST_AT_C = -2.0  # and colder, between 15 March and 15 May (wheat heading)
HEAT_FROM_C = 31.0
ARCHIVE_SNAP_DEG = 0.05

# ---- satellite (methods of farm_doctor/field_eye.py and
# evidence/past_seasons/backtest/planted_area_test.py)
S2_VALID_SCL = {4, 5, 6, 7}  # vegetation, bare, water, unclassified
MIN_VALID = 0.60  # a picture counts when 60% of the box is clear
S2_MAX_CLOUD = 60
LANDSAT_MAX_CLOUD = 50
T_SUMMER = 0.25  # planted_area_test.py: green in July-August means watered or perennial
WEAK_BELOW = 0.7  # field_eye.py: a pixel under 70% of the field's median is weak
WEAK_MIN_MEDIAN = 0.15  # field_eye.py: no weak pixels on a bare field
WEAK_SEASON_MIN_PEAK = 0.30  # a season tells weak spots apart only when a crop grew
WEAK_IN_SHARE = 0.8  # "almost every season"
NOW_LOOKBACK_DAYS = 30
NOW_WINDOW_DAYS = 20  # field_eye.py: the same weeks of earlier years, 20 days each side
NOW_MIN_YEARS = 3
S2_FIRST_SPRING = 2016
LANDSAT_FIRST_SPRING = 1984
MAX_GRID = 60
POOL = 6
FIRE_RADIUS_KM = 1.0
FIRE_HOURS = 168  # the longest window GET /v1/fires answers

CALLS = {}
CALLS_LOCK = threading.Lock()


def log(message):
    print(f"{dt.datetime.now(dt.timezone.utc):%Y-%m-%d %H:%M:%S}Z {message}", flush=True)


class OutsideError(Exception):
    """An outside service did not answer, or answered with something unusable."""


class NothingToPush(Exception):
    """The topic cannot be computed honestly for this farm right now."""


# ---------------------------------------------------------------- plumbing
def fetch(url, body=None, what="outside", timeout=60, tries=3, wait=3):
    """GET (or POST when a body is given). Counts every attempt per service."""
    last = None
    for attempt in range(tries):
        if attempt:
            time.sleep(wait * attempt)
        headers = {"User-Agent": UA, "Accept": "*/*"}
        data = None
        if body is not None:
            data = json.dumps(body).encode()
            headers["Content-Type"] = "application/json"
        with CALLS_LOCK:
            CALLS[what] = CALLS.get(what, 0) + 1
        try:
            with urllib.request.urlopen(urllib.request.Request(url, data=data, headers=headers), timeout=timeout) as answer:
                return answer.read()
        except urllib.error.HTTPError as error:
            last = f"HTTP {error.code} {error.read()[:160]!r}"
            if error.code in (400, 404, 422, 429):
                break  # asking again does not help, and 429 means slow down
        except (urllib.error.URLError, OSError) as error:
            last = error
    raise OutsideError(f"{what} did not answer: {last}")


def read_json(path):
    try:
        return json.loads(Path(path).read_text())
    except (OSError, ValueError):
        return None


def write_json(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, separators=(",", ":")))
    temporary.replace(path)


def backend(method, path, body=None, timeout=60):
    data = json.dumps(body).encode() if body is not None else None
    headers = {"x-service-key": KEY, "content-type": "application/json"}
    request = urllib.request.Request(f"{API}{path}", data=data, method=method, headers=headers)
    with CALLS_LOCK:
        CALLS["backend"] = CALLS.get("backend", 0) + 1
    with urllib.request.urlopen(request, timeout=timeout) as answer:
        raw = answer.read()
        return json.loads(raw) if raw else {}


def season_name(start_year):
    return f"{start_year}/{(start_year + 1) % 100:02d}"


def slope_per_decade(points):
    """Least-squares slope of (year, value) pairs, times ten. None under 10 points."""
    if len(points) < 10:
        return None
    mean_x = st.mean(x for x, _ in points)
    mean_y = st.mean(y for _, y in points)
    spread = sum((x - mean_x) ** 2 for x, _ in points)
    return 10 * sum((x - mean_x) * (y - mean_y) for x, y in points) / spread if spread else None


def pearson(pairs):
    if len(pairs) < 3:
        return None
    mean_x = st.mean(x for x, _ in pairs)
    mean_y = st.mean(y for _, y in pairs)
    sxx = sum((x - mean_x) ** 2 for x, _ in pairs)
    syy = sum((y - mean_y) ** 2 for _, y in pairs)
    if sxx <= 0 or syy <= 0:
        return None
    return sum((x - mean_x) * (y - mean_y) for x, y in pairs) / math.sqrt(sxx * syy)


def measure(code, value, unit, label):
    return {"code": code, "value": value, "unit": unit, "label_en": label, "label_ku": None}


def fit_summary(sentences):
    """Joins the sentences in order, leaving out the ones that no longer fit."""
    text = ""
    for sentence in sentences:
        if sentence and len(text) + len(sentence) + (1 if text else 0) <= SUMMARY_LIMIT:
            text = f"{text} {sentence}".strip()
    return text or None


def topic_body(as_of, source, confidence, sentences, measures):
    measures = [m for m in measures if m["value"] is not None and math.isfinite(m["value"])]
    body = {
        "as_of": as_of,
        "source": source,
        "confidence": confidence,
        "summary_en": fit_summary(sentences),
        "summary_ku": None,  # the app builds its own Sorani sentences from the measures
        "measures": measures,
    }
    if not measures:
        raise NothingToPush("no measure could be computed")
    if len(source) > SOURCE_LIMIT:
        raise ValueError(f"source is {len(source)} characters, the backend takes {SOURCE_LIMIT}: {source}")
    if len(measures) > MEASURES_LIMIT:
        raise ValueError(f"{len(measures)} measures, the backend takes {MEASURES_LIMIT}")
    for one in measures:
        if len(one["label_en"]) > LABEL_LIMIT or len(one["unit"]) > UNIT_LIMIT or len(one["code"]) > 40:
            raise ValueError(f"measure {one['code']} is longer than the backend takes")
    return body


# ---------------------------------------------------------------- ERA5 series
def archive_call(lat, lon, start, end):
    query = urllib.parse.urlencode(
        {
            "latitude": lat,
            "longitude": lon,
            "start_date": start.isoformat(),
            "end_date": end.isoformat(),
            "daily": "precipitation_sum,temperature_2m_min,temperature_2m_max",
            "models": "era5_seamless",
            "timezone": "Asia/Baghdad",
        }
    )
    try:
        daily = json.loads(fetch(f"{ARCHIVE}?{query}", what="open-meteo archive", timeout=120))["daily"]
        return {
            day: [daily["precipitation_sum"][i], daily["temperature_2m_min"][i], daily["temperature_2m_max"][i]]
            for i, day in enumerate(daily["time"])
        }
    except (KeyError, IndexError, TypeError, ValueError) as error:
        raise OutsideError(f"open-meteo archive answered with an unexpected shape: {error}") from error


def weather_days(lat, lon, today):
    """Daily [rain, min, max] since 1981 for the place, as {day: [..]}.

    The 45 years are downloaded once per place (the archive counts such a
    call as about a thousand, and allows ten thousand a day). After that only
    the last days are asked for, once a day.
    """
    snap = lambda value: round(round(value / ARCHIVE_SNAP_DEG) * ARCHIVE_SNAP_DEG, 2)
    lat, lon = snap(lat), snap(lon)
    path = CACHE / "archive" / f"{lat:.2f}_{lon:.2f}.json"
    cached = read_json(path)
    if cached and cached.get("fetched") == today.isoformat():
        return cached["days"]
    if cached:
        days = cached["days"]
        known = max(day for day, row in days.items() if row[0] is not None)
        start = dt.date.fromisoformat(known) - dt.timedelta(days=10)
        days.update(archive_call(lat, lon, start, today))
    else:
        days = archive_call(lat, lon, dt.date(FIRST_YEAR, 1, 1), today)
    write_json(path, {"fetched": today.isoformat(), "lat": lat, "lon": lon, "days": days})
    return days


def rows_of(days):
    return sorted((dt.date.fromisoformat(day), row[0], row[1], row[2]) for day, row in days.items())


def rain_seasons(rows):
    """October-May rain per season start year, only seasons with every day present."""
    totals, counts = {}, {}
    for day, rain, _, _ in rows:
        if 6 <= day.month <= 9 or rain is None:
            continue
        season = day.year if day.month >= 10 else day.year - 1
        totals[season] = totals.get(season, 0.0) + rain
        counts[season] = counts.get(season, 0) + 1
    full = lambda season: 243 + (1 if (season + 1) % 4 == 0 and ((season + 1) % 100 != 0 or (season + 1) % 400 == 0) else 0)
    return {season: totals[season] for season in sorted(totals) if season >= FIRST_YEAR and counts[season] == full(season)}


def rain_normal(seasons):
    base = [seasons[season] for season in NORMAL_SEASONS if season in seasons]
    if len(base) < 25:
        raise NothingToPush("fewer than 25 of the 30 normal seasons have complete rain")
    return st.mean(base)


def rain_topic(rows):
    seasons = rain_seasons(rows)
    normal = rain_normal(seasons)
    last_day = max(day for day, rain, _, _ in rows if rain is not None)
    last = max(seasons)
    droughts = [season for season in seasons if 100 * seasons[season] / normal < DROUGHT_BELOW_PCT]
    trend = slope_per_decade(list(seasons.items()))
    first = min(seasons)
    measures = [
        measure("normal_mm_oct_may", round(normal), "mm", "Normal October-May rain"),
        measure("last_season_mm", round(seasons[last]), "mm", f"Rain in {season_name(last)}"),
        measure("last_season_pct_of_normal", round(100 * seasons[last] / normal), "%", f"{season_name(last)} vs normal"),
        measure("drought_seasons", len(droughts), "seasons", f"Drought seasons since {season_name(first)}"),
        measure("latest_drought_season", max(droughts) if droughts else None, "season start year", "Latest drought season"),
        measure("trend_mm_per_decade", None if trend is None else round(trend, 1), "mm per decade", "Rain trend"),
    ]
    latest = ", ".join(season_name(season) for season in droughts[-3:])
    sentences = [
        f"Normal October to May rain here: {normal:.0f} mm.",
        f"Since {season_name(first)}, {len(droughts)} seasons were droughts" + (f" (latest: {latest})." if droughts else "."),
        f"Last full season {season_name(last)}: {seasons[last]:.0f} mm, {100 * seasons[last] / normal:.0f}% of normal.",
    ]
    if last_day.month >= 10 or last_day.month <= 5:
        current = last_day.year if last_day.month >= 10 else last_day.year - 1
        order = lambda day: ((day.month - 10) % 12, min(day.day, 28) if day.month == 2 else day.day)
        cut = order(last_day)
        so_far, usual = 0.0, {}
        for day, rain, _, _ in rows:
            if 6 <= day.month <= 9 or rain is None or order(day) > cut:
                continue
            season = day.year if day.month >= 10 else day.year - 1
            if season == current:
                so_far += rain
            elif season in NORMAL_SEASONS and season in seasons:
                usual[season] = usual.get(season, 0.0) + rain
        usual_so_far = sum(usual.values()) / sum(1 for season in NORMAL_SEASONS if season in seasons)
        if current > last:
            measures += [
                measure("this_season_so_far_mm", round(so_far), "mm", f"Rain so far in {season_name(current)}"),
                measure("this_season_normal_so_far_mm", round(usual_so_far), "mm", "Normal for the same days"),
            ]
            sentences.append(
                f"This season so far ({season_name(current)}, to {last_day}): {so_far:.0f} mm against "
                f"{usual_so_far:.0f} mm normal for the same days."
            )
    source = "ERA5 ~25 km, daily since 1981 (Open-Meteo era5_seamless); Oct-May totals; normal 1991-2020; drought <80%"
    return topic_body(last_day.isoformat(), source, "likely", sentences, measures)


def weather_topic(rows):
    last_day = max(day for day, _, low, _ in rows if low is not None)
    seasons = {}
    for day, _, low, high in rows:
        if low is None or high is None:
            continue
        season = day.year if day.month >= 7 else day.year - 1
        one = seasons.setdefault(season, {"days": 0, "frost": 0, "hard": 0, "heat": 0, "last": None, "first": None})
        one["days"] += 1
        if low < FROST_BELOW_C:
            one["frost"] += 1
            if one["first"] is None:
                one["first"] = (day - dt.date(season, 7, 1)).days
            if day.month <= 6:
                one["last"] = (day - dt.date(season + 1, 1, 1)).days
        if low <= HARD_FROST_AT_C and (3, 15) <= (day.month, day.day) <= (5, 15):
            one["hard"] += 1
        if high >= HEAT_FROM_C and day.month in (4, 5):
            one["heat"] += 1
    # A season counts when it has every day from 1 July to 30 June.
    complete = {s: one for s, one in seasons.items() if s >= FIRST_YEAR and one["days"] >= 365}
    base = [complete[s] for s in NORMAL_SEASONS if s in complete]
    if len(base) < 25:
        raise NothingToPush("fewer than 25 of the 30 normal seasons have complete temperatures")
    frost = st.mean(one["frost"] for one in base)
    hard = st.mean(one["hard"] for one in base)
    heat = st.mean(one["heat"] for one in base)
    frost_trend = slope_per_decade([(s, one["frost"]) for s, one in complete.items()])
    heat_trend = slope_per_decade([(s, one["heat"]) for s, one in complete.items()])
    hard_seasons = sorted(s for s, one in complete.items() if one["hard"] > 0)
    lasts = [one["last"] for one in base if one["last"] is not None]
    firsts = [one["first"] for one in base if one["first"] is not None]
    # A usual date is given only when frost came in at least 20 of the 30 seasons.
    last_frost = dt.date(2001, 1, 1) + dt.timedelta(days=round(st.median(lasts))) if len(lasts) >= 20 else None
    first_frost = dt.date(2001, 7, 1) + dt.timedelta(days=round(st.median(firsts))) if len(firsts) >= 20 else None
    measures = [
        measure("frost_days_normal", round(frost, 1), "days", "Frost nights a season (normal)"),
        measure("frost_days_trend_per_decade", None if frost_trend is None else round(frost_trend, 2), "days per decade", "Frost nights trend"),
        measure("hard_spring_frost_days_normal", round(hard, 1), "days", "Hard spring frost nights (normal)"),
        measure("spring_heat_days_normal", round(heat, 1), "days", "Days of 31 C or more in April-May (normal)"),
        measure("spring_heat_days_trend_per_decade", None if heat_trend is None else round(heat_trend, 2), "days per decade", "Spring heat trend"),
        measure("last_spring_frost_month", last_frost and last_frost.month, "month", "Usual last spring frost: month"),
        measure("last_spring_frost_day", last_frost and last_frost.day, "day", "Usual last spring frost: day"),
        measure("first_autumn_frost_month", first_frost and first_frost.month, "month", "Usual first autumn frost: month"),
        measure("first_autumn_frost_day", first_frost and first_frost.day, "day", "Usual first autumn frost: day"),
        measure("hard_spring_frost_seasons", len(hard_seasons), "seasons", "Seasons with a hard spring frost"),
        measure("latest_hard_spring_frost_season", hard_seasons[-1] if hard_seasons else None, "season start year", "Latest hard spring frost"),
    ]
    dates = ""
    if last_frost and first_frost:
        dates = (
            f"The last spring frost usually falls around {last_frost:%m-%d} and the first autumn frost around "
            f"{first_frost:%m-%d} (month-day)."
        )
    if hard_seasons:
        named = ", ".join(season_name(s) for s in hard_seasons[-4:])
        hard_text = (
            f"Hard spring frost, which hurts heading wheat, came in: {named}."
            if len(hard_seasons) <= 4
            else f"Hard spring frost, which hurts heading wheat, came in {len(hard_seasons)} seasons, latest: {named}."
        )
    else:
        hard_text = f"No hard spring frost since {season_name(min(complete))}."
    sentences = [
        f"About {frost:.1f} frost nights a season.",
        dates,
        hard_text,
        f"About {heat:.1f} days of 31 C or more in April and May.",
    ]
    source = "ERA5-Land ~9 km, daily air temperature since 1981 (Open-Meteo); normal 1991-2020; hard frost <=-2 C 15 Mar-15 May"
    return topic_body(last_day.isoformat(), source, "likely", sentences, measures)


# ---------------------------------------------------------------- soil and land
def soilgrids(lat, lon, patient):
    """Topsoil (0-30 cm) clay %, sand %, organic carbon g/kg and pH, or None each.

    Asks the query service first, then the map service when the first is down or
    answers without values. An answer without any value is never cached, so the
    field is asked again on the next run. Raises OutsideError when both fail.
    """
    lat, lon = round(lat, 3), round(lon, 3)
    path = CACHE / "soilgrids" / f"{lat:.3f}_{lon:.3f}.json"
    cached = read_json(path)
    if cached is not None and any(value is not None for value in cached.values()):
        return cached
    try:
        result = soilgrids_query(lat, lon, patient)
    except OutsideError as error:
        log(f"  soil: {error}; trying the SoilGrids map service")
        result = None
    if result is None or all(value is None for value in result.values()):
        result = soilgrids_maps(lat, lon)
    if all(value is None for value in result.values()):
        raise OutsideError("SoilGrids has no soil values for this point")
    write_json(path, result)
    return result


def soilgrids_query(lat, lon, patient):
    query = [("lon", lon), ("lat", lat), ("value", "mean")]
    query += [("property", name) for name in SOIL_PROPERTIES]
    query += [("depth", depth) for depth in SOIL_DEPTHS]
    # The quick try keeps a slow SoilGrids from holding up a new farm's other topics.
    raw = fetch(f"{SOILGRIDS}?{urllib.parse.urlencode(query)}", what="soilgrids", timeout=60 if patient else 25, tries=2 if patient else 1, wait=10)
    try:
        layers = json.loads(raw)["properties"]["layers"]
        result = {}
        for layer in layers:
            total = weight = 0.0
            for depth in layer["depths"]:
                value = depth["values"].get("mean")
                thick = depth["range"]["bottom_depth"] - depth["range"]["top_depth"]
                if value is not None:
                    total += value * thick
                    weight += thick
            # Every depth must be there: a mean of part of the topsoil is not the topsoil.
            result[layer["name"]] = total / weight / layer["unit_measure"]["d_factor"] if weight == 30 else None
    except (KeyError, TypeError, ValueError) as error:
        raise OutsideError(f"soilgrids answered with an unexpected shape: {error}") from error
    return result


SOIL_PROPERTIES = ("clay", "sand", "soc", "phh2o")
SOIL_DEPTHS = ("0-5cm", "5-15cm", "15-30cm")
# Stored as whole numbers 10 times the unit: g/kg for clay and sand (so /10 = %),
# dg/kg for organic carbon (/10 = g/kg), pH x 10.
SOIL_D_FACTOR = 10


def soilgrids_maps(lat, lon):
    """The 0-30 cm means from the SoilGrids map service: the median of the 250 m
    pixels within about 300 m of the point, per depth, weighted by depth like the
    query service. A property with any depth missing (towns, water) is None."""
    half = 0.003
    result = {}
    for name in SOIL_PROPERTIES:
        total = weight = 0.0
        for depth in SOIL_DEPTHS:
            query = [
                ("map", f"/map/{name}.map"), ("SERVICE", "WCS"), ("VERSION", "2.0.1"), ("REQUEST", "GetCoverage"),
                ("COVERAGEID", f"{name}_{depth}_mean"), ("FORMAT", "image/tiff"),
                ("SUBSET", f"long({lon - half:.5f},{lon + half:.5f})"), ("SUBSET", f"lat({lat - half:.5f},{lat + half:.5f})"),
                ("SUBSETTINGCRS", "http://www.opengis.net/def/crs/EPSG/0/4326"),
                ("OUTPUTCRS", "http://www.opengis.net/def/crs/EPSG/0/4326"),
            ]
            try:
                raw = fetch(f"{SOILGRIDS_MAPS}?{urllib.parse.urlencode(query)}", what="soilgrids_maps", timeout=30, tries=2, wait=5)
                pixels, nodata = tiff_int16(raw)
            except (OutsideError, ValueError, struct.error, zlib.error) as error:
                log(f"  soil map {name} {depth}: {error}")
                pixels, nodata = [], None
            # 0 is what the maps hold where there is no soil (towns, water).
            good = [value for value in pixels if value != nodata and value > 0]
            if good:
                top, bottom = (int(part) for part in depth[:-2].split("-"))
                total += st.median(good) * (bottom - top)
                weight += bottom - top
        result[name] = total / weight / SOIL_D_FACTOR if weight == 30 else None
    return result


def tiff_int16(raw):
    """Pixel values and the no-data value of a small single-band int16 GeoTIFF,
    as the map service sends it (strips or tiles, deflate or none, predictor 1 or 2)."""
    if raw[:2] != b"II":
        raise ValueError("not a little-endian TIFF")
    start = struct.unpack("<I", raw[4:8])[0]
    count = struct.unpack("<H", raw[start:start + 2])[0]
    tags = {}
    for i in range(count):
        tag, kind, n, value = struct.unpack("<HHII", raw[start + 2 + 12 * i:start + 14 + 12 * i])
        tags[tag] = (kind, n, value)

    def ints(tag, default=None):
        if tag not in tags:
            return default
        kind, n, value = tags[tag]
        size = {3: 2, 4: 4}[kind]
        data = struct.pack("<I", value)[:n * size] if n * size <= 4 else raw[value:value + n * size]
        return list(struct.unpack("<" + ("H" if size == 2 else "I") * n, data))

    width, height = ints(256)[0], ints(257)[0]
    compression, predictor = ints(259, [1])[0], ints(317, [1])[0]
    if compression not in (1, 8):
        raise ValueError(f"TIFF compression {compression}")
    tiled = 324 in tags
    offsets, sizes = (ints(324), ints(325)) if tiled else (ints(273), ints(279))
    block_w = ints(322)[0] if tiled else width
    block_h = ints(323)[0] if tiled else ints(278, [height])[0]
    nodata = None
    if 42113 in tags:
        kind, n, value = tags[42113]
        text = raw[value:value + n] if n > 4 else struct.pack("<I", value)[:n]
        try:
            nodata = int(float(text.rstrip(b"\x00").decode()))
        except ValueError:
            nodata = None
    grid = {}
    across = (width + block_w - 1) // block_w
    for block, (offset, size) in enumerate(zip(offsets, sizes)):
        data = raw[offset:offset + size]
        if compression == 8:
            data = zlib.decompress(data)
        rows = len(data) // (2 * block_w)
        values = list(struct.unpack("<" + "h" * (rows * block_w), data[:rows * block_w * 2]))
        for r in range(rows):
            row = values[r * block_w:(r + 1) * block_w]
            if predictor == 2:
                for x in range(1, block_w):
                    row[x] = (row[x] + row[x - 1] + 32768) % 65536 - 32768
            for x in range(block_w):
                gx = (block % across) * block_w + x if tiled else x
                gy = (block // across) * block_h + r if tiled else block * block_h + r
                if gx < width and gy < height:
                    grid[(gx, gy)] = row[x]
    return [grid[(x, y)] for y in range(height) for x in range(width) if (x, y) in grid], nodata


def land(lat, lon):
    """Height at the centre and the mean slope of a 5x5 grid of heights 90 m apart."""
    lat, lon = round(lat, 4), round(lon, 4)
    path = CACHE / "elevation" / f"{lat:.4f}_{lon:.4f}.json"
    cached = read_json(path)
    if cached is not None:
        return cached
    step = 90.0
    dlat = step / 110570
    dlon = step / (111320 * math.cos(math.radians(lat)))
    points = [(lat + (2 - row) * dlat, lon + (col - 2) * dlon) for row in range(5) for col in range(5)]
    query = urllib.parse.urlencode(
        {"latitude": ",".join(f"{p[0]:.6f}" for p in points), "longitude": ",".join(f"{p[1]:.6f}" for p in points)}
    )
    try:
        heights = json.loads(fetch(f"{ELEVATION}?{query}", what="open-meteo elevation"))["elevation"]
        if len(heights) != 25 or any(h is None or h != h for h in heights):
            raise ValueError("a height is missing")
    except (KeyError, TypeError, ValueError) as error:
        raise OutsideError(f"open-meteo elevation answered with an unexpected shape: {error}") from error
    z = lambda row, col: heights[row * 5 + col]
    slopes = []
    for row in range(1, 4):
        for col in range(1, 4):
            # Horn's method on the 3x3 heights around the point.
            dzdx = ((z(row - 1, col + 1) + 2 * z(row, col + 1) + z(row + 1, col + 1)) - (z(row - 1, col - 1) + 2 * z(row, col - 1) + z(row + 1, col - 1))) / (8 * step)
            dzdy = ((z(row - 1, col - 1) + 2 * z(row - 1, col) + z(row - 1, col + 1)) - (z(row + 1, col - 1) + 2 * z(row + 1, col) + z(row + 1, col + 1))) / (8 * step)
            slopes.append(math.degrees(math.atan(math.hypot(dzdx, dzdy))))
    result = {"elevation_m": z(2, 2), "slope_deg": st.mean(slopes)}
    write_json(path, result)
    return result


def soil_topic(lat, lon, today, patient):
    """Returns (body, complete). Not complete when SoilGrids did not answer:
    the body then holds the height and slope alone."""
    ground = land(lat, lon)
    try:
        soil = soilgrids(lat, lon, patient)
    except OutsideError as error:
        log(f"  soil: {error}")
        soil = None
    get = lambda name: None if soil is None or soil.get(name) is None else soil[name]
    clay, sand, carbon, ph = get("clay"), get("sand"), get("soc"), get("phh2o")
    measures = [
        measure("clay_pct_topsoil", clay and round(clay, 1), "%", "Clay in the topsoil"),
        measure("sand_pct_topsoil", sand and round(sand, 1), "%", "Sand in the topsoil"),
        measure("organic_carbon_g_kg_topsoil", carbon and round(carbon, 1), "g/kg", "Organic carbon in the topsoil"),
        measure("ph_topsoil", ph and round(ph, 1), "pH", "Topsoil pH"),
        measure("elevation_m", round(ground["elevation_m"], 1), "m", "Height above sea level"),
        measure("slope_deg", round(ground["slope_deg"], 2), "deg", "Average slope"),
    ]
    parts = [
        clay is not None and f"clay {clay:.1f}%",
        sand is not None and f"sand {sand:.1f}%",
        carbon is not None and f"organic carbon {carbon:.1f} g/kg",
        ph is not None and f"pH {ph:.1f}",
    ]
    parts = [part for part in parts if part]
    land_source = "Copernicus DEM 90 m (Open-Meteo): slope = mean of a 5x5 grid of heights 90 m apart"
    sentences = [
        f"Topsoil (modelled, 250 m): {', '.join(parts)}." if parts else "",
        f"Height {ground['elevation_m']:.0f} m, slope {ground['slope_deg']:.2f} degrees on average.",
    ]
    source = f"SoilGrids 2.0 250 m (modelled, not sampled) 0-30 cm; {land_source}" if parts else land_source
    if len(source) > SOURCE_LIMIT:
        source = "SoilGrids 2.0 250 m (modelled, not sampled) 0-30 cm; Copernicus DEM 90 m (Open-Meteo), slope from a 5x5 grid"
    return topic_body(today.isoformat(), source, "unsure", sentences, measures), soil is not None


# ---------------------------------------------------------------- satellite pictures
class Box:
    """The square around the farm centre that has the farm's area."""

    def __init__(self, lat, lon, area_dunam):
        self.lat, self.lon = lat, lon
        self.side_m = max(20.0, math.sqrt(max(area_dunam, 0.0) * 2500.0))
        half = self.side_m / 2
        dlat = half / 110570
        dlon = half / (111320 * math.cos(math.radians(lat)))
        self.bounds = (round(lon - dlon, 6), round(lat - dlat, 6), round(lon + dlon, 6), round(lat + dlat, 6))
        self.n10 = max(2, min(MAX_GRID, round(self.side_m / 10)))
        self.n30 = max(2, min(MAX_GRID, round(self.side_m / 30)))
        self.key = f"{lat:.5f}_{lon:.5f}_{self.side_m:.0f}"

    def pixel_m(self, n):
        return self.side_m / n


def parse_npy(raw):
    if raw[:6] != b"\x93NUMPY":
        raise OutsideError("planetary computer sent something that is not a picture: " + raw[:80].decode("latin1"))
    if raw[6] == 1:
        length, start = struct.unpack("<H", raw[8:10])[0], 10
    else:
        length, start = struct.unpack("<I", raw[8:12])[0], 12
    header = ast.literal_eval(raw[start : start + length].decode("latin1"))
    try:
        code = {"<u2": "H", "<i2": "h", "|u1": "B", "<f4": "f", "<f8": "d", "<i4": "i", "<u4": "I"}[header["descr"]]
    except KeyError as error:
        raise OutsideError(f"picture has an unexpected number type {header['descr']}") from error
    values = array.array(code)
    values.frombytes(raw[start + length :])
    return header["shape"], values


def crop(collection, item, box, n, assets):
    bounds = ",".join(str(v) for v in box.bounds)
    names = "".join(f"&assets={name}" for name in assets)
    url = f"{DATA}/item/bbox/{bounds}/{n}x{n}.npy?collection={collection}&item={item}{names}"
    shape, values = parse_npy(fetch(url, what="planetary computer picture", timeout=90))
    if len(shape) != 3 or shape[0] < 3 or shape[1] * shape[2] * shape[0] != len(values):
        raise OutsideError(f"picture {item} has an unexpected shape {shape}")
    return shape[0], shape[1] * shape[2], values


def s2_picture(item, box):
    """NDVI of every pixel of the box (None where not clear) and the clear share."""
    bands, count, a = crop("sentinel-2-l2a", item["id"], box, box.n10, ("B04", "B08", "SCL"))
    offset = 1000 if item["baseline"] >= 4.0 else 0
    if offset:
        # A few items carry no +1000 offset despite their baseline (seen in the
        # backtest): then red under 1000 is common on land, and no offset is taken.
        land_pixels = [i for i in range(count) if a[2 * count + i] in (4, 5)]
        if land_pixels and sum(1 for i in land_pixels if a[i] < 1000) / len(land_pixels) > 0.05:
            offset = 0
    grid, good = [None] * count, 0
    for i in range(count):
        if bands >= 4 and a[3 * count + i] == 0:
            continue
        if a[2 * count + i] not in S2_VALID_SCL:
            continue
        red, nir = a[i] - offset, a[count + i] - offset
        if red + nir <= 0:
            continue
        grid[i] = max(-1.0, min(1.0, (nir - red) / (nir + red)))
        good += 1
    return grid, good / count


def landsat_picture(item, box):
    bands, count, a = crop("landsat-c2-l2", item["id"], box, box.n30, ("red", "nir08", "qa_pixel"))
    grid, good = [None] * count, 0
    for i in range(count):
        if bands >= 4 and a[3 * count + i] == 0:
            continue
        quality = a[2 * count + i]
        # Clear bit set; no fill, dilated cloud, cirrus, cloud, shadow or snow.
        if not quality & 0x40 or quality & 0x3F:
            continue
        red, nir = a[i] * 0.0000275 - 0.2, a[count + i] * 0.0000275 - 0.2
        if red <= 0 or nir <= 0 or red > 1 or nir > 1:
            continue
        grid[i] = (nir - red) / (nir + red)
        good += 1
    return grid, good / count


def mean_of(grid):
    values = [v for v in grid if v is not None]
    return st.mean(values) if values else None


def search(collection, box, start, end, max_cloud, fields):
    body = {
        "collections": [collection],
        "intersects": {"type": "Point", "coordinates": [box.lon, box.lat]},
        "datetime": f"{start}T00:00:00Z/{end}T23:59:59Z",
        "query": {"eo:cloud_cover": {"lt": max_cloud}},
        "limit": 1000,
        "fields": {
            "include": ["id", "properties.datetime", "properties.eo:cloud_cover", "properties.platform"] + [f"properties.{f}" for f in fields],
            "exclude": ["assets", "links", "geometry", "bbox", "stac_extensions", "collection", "type", "stac_version"],
        },
    }
    found = []
    for _ in range(20):
        try:
            page = json.loads(fetch(STAC, body, what="planetary computer search", timeout=120))
        except ValueError as error:
            raise OutsideError(f"planetary computer search answered with something unreadable: {error}") from error
        found += page.get("features", [])
        following = [link for link in page.get("links", []) if link.get("rel") == "next"]
        if not following or not page.get("features") or not following[0].get("body"):
            break
        body = following[0]["body"]
    return found


def s2_items(box, start, end):
    """Sentinel-2 passes over the centre, one per day, from the tile seen most."""
    items = []
    for feature in search("sentinel-2-l2a", box, start, end, S2_MAX_CLOUD, ["s2:mgrs_tile", "s2:processing_baseline"]):
        p = feature["properties"]
        items.append(
            {
                "id": feature["id"],
                "date": p["datetime"][:10],
                "cloud": p.get("eo:cloud_cover", 99),
                "tile": p.get("s2:mgrs_tile"),
                "baseline": float(p.get("s2:processing_baseline", "0") or 0),
            }
        )
    return items


def one_per_day(items, tile_key):
    """Keeps one item per day: from the tile (or path/row) seen most, newest processing."""
    seen = {}
    for item in items:
        seen[item[tile_key]] = seen.get(item[tile_key], 0) + 1
    by_day = {}
    for item in items:
        best = by_day.get(item["date"])
        rank = (seen[item[tile_key]], item["id"])
        if best is None or rank > (seen[best[tile_key]], best["id"]):
            by_day[item["date"]] = item
    return sorted(by_day.values(), key=lambda item: item["date"])


def landsat_items(box, start, end):
    items = []
    for feature in search("landsat-c2-l2", box, start, end, LANDSAT_MAX_CLOUD, ["landsat:wrs_path", "landsat:wrs_row"]):
        p = feature["properties"]
        items.append(
            {
                "id": feature["id"],
                "date": p["datetime"][:10],
                "cloud": p.get("eo:cloud_cover", 99),
                "tile": f"{p.get('landsat:wrs_path')}/{p.get('landsat:wrs_row')}",
            }
        )
    return one_per_day(items, "tile")


# ---------------------------------------------------------------- season history
def window_of(item_date):
    """('spring' | 'summer' | None, season start year) of a picture's day."""
    day = dt.date.fromisoformat(item_date)
    if 2 <= day.month <= 5:
        return "spring", day.year - 1
    if day.month in (7, 8):
        return "summer", day.year - 1
    return None, None


def first_clear(candidates, reader, box):
    """The first candidate with enough of the box clear, as (item, grid), else None.
    Raises OutsideError when a picture could not be read at all."""
    for item in candidates:
        grid, share = reader(item, box)
        if share >= MIN_VALID:
            return item, grid
    return None


def read_seasons(box, items, reader, picks, keep_pixels):
    """Reads the chosen pictures of each season. Returns {start year: record},
    and the start years where a picture could not be read (not stored)."""
    jobs = []  # (season, window, candidates)
    for season, windows in picks(items).items():
        for window, groups in windows.items():
            for candidates in groups:
                jobs.append((season, window, candidates))

    def run(job):
        try:
            return job, first_clear(job[2], reader, box), None
        except OutsideError as error:
            return job, None, error

    seasons, broken = {}, set()
    with cf.ThreadPoolExecutor(POOL) as pool:
        for (season, window, _), found, error in pool.map(run, jobs):
            record = seasons.setdefault(season, {"spring": [], "summer": [], "px": None})
            if error is not None:
                broken.add(season)
                continue
            if found is None:
                continue
            item, grid = found
            record[window].append([item["date"], round(mean_of(grid), 4)])
            if window == "spring" and keep_pixels:
                old = record["px"] or [None] * len(grid)
                record["px"] = [
                    v if o is None else o if v is None else max(o, v) for o, v in zip(old, grid)
                ]
    for season, record in seasons.items():
        record["spring"].sort()
        record["summer"].sort()
        if record["px"]:
            record["px"] = [None if v is None else round(v, 3) for v in record["px"]]
    return {s: r for s, r in seasons.items() if s not in broken}, broken


def s2_picks(items):
    """Per season: the clearest pass of every 10 days of February-May and every
    15 days of July-August, with one spare each."""
    bins = {}
    for item in items:
        window, season = window_of(item["date"])
        if window is None:
            continue
        day = dt.date.fromisoformat(item["date"])
        start = dt.date(day.year, 2 if window == "spring" else 7, 1)
        slot = (day - start).days // (10 if window == "spring" else 15)
        bins.setdefault(season, {}).setdefault(window, {}).setdefault(slot, []).append(item)
    return {
        season: {window: [sorted(group, key=lambda i: i["cloud"])[:2] for group in slots.values()] for window, slots in windows.items()}
        for season, windows in bins.items()
    }


def landsat_picks(items):
    """Per season: the 8 clearest spring scenes and the 3 clearest of July-August."""
    groups = {}
    for item in items:
        window, season = window_of(item["date"])
        if window is not None:
            groups.setdefault(season, {}).setdefault(window, []).append(item)
    return {
        season: {
            window: [[item] for item in sorted(found, key=lambda i: i["cloud"])[: 8 if window == "spring" else 3]]
            for window, found in windows.items()
        }
        for season, windows in groups.items()
    }


def season_closed(season, today):
    return today > dt.date(season + 1, 9, 10)


def season_history(box, today, with_landsat=True):
    """Every season's spring and summer greenness, kept per box. A season that
    is over is read once and never again."""
    path = CACHE / "history" / f"{box.key}.json"
    stored = read_json(path) or {"seasons": {}}
    seasons = stored["seasons"]
    current = today.year if today.month >= 10 else today.year - 1
    failed = set()

    have_s2 = [int(s) for s, r in seasons.items() if r["sensor"] == "s2"]
    s2_from = max(have_s2) + 1 if have_s2 else S2_FIRST_SPRING - 1
    if s2_from <= current:
        start = dt.date(s2_from + 1, 2, 1)
        if start <= today:
            items = one_per_day(s2_items(box, start.isoformat(), today.isoformat()), "tile")
            fresh, broken = read_seasons(box, items, s2_picture, s2_picks, keep_pixels=True)
            failed |= broken
            for season, record in fresh.items():
                record.update(sensor="s2", closed=season_closed(season, today))
                seasons[str(season)] = record
    if with_landsat and not stored.get("landsat_done"):
        items = landsat_items(box, f"{LANDSAT_FIRST_SPRING}-01-01", f"{S2_FIRST_SPRING - 1}-09-30")
        fresh, broken = read_seasons(box, items, landsat_picture, landsat_picks, keep_pixels=False)
        failed |= broken
        for season, record in fresh.items():
            record.update(sensor="landsat", closed=True)
            seasons[str(season)] = record
        stored["landsat_done"] = not broken

    # Only finished seasons are kept: an open one is read again on the next weekly round.
    write_json(path, {"seasons": {s: r for s, r in seasons.items() if r["closed"]}, "landsat_done": stored.get("landsat_done", False)})
    if failed:
        raise OutsideError(f"pictures of {len(failed)} seasons could not be read; the seasons that were read are kept")
    return {int(s): r for s, r in seasons.items()}


def measured(record):
    """A season's spring peak counts with two clear spring pictures, one of them
    between 1 March and 15 May (when winter crops peak here)."""
    spring = record["spring"]
    return len(spring) >= 2 and any("03-01" <= day[5:] <= "05-15" for day, _ in spring)


def history_numbers(seasons):
    raw = {s: max(v for _, v in r["spring"]) for s, r in seasons.items() if measured(r)}
    if not raw:
        return None
    # Landsat at 30 m reads this NDVI lower than Sentinel-2 at 10 m (on the test
    # field its median peak was 0.55 against 0.77). So every season is compared
    # with the median of its own satellite, and a Landsat peak is scaled to the
    # Sentinel-2 level by the ratio of the two medians. This hides any real
    # change between the two periods, which is why no trend is pushed.
    reference = {}
    for sensor in ("s2", "landsat"):
        own = [peak for s, peak in raw.items() if seasons[s]["sensor"] == sensor]
        reference[sensor] = st.median(own) if len(own) >= 5 else st.median(raw.values())
    normal = reference[seasons[max(raw)]["sensor"]]
    peaks = {s: peak / reference[seasons[s]["sensor"]] * normal for s, peak in raw.items()}
    ranked = sorted(peaks, key=lambda s: -peaks[s])
    out = {
        "peaks": peaks,
        "normal": normal,
        "last": max(peaks),
        "best": ranked[:3],
        "worst": ranked[::-1][:3] if len(ranked) >= 6 else [],
        "pictures_s2": sum((len(r["spring"]) if s in raw else 0) + len(r["summer"]) for s, r in seasons.items() if r["sensor"] == "s2"),
        "pictures_landsat": sum((len(r["spring"]) if s in raw else 0) + len(r["summer"]) for s, r in seasons.items() if r["sensor"] == "landsat"),
        "normal_of": "Sentinel-2" if seasons[max(raw)]["sensor"] == "s2" else "Landsat",
        "last_picture": max(day for r in seasons.values() for day, _ in r["spring"] + r["summer"]),
        "first_s2": min((s for s in peaks if seasons[s]["sensor"] == "s2"), default=None),
        "first_landsat": min((s for s in peaks if seasons[s]["sensor"] == "landsat"), default=None),
    }
    # Summer: green when the lowest July-August picture is still at 0.25 or more.
    summers = {s: min(v for _, v in r["summer"]) for s, r in seasons.items() if r["summer"]}
    out["summers_seen"] = len(summers)
    out["summers_green"] = sum(1 for v in summers.values() if v >= T_SUMMER)
    # Weak spots, on 10 m pixels only: a pixel whose own spring peak stays under
    # 70% of the field's median pixel in at least 8 of 10 seasons with a crop.
    counted, weak_counts, total = 0, None, 0
    for season, record in seasons.items():
        if record["sensor"] != "s2" or season not in peaks or not record.get("px"):
            continue
        values = [v for v in record["px"] if v is not None]
        if len(values) < len(record["px"]) or st.median(values) < WEAK_SEASON_MIN_PEAK:
            continue
        median = st.median(values)
        weak_counts = weak_counts or [0] * len(values)
        total = len(values)
        counted += 1
        for i, value in enumerate(values):
            if value < WEAK_BELOW * median:
                weak_counts[i] += 1
    if counted >= 3:
        out["weak_seasons"] = counted
        out["weak_share"] = 100 * sum(1 for c in weak_counts if c >= WEAK_IN_SHARE * counted) / total
    return out


# ---------------------------------------------------------------- the newest picture
def stage_of(ndvi, day):
    """0 bare, 1 coming up, 2 growing, 3 dense, 4 after the season. A rule of
    thumb on field_eye.py's greenness bands (0.2, 0.35, 0.6) and the calendar."""
    after_harvest = 6 <= day.month <= 9
    if ndvi < 0.2:
        return 4 if after_harvest else 0
    if ndvi < 0.35:
        if after_harvest:
            return 4
        return 1 if day.month >= 10 or day.month <= 2 else 2
    return 2 if ndvi < 0.6 else 3


def newest_picture(box, today, known):
    """The newest clear Sentinel-2 picture against the same weeks of earlier
    years. `known` is the last result; it is returned unchanged when no newer
    clear picture exists. None when no picture of the last 30 days is clear."""
    pictures_path = CACHE / "pictures" / f"{box.key}.json"
    pictures = read_json(pictures_path) or {}  # item id -> [mean NDVI or None, clear share]
    start = today - dt.timedelta(days=NOW_LOOKBACK_DAYS)
    recent = one_per_day(s2_items(box, start.isoformat(), today.isoformat()), "tile")
    chosen = None
    for item in sorted(recent, key=lambda i: i["date"], reverse=True)[:6]:
        if known and known.get("item") == item["id"]:
            return known
        if item["id"] in pictures and pictures[item["id"]][1] < MIN_VALID:
            continue
        grid, share = s2_picture(item, box)
        pictures[item["id"]] = [mean_of(grid), round(share, 3)]
        if share >= MIN_VALID:
            chosen = (item, grid, share)
            break
    write_json(pictures_path, pictures)
    if chosen is None:
        return None
    item, grid, share = chosen
    day = dt.date.fromisoformat(item["date"])
    now = mean_of(grid)
    result = {"item": item["id"], "date": item["date"], "ndvi": round(now, 4), "clear": round(share, 3), "stage": stage_of(now, day)}

    def year_value(year):
        middle = dt.date(year, day.month, min(day.day, 28))
        window = dt.timedelta(days=NOW_WINDOW_DAYS)
        found = one_per_day(s2_items(box, (middle - window).isoformat(), (middle + window).isoformat()), "tile")
        for old in sorted(found, key=lambda i: i["cloud"])[:3]:
            if old["id"] not in pictures:
                old_grid, old_share = s2_picture(old, box)
                pictures[old["id"]] = [mean_of(old_grid), round(old_share, 3)]
            value, old_share = pictures[old["id"]]
            if value is not None and old_share >= MIN_VALID:
                return year, value
        return year, None

    years = range(max(S2_FIRST_SPRING, day.year - 10), day.year)
    with cf.ThreadPoolExecutor(POOL) as pool:
        history = [(year, value) for year, value in pool.map(year_value, years) if value is not None]
    write_json(pictures_path, pictures)
    if len(history) >= NOW_MIN_YEARS:
        normal = st.mean(value for _, value in history)
        result.update(normal=round(normal, 4), years=len(history), pct=round(100 * now / max(normal, 0.05)))
    # Inside the field (field_eye.py): pixels under 70% of the field's median.
    cells = [(i, v) for i, v in enumerate(grid) if v is not None]
    median = st.median(v for _, v in cells)
    weak = [i for i, v in cells if v < WEAK_BELOW * median] if median > WEAK_MIN_MEDIAN else []
    result["behind"] = round(100 * len(weak) / len(cells), 1)
    if weak:
        n, pixel = box.n10, box.pixel_m(box.n10)
        result["dx"] = round((st.mean(i % n for i in weak) + 0.5 - n / 2) * pixel, 1)
        result["dy"] = round((n / 2 - (st.mean(i // n for i in weak) + 0.5)) * pixel, 1)
    return result


def greenness_topic(box, numbers, now):
    if numbers is None and now is None:
        raise NothingToPush("no clear picture in the last 30 days and no season measured yet")
    measures, sentences = [], []
    if numbers:
        peaks, last = numbers["peaks"], numbers["last"]
        pct = 100 * peaks[last] / numbers["normal"]
        measures += [
            measure("seasons_measured", len(peaks), "seasons", "Seasons measured from space"),
            measure("normal_peak_ndvi", round(numbers["normal"], 3), "NDVI", "Normal spring peak greenness"),
            measure("last_season_pct_of_normal", round(pct), "%", f"Last season ({season_name(last)}) vs normal"),
            measure("weak_share_pct", round(numbers["weak_share"], 1) if "weak_share" in numbers else None, "%", "Share of the field weak almost every season"),
            measure("weak_spots_10m_seasons", numbers.get("weak_seasons"), "seasons", "Seasons with 10 m pictures used for weak spots"),
            measure("pictures_landsat", numbers["pictures_landsat"] or None, "pictures", "Clear Landsat pictures used"),
            measure("pictures_sentinel2", numbers["pictures_s2"] or None, "pictures", "Clear Sentinel-2 pictures used"),
        ]
        for place, season in enumerate(numbers["best"], start=1):
            measures.append(measure(f"best_season_{place}", season, "season start year", f"Greenest season, place {place}"))
        sentences += [
            f"Seen from space for {len(peaks)} seasons ({season_name(min(peaks))} to {season_name(max(peaks))}).",
            f"Normal spring peak NDVI {numbers['normal']:.3f} ({numbers['normal_of']} seasons).",
            f"Best seasons: {', '.join(season_name(s) for s in numbers['best'])}.",
            f"Last season {season_name(last)}: {pct:.0f}% of normal.",
            f"Worst: {', '.join(season_name(s) for s in numbers['worst'])}." if numbers["worst"] else "",
        ]
        if "weak_share" in numbers:
            sentences.append(
                f"{numbers['weak_share']:.0f}% of the field stays behind the rest almost every season. "
                f"Measured on 10 m pixels over {numbers['weak_seasons']} seasons."
            )
    if now:
        day = dt.date.fromisoformat(now["date"])
        measures += [
            measure("now_picture_year", day.year, "year", "Latest clear picture: year"),
            measure("now_picture_month", day.month, "month", "Latest clear picture: month"),
            measure("now_picture_day", day.day, "day", "Latest clear picture: day"),
            measure("now_ndvi", round(now["ndvi"], 3), "NDVI", "Greenness in the latest clear picture"),
            measure("now_stage", now["stage"], "stage", "Now: 0 bare, 1 coming up, 2-3 growing, 4 after season"),
            measure("now_normal_ndvi", now.get("normal") and round(now["normal"], 3), "NDVI", "Usual greenness in these weeks, earlier years"),
            measure("now_pct_of_normal", now.get("pct"), "%", "Greenness now vs the same weeks of earlier years"),
            measure("now_behind_share_pct", now["behind"], "%", "Share of the field behind the rest now"),
            measure("now_behind_dx_m", now.get("dx"), "m", "Behind part: metres east of the centre"),
            measure("now_behind_dy_m", now.get("dy"), "m", "Behind part: metres north of the centre"),
        ]
        sentences.append(f"Latest clear picture {now['date']}: NDVI {now['ndvi']:.2f}.")
    side = f"{box.side_m:.0f} m box at centre"
    if numbers and numbers["first_landsat"] is not None:
        source = f"Sentinel-2 10 m (2016-), Landsat 30 m (1984-2015) NDVI; spring peak vs median of the same satellite; {side}"
    elif numbers:
        source = f"Sentinel-2 10 m NDVI, spring peak vs own median; {side}, not the outline"
    else:
        source = f"Sentinel-2 10 m NDVI, newest clear picture vs same weeks of earlier years; {side}, not the outline"
    as_of = max(([now["date"]] if now else []) + ([numbers["last_picture"]] if numbers else []))
    return topic_body(as_of, source, "likely", sentences, measures)


# ---------------------------------------------------------------- dryness
def km_between(lat1, lon1, lat2, lon2):
    return math.hypot((lon2 - lon1) * 111.32 * math.cos(math.radians(lat1)), (lat2 - lat1) * 110.57)


def dryness_topic(box, rows, numbers, fires):
    """Rain against the spring peak over the measured seasons, summer greenness
    and the stored fires. Not pushed under 5 seasons with both rain and a peak."""
    if numbers is None:
        raise NothingToPush("no season measured from space yet")
    rain = rain_seasons(rows)
    normal = rain_normal(rain)
    both = {s: (100 * rain[s] / normal, peak) for s, peak in numbers["peaks"].items() if s in rain}
    if len(both) < 5:
        raise NothingToPush(f"only {len(both)} seasons have both rain and a spring peak; 5 are needed")
    r = pearson(list(both.values()))
    dry = [peak for pct, peak in both.values() if pct < DROUGHT_BELOW_PCT]
    wet = [peak for pct, peak in both.values() if pct >= WET_FROM_PCT]
    how = "closely" if r is not None and r >= 0.6 else "partly" if r is not None and r >= 0.3 else "hardly"
    measures = [
        measure("rain_green_r", r and round(r, 2), "r", "How closely the crop follows the rain"),
        measure("peak_ndvi_in_droughts", round(st.mean(dry), 3) if dry else None, "NDVI", "Spring peak in drought seasons"),
        measure("peak_ndvi_in_wet_seasons", round(st.mean(wet), 3) if wet else None, "NDVI", "Spring peak in wet seasons"),
    ]
    sentences = [f"Over {len(both)} seasons the crop follows the rain {how} (r = {r:.2f})." if r is not None else ""]
    if dry and wet:
        sentences.append(
            f"Spring peak in {len(dry)} drought seasons {st.mean(dry):.3f}, in {len(wet)} wet seasons {st.mean(wet):.3f}."
        )
    if numbers["summers_seen"]:
        measures += [
            measure("summer_green_seasons", numbers["summers_green"], "seasons", "Seasons green in summer"),
            measure("summer_seasons_seen", numbers["summers_seen"], "seasons", "Summers seen from space"),
        ]
        sentences.append(f"Green in summer in {numbers['summers_green']} of {numbers['summers_seen']} seasons.")
    fire_part = ""
    if fires is not None:
        near = sum(1 for fire in fires if km_between(box.lat, box.lon, fire["lat"], fire["lon"]) <= FIRE_RADIUS_KM)
        # Its own code: the app reads `fire_detections` as the whole satellite
        # record since 2000, and the backend keeps only the last week on show.
        measures.append(measure("fire_detections_7d", near, "count", "Fires seen within about 1 km, last 7 days"))
        sentences.append(f"Fires stored within about 1 km in the last 7 days: {near}.")
        fire_part = "; fires stored 7 d"
    last_day = max(day for day, value, _, _ in rows if value is not None)
    source = f"Derived: ERA5 Oct-May rain vs spring peak NDVI (Landsat scaled to Sentinel-2); wet >=115%, dry <80%{fire_part}"
    confidence = "likely" if len(both) >= 15 else "unsure"
    return topic_body(last_day.isoformat(), source, confidence, sentences, measures)


# ---------------------------------------------------------------- the run
class Run:
    def __init__(self, dry_run, force):
        self.dry_run, self.force = dry_run, force
        self.started = dt.datetime.now(dt.timezone.utc)
        self.clock = time.time()
        self.pushed = self.failed = self.skipped = self.waiting = 0
        self.state = read_json(CACHE / "state.json") or {}
        self.fires = None
        self.fires_asked = False
        self.notes = []

    def now(self):
        return dt.datetime.now(dt.timezone.utc)

    def age_s(self, stamp):
        if not stamp:
            return float("inf")
        return (self.now() - dt.datetime.fromisoformat(stamp)).total_seconds()

    def save(self):
        if not self.dry_run:
            write_json(CACHE / "state.json", self.state)

    def over_budget(self):
        return time.time() - self.clock > BUDGET_S

    def stored_fires(self):
        """The backend's fires of the last 7 days, asked once per run. None when
        the backend did not answer: then the fire figure is left out."""
        if not self.fires_asked:
            self.fires_asked = True
            try:
                self.fires = backend("GET", f"/fires?hours={FIRE_HOURS}").get("fires", [])
            except (urllib.error.URLError, OSError, ValueError) as error:
                log(f"  the stored fires could not be read ({error}); the fire figure is left out")
        return self.fires

    def push(self, farm, topic, body):
        what = f"farm {farm['id']} {topic}: {len(body['measures'])} measures, as of {body['as_of']}"
        if self.dry_run:
            log(f"would push {what}")
            print(json.dumps(body, indent=1, ensure_ascii=False), flush=True)
            self.pushed += 1
            return
        backend("PUT", f"/ingest/farms/{farm['id']}/insights/{topic}", body)
        self.pushed += 1
        log(f"pushed {what} ({time.time() - self.clock:.0f} s into the run)")

    def attempt(self, farm, topic, work, key=None):
        """Runs one piece of work (`key` names it in the state file when it is
        not a whole topic). A failure is logged, counted and backed off; it
        never stops the other topics or farms."""
        key = key or topic
        mine = self.state.setdefault(str(farm["id"]), {})
        try:
            work()
            mine.pop(f"fail_{key}", None)
            return True
        except NothingToPush as reason:
            log(f"farm {farm['id']} {topic}: not pushed, {reason}")
            self.skipped += 1
            mine[key] = {"at": self.now().isoformat(), "nothing": str(reason)}
        except urllib.error.HTTPError as error:
            log(f"farm {farm['id']} {topic}: backend refused with {error.code}: {error.read()[:200]!r}")
            self.fail(mine, key)
        except (OutsideError, OSError, ValueError, KeyError, TypeError, ZeroDivisionError, st.StatisticsError) as error:
            log(f"farm {farm['id']} {topic}: failed, {type(error).__name__}: {error}")
            self.fail(mine, key)
        finally:
            self.save()
        return False

    def fail(self, mine, key):
        self.failed += 1
        before = mine.get(f"fail_{key}", {}).get("n", 0)
        wait = min(RETRY_MAX_S, RETRY_FIRST_S * 2**before) - 60
        mine[f"fail_{key}"] = {"n": before + 1, "retry": (self.now() + dt.timedelta(seconds=wait)).isoformat()}

    def backing_off(self, mine, key):
        retry = mine.get(f"fail_{key}", {}).get("retry")
        return bool(retry) and not self.force and dt.datetime.fromisoformat(retry) > self.now()

    def due(self, farm):
        """Which pieces of work the farm needs now."""
        mine = self.state.get(str(farm["id"]), {})
        place = f"{farm['lat']:.5f},{farm['lon']:.5f},{farm['area_dunam']:.2f}"
        if mine.get("place") != place:
            mine = self.state[str(farm["id"])] = {"place": place}
        have = {one["topic"] for one in farm.get("topics", [])}
        old = lambda key, limit: self.force or self.age_s(mine.get(key, {}).get("at")) >= limit
        work = set()
        for topic in ("rain", "weather"):
            if topic not in have or old(topic, DAILY_S):
                work.add(topic)
        if "soil" not in have or (not mine.get("soil", {}).get("complete", True) and old("soil", RETRY_FIRST_S)):
            work.add("soil")
        # Work that had nothing to push is looked at again at its refresh age, not every run.
        missing = lambda topic, key: topic not in have and "nothing" not in mine.get(key, {})
        if missing("greenness", "now") or old("now", DAILY_S):
            work.add("now")
        if old("history", WEEKLY_S):
            work.add("history")
        if missing("dryness", "dryness") or old("dryness", DAILY_S):
            work.add("dryness")
        for key in list(work):
            if self.backing_off(mine, key):
                work.discard(key)
                self.waiting += 1
        return work


def work_on(run, farms, today):
    plans = {farm["id"]: run.due(farm) for farm in farms}
    busy = [farm for farm in farms if plans[farm["id"]]]
    if not busy:
        return 0
    log(f"{len(busy)} of {len(farms)} farms have work: " + ", ".join(f"{f['id']} ({' '.join(sorted(plans[f['id']]))})" for f in busy[:20]))
    series = {}

    def rows_for(farm):
        if farm["id"] not in series:
            series[farm["id"]] = rows_of(weather_days(farm["lat"], farm["lon"], today))
        return series[farm["id"]]

    stamp = lambda farm, key, **more: run.state[str(farm["id"])].__setitem__(key, {"at": run.now().isoformat(), **more})

    soil_later = []

    # Round 1, quick: rain, weather and soil for every farm, so a new farm has
    # them within a couple of minutes however long the satellite work takes.
    for farm in busy:
        plan = plans[farm["id"]]
        for topic, build in (("rain", rain_topic), ("weather", weather_topic)):
            if topic in plan:
                def quick(topic=topic, build=build):
                    run.push(farm, topic, build(rows_for(farm)))
                    stamp(farm, topic)
                run.attempt(farm, topic, quick)
        if "soil" in plan:
            def soil():
                body, complete = soil_topic(farm["lat"], farm["lon"], today, patient=False)
                if complete or not any(one["topic"] == "soil" for one in farm.get("topics", [])):
                    run.push(farm, "soil", body)
                stamp(farm, "soil", complete=complete)
                if not complete:
                    log(f"farm {farm['id']} soil: SoilGrids did not answer; height and slope are stored, the soil map is asked again at the end of the run")
                    soil_later.append(farm)
            run.attempt(farm, "soil", soil)

    # Round 2: the newest picture of every farm.
    boxes = {farm["id"]: Box(farm["lat"], farm["lon"], farm["area_dunam"]) for farm in busy}
    history_path = lambda farm: CACHE / "history" / f"{boxes[farm['id']].key}.json"

    def kept_numbers(farm):
        stored = read_json(history_path(farm))
        return history_numbers({int(s): r for s, r in stored["seasons"].items()}) if stored and stored["seasons"] else None

    for farm in busy:
        if "now" not in plans[farm["id"]]:
            continue
        if run.over_budget():
            run.notes.append("ran out of time; the rest is left for the next run")
            break

        def now():
            mine = run.state[str(farm["id"])]
            known = mine.get("now", {}).get("result")
            result = newest_picture(boxes[farm["id"]], today, known)
            have = {one["topic"] for one in farm.get("topics", [])}
            if result is None and "greenness" in have:
                stamp(farm, "now", result=known)
                log(f"farm {farm['id']} greenness: no clear picture in the last {NOW_LOOKBACK_DAYS} days, what is stored stays")
                return
            if result is not None and known is not None and result.get("item") == known.get("item") and "greenness" in have and not run.force:
                stamp(farm, "now", result=result)
                log(f"farm {farm['id']} greenness: no newer clear picture than {result['date']}, nothing pushed")
                return
            result = result or known
            run.push(farm, "greenness", greenness_topic(boxes[farm["id"]], kept_numbers(farm), result))
            stamp(farm, "now", result=result)
        run.attempt(farm, "greenness", now, key="now")

    # Round 3, slow: the season history, then what is derived from it.
    for farm in busy:
        plan = plans[farm["id"]]
        if not plan & {"history", "dryness"}:
            continue
        if run.over_budget():
            run.notes.append("ran out of time; the rest is left for the next run")
            break
        box = boxes[farm["id"]]
        numbers = None
        if "history" in plan:
            def history():
                nonlocal numbers
                numbers = history_numbers(season_history(box, today))
                stamp(farm, "history")
                now = run.state[str(farm["id"])].get("now", {}).get("result")
                run.push(farm, "greenness", greenness_topic(box, numbers, now))
            if not run.attempt(farm, "greenness", history, key="history"):
                # Dryness needs the full history: with part of it the figures would be wrong.
                continue
            plan.add("dryness")
        if "dryness" in plan:
            def dryness():
                body = dryness_topic(box, rows_for(farm), numbers or kept_numbers(farm), run.stored_fires())
                run.push(farm, "dryness", body)
                stamp(farm, "dryness")
            run.attempt(farm, "dryness", dryness)

    # Last: SoilGrids once more, with patience, where it did not answer.
    for farm in soil_later:
        def soil_again():
            body, complete = soil_topic(farm["lat"], farm["lon"], today, patient=True)
            if not complete:
                raise OutsideError("SoilGrids still does not answer; height and slope stay, the soil is tried again later")
            run.push(farm, "soil", body)
            stamp(farm, "soil", complete=True)
        run.attempt(farm, "soil", soil_again)
    return len(busy)


def report(run, message):
    """Tells the dashboard's job status page about a run that did something."""
    body = {
        "finished_at": dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "ok": run.failed == 0,
        "rows": run.pushed,
        "message": message[:500],
    }
    started = urllib.parse.quote(run.started.strftime("%Y-%m-%dT%H:%M:%SZ"), safe="")
    try:
        backend("PUT", f"/ingest/jobs/farm_analysis/runs/{started}", body, timeout=20)
    except (urllib.error.URLError, OSError) as error:
        log(f"could not report the run to the job status: {error}")


def option(name):
    return sys.argv[sys.argv.index(name) + 1] if name in sys.argv else None


def main():
    dry_run = "--dry-run" in sys.argv
    at = option("--at")
    if at and not dry_run:
        sys.exit("--at needs --dry-run")
    if not KEY and not at:
        sys.exit("INGEST__SERVICE_KEY is not set")

    CACHE.mkdir(parents=True, exist_ok=True)
    lock = open(CACHE / "farm_analysis.lock", "w")
    try:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except OSError:
        log("done: 0 pushed, another run is still working (lock held)")
        return

    run = Run(dry_run, "--force" in sys.argv)
    today = run.started.date()
    if at:
        parts = [float(part) for part in at.split(",")]
        farms = [{"id": "at", "lat": parts[0], "lon": parts[1], "area_dunam": parts[2] if len(parts) > 2 else 12.0, "topics": []}]
        run.state = {}
    else:
        try:
            farms = backend("GET", "/ingest/farms").get("farms", [])
        except (urllib.error.URLError, OSError, ValueError) as error:
            log(f"the backend's farm list could not be read: {error}")
            log("done: 0 pushed, 1 failed (no farm list)")
            sys.exit(1)
        only = option("--farm")
        if only:
            farms = [farm for farm in farms if str(farm["id"]) == only]

    busy = work_on(run, farms, today)
    run.save()
    calls = ", ".join(f"{name} {count}" for name, count in sorted(CALLS.items()))
    message = (
        f"done: {run.pushed} {'would be ' if dry_run else ''}pushed, {run.failed} failed, {run.skipped} not computable, "
        f"{len(farms) - busy} of {len(farms)} farms up to date"
        + (f", {run.waiting} waiting to retry" if run.waiting else "")
        + (f"; {'; '.join(sorted(set(run.notes)))}" if run.notes else "")
        + f" ({time.time() - run.clock:.0f} s; calls: {calls})"
    )
    if not dry_run and not at and (run.pushed or run.failed):
        report(run, message)
    log(message)
    if run.failed:
        sys.exit(1)


if __name__ == "__main__":
    main()
