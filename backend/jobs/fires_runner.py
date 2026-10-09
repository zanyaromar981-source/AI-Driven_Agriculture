#!/usr/bin/env python3
"""Fires runner: satellite fire detections in the 33 districts, without the
gas flares.

Runs on a timer (every 3 hours). It downloads NASA FIRMS active-fire files
(three VIIRS satellites and MODIS, the last 7 days, no key needed), keeps the
detections that fall inside a district, removes the ones that are permanent
heat sources (gas flares at oil and gas fields), groups what is left into
fires and pushes them to the backend.

Telling a flare from a fire:
- A flare burns at the same spot day and night, for weeks. A crop or grass
  fire moves and is over in hours; most are seen in daylight.
- So each spot (a cell of about 1 km) is remembered with the days it was hot
  and whether it was hot at night. A spot that was hot on 3 or more different
  days in the last 30, at night on at least 2 of them, is treated as a flare,
  and so are the cells touching it. Detections there are not pushed.
- This memory builds up over time (`cache/fire_cells.json`). On the first
  run it only knows the last 7 days.
What this gets wrong: a real fire that burns in one place for 3 days and
nights (a forest or peat fire) is hidden from its third day; a new flare is
shown as a fire for its first two days. The count of masked detections is
logged on every run.

Other limits: a detection reaches NASA's files about 3 hours after the
satellite passed, and a satellite passes a few times a day, so a short fire
between passes is never seen. The burned area is not measured.

Needs only the Python standard library.

Environment:
  FARM_DOCTOR_API       base address, default http://localhost:8790/v1
  INGEST__SERVICE_KEY   the backend's ingest key (required)
  FIRES_RUNNER_CACHE    folder for the flare memory, default ./cache
  FIRES_PUSH_HOURS      push fires seen in the last N hours, default 48

Run with --dry-run to print what would be pushed without pushing.
"""

import csv
import datetime as dt
import io
import json
import math
import os
import sys
import urllib.error
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
CACHE = Path(os.environ.get("FIRES_RUNNER_CACHE", HERE / "cache"))
API = os.environ.get("FARM_DOCTOR_API", "http://localhost:8790/v1").rstrip("/")
KEY = os.environ.get("INGEST__SERVICE_KEY", "")
PUSH_HOURS = int(os.environ.get("FIRES_PUSH_HOURS", "48"))

FIRMS = "https://firms.modaps.eosdis.nasa.gov/data/active_fire"
FILES = {
    "VIIRS NOAA-20": f"{FIRMS}/noaa-20-viirs-c2/csv/J1_VIIRS_C2_Russia_Asia_7d.csv",
    "VIIRS NOAA-21": f"{FIRMS}/noaa-21-viirs-c2/csv/J2_VIIRS_C2_Russia_Asia_7d.csv",
    "VIIRS Suomi NPP": f"{FIRMS}/suomi-npp-viirs-c2/csv/SUOMI_VIIRS_C2_Russia_Asia_7d.csv",
    "MODIS": f"{FIRMS}/modis-c6.1/csv/MODIS_C6_1_Russia_Asia_7d.csv",
}
# A box around the region, to throw most of the file away before the exact
# district test.
BOX = (34.0, 37.6, 42.0, 46.6)  # south, north, west, east

FLARE_CELL_DEG = 0.01  # about 1 km
FLARE_MIN_DAYS = 3
FLARE_MIN_NIGHT_DAYS = 2
FLARE_MEMORY_DAYS = 30
FIRE_CELL_DEG = 0.02  # detections this close on one day are one fire
MODIS_MIN_CONFIDENCE = 30
SOURCE = "NASA FIRMS (VIIRS, MODIS), about 3 h behind the pass; gas flare spots removed"


def log(message):
    print(f"{dt.datetime.now(dt.timezone.utc):%Y-%m-%d %H:%M:%S}Z {message}", flush=True)


def download(url):
    request = urllib.request.Request(url, headers={"user-agent": "farm-doctor-fires-runner"})
    with urllib.request.urlopen(request, timeout=180) as response:
        return response.read().decode("utf-8", "replace")


def detections():
    """Every usable detection inside the box, from all four files."""
    south, north, west, east = BOX
    found = []
    for satellite, url in FILES.items():
        try:
            rows = csv.DictReader(io.StringIO(download(url)))
        except (urllib.error.URLError, OSError) as error:
            log(f"{satellite}: could not download ({error}); continuing without it")
            continue
        kept = 0
        for row in rows:
            try:
                lat, lon = float(row["latitude"]), float(row["longitude"])
            except (KeyError, ValueError):
                continue
            if not (south <= lat <= north and west <= lon <= east):
                continue
            confidence = row.get("confidence", "")
            if confidence in ("low", "l"):
                continue
            if confidence.isdigit() and int(confidence) < MODIS_MIN_CONFIDENCE:
                continue
            try:
                when = dt.datetime.strptime(
                    f"{row['acq_date']} {int(row['acq_time']):04d}", "%Y-%m-%d %H%M"
                ).replace(tzinfo=dt.timezone.utc)
            except (KeyError, ValueError):
                continue
            found.append(
                {
                    "lat": lat,
                    "lon": lon,
                    "when": when,
                    "night": row.get("daynight", "") == "N",
                    "frp": float(row.get("frp") or 0),
                    "satellite": satellite,
                }
            )
            kept += 1
        log(f"{satellite}: {kept} detections in the box")
    return found


def inside(lat, lon, ring):
    hit = False
    for (x1, y1), (x2, y2) in zip(ring, ring[1:] + ring[:1]):
        if (y1 > lat) != (y2 > lat) and lon < (x2 - x1) * (lat - y1) / (y2 - y1) + x1:
            hit = not hit
    return hit


def district_of(lat, lon, districts):
    for district in districts:
        if any(inside(lat, lon, ring) for ring in district["rings"]):
            return district
    return None


def cell(lat, lon, size):
    return (math.floor(lat / size), math.floor(lon / size))


def load_memory():
    path = CACHE / "fire_cells.json"
    return json.loads(path.read_text()) if path.exists() else {}


def save_memory(memory):
    CACHE.mkdir(parents=True, exist_ok=True)
    temporary = CACHE / "fire_cells.json.tmp"
    temporary.write_text(json.dumps(memory))
    temporary.replace(CACHE / "fire_cells.json")


def remember(memory, found, today):
    """Adds what was seen to the memory and forgets what is too old."""
    for one in found:
        key = "{},{}".format(*cell(one["lat"], one["lon"], FLARE_CELL_DEG))
        day = one["when"].date().isoformat()
        seen = memory.setdefault(key, {})
        seen[day] = bool(seen.get(day)) or one["night"]
    oldest = (today - dt.timedelta(days=FLARE_MEMORY_DAYS)).isoformat()
    for key in list(memory):
        memory[key] = {day: night for day, night in memory[key].items() if day >= oldest}
        if not memory[key]:
            del memory[key]


def flare_cells(memory):
    """Cells that behave like a flare, and the cells touching them."""
    flares = set()
    for key, days in memory.items():
        if len(days) >= FLARE_MIN_DAYS and sum(1 for night in days.values() if night) >= FLARE_MIN_NIGHT_DAYS:
            row, column = (int(part) for part in key.split(","))
            for d_row in (-1, 0, 1):
                for d_column in (-1, 0, 1):
                    flares.add((row + d_row, column + d_column))
    return flares


def group(found):
    """Detections close together on one day become one fire."""
    fires = {}
    for one in found:
        row, column = cell(one["lat"], one["lon"], FIRE_CELL_DEG)
        key = (one["when"].date().isoformat(), row, column)
        fires.setdefault(key, []).append(one)
    return fires


def farms_near(lat, lon, farms, km=5.0):
    count = 0
    for farm in farms:
        d_lat = (farm["lat"] - lat) * 111.2
        d_lon = (farm["lon"] - lon) * 111.2 * math.cos(math.radians(lat))
        if d_lat * d_lat + d_lon * d_lon <= km * km:
            count += 1
    return count


def call(method, path, body=None):
    data = json.dumps(body).encode() if body is not None else None
    headers = {"x-service-key": KEY, "content-type": "application/json"}
    request = urllib.request.Request(f"{API}{path}", data=data, method=method, headers=headers)
    with urllib.request.urlopen(request, timeout=60) as response:
        raw = response.read()
        return json.loads(raw) if raw else {}


def main():
    dry_run = "--dry-run" in sys.argv
    if not KEY and not dry_run:
        sys.exit("INGEST__SERVICE_KEY is not set")

    districts = json.loads((HERE / "district_shapes.json").read_text())
    now = dt.datetime.now(dt.timezone.utc)

    found = detections()
    if not found:
        sys.exit("no fire file could be read")

    memory = load_memory()
    remember(memory, found, now.date())
    if not dry_run:
        save_memory(memory)
    flares = flare_cells(memory)

    in_districts, masked = [], 0
    for one in found:
        district = district_of(one["lat"], one["lon"], districts)
        if district is None:
            continue
        if cell(one["lat"], one["lon"], FLARE_CELL_DEG) in flares:
            masked += 1
            continue
        one["district"] = district
        in_districts.append(one)
    log(
        f"{len(found)} detections in the box, {len(in_districts) + masked} inside the districts, "
        f"{masked} of those at flare spots and removed ({len(flares)} flare cells known)"
    )

    try:
        farms = [] if dry_run and not KEY else call("GET", "/ingest/farms").get("farms", [])
    except (urllib.error.URLError, OSError) as error:
        log(f"could not read the farms ({error}); pushing fires without a farm count")
        farms = []

    since = now - dt.timedelta(hours=PUSH_HOURS)
    pushed, failed = 0, 0
    for (day, row, column), members in sorted(group(in_districts).items()):
        latest = max(one["when"] for one in members)
        if latest < since:
            continue
        lat = sum(one["lat"] for one in members) / len(members)
        lon = sum(one["lon"] for one in members) / len(members)
        district = members[0]["district"]
        body = {
            "lat": round(lat, 5),
            "lon": round(lon, 5),
            "zone_slug": district["slug"],
            "place_en": f"{district['name']} district",
            "place_ku": None,
            "detected_at": latest.isoformat(),
            "area_ha": None,
            "wind_kmh": None,
            "wind_direction": None,
            "status": "active",
            "farms_within_5km": farms_near(lat, lon, farms) if farms else None,
            "farmers_alerted": None,
            "source": f"{SOURCE}; {len(members)} detections"[:120],
        }
        external_id = f"firms-{day}-{row}-{column}"
        if dry_run:
            log(f"would push {external_id}: {district['slug']} {body['lat']},{body['lon']} at {latest:%H:%M} UTC, {len(members)} detections")
            pushed += 1
            continue
        try:
            call("PUT", f"/ingest/fires/{external_id}", body)
            pushed += 1
        except urllib.error.HTTPError as error:
            log(f"{external_id}: backend refused with {error.code}: {error.read()[:200]!r}")
            failed += 1
        except OSError as error:
            log(f"{external_id}: backend unreachable: {error}")
            failed += 1

    log(f"done: {pushed} fires {'would be ' if dry_run else ''}pushed, {failed} failed")
    if failed:
        sys.exit(1)


if __name__ == "__main__":
    main()
