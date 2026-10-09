#!/usr/bin/env python3
"""Region runner: rain against normal for each of the 33 districts.

Runs on a timer (every 12 hours). For each district it compares the rain of
the last 365 days at the district's centre with the same 365-day window in
each of the ten years before, and pushes the result to the backend's ingest
route as that district's reading for the current month.

What it measures and what it does not:
- rain_pct_of_normal: measured. Source: Open-Meteo archive (ERA5 reanalysis,
  about 9 to 25 km cells), one point per district, so it is a district-scale
  figure, not a field-scale one.
- dryness (0 to 100): NOT a separate measurement. It is the rain figure put
  on the dashboard's scale: dryness = 100 - rain_pct_of_normal / 2, so normal
  rain is 50, half the normal rain is 75, and 200% or more is 0. Soil moisture
  and greenness are not in it yet.
- greenness, water need, nitrogen hold, best crops: not computed; sent empty.

Needs only the Python standard library.

Environment:
  FARM_DOCTOR_API       base address, default http://localhost:8790/v1
  INGEST__SERVICE_KEY   the backend's ingest key (required)
  REGION_RUNNER_CACHE   folder for the rain cache, default ./cache
"""

import datetime as dt
import json
import os
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

ARCHIVE = "https://archive-api.open-meteo.com/v1/archive"
HERE = Path(__file__).resolve().parent
CACHE = Path(os.environ.get("REGION_RUNNER_CACHE", HERE / "cache"))
API = os.environ.get("FARM_DOCTOR_API", "http://localhost:8790/v1").rstrip("/")
KEY = os.environ.get("INGEST__SERVICE_KEY", "")

WINDOW_DAYS = 365
NORMAL_YEARS = 10
# The free archive counts a request as locations x (days / 14). Two districts
# over eleven years stay under its per-minute allowance; a pause between
# requests keeps the first, long fetch polite.
BACKFILL_BATCH = 2
BACKFILL_PAUSE_S = 65
# The backend keeps a source of at most 100 characters; the method is spelled
# out in full in the README next to this file.
SOURCE = "Open-Meteo ERA5, district centre: 365-day rain vs 10-yr normal; dryness=100-rain%/2"


def log(message):
    print(f"{dt.datetime.now(dt.timezone.utc):%Y-%m-%d %H:%M:%S}Z {message}", flush=True)


def fetch_rain(districts, start, end):
    """Daily rain in mm for each district between two dates, by slug."""
    query = urllib.parse.urlencode(
        {
            "latitude": ",".join(str(d["lat"]) for d in districts),
            "longitude": ",".join(str(d["lon"]) for d in districts),
            "start_date": start.isoformat(),
            "end_date": end.isoformat(),
            "daily": "precipitation_sum",
            "timezone": "UTC",
        }
    )
    for attempt in range(5):
        try:
            with urllib.request.urlopen(f"{ARCHIVE}?{query}", timeout=120) as response:
                body = json.load(response)
            break
        except urllib.error.HTTPError as error:
            if error.code != 429 or attempt == 4:
                raise
            log(f"weather archive asked us to slow down, waiting {BACKFILL_PAUSE_S} s")
            time.sleep(BACKFILL_PAUSE_S)
    if isinstance(body, dict):
        body = [body]
    rain = {}
    for district, answer in zip(districts, body):
        days = answer["daily"]["time"]
        values = answer["daily"]["precipitation_sum"]
        rain[district["slug"]] = {day: value for day, value in zip(days, values) if value is not None}
    return rain


def load_cache():
    path = CACHE / "rain.json"
    return json.loads(path.read_text()) if path.exists() else {}


def save_cache(cache):
    CACHE.mkdir(parents=True, exist_ok=True)
    temporary = CACHE / "rain.json.tmp"
    temporary.write_text(json.dumps(cache))
    temporary.replace(CACHE / "rain.json")


def update_cache(districts, cache, today):
    """Fetch what the cache lacks: eleven years once, then only recent days."""
    first_needed = today - dt.timedelta(days=366 * (NORMAL_YEARS + 1) + 10)
    missing = [d for d in districts if not cache.get(d["slug"])]
    for start in range(0, len(missing), BACKFILL_BATCH):
        batch = missing[start : start + BACKFILL_BATCH]
        log(f"first fetch of eleven years of rain: {', '.join(d['slug'] for d in batch)}")
        cache.update(fetch_rain(batch, first_needed, today))
        save_cache(cache)
        if start + BACKFILL_BATCH < len(missing):
            time.sleep(BACKFILL_PAUSE_S)

    # The newest days are revised by the archive for about a week, so the
    # last 40 days are always fetched again.
    recent = fetch_rain(districts, today - dt.timedelta(days=40), today)
    for slug, days in recent.items():
        cache.setdefault(slug, {}).update(days)
    save_cache(cache)


def shift_years(day, years):
    try:
        return day.replace(year=day.year - years)
    except ValueError:  # 29 February
        return day.replace(year=day.year - years, day=28)


def window_total(days, end):
    """Rain over the 365 days ending on `end`, or None if days are missing."""
    total, found = 0.0, 0
    for offset in range(WINDOW_DAYS):
        value = days.get((end - dt.timedelta(days=offset)).isoformat())
        if value is not None:
            total += value
            found += 1
    # A few missing days do not change a yearly total; a hole does.
    return total if found >= WINDOW_DAYS - 5 else None


def reading_for(days):
    """Returns (as_of, rain_pct_of_normal, dryness) or None if it cannot be worked out."""
    if not days:
        return None
    as_of = dt.date.fromisoformat(max(days))
    now = window_total(days, as_of)
    past = [window_total(days, shift_years(as_of, years)) for years in range(1, NORMAL_YEARS + 1)]
    past = [total for total in past if total is not None]
    if now is None or len(past) < NORMAL_YEARS - 2:
        return None
    normal = sum(past) / len(past)
    if normal <= 0:
        return None
    rain_pct = min(now / normal * 100.0, 400.0)
    dryness = max(0, min(100, round(100 - rain_pct / 2)))
    return as_of, round(rain_pct, 1), dryness


def push(slug, month, rain_pct, dryness, as_of):
    body = json.dumps(
        {
            "dryness": dryness,
            "rain_pct_of_normal": rain_pct,
            "greenness_pct_vs_normal": None,
            "water_need": None,
            "nitrogen_hold": False,
            "best_crops": [],
            "source": f"{SOURCE}; to {as_of}"[:100],
        }
    ).encode()
    request = urllib.request.Request(
        f"{API}/ingest/zones/{slug}/readings/{month}",
        data=body,
        method="PUT",
        headers={"content-type": "application/json", "x-service-key": KEY},
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        return response.status


def main():
    if not KEY:
        sys.exit("INGEST__SERVICE_KEY is not set")

    districts = json.loads((HERE / "districts.json").read_text())
    today = dt.datetime.now(dt.timezone.utc).date()
    cache = load_cache()

    update_cache(districts, cache, today)

    pushed, skipped, failed = 0, 0, 0
    for district in districts:
        slug = district["slug"]
        reading = reading_for(cache.get(slug, {}))
        if reading is None:
            log(f"{slug}: not enough rain data, nothing pushed")
            skipped += 1
            continue
        as_of, rain_pct, dryness = reading
        month = f"{as_of:%Y-%m}"
        try:
            push(slug, month, rain_pct, dryness, as_of)
            log(f"{slug}: {month} rain {rain_pct}% of normal, dryness {dryness}")
            pushed += 1
        except urllib.error.HTTPError as error:
            log(f"{slug}: backend refused with {error.code}: {error.read()[:200]!r}")
            failed += 1
        except OSError as error:
            log(f"{slug}: backend unreachable: {error}")
            failed += 1

    log(f"done: {pushed} pushed, {skipped} skipped, {failed} failed")
    if failed:
        sys.exit(1)


if __name__ == "__main__":
    main()
