#!/usr/bin/env python3
"""Farm plan: the next 10 days of weather at each farm, turned into farm work.

Runs on a timer every 10 minutes, but only works on farms whose plan is
missing or older than 6 hours, so a new farm gets its plan within minutes and
a run with nothing to do costs one call to the backend.

For each farm that is due it fetches the Open-Meteo forecast (10 days, daily
and hourly) and the air-quality forecast (PM10, 5 days), applies the rules of
farm_doctor/weather_planner.py, and pushes the plan in the shape the farmer
app reads (BACKEND.md 2.4): PUT /v1/ingest/farms/{id}/plan. The alerts,
decisions and English sentences are the ones planFromWeather in the app's
fake_api.dart produces. The app has no Sorani for them yet, so `ku` repeats
`en`.

What it does not know:
- the crops on a farm. Like weather_planner.py it assumes winter wheat, so a
  farm with only vegetables still gets the sowing, urea and rust lines.
- anything the forecast does not: a forecast is one model cell of several km.

Farms close together share a forecast: farm centres are grouped on a grid of
0.05 degrees (about 5 km), the forecast is fetched once per group at the mean
of its farms' centres, and several groups go into one Open-Meteo call.

The numbers of the rules are read from the backend at the start of each run
(GET /v1/ingest/rules?used_by=weather_planner), so a change on the
dashboard's Rules page changes the next plans. A rule that is missing, or a
backend that does not answer that call, falls back to the number built in
here.

Reporting: the job reports itself to the job status page (job `plans`), and
only the runs worth a line: those that pushed or failed, plus one quiet run
every 6 hours so a healthy job is not shown as late.

Needs only the Python standard library.

Usage:
  farm_plan.py              plan the farms that are due
  farm_plan.py --force      plan every farm, however fresh its plan is
  farm_plan.py --dry-run    fetch and print the plans, push and report nothing
  farm_plan.py --keep-raw DIR   also save the raw forecasts there, to check a plan by hand

Environment:
  FARM_DOCTOR_API       base address, default http://localhost:8790/v1
  INGEST__SERVICE_KEY   the backend's ingest key (required)
  FARM_PLAN_CACHE       folder for the lock and the small caches, default ./cache
"""

import datetime as dt
import fcntl
import json
import math
import os
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

FORECAST = "https://api.open-meteo.com/v1/forecast"
AIR_QUALITY = "https://air-quality-api.open-meteo.com/v1/air-quality"
ARCHIVE = "https://archive-api.open-meteo.com/v1/archive"
HERE = Path(__file__).resolve().parent
CACHE = Path(os.environ.get("FARM_PLAN_CACHE", HERE / "cache"))
API = os.environ.get("FARM_DOCTOR_API", "http://localhost:8790/v1").rstrip("/")
KEY = os.environ.get("INGEST__SERVICE_KEY", "")

JOB = "plans"
SOURCE = "Open-Meteo (ECMWF/GraphCast family)"
TIMEZONE = "Asia/Baghdad"
DAYS = 10  # no forecast beyond 10 days, ever: the backend refuses an 11th
MAX_AGE_H = 6
GRID_DEG = 0.05
POINTS_PER_CALL = 40
ARCHIVE_POINTS_PER_CALL = 10
MESSAGE_LIMIT = 500

DAILY = "precipitation_sum,temperature_2m_max,temperature_2m_min"
HOURLY = "temperature_2m,relative_humidity_2m,precipitation,wind_speed_10m"

# The numbers of weather_planner.py, used when the backend has no rule.
DEFAULTS = {
    "frost_c": 0.0,
    "hard_frost_c": -2.0,
    "heat_c": 31.0,
    "heavy_rain_mm": 12.0,
    "sowing_rain_mm": 20.0,
    "rust_weather_hours": 24.0,
    "spray_window_hours": 6.0,
    "sunn_pest_degree_days": 84.0,
    "dust_pm10": 150.0,
}
# Parts of the rules the backend keeps no number for.
RUST_SOME_HOURS = 8  # from this many cool wet hours rust weather is "some"
SUNN_PEST_BASE_C = 13.3
SUNN_PEST_END_DD = 223  # past this the spray window is over for the year
WEEKDAYS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]


def log(message):
    print(f"{dt.datetime.now(dt.timezone.utc):%Y-%m-%d %H:%M:%S}Z {message}", flush=True)


def utc_now():
    return dt.datetime.now(dt.timezone.utc).replace(microsecond=0)


def stamp(moment):
    return moment.strftime("%Y-%m-%dT%H:%M:%SZ")


# ---- the backend ----


def backend(method, path, body=None):
    request = urllib.request.Request(
        f"{API}{path}",
        data=None if body is None else json.dumps(body).encode(),
        method=method,
        headers={"content-type": "application/json", "x-service-key": KEY},
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.load(response)


def load_thresholds():
    """The rule numbers from the backend, each falling back to the one built in."""
    thresholds, missing = dict(DEFAULTS), sorted(DEFAULTS)
    try:
        rules = backend("GET", "/ingest/rules?used_by=weather_planner")["rules"]
    except (urllib.error.URLError, OSError, ValueError, KeyError) as error:
        log(f"rules: backend did not answer ({error}); using the built-in numbers")
        return thresholds
    for rule in rules:
        code, value = rule.get("code"), rule.get("value")
        if code in DEFAULTS and isinstance(value, (int, float)) and math.isfinite(value):
            thresholds[code] = float(value)
            missing.remove(code)
    if missing:
        log(f"rules: no stored value for {', '.join(missing)}; using the built-in numbers")
    changed = {code: value for code, value in thresholds.items() if value != DEFAULTS[code]}
    if changed:
        log(f"rules: changed on the dashboard: {changed}")
    return thresholds


def report_run(started, finished, ok, rows, message):
    """Tells the job status page about this run. Never stops the job."""
    path = f"/ingest/jobs/{JOB}/runs/{urllib.parse.quote(stamp(started), safe='')}"
    body = {
        "finished_at": finished and stamp(finished),
        "ok": ok,
        "rows": rows,
        "message": message and message[:MESSAGE_LIMIT],
    }
    try:
        backend("PUT", path, body)
        (CACHE / "farm_plan_reported").write_text(stamp(started))
    except (urllib.error.URLError, OSError, ValueError) as error:
        log(f"could not report the run: {error}")


def heartbeat_due(now):
    """A quiet run is reported when nothing was reported for a whole period."""
    try:
        last = dt.datetime.strptime(
            (CACHE / "farm_plan_reported").read_text().strip(), "%Y-%m-%dT%H:%M:%SZ"
        ).replace(tzinfo=dt.timezone.utc)
    except (OSError, ValueError):
        return True
    return now - last >= dt.timedelta(hours=MAX_AGE_H)


# ---- Open-Meteo ----


def open_meteo(url, points, params):
    """One call for several points. Returns one answer per point, in order."""
    query = urllib.parse.urlencode(
        {
            "latitude": ",".join(str(lat) for lat, _ in points),
            "longitude": ",".join(str(lon) for _, lon in points),
            "timezone": TIMEZONE,
            **params,
        }
    )
    request = urllib.request.Request(f"{url}?{query}", headers={"User-Agent": "farm-doctor-jobs"})
    for attempt in range(3):
        try:
            with urllib.request.urlopen(request, timeout=60) as response:
                body = json.load(response)
            break
        except urllib.error.HTTPError as error:
            if error.code not in (429, 500, 502, 503, 504) or attempt == 2:
                raise
            time.sleep(20)
        except OSError:
            if attempt == 2:
                raise
            time.sleep(5)
    if isinstance(body, dict):
        body = [body]
    if len(body) != len(points):
        raise ValueError(f"asked for {len(points)} points, got {len(body)}")
    return body


def chunks(items, size):
    for start in range(0, len(items), size):
        yield items[start : start + size]


def load_json(name):
    try:
        return json.loads((CACHE / name).read_text())
    except (OSError, ValueError):
        return {}


def save_json(name, value):
    temporary = CACHE / f"{name}.tmp"
    temporary.write_text(json.dumps(value))
    temporary.replace(CACHE / name)


def degree_days(cells, today):
    """Sunn pest degree-days since 1 January (base 13.3 C) per cell key.

    The sum of the days already counted is kept per cell, so a run fetches
    only the days since, and nothing at all once the year's window is over.
    A cell whose archive call fails is left out: no sunn pest line for it.
    """
    cache = load_json("farm_plan_degree_days.json")
    year_start = dt.date(today.year, 1, 1)
    result, to_fetch = {}, []
    for key, point in cells.items():
        kept = cache.get(key)
        if not kept or kept.get("year") != today.year:
            kept = cache[key] = {"year": today.year, "through": None, "dd": 0.0}
        if kept["dd"] >= SUNN_PEST_END_DD:
            result[key] = kept["dd"]
            continue
        start = (
            dt.date.fromisoformat(kept["through"]) + dt.timedelta(days=1)
            if kept["through"]
            else year_start
        )
        if start > today:
            result[key] = kept["dd"]
        else:
            to_fetch.append((key, point, start))
    # Cells that start on the same day share a call.
    by_start = {}
    for key, point, start in to_fetch:
        by_start.setdefault(start, []).append((key, point))
    for start, group in by_start.items():
        for batch in chunks(group, ARCHIVE_POINTS_PER_CALL):
            try:
                answers = open_meteo(
                    ARCHIVE,
                    [point for _, point in batch],
                    {
                        "start_date": start.isoformat(),
                        "end_date": today.isoformat(),
                        "daily": "temperature_2m_mean",
                    },
                )
            except (urllib.error.URLError, OSError, ValueError) as error:
                log(f"sunn pest: archive did not answer for {len(batch)} cells: {error}")
                continue
            for (key, _), answer in zip(batch, answers):
                kept = cache[key]
                # The archive is some days behind: the newest days come back
                # empty and are asked for again next time.
                for day, mean in zip(answer["daily"]["time"], answer["daily"]["temperature_2m_mean"]):
                    if mean is None:
                        break
                    kept["dd"] += max(0.0, mean - SUNN_PEST_BASE_C)
                    kept["through"] = day
                result[key] = kept["dd"]
    save_json("farm_plan_degree_days.json", cache)
    return result


# ---- the rules ----


def half_up(value):
    """Rounds like the app does (halves away from zero), never to "-0"."""
    rounded = int(math.floor(abs(value) + 0.5))
    return -rounded if value < 0 and rounded else rounded


def number(value):
    """A threshold as a person writes it: 20, not 20.0."""
    return str(int(value)) if float(value).is_integer() else str(value)


def weekday(day, first):
    """ "Sun" for a day of the plan's first week. Ten days hold some weekdays
    twice, so a later day carries its date ("Sun 18"): the bare name would
    be read as the Sunday that comes first."""
    date = dt.date.fromisoformat(day)
    name = WEEKDAYS[date.weekday()]
    return name if (date - dt.date.fromisoformat(first)).days < 7 else f"{name} {date.day}"


def build_plan(forecast, air, sunn_dd, rules, issued):
    """The rules of weather_planner.py on one Open-Meteo answer, in the app's shape."""
    daily, hourly = forecast["daily"], forecast["hourly"]
    days = daily["time"][:DAYS]
    count = len(days)
    rain = daily["precipitation_sum"][:count]
    tmin = daily["temperature_2m_min"][:count]
    tmax = daily["temperature_2m_max"][:count]
    hours = hourly["time"]
    temp = hourly["temperature_2m"]
    humidity = hourly["relative_humidity_2m"]
    hourly_rain = hourly["precipitation"]
    wind = hourly["wind_speed_10m"]
    month = int(days[0][5:7])
    wet = [value or 0 for value in rain]

    alerts, decisions = [], []

    def alert(kind, day, value, level, text):
        alerts.append({"type": kind, "day": day, "value": value, "level": level, "ku": text, "en": text})

    def decide(code, text):
        decisions.append({"code": code, "ku": text, "en": text})

    # Sowing: enough rain within 3 days, October to December.
    if month >= 10:
        need = rules["sowing_rain_mm"]
        first = next((i for i in range(count - 2) if sum(wet[i : i + 3]) >= need), None)
        if first is not None:
            total = sum(wet[first : first + 3])
            alert(
                "sowing_rain",
                days[first],
                round(total, 1),
                "watch",
                f"Sowing rain: {half_up(total)} mm in 3 days from {weekday(days[first], days[0])}",
            )
            decide(
                "sow_go",
                f"Good sowing rain ({number(need)} mm or more in 3 days) starts {weekday(days[first], days[0])}.",
            )
        else:
            decide(
                "sow_wait",
                f"No sowing rain ({number(need)} mm) in the next 10 days. Wait, do not dry-sow.",
            )

    # Urea: spread on dry soil just before a day of heavy rain, January to March.
    if month <= 3:
        need = rules["heavy_rain_mm"]
        first = next((i for i in range(count) if wet[i] >= need), None)
        if first is not None:
            alert(
                "urea_rain",
                days[first],
                wet[first],
                "watch",
                f"{half_up(wet[first])} mm rain {weekday(days[first], days[0])}: urea goes on just before",
            )
            decide(
                "urea_go",
                f"Spread urea on dry soil just before {weekday(days[first], days[0])} ({half_up(wet[first])} mm).",
            )
        else:
            decide("urea_hold", f"No {number(need)} mm rain coming. Hold the urea top-dressing.")

    # Spray windows: hours in a row, dry, 15 to 24 C, wind under 15 km/h, 7:00 to 18:00.
    window = max(1, int(rules["spray_window_hours"]))
    spray_days = set()
    for start in range(len(hours) - window + 1):
        if all(
            (hourly_rain[j] or 0) == 0
            and temp[j] is not None
            and 15 <= temp[j] <= 24
            and (wind[j] or 0) < 15
            and 7 <= int(hours[j][11:13]) <= 18
            for j in range(start, start + window)
        ):
            spray_days.add(hours[start][:10])
    if spray_days:
        names = ", ".join(weekday(day, days[0]) for day in sorted(spray_days)[:5])
        decide("spray_ok", f"Safe spray days: {names}.")

    # Frost.
    hard = []
    for i in range(count):
        if tmin[i] is not None and tmin[i] <= rules["frost_c"]:
            is_hard = tmin[i] <= rules["hard_frost_c"]
            alert(
                "frost",
                days[i],
                tmin[i],
                "alarm" if is_hard else "watch",
                f"Frost {half_up(tmin[i])} °C {weekday(days[i], days[0])} night",
            )
            if is_hard:
                hard.append(weekday(days[i], days[0]))
    if hard:
        decide("frost_check", f"Hard frost on {', '.join(hard)}. Check the heads 7 to 10 days after.")

    # Heat during flowering, April and May.
    if month in (4, 5):
        hot = []
        for i in range(count):
            if tmax[i] is not None and tmax[i] >= rules["heat_c"]:
                alert("heat", days[i], tmax[i], "watch", f"Heat {half_up(tmax[i])} °C {weekday(days[i], days[0])}")
                hot.append(weekday(days[i], days[0]))
        if hot:
            decide(
                "heat_check",
                f"Over {number(rules['heat_c'])} °C on {', '.join(hot)} during flowering.",
            )

    # Rust weather: hours at 6 to 16 C with humidity of 90% or more.
    cool_wet = [
        j
        for j in range(len(hours))
        if temp[j] is not None and 6 <= temp[j] <= 16 and (humidity[j] or 0) >= 90
    ]
    if len(cool_wet) >= min(RUST_SOME_HOURS, rules["rust_weather_hours"]):
        high = len(cool_wet) >= rules["rust_weather_hours"]
        first_day = hours[cool_wet[0]][:10]
        if first_day in days:
            alert(
                "rust_weather",
                first_day,
                float(len(cool_wet)),
                "alarm" if high else "watch",
                f"Rust weather: {len(cool_wet)} cool wet hours",
            )
        decide(
            "check_rust",
            f"{'High' if high else 'Some'} rust weather ({len(cool_wet)} cool wet hours). Check the leaves.",
        )

    # Sunn pest: nymphs between two sums of degree-days since 1 January.
    if sunn_dd is not None and rules["sunn_pest_degree_days"] <= sunn_dd < SUNN_PEST_END_DD:
        decide(
            "count_sunn_pest",
            "Sunn pest nymphs are likely. Count them with a 0.5 x 0.5 m frame; spray only above 8 per m2.",
        )

    # Dust: the highest PM10 of the next 5 days.
    if air is not None:
        pm10 = air["hourly"]["pm10"]
        known = [j for j in range(len(pm10)) if pm10[j] is not None]
        if known:
            top = max(known, key=lambda j: pm10[j])
            if pm10[top] >= rules["dust_pm10"]:
                day = air["hourly"]["time"][top][:10]
                if day in days:
                    alert(
                        "dust",
                        day,
                        pm10[top],
                        "watch",
                        f"Dust {weekday(day, days[0])}: PM10 up to {half_up(pm10[top])}",
                    )
                decide(
                    "dust_delay",
                    f"Dust (PM10 up to {half_up(pm10[top])}) {weekday(day, days[0])}. "
                    "Delay spraying and harvest, shelter animals.",
                )

    alerts.sort(key=lambda item: item["day"])
    return {
        "from": days[0],
        "days": count,
        "rain_mm": rain,
        "tmin": tmin,
        "tmax": tmax,
        "alerts": alerts,
        "decisions": decisions,
        "source": SOURCE,
        "issued": stamp(issued),
    }


# ---- one run ----


def cell_of(farm):
    return f"{round(farm['lat'] / GRID_DEG) * GRID_DEG:.2f},{round(farm['lon'] / GRID_DEG) * GRID_DEG:.2f}"


def is_due(farm, now):
    if not farm.get("issued"):
        return True
    issued = dt.datetime.fromisoformat(farm["issued"].replace("Z", "+00:00"))
    return now - issued > dt.timedelta(hours=MAX_AGE_H)


def run(force, dry_run, keep_raw):
    started = utc_now()
    farms = backend("GET", "/ingest/farms/plans/coverage")["farms"]
    due = [farm for farm in farms if force or is_due(farm, started)]
    fresh = len(farms) - len(due)

    if not due:
        summary = f"done: 0 pushed, {fresh} fresh, 0 failed"
        if not dry_run and heartbeat_due(started):
            report_run(started, utc_now(), True, 0, summary)
        log(summary)
        return 0

    if not dry_run:
        report_run(started, None, None, None, None)

    rules = load_thresholds()

    groups = {}
    for farm in due:
        groups.setdefault(cell_of(farm), []).append(farm)
    # One forecast per group, at the middle of the farms that are in it.
    points = {
        key: (
            round(sum(farm["lat"] for farm in members) / len(members), 4),
            round(sum(farm["lon"] for farm in members) / len(members), 4),
        )
        for key, members in groups.items()
    }
    log(f"{len(due)} farms due in {len(groups)} forecast groups, {fresh} fresh")

    pushed, failed = 0, 0
    for batch in chunks(sorted(groups), POINTS_PER_CALL):
        where = [points[key] for key in batch]
        try:
            forecasts = open_meteo(
                FORECAST, where, {"forecast_days": DAYS, "daily": DAILY, "hourly": HOURLY}
            )
        except (urllib.error.URLError, OSError, ValueError) as error:
            count = sum(len(groups[key]) for key in batch)
            log(f"forecast did not answer for {len(batch)} groups ({count} farms): {error}")
            failed += count
            continue
        issued = utc_now()
        try:
            airs = open_meteo(AIR_QUALITY, where, {"forecast_days": 5, "hourly": "pm10"})
        except (urllib.error.URLError, OSError, ValueError) as error:
            # Dust is a bonus; the plan still works without it.
            log(f"air quality did not answer for {len(batch)} groups, no dust line: {error}")
            airs = [None] * len(batch)
        today = dt.date.fromisoformat(forecasts[0]["daily"]["time"][0])
        sunn = degree_days({key: points[key] for key in batch}, today)

        for key, forecast, air in zip(batch, forecasts, airs):
            if keep_raw:
                name = key.replace(",", "_")
                (keep_raw / f"{name}.json").write_text(
                    json.dumps({"point": points[key], "forecast": forecast, "air": air, "sunn_dd": sunn.get(key)})
                )
            try:
                plan = build_plan(forecast, air, sunn.get(key), rules, issued)
            except (KeyError, IndexError, TypeError, ValueError) as error:
                log(f"group {key}: forecast could not be read: {error!r}")
                failed += len(groups[key])
                continue
            for farm in groups[key]:
                summary = (
                    f"farm {farm['id']} ({key}): {len(plan['alerts'])} alerts, "
                    f"decisions {[d['code'] for d in plan['decisions']]}"
                )
                if dry_run:
                    log(summary)
                    print(json.dumps(plan, ensure_ascii=False))
                    continue
                try:
                    backend("PUT", f"/ingest/farms/{farm['id']}/plan", plan)
                    log(summary)
                    pushed += 1
                except urllib.error.HTTPError as error:
                    log(f"farm {farm['id']}: backend refused with {error.code}: {error.read()[:200]!r}")
                    failed += 1
                except OSError as error:
                    log(f"farm {farm['id']}: backend unreachable: {error}")
                    failed += 1

    summary = f"done: {pushed} pushed, {fresh} fresh, {failed} failed"
    if not dry_run:
        report_run(started, utc_now(), failed == 0, pushed, summary)
    log(summary)
    return 1 if failed else 0


def main():
    arguments = sys.argv[1:]
    keep_raw = None
    if "--keep-raw" in arguments:
        keep_raw = Path(arguments[arguments.index("--keep-raw") + 1])
        keep_raw.mkdir(parents=True, exist_ok=True)
    if not KEY:
        sys.exit("INGEST__SERVICE_KEY is not set")

    CACHE.mkdir(parents=True, exist_ok=True)
    # The timer fires every 10 minutes; a run that is still going keeps the
    # lock and the next one leaves at once.
    lock = open(CACHE / "farm_plan.lock", "w")
    try:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except OSError:
        log("another run is still going, leaving")
        return

    try:
        code = run("--force" in arguments, "--dry-run" in arguments, keep_raw)
    except (urllib.error.URLError, OSError, ValueError, KeyError) as error:
        # The coverage call failed: nothing can be planned or reported.
        log(f"backend did not answer: {error}")
        log("done: 0 pushed, 0 fresh, 0 failed (backend unreachable)")
        code = 1
    sys.exit(code)


if __name__ == "__main__":
    main()
