#!/usr/bin/env python3
"""Greenness per district from satellite: MODIS NDVI against its own past.

Used by region_runner.py, which pushes the figure with the district's
reading. Run on its own it only fills the cache and prints what it found:

  python3 ndvi.py                 the newest 16-day picture, all districts
  python3 ndvi.py --months 36     also what the last 36 months need
  python3 ndvi.py sulaymaniyah    only the named districts

What is measured:
- NDVI from the MODIS product MOD13Q1 (Terra, 250 m pixels, one picture per
  16 days, the best cloud-free look in those days), read through the free
  ORNL DAAC subset service, no key.
- For each district: the mean NDVI of the pixels in a square of about 19 km
  around the district's centre that also lie inside the district's outline.
  Pixels the product marks as cloud, snow or ice, or leaves empty, are
  dropped (pixel reliability 2, 3 or fill), and so are pixels that read as
  open water (NDVI below zero), so a lake that is fuller than usual does not
  count as land that lost its green. A picture with fewer than half of its
  pixels left after clouds and snow is not used.
- Greenness = that mean against the mean of the same 16 days in each of the
  ten years before, in percent: 0 is normal, -20 is a fifth less green. At
  least five of the ten years must be usable.

What it is not:
- Not cropland. Every kind of land in the square counts: fields, rangeland,
  forest, towns. Nothing here tells a wheat field from a hillside.
- Not the whole district: a square around its centre.
- Not today: a picture reaches the service about two to four weeks after its
  16 days began. The reading's source names the day the picture starts.

Needs only the Python standard library.

Environment:
  REGION_RUNNER_CACHE   folder for the cache (ndvi.json), default ./cache
  GREENNESS_MINUTES     stop asking the service after this long, default 20
"""

import datetime as dt
import json
import math
import os
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

ORNL = "https://modis.ornl.gov/rst/api/v1/MOD13Q1"
HERE = Path(__file__).resolve().parent
CACHE = Path(os.environ.get("REGION_RUNNER_CACHE", HERE / "cache"))

NDVI_BAND = "250m_16_days_NDVI"
RELIABILITY_BAND = "250m_16_days_pixel_reliability"
NDVI_SCALE = 0.0001
NDVI_VALID = (-2000, 10000)  # the fill value is -3000
# Pixel reliability: 0 good, 1 marginal, 2 snow or ice, 3 cloud, -1 no data.
USABLE_RELIABILITY = (0, 1)
WINDOW_KM = 10  # the service answers with 81 x 81 pixels, about 19 km a side
SPHERE_RADIUS_M = 6371007.181  # of the MODIS sinusoidal grid
MOST_PICTURES_PER_CALL = 10  # the service refuses more
NORMAL_YEARS = 10
MIN_NORMAL_YEARS = 5
MIN_USABLE_SHARE = 0.5
MIN_PIXELS_INSIDE = 200
# A picture counts for a day once half of its 16 days lie before that day.
HALF_PICTURE_DAYS = 8
# The newest picture can be this old and still stand for "now".
OLDEST_PICTURE_DAYS = 48
PAUSE_S = 1.0
GIVE_UP_AFTER_FAILURES = 5
DATES_POINT = (36.19, 44.01)  # the list of pictures is the same everywhere here


def log(message):
    print(f"{dt.datetime.now(dt.timezone.utc):%Y-%m-%d %H:%M:%S}Z {message}", flush=True)


class ServiceDown(Exception):
    pass


def ask(path, query):
    """One call to the subset service, with retries. Raises OSError when it will not answer."""
    request = urllib.request.Request(
        f"{ORNL}/{path}?{urllib.parse.urlencode(query)}",
        headers={"accept": "application/json", "user-agent": "farm-doctor-region-runner"},
    )
    for attempt in range(3):
        try:
            with urllib.request.urlopen(request, timeout=120) as response:
                return json.load(response)
        except urllib.error.HTTPError as error:
            # A 4xx is our mistake and will not get better by asking again.
            if error.code < 500 and error.code != 429:
                raise OSError(f"{error.code}: {error.read()[:200]!r}") from error
            problem = error
        except (OSError, ValueError) as error:
            problem = error
        if attempt == 2:
            raise OSError(str(problem))
        time.sleep(10 * (attempt + 1))


def picture_day(code):
    """The first day of a picture named like A2026257."""
    return dt.date(int(code[1:5]), 1, 1) + dt.timedelta(days=int(code[5:8]) - 1)


def load_cache():
    path = CACHE / "ndvi.json"
    cache = json.loads(path.read_text()) if path.exists() else {}
    cache.setdefault("pictures", [])
    cache.setdefault("pictures_checked", "")
    cache.setdefault("districts", {})
    return cache


def save_cache(cache):
    CACHE.mkdir(parents=True, exist_ok=True)
    temporary = CACHE / "ndvi.json.tmp"
    temporary.write_text(json.dumps(cache))
    temporary.replace(CACHE / "ndvi.json")


def refresh_pictures(cache, today):
    """The list of pictures the service holds, asked for once a day."""
    if cache["pictures_checked"] == today.isoformat() and cache["pictures"]:
        return
    try:
        body = ask("dates", {"latitude": DATES_POINT[0], "longitude": DATES_POINT[1]})
        pictures = sorted(entry["modis_date"] for entry in body["dates"])
    except (OSError, KeyError, TypeError) as error:
        log(f"greenness: could not read the list of pictures ({error}); using the last known list")
        return
    # The list only ever grows; an answer that lost pictures is a bad answer.
    if len(pictures) >= len(cache["pictures"]):
        cache["pictures"] = pictures
        cache["pictures_checked"] = today.isoformat()
        save_cache(cache)


def picture_for(as_of, pictures):
    """The picture that stands for a day: the latest one with half of its 16
    days before that day. None when the service has nothing recent enough."""
    best = None
    for code in pictures:
        start = picture_day(code)
        if start + dt.timedelta(days=HALF_PICTURE_DAYS) <= as_of:
            best = code
    if best is None or (as_of - picture_day(best)).days > OLDEST_PICTURE_DAYS:
        return None
    return best


def same_days_before(code):
    """The same 16 days in each of the ten years before, newest first."""
    year, day_of_year = int(code[1:5]), code[5:8]
    return [f"A{year - back}{day_of_year}" for back in range(1, NORMAL_YEARS + 1)]


def wanted_pictures(as_of, pictures):
    code = picture_for(as_of, pictures)
    if code is None:
        return []
    known = set(pictures)
    return [code] + [past for past in same_days_before(code) if past in known]


def inside(lat, lon, ring):
    hit = False
    for (x1, y1), (x2, y2) in zip(ring, ring[1:] + ring[:1]):
        if (y1 > lat) != (y2 > lat) and lon < (x2 - x1) * (lat - y1) / (y2 - y1) + x1:
            hit = not hit
    return hit


_INSIDE = {}


def pixels_inside(answer, rings):
    """Which pixels of an answer have their centre inside the district.

    The answer gives the lower left corner of the square on the MODIS
    sinusoidal grid; its values run row by row from the north-west corner
    (checked against the service with two overlapping squares).
    """
    x0, y0 = float(answer["xllcorner"]), float(answer["yllcorner"])
    size, rows, columns = float(answer["cellsize"]), int(answer["nrows"]), int(answer["ncols"])
    # Every call for one district answers with the same square.
    square = (x0, y0, rows, columns)
    if square in _INSIDE:
        return _INSIDE[square]
    chosen = []
    for row in range(rows):
        y = y0 + (rows - row - 0.5) * size
        lat = y / SPHERE_RADIUS_M
        for column in range(columns):
            x = x0 + (column + 0.5) * size
            lon = math.degrees(x / (SPHERE_RADIUS_M * math.cos(lat)))
            if any(inside(math.degrees(lat), lon, ring) for ring in rings):
                chosen.append(row * columns + column)
    _INSIDE[square] = chosen
    return chosen


def fetch_pictures(district, rings, first, last):
    """Mean NDVI of the usable land pixels for every picture from `first` to
    `last`: {code: [mean or None, pixels used, pixels inside the district]}."""
    query = {
        "latitude": district["lat"],
        "longitude": district["lon"],
        "startDate": first,
        "endDate": last,
        "kmAboveBelow": WINDOW_KM,
        "kmLeftRight": WINDOW_KM,
    }
    ndvi = ask("subset", {**query, "band": NDVI_BAND})
    time.sleep(PAUSE_S)
    reliability = ask("subset", {**query, "band": RELIABILITY_BAND})
    time.sleep(PAUSE_S)

    chosen = pixels_inside(ndvi, rings)
    if len(chosen) < MIN_PIXELS_INSIDE:
        raise ValueError(f"only {len(chosen)} pixels of the square lie inside the district")
    ranks = {entry["modis_date"]: entry["data"] for entry in reliability["subset"]}
    found = {}
    for entry in ndvi["subset"]:
        code, values = entry["modis_date"], entry["data"]
        rank = ranks.get(code)
        if rank is None or len(rank) != len(values):
            continue
        usable = [
            values[index]
            for index in chosen
            if rank[index] in USABLE_RELIABILITY and NDVI_VALID[0] <= values[index] <= NDVI_VALID[1]
        ]
        # Open water reads below zero. A lake that is fuller than in other
        # years would otherwise show as land that lost its green.
        land = [value for value in usable if value >= 0]
        enough = len(usable) >= MIN_USABLE_SHARE * len(chosen) and len(land) >= MIN_PIXELS_INSIDE
        mean = round(sum(land) / len(land) * NDVI_SCALE, 4) if enough else None
        found[code] = [mean, len(land), len(chosen)]
    return found


def spans(missing, pictures):
    """Groups the missing pictures into calls: one call per block of ten
    neighbouring pictures, from the first to the last one missing in it."""
    position = {code: index for index, code in enumerate(pictures)}
    blocks = {}
    for code in missing:
        blocks.setdefault(position[code] // MOST_PICTURES_PER_CALL, []).append(position[code])
    return [(pictures[min(found)], pictures[max(found)]) for _, found in sorted(blocks.items(), reverse=True)]


def update(cache, districts, shapes, days, deadline):
    """Fetches what the given days need and the cache lacks, district by
    district, until done or out of time. Returns (calls made, finished)."""
    pictures = cache["pictures"]
    rings = {shape["slug"]: shape["rings"] for shape in shapes}
    calls, failures = 0, 0
    for district in districts:
        slug = district["slug"]
        have = cache["districts"].setdefault(slug, {})
        missing = {
            code for day in days for code in wanted_pictures(day, pictures) if code not in have
        }
        for first, last in spans(missing, pictures):
            if time.monotonic() > deadline:
                log(f"greenness: out of time at {slug}; the next run carries on from the cache")
                return calls, False
            try:
                have.update(fetch_pictures(district, rings.get(slug, []), first, last))
                calls += 2
                failures = 0
            except ValueError as error:
                log(f"greenness: {slug}: {error}; no greenness for it")
                break
            except (OSError, KeyError, TypeError) as error:
                calls += 2
                failures += 1
                log(f"greenness: {slug} {first} to {last}: the service did not answer ({error})")
                if failures >= GIVE_UP_AFTER_FAILURES:
                    save_cache(cache)
                    raise ServiceDown(f"{failures} calls in a row failed") from error
                continue
            save_cache(cache)
    return calls, True


def greenness_for(have, as_of, pictures):
    """What the cache says about one district on one day, or None:
    {pct, ndvi, normal, years, picture (its first day)}."""
    code = picture_for(as_of, pictures)
    now = (have.get(code) or [None])[0] if code else None
    if now is None:
        return None
    past = [(have.get(earlier) or [None])[0] for earlier in same_days_before(code)]
    past = [mean for mean in past if mean is not None]
    if len(past) < MIN_NORMAL_YEARS:
        return None
    normal = sum(past) / len(past)
    # Against bare ground or water a ratio says nothing.
    if normal < 0.05:
        return None
    pct = max(-100.0, min(300.0, (now / normal - 1) * 100))
    return {
        "pct": round(pct, 1),
        "ndvi": now,
        "normal": round(normal, 4),
        "years": len(past),
        "picture": picture_day(code),
    }


def month_ends(today, months):
    """The last day of each of the `months` whole months before this one, newest first."""
    ends, first = [], today.replace(day=1)
    for _ in range(months):
        end = first - dt.timedelta(days=1)
        ends.append(end)
        first = end.replace(day=1)
    return ends


def minutes_allowed():
    return float(os.environ.get("GREENNESS_MINUTES", "20"))


def main():
    arguments = sys.argv[1:]
    months = 0
    if "--months" in arguments:
        at = arguments.index("--months")
        months = int(arguments[at + 1])
        del arguments[at : at + 2]
    districts = json.loads((HERE / "districts.json").read_text())
    if arguments:
        districts = [d for d in districts if d["slug"] in arguments]
    shapes = json.loads((HERE / "district_shapes.json").read_text())
    today = dt.datetime.now(dt.timezone.utc).date()

    cache = load_cache()
    refresh_pictures(cache, today)
    if not cache["pictures"]:
        sys.exit("the list of pictures could not be read and none is cached")
    deadline = time.monotonic() + minutes_allowed() * 60
    calls, finished = 0, True
    try:
        # Today first, for every district; the past months after.
        for days in ([today], month_ends(today, months)):
            if days and finished:
                made, finished = update(cache, districts, shapes, days, deadline)
                calls += made
    except ServiceDown as error:
        log(f"greenness: stopped, {error}")
        finished = False

    found = 0
    for district in districts:
        result = greenness_for(cache["districts"].get(district["slug"], {}), today, cache["pictures"])
        if result is None:
            log(f"{district['slug']}: no greenness yet")
            continue
        found += 1
        log(
            f"{district['slug']}: NDVI {result['ndvi']:.3f} from {result['picture']}, "
            f"{result['years']}-yr mean {result['normal']:.3f}, greenness {result['pct']:+.1f}%"
        )
    log(f"done: {found} districts with greenness, {calls} calls, {'complete' if finished else 'not complete'}")
    if not finished:
        sys.exit(1)


if __name__ == "__main__":
    main()
