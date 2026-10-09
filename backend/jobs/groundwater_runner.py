#!/usr/bin/env python3
"""Groundwater runner: how wet the ground is around each farm, deep and
shallow, against its own history.

Runs once a day. NASA publishes three world maps every week (GRACE-DA
drought indicators): groundwater storage, root-zone soil moisture and surface
soil moisture, each as a percentile: 50 is normal for the time of year, 10
means only 10% of past years were this dry. The job reads the three values at
each farm's centre and pushes them as the farm's `groundwater` topic.

Read this before trusting the number:
- It is NOT a well depth and not a measurement at the farm. One value covers
  a square of about 25 km. Every farm in that square gets the same number.
- It is a model fed with satellite gravity readings (GRACE-FO), not a probe
  in the ground. NASA describes the percentiles as relative to 1948 to 2012.
- It cannot see one village's wells falling because of pumping.
- No free source gives groundwater depth at farm scale in Iraq. This is the
  only current figure there is, so it is pushed with confidence "unsure".

Needs the `rasterio` package (see the README for the one-line install).

Environment:
  FARM_DOCTOR_API       base address, default http://localhost:8790/v1
  INGEST__SERVICE_KEY   the backend's ingest key (required)

Run with --dry-run to print the values at the 33 district centres and stop.
"""

import datetime as dt
import email.utils
import json
import os
import sys
import tempfile
import urllib.error
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
API = os.environ.get("FARM_DOCTOR_API", "http://localhost:8790/v1").rstrip("/")
KEY = os.environ.get("INGEST__SERVICE_KEY", "")

FOLDER = "https://nasagrace.unl.edu/globaldata/current"
MAPS = {
    "groundwater_percentile": ("gws_perc_025deg_GL.tif", "Groundwater storage"),
    "root_zone_moisture_percentile": ("rtzsm_perc_025deg_GL.tif", "Soil moisture, root zone"),
    "surface_moisture_percentile": ("sfsm_perc_025deg_GL.tif", "Soil moisture, top layer"),
}
SOURCE = "NASA GRACE-DA weekly percentiles, 25 km model cell; not a well depth, not measured at the farm"


def log(message):
    print(f"{dt.datetime.now(dt.timezone.utc):%Y-%m-%d %H:%M:%S}Z {message}", flush=True)


def download(name, folder):
    """Saves one map and returns its path and the day NASA last changed it."""
    path = Path(folder) / name
    request = urllib.request.Request(f"{FOLDER}/{name}", headers={"user-agent": "farm-doctor-groundwater-runner"})
    with urllib.request.urlopen(request, timeout=180) as response:
        path.write_bytes(response.read())
        changed = response.headers.get("Last-Modified")
    day = email.utils.parsedate_to_datetime(changed).date() if changed else dt.date.today()
    return path, day


def read_points(path, points):
    """The map's value at each (lat, lon), or None where the map has no data."""
    import rasterio  # imported here so --help works without it

    with rasterio.open(path) as raster:
        values = []
        for sample in raster.sample([(lon, lat) for lat, lon in points]):
            value = float(sample[0])
            missing = value != value or (raster.nodata is not None and value == raster.nodata)
            values.append(None if missing or not 0 <= value <= 100 else round(value, 1))
        return values


def words(percentile):
    if percentile < 10:
        return "much lower than usual"
    if percentile < 30:
        return "lower than usual"
    if percentile <= 70:
        return "about usual"
    if percentile <= 90:
        return "higher than usual"
    return "much higher than usual"


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

    if dry_run:
        districts = json.loads((HERE / "districts.json").read_text())
        places = [(d["slug"], d["lat"], d["lon"]) for d in districts]
    else:
        farms = call("GET", "/ingest/farms").get("farms", [])
        places = [(farm["id"], farm["lat"], farm["lon"]) for farm in farms]
        if not places:
            log("no farms yet: nothing to do")
            return

    with tempfile.TemporaryDirectory(prefix="groundwater-") as folder:
        columns, as_of = {}, None
        for code, (name, _) in MAPS.items():
            path, day = download(name, folder)
            columns[code] = read_points(path, [(lat, lon) for _, lat, lon in places])
            as_of = day if as_of is None else min(as_of, day)
    log(f"maps last changed by NASA on {as_of}; {len(places)} places read")

    pushed, skipped, failed = 0, 0, 0
    for index, (place, _, _) in enumerate(places):
        row = {code: columns[code][index] for code in MAPS}
        if row["groundwater_percentile"] is None:
            log(f"{place}: the map has no groundwater value here, nothing pushed")
            skipped += 1
            continue
        if dry_run:
            log(f"{place}: {row}")
            pushed += 1
            continue

        ground = row["groundwater_percentile"]
        body = {
            "as_of": as_of.isoformat(),
            "source": SOURCE[:120],
            "confidence": "unsure",
            "summary_en": (
                f"Groundwater in the 25 km area around this farm is {words(ground)} for this time of year "
                f"(drier than {100 - round(ground)}% of past years). This is a satellite and model estimate "
                "for the wider area, not a measurement of your well."
            ),
            "summary_ku": None,
            "measures": [
                {"code": code, "value": row[code], "unit": "percentile", "label_en": MAPS[code][1], "label_ku": None}
                for code in MAPS
                if row[code] is not None
            ],
        }
        try:
            call("PUT", f"/ingest/farms/{place}/insights/groundwater", body)
            pushed += 1
        except urllib.error.HTTPError as error:
            log(f"farm {place}: backend refused with {error.code}: {error.read()[:200]!r}")
            failed += 1
        except OSError as error:
            log(f"farm {place}: backend unreachable: {error}")
            failed += 1

    log(f"done: {pushed} {'read' if dry_run else 'pushed'}, {skipped} skipped, {failed} failed")
    if failed:
        sys.exit(1)


if __name__ == "__main__":
    main()
