#!/usr/bin/env python3
"""Dams runner: the water area of Dukan and Darbandikhan lakes from satellite.

Runs on a timer (once a day). For each lake it looks for Sentinel-2 passes of
the last 30 days, measures the water area on every clear pass that is not
stored yet, and pushes one reading per day to the backend.

The method is the one tested in evidence/past_seasons/backtest/
dam_water_test.py (165 measurements, 2008 to 2026), ported here unchanged:
- Sentinel-2 L2A from Microsoft Planetary Computer (free, no key). Its
  statistics service counts the pixels; no image is downloaded.
- Each lake is one rectangle in UTM 38N that holds the whole reservoir, read
  at exactly 20 m per pixel. The rectangle is cut along the tile borders and
  each piece is read from its own tile of the same pass.
- Water = NDWI (B03-B08)/(B03+B08) > 0. Area = water pixels x 0.0004 km2.
- Dukan is measured only from relative orbit 135 (orbit 92 cuts a corner of
  the rectangle), Darbandikhan only from orbit 92 (the only one covering it).
- A pass is tried when every tile reports under 20% cloud. It is "clear", and
  pushed, when cloud and cloud shadow (scene classes 3, 8, 9, 10) cover under
  0.5% of the rectangle and every piece is at least 99.5% inside the swath.

What is pushed, and what it is not:
- lake_area_km2: measured.
- pct_full: the backend requires it. It is the lake AREA as a share of the
  full lake area (Dukan 270 km2, Darbandikhan 113 km2: the official full
  areas quoted in evidence/past_seasons/BACKTEST_RESULTS.md section 6d, where
  our own fullest measurements are 271 to 276 and 103 to 112). It is NOT the
  stored volume as a share of capacity. A lake loses volume faster than
  area, so this figure reads higher than the true volume share: in June 2025
  Dukan was reported at 24% of its volume while its area was 42% of full.
  An area a little above the full area is sent as 100.
- volume_bn_m3 and farm_supply_bn_m3: not known, sent empty. No tested
  area-to-volume curve exists for these lakes (see jobs/README.md).

Other limits: a lake is passed every 2 to 5 days and winter passes are often
cloudy, so readings can be weeks apart; a pass reaches Planetary Computer
some hours to two days after it was flown; haze moves a reading by a few km2.

--backfill pushes the tested history in dam_history.csv (the clear rows of
the backtest: Landsat 5 and 8 at 30 m for 2008, 2013 and 2019, Sentinel-2
from 2017) and measures nothing new. Run it once on a new server.

Needs only the Python standard library.

Environment:
  FARM_DOCTOR_API       base address, default http://localhost:8790/v1
  INGEST__SERVICE_KEY   the backend's ingest key (required)
  DAMS_LOOKBACK_DAYS    how far back to look for passes, default 30

Run with --dry-run to print what would be pushed without pushing.
"""

import concurrent.futures as cf
import csv
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

HERE = Path(__file__).resolve().parent
API = os.environ.get("FARM_DOCTOR_API", "http://localhost:8790/v1").rstrip("/")
KEY = os.environ.get("INGEST__SERVICE_KEY", "")
LOOKBACK_DAYS = int(os.environ.get("DAMS_LOOKBACK_DAYS", "30"))

STAC = "https://planetarycomputer.microsoft.com/api/stac/v1/search"
STATS = "https://planetarycomputer.microsoft.com/api/data/v1/item/statistics"
UA = (
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 "
    "(KHTML, like Gecko) Chrome/128.0 Safari/537.36"
)
# The statistics service answers in 3 to 7 seconds when it is well and now
# and then with a 504. A pass that cannot be read is tried again on the next
# run, so a run gives up on a lake after a few failures rather than waiting.
HTTP_TIMEOUT_S = 60
HTTP_ATTEMPTS = 4
MAX_FAILED_PASSES = 2

# Lake rectangles in UTM 38N metres (x0, y0, x1, y1) and their pieces by
# Sentinel-2 tile. Cuts at x=505000, y=3995000 and y=3895000, the middle of
# the tile overlaps. Copied from the tested script; do not change one without
# measuring the history again.
LAKES = {
    "dukan": {
        "name": "Dukan",
        "full_km2": 270.0,
        "orbits": {135},
        "rect": (471000, 3977000, 522000, 4020000),
        "pieces": {
            "38SME": (471000, 3977000, 505000, 3995000),
            "38SMF": (471000, 3995000, 505000, 4020000),
            "38SNE": (505000, 3977000, 522000, 3995000),
            "38SNF": (505000, 3995000, 522000, 4020000),
        },
    },
    "darbandikhan": {
        "name": "Darbandikhan",
        "full_km2": 113.0,
        "orbits": {92},
        "rect": (545000, 3885000, 595000, 3925000),
        "pieces": {
            "38SND": (545000, 3885000, 595000, 3895000),
            "38SNE": (545000, 3895000, 595000, 3925000),
        },
    },
}
MAX_TILE_CLOUD = 20
MAX_CLOUD_PCT = 0.5
MIN_VALID_PCT = 99.5
PIXEL_M = 20

# The three bands and the histogram edges of the tested script. Only NDWI and
# the scene classes are used here; MNDWI stays in the request so the service
# is asked exactly what it was asked in the test.
S2_EXPR = "(B03-B11)/(B03+B11);(B03-B08)/(B03+B08);SCL"
IDX_EDGES = [-1.0001, -0.5, -0.4, -0.3, -0.25, -0.2, -0.15, -0.1, -0.05, 0, 0.05, 0.1, 0.15, 0.2, 0.3, 0.5, 1.0001]
BINS = IDX_EDGES + [2.5, 3.5, 5.5, 6.5, 7.5, 10.5, 11.5]

# The backend keeps a source of at most 100 characters.
SOURCE_LIMIT = 100


def log(message):
    print(f"{dt.datetime.now(dt.timezone.utc):%Y-%m-%d %H:%M:%S}Z {message}", flush=True)


class SatelliteError(Exception):
    """Planetary Computer did not answer, or answered with something unusable."""


# ---------------------------------------------------------------- the tested method
def bin_of(value):
    for i in range(len(BINS) - 1):
        if BINS[i] <= value < BINS[i + 1]:
            return i
    raise ValueError(value)


SCL_SHADOW, SCL_CLOUD = bin_of(3), bin_of(9)


def km2_above(hist, threshold, pixel_km2):
    """Area of the pixels in the index bins that start at the threshold or above."""
    return sum(hist[i] for i in range(len(IDX_EDGES) - 1) if IDX_EDGES[i] >= threshold - 1e-9) * pixel_km2


def utm2ll(easting, northing, zone=38):
    """UTM to latitude and longitude (Krueger series, WGS84). Only for the search box."""
    a = 6378137.0
    f = 1 / 298.257223563
    k0 = 0.9996
    n = f / (2 - f)
    big_a = a / (1 + n) * (1 + n**2 / 4 + n**4 / 64)
    be = [n / 2 - 2 / 3 * n**2 + 37 / 96 * n**3, 1 / 48 * n**2 + 1 / 15 * n**3, 17 / 480 * n**3]
    de = [2 * n - 2 / 3 * n**2 - 2 * n**3, 7 / 3 * n**2 - 8 / 5 * n**3, 56 / 15 * n**3]
    xi = northing / (k0 * big_a)
    eta = (easting - 500000) / (k0 * big_a)
    xi_ = xi - sum(be[j] * math.sin(2 * (j + 1) * xi) * math.cosh(2 * (j + 1) * eta) for j in range(3))
    eta_ = eta - sum(be[j] * math.cos(2 * (j + 1) * xi) * math.sinh(2 * (j + 1) * eta) for j in range(3))
    chi = math.asin(math.sin(xi_) / math.cosh(eta_))
    lat = chi + sum(de[j] * math.sin(2 * (j + 1) * chi) for j in range(3))
    lon = math.radians(zone * 6 - 183) + math.atan2(math.sinh(eta_), math.cos(xi_))
    return math.degrees(lat), math.degrees(lon)


def rect_ll_polygon(rect):
    x0, y0, x1, y1 = rect
    points = [utm2ll(x, y) for x, y in ((x0, y0), (x1, y0), (x1, y1), (x0, y1))]
    lats = [p[0] for p in points]
    lons = [p[1] for p in points]
    a, b, c, d = min(lons) - 0.01, min(lats) - 0.01, max(lons) + 0.01, max(lats) + 0.01
    return {"type": "Polygon", "coordinates": [[[a, b], [c, b], [c, d], [a, d], [a, b]]]}


def utm_feature(rect):
    x0, y0, x1, y1 = rect
    return {
        "type": "Feature",
        "properties": {},
        "geometry": {"type": "Polygon", "coordinates": [[[x0, y0], [x1, y0], [x1, y1], [x0, y1], [x0, y0]]]},
    }


def post_json(url, body):
    last = None
    for attempt in range(HTTP_ATTEMPTS):
        if attempt:
            time.sleep(2 + 3 * attempt)
        request = urllib.request.Request(
            url, data=json.dumps(body).encode(), headers={"User-Agent": UA, "Content-Type": "application/json"}
        )
        try:
            with urllib.request.urlopen(request, timeout=HTTP_TIMEOUT_S) as response:
                return json.loads(response.read())
        except (urllib.error.URLError, OSError, ValueError) as error:
            last = error
    raise SatelliteError(f"{url.split('?')[0]} did not answer after {HTTP_ATTEMPTS} tries: {last}")


def stac_search(intersects, datetime_, query):
    body = {"collections": ["sentinel-2-l2a"], "intersects": intersects, "datetime": datetime_, "limit": 250, "query": query}
    found = []
    while True:
        page = post_json(STAC, body)
        found += page.get("features", [])
        following = [link for link in page.get("links", []) if link.get("rel") == "next"]
        if not following or not page.get("features") or not following[0].get("body"):
            return found
        body = following[0]["body"]


def item_stats(item, rect):
    x0, y0, x1, y1 = rect
    width = int(round((x1 - x0) / PIXEL_M))
    height = int(round((y1 - y0) / PIXEL_M))
    query = {
        "collection": "sentinel-2-l2a",
        "item": item,
        "coord_crs": "EPSG:32638",
        "dst_crs": "EPSG:32638",
        "width": width,
        "height": height,
        "expression": S2_EXPR,
        "asset_as_band": "true",
        "histogram_bins": ",".join(str(edge) for edge in BINS),
    }
    answer = post_json(STATS + "?" + urllib.parse.urlencode(query), utm_feature(rect))
    try:
        statistics = answer["properties"]["statistics"]
    except (KeyError, TypeError) as error:
        raise SatelliteError(f"statistics answer for {item} has no statistics: {error}") from error
    pixel_km2 = ((x1 - x0) / width) * ((y1 - y0) / height) / 1e6
    return statistics, pixel_km2, width * height


def passes(lake, start, end):
    """Passes that have every tile of the lake, from the lake's orbit, with each tile under 20% cloud."""
    pieces = lake["pieces"]
    features = stac_search(rect_ll_polygon(lake["rect"]), f"{start}/{end}", {"eo:cloud_cover": {"lt": MAX_TILE_CLOUD}})
    groups = {}
    for feature in features:
        properties = feature["properties"]
        tile = properties.get("s2:mgrs_tile")
        if tile not in pieces:
            continue
        key = (properties["datetime"][:10], properties.get("sat:relative_orbit"), properties.get("platform"))
        group = groups.setdefault(key, {})
        # When a tile was processed twice, the newer processing is used.
        if tile not in group or feature["id"] > group[tile]:
            group[tile] = feature["id"]
    full = []
    for (day, orbit, platform), tiles in groups.items():
        if all(tile in tiles for tile in pieces) and orbit in lake["orbits"]:
            full.append({"day": day, "orbit": orbit, "platform": platform or "Sentinel-2", "tiles": tiles})
    return sorted(full, key=lambda one: (one["day"], one["platform"]))


def measure(lake, one):
    """Water area, cloud share and swath cover of one pass. All pieces are read at once."""
    pieces = lake["pieces"]
    with cf.ThreadPoolExecutor(len(pieces)) as pool:
        results = list(pool.map(lambda piece: item_stats(one["tiles"][piece[0]], piece[1]), pieces.items()))
    water = cloud = area = 0.0
    min_valid = 100.0
    for statistics, pixel_km2, pixel_count in results:
        try:
            mndwi, ndwi, scl = (statistics[key] for key in list(statistics)[:3])
            water += km2_above(ndwi["histogram"][0], 0, pixel_km2)
            cloud += (scl["histogram"][0][SCL_SHADOW] + scl["histogram"][0][SCL_CLOUD]) * pixel_km2
            min_valid = min(min_valid, mndwi["valid_percent"])
        except (KeyError, IndexError, TypeError, ValueError) as error:
            raise SatelliteError(f"statistics answer has an unexpected shape: {error}") from error
        area += pixel_count * pixel_km2
    cloud_pct = 100 * cloud / area
    return {
        "water_km2": water,
        "cloud_pct": cloud_pct,
        "min_valid": min_valid,
        "clear": cloud_pct < MAX_CLOUD_PCT and min_valid >= MIN_VALID_PCT,
    }


# ---------------------------------------------------------------- readings
def reading(lake, day, satellite, resolution_m, water_km2):
    """The body of one push. pct_full is the area share, and the source says so."""
    full = lake["full_km2"]
    source = (
        f"{satellite} {day}, NDWI>0 water at {resolution_m} m. "
        f"pct_full = lake AREA / full {full:.0f} km2, not volume"
    )
    if len(source) > SOURCE_LIMIT:
        raise ValueError(f"source is {len(source)} characters, the backend takes {SOURCE_LIMIT}: {source}")
    return {
        "pct_full": round(min(100.0, 100 * water_km2 / full), 1),
        "volume_bn_m3": None,
        "lake_area_km2": round(water_km2, 2),
        "farm_supply_bn_m3": None,
        "source": source,
    }


def history_readings():
    """The clear rows of the backtest, one per lake and day, as (slug, day, body)."""
    slugs = {lake["name"]: slug for slug, lake in LAKES.items()}
    chosen = {}
    with open(HERE / "dam_history.csv", newline="") as handle:
        for row in csv.DictReader(handle):
            slug = slugs[row["lake"]]
            parts = row["source"].split()
            if parts[0] == "Sentinel-2":
                satellite, resolution_m = f"Sentinel-{parts[1]}", 20
            else:
                satellite, resolution_m = parts[0].replace("landsat-", "Landsat "), 30
            key = (slug, row["date"])
            # One day was measured by both; the 20 m Sentinel-2 reading is kept.
            if key in chosen and resolution_m > chosen[key][0]:
                continue
            chosen[key] = (resolution_m, reading(LAKES[slug], row["date"], satellite, resolution_m, float(row["water_km2"])))
    return [(slug, day, body) for (slug, day), (_, body) in sorted(chosen.items())]


def call(method, path, body=None):
    data = json.dumps(body).encode() if body is not None else None
    headers = {"x-service-key": KEY, "content-type": "application/json"}
    request = urllib.request.Request(f"{API}{path}", data=data, method=method, headers=headers)
    with urllib.request.urlopen(request, timeout=60) as response:
        raw = response.read()
        return json.loads(raw) if raw else {}


def push(slug, day, body, dry_run):
    """True when the reading was stored (or would be, in a dry run)."""
    what = f"{slug} {day}: {body['lake_area_km2']} km2, {body['pct_full']}% of the full area"
    if dry_run:
        log(f"would push {what}")
        return True
    try:
        call("PUT", f"/ingest/dams/{slug}/readings/{day}", body)
    except urllib.error.HTTPError as error:
        log(f"{slug} {day}: backend refused with {error.code}: {error.read()[:200]!r}")
        return False
    except OSError as error:
        log(f"{slug} {day}: backend unreachable: {error}")
        return False
    log(f"pushed {what}")
    return True


def stored_days(slug, start, end):
    """Days the backend already holds for the lake. A stored day is left alone,
    so a correction made by staff is not written over."""
    answer = call("GET", f"/dams/{slug}/history?from={start}&to={end}")
    return {one["day"] for one in answer.get("readings", [])}


def main():
    dry_run = "--dry-run" in sys.argv
    backfill = "--backfill" in sys.argv
    if not KEY and not dry_run:
        sys.exit("INGEST__SERVICE_KEY is not set")

    pushed = failed = skipped = unclear = 0

    if backfill:
        for slug, day, body in history_readings():
            if push(slug, day, body, dry_run):
                pushed += 1
            else:
                failed += 1
        log(f"done: {pushed} readings {'would be ' if dry_run else ''}pushed from the tested history, {failed} failed")
        sys.exit(1 if failed else 0)

    today = dt.datetime.now(dt.timezone.utc).date()
    start = today - dt.timedelta(days=LOOKBACK_DAYS)
    for slug, lake in LAKES.items():
        try:
            found = passes(lake, start.isoformat(), today.isoformat())
        except SatelliteError as error:
            log(f"{slug}: could not search for passes ({error}); nothing measured, the next run tries again")
            failed += 1
            continue
        try:
            known = stored_days(slug, start, today)
        except (urllib.error.URLError, OSError, ValueError) as error:
            if not dry_run:
                log(f"{slug}: could not read what is stored ({error}); nothing measured")
                failed += 1
                continue
            known = set()
        log(f"{slug}: {len(found)} passes since {start} with every tile under {MAX_TILE_CLOUD}% cloud, {len(known)} days stored")

        failed_here = 0
        done_days = set(known)
        for one in found:
            if one["day"] in done_days:
                skipped += 1
                continue
            if failed_here >= MAX_FAILED_PASSES:
                log(f"{slug}: the satellite service keeps failing; leaving the rest for the next run")
                break
            try:
                result = measure(lake, one)
            except SatelliteError as error:
                log(f"{slug} {one['day']}: could not measure ({error})")
                failed += 1
                failed_here += 1
                continue
            if not result["clear"]:
                log(
                    f"{slug} {one['day']}: not clear (cloud and shadow {result['cloud_pct']:.2f}% of the rectangle, "
                    f"least swath cover {result['min_valid']:.1f}%), not pushed"
                )
                unclear += 1
                continue
            body = reading(lake, one["day"], one["platform"], PIXEL_M, result["water_km2"])
            if push(slug, one["day"], body, dry_run):
                pushed += 1
                done_days.add(one["day"])
            else:
                failed += 1

    log(
        f"done: {pushed} readings {'would be ' if dry_run else ''}pushed, "
        f"{skipped} passes already stored, {unclear} not clear, {failed} failed"
    )
    if failed:
        sys.exit(1)


if __name__ == "__main__":
    main()
