#!/usr/bin/env python3
"""Region runner: rain and greenness against normal for each of the 33 districts.

Runs on a timer (every 12 hours). For each district it compares the rain of
the last 365 days at the district's centre with the same 365-day window in
each of the ten years before, adds how green the district looks from
satellite against the same weeks of the ten years before, and pushes the
result to the backend's ingest route as that district's reading for the
current month.

With --backfill-months N it also pushes one reading for each of the N whole
months before this one, each worked out exactly as today's is, as it stood
on that month's last day: the 365 days ending then, against the ten years
before them. That is what gives the dashboard a "last year" and "earlier
years" to compare with. It needs N months more rain than the daily run
keeps, and fetches them once.

What it measures and what it does not:
- rain_pct_of_normal: measured. Source: Open-Meteo archive (ERA5 reanalysis,
  about 9 to 25 km cells), one point per district, so it is a district-scale
  figure, not a field-scale one.
- dryness (0 to 100): NOT a separate measurement. It is the rain figure put
  on the dashboard's scale: dryness = 100 - rain_pct_of_normal / 2, so normal
  rain is 50, half the normal rain is 75, and 200% or more is 0. Soil moisture
  and greenness are not in it.
- greenness_pct_vs_normal: measured, see ndvi.py. MODIS NDVI in a square of
  about 19 km around the district's centre, every kind of land in it (not
  cropland alone), against the same 16 days of the ten years before. Sent
  empty when the satellite service has no usable picture.
- water need, nitrogen hold, best crops: not computed; sent empty.

Needs only the Python standard library.

Environment:
  FARM_DOCTOR_API       base address, default http://localhost:8790/v1
  INGEST__SERVICE_KEY   the backend's ingest key (required)
  REGION_RUNNER_CACHE   folder for the rain and greenness caches, default ./cache
  GREENNESS_MINUTES     stop asking the satellite service after this long,
                        default 20; what is missing is fetched by later runs

Options:
  --backfill-months N   also push the N whole months before this one
  --no-greenness        do not ask the satellite service; push what is cached
  --dry-run             fetch and work out everything, push nothing
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

import ndvi

ARCHIVE = "https://archive-api.open-meteo.com/v1/archive"
HERE = Path(__file__).resolve().parent
CACHE = Path(os.environ.get("REGION_RUNNER_CACHE", HERE / "cache"))
API = os.environ.get("FARM_DOCTOR_API", "http://localhost:8790/v1").rstrip("/")
KEY = os.environ.get("INGEST__SERVICE_KEY", "")

WINDOW_DAYS = 365
NORMAL_YEARS = 10
# The free archive counts a request as locations x (days / 14) and allows
# 600 a minute. Each first fetch is cut to stay under that (two districts for
# eleven years), and a pause between requests keeps the long fetch polite.
BACKFILL_WEIGHT = 580
BACKFILL_PAUSE_S = 65
# How far the oldest day in the cache may be from the day asked for before
# the older days are fetched.
CACHE_START_SLACK_DAYS = 10
# The backend keeps a source of at most 100 characters; the method is spelled
# out in full in the README next to this file.
SOURCE_LIMIT = 100


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
            reason = error.read()[:200].decode("utf-8", "replace")
            # The archive also has an allowance per hour and per day; waiting
            # a minute does not bring those back.
            if error.code != 429 or attempt == 4 or "inutely" not in reason:
                raise OSError(f"weather archive answered {error.code}: {reason}") from error
            log(f"weather archive asked us to slow down, waiting {BACKFILL_PAUSE_S} s")
            time.sleep(BACKFILL_PAUSE_S)
        except (OSError, ValueError) as error:
            if attempt == 4:
                raise OSError(f"weather archive did not answer: {error}") from error
            time.sleep(15 * (attempt + 1))
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


def fetch_in_batches(districts, cache, start, end, what):
    """A long fetch, a few districts at a time. Returns the number of calls."""
    days = (end - start).days + 1
    size = max(1, int(BACKFILL_WEIGHT / max(1.0, days / 14)))
    calls = 0
    for at in range(0, len(districts), size):
        batch = districts[at : at + size]
        log(f"{what}, {start} to {end}: {', '.join(d['slug'] for d in batch)}")
        for slug, found in fetch_rain(batch, start, end).items():
            cache.setdefault(slug, {}).update(found)
        save_cache(cache)
        calls += 1
        if at + size < len(districts):
            time.sleep(BACKFILL_PAUSE_S)
    return calls


def update_cache(districts, cache, today, oldest_reading):
    """Fetch what the cache lacks: eleven years before the oldest reading
    once, then only recent days. Returns the number of calls made."""
    first_needed = oldest_reading - dt.timedelta(days=366 * (NORMAL_YEARS + 1) + 10)
    missing = [d for d in districts if not cache.get(d["slug"])]
    calls = fetch_in_batches(missing, cache, first_needed, today, "first fetch of rain")

    # A cache made for today's reading alone starts too late for a backfill.
    short = {}
    for district in districts:
        days = cache.get(district["slug"])
        if not days:
            continue
        oldest = dt.date.fromisoformat(min(days))
        if (oldest - first_needed).days > CACHE_START_SLACK_DAYS:
            short.setdefault(oldest, []).append(district)
    for oldest, group in sorted(short.items()):
        if calls:
            time.sleep(BACKFILL_PAUSE_S)
        calls += fetch_in_batches(
            group, cache, first_needed, oldest - dt.timedelta(days=1), "older rain for the backfill"
        )

    # The newest days are revised by the archive for about a week, so the
    # last 40 days are always fetched again.
    recent = fetch_rain(districts, today - dt.timedelta(days=40), today)
    for slug, days in recent.items():
        cache.setdefault(slug, {}).update(days)
    save_cache(cache)
    return calls + 1


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


def reading_for(days, as_of=None):
    """The reading as it stood on `as_of` (the newest day with rain when left
    out). Returns (as_of, rain_pct_of_normal, dryness, years in the normal)
    or None if it cannot be worked out."""
    if not days:
        return None
    if as_of is None:
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
    return as_of, round(rain_pct, 1), dryness, len(past)


def source_text(as_of, rain_years, green):
    """Says what the numbers of one reading are made of, in 100 characters."""
    if green is None:
        text = (
            f"Open-Meteo ERA5, district centre: 365-day rain vs {rain_years}-yr normal; "
            f"dryness=100-rain%/2; to {as_of}"
        )
    else:
        text = (
            f"ERA5 rain 365d to {as_of} vs {rain_years}y; dryness=100-rain%/2; "
            f"MODIS NDVI all land from {green['picture']} vs {green['years']}y"
        )
    if len(text) > SOURCE_LIMIT:
        raise ValueError(f"source text is {len(text)} characters: {text}")
    return text


def push(slug, month, rain_pct, dryness, green, source):
    body = json.dumps(
        {
            "dryness": dryness,
            "rain_pct_of_normal": rain_pct,
            "greenness_pct_vs_normal": green["pct"] if green else None,
            "water_need": None,
            "nitrogen_hold": False,
            "best_crops": [],
            "source": source,
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


def option_number(name):
    """The whole number after an option such as --backfill-months, or 0."""
    if name not in sys.argv:
        return 0
    try:
        value = int(sys.argv[sys.argv.index(name) + 1])
    except (IndexError, ValueError):
        sys.exit(f"{name} needs a whole number")
    if value < 0:
        sys.exit(f"{name} cannot be negative")
    return value


def update_greenness(districts, days, today):
    """Brings the greenness cache up to date as far as the time allows and
    returns it. A satellite service that is down never stops the rain."""
    cache = ndvi.load_cache()
    if "--no-greenness" in sys.argv:
        return cache
    try:
        ndvi.refresh_pictures(cache, today)
        if not cache["pictures"]:
            log("greenness: no list of satellite pictures; readings go out without greenness")
            return cache
        shapes = json.loads((HERE / "district_shapes.json").read_text())
        deadline = time.monotonic() + ndvi.minutes_allowed() * 60
        # Today's reading first, for every district; the past months after.
        calls, finished = ndvi.update(cache, districts, shapes, days[:1], deadline)
        if finished and days[1:]:
            more, finished = ndvi.update(cache, districts, shapes, days[1:], deadline)
            calls += more
        log(f"greenness: {calls} calls to the satellite service, {'complete' if finished else 'not complete'}")
    except ndvi.ServiceDown as error:
        log(f"greenness: the satellite service is not answering ({error}); using what is cached")
    return cache


def main():
    dry_run = "--dry-run" in sys.argv
    backfill_months = option_number("--backfill-months")
    if not KEY and not dry_run:
        sys.exit("INGEST__SERVICE_KEY is not set")

    districts = json.loads((HERE / "districts.json").read_text())
    today = dt.datetime.now(dt.timezone.utc).date()
    month_ends = ndvi.month_ends(today, backfill_months)
    cache = load_cache()

    try:
        calls = update_cache(districts, cache, today, month_ends[-1] if month_ends else today)
    except OSError as error:
        log(f"done: 0 pushed, 0 skipped, {len(districts)} failed: {error}")
        sys.exit(1)
    log(f"rain: {calls} calls to the weather archive")

    # Each district's readings, newest first: today's, then one per past
    # month. The newest days are missing for a few days after a month ends,
    # so today's reading can still belong to last month; it then stands for
    # that month and the month-end one is left out.
    readings = []
    skipped = 0
    for district in districts:
        days = cache.get(district["slug"], {})
        months_seen = set()
        for as_of in [None] + month_ends:
            reading = reading_for(days, as_of)
            if reading is None:
                when = f"{as_of:%Y-%m}" if as_of else "today"
                log(f"{district['slug']}: {when}: not enough rain data, nothing pushed")
                skipped += 1
                continue
            month = f"{reading[0]:%Y-%m}"
            if month not in months_seen:
                months_seen.add(month)
                readings.append((district, month, reading))

    greenness_days = list(dict.fromkeys(reading[0] for _, _, reading in readings))
    green_cache = update_greenness(districts, sorted(greenness_days, reverse=True), today)

    pushed, failed, with_greenness = 0, 0, 0
    for district, month, (as_of, rain_pct, dryness, rain_years) in readings:
        slug = district["slug"]
        green = ndvi.greenness_for(green_cache["districts"].get(slug, {}), as_of, green_cache["pictures"])
        source = source_text(as_of, rain_years, green)
        with_greenness += green is not None
        if green is None:
            greenness = "no greenness"
        else:
            greenness = f"greenness {green['pct']:+.1f}% (NDVI {green['ndvi']:.3f} from {green['picture']})"
        line = f"{slug}: {month} rain {rain_pct}% of normal, dryness {dryness}, {greenness}"
        if dry_run:
            # A backfill is thousands of lines; today's are the ones to read.
            if month == f"{today:%Y-%m}" or not backfill_months:
                log(f"would push {line}")
            pushed += 1
            continue
        try:
            push(slug, month, rain_pct, dryness, green, source)
            if month == f"{today:%Y-%m}" or not backfill_months:
                log(line)
            pushed += 1
        except urllib.error.HTTPError as error:
            log(f"{slug}: {month}: backend refused with {error.code}: {error.read()[:200]!r}")
            failed += 1
        except OSError as error:
            log(f"{slug}: {month}: backend unreachable: {error}")
            failed += 1

    log(
        f"done: {pushed} {'would be ' if dry_run else ''}pushed ({with_greenness} with greenness), "
        f"{skipped} skipped, {failed} failed"
    )
    if failed:
        sys.exit(1)


if __name__ == "__main__":
    main()
