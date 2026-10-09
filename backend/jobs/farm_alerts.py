#!/usr/bin/env python3
"""Farm alerts: turns what the backend already stores into alerts for each
farm.

Runs on a timer (every 30 minutes). It makes nothing up and measures
nothing: it reads the fires and the weekly plan the other jobs stored and
writes one alert per farm and event through the ingest routes. The app lists
them; a push sender (not built yet) will send the red ones.

Two kinds of alert:
- Fire near a farm. For each fire of the last 24 hours within 5 km of a
  farm's centre: level `alarm` within 2 km, `watch` beyond. Confidence is
  always `unsure`: a satellite detection is unchecked and can be a gas flare
  or a controlled burn, so the text says "satellite fire detection" and
  never that there is a fire. Fires the backend marks as out are skipped.
  The distance is from the centre of the farm, not its edge, so a large farm
  can be closer to the fire than the text says.
- Plan alerts. The `alerts` of a farm's weekly plan, one alert per type and
  day. The route that serves the plan to jobs is not built yet; until it is,
  every farm answers 404 and this part does nothing (see `fetch_plan`).

Every alert has a key (`fire:<fire id>`, `<type>:<day>`). The backend keeps
one alert per farm and key, so running again replaces the wording and never
adds a second alert or unticks one the farmer ticked.

Sorani: this job writes no Sorani. Plan alerts carry the plan's own `ku`.
Fire alerts carry the English text in `ku` until a native speaker has
translated the sentences in `fire_alert`.

Needs only the Python standard library.

Environment:
  FARM_DOCTOR_API       base address, default http://localhost:8790/v1
  INGEST__SERVICE_KEY   the backend's ingest key (required)
  FARM_ALERTS_LOCK      lock file, default ./cache/farm_alerts.lock

Run with --dry-run to print what would be pushed without pushing.
"""

import datetime as dt
import fcntl
import json
import math
import os
import sys
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
API = os.environ.get("FARM_DOCTOR_API", "http://localhost:8790/v1").rstrip("/")
KEY = os.environ.get("INGEST__SERVICE_KEY", "")
LOCK = Path(os.environ.get("FARM_ALERTS_LOCK", HERE / "cache" / "farm_alerts.lock"))

FIRE_HOURS = 24
FIRE_WATCH_KM = 5.0
FIRE_ALARM_KM = 2.0
IRAQ = dt.timezone(dt.timedelta(hours=3))  # no daylight saving

FIRE_SOURCE = "farm_alerts.py: stored NASA FIRMS detection, distance from the farm's centre"
PLAN_SOURCE = "farm_alerts.py: the alerts of the farm's stored weekly plan"

DIRECTIONS = ["north", "north-east", "east", "south-east", "south", "south-west", "west", "north-west"]

PLAN_TYPES = {
    "frost", "heat", "heavy_rain", "dry_spell", "rust_weather", "sunn_pest",
    "dust", "spray_window", "sowing_rain", "urea_rain",
}
LEVELS = {"watch", "alarm"}
# The plan's decision that says what to do about an alert of each type. An
# alert whose plan has no such decision repeats its own text as the action.
DECISIONS = {
    "frost": ["frost_check"],
    "heat": ["heat_check"],
    "dust": ["dust_delay"],
    "rust_weather": ["check_rust"],
    "sunn_pest": ["count_sunn_pest"],
    "spray_window": ["spray_ok"],
    "sowing_rain": ["sow_go", "sow_wait"],
    "urea_rain": ["urea_go", "urea_hold"],
}


def log(message):
    print(f"{dt.datetime.now(dt.timezone.utc):%Y-%m-%d %H:%M:%S}Z {message}", flush=True)


def call(method, path, body=None):
    data = json.dumps(body).encode() if body is not None else None
    headers = {"x-service-key": KEY, "content-type": "application/json"}
    request = urllib.request.Request(f"{API}{path}", data=data, method=method, headers=headers)
    with urllib.request.urlopen(request, timeout=60) as response:
        raw = response.read()
        return json.loads(raw) if raw else {}


def distance_km(lat1, lon1, lat2, lon2):
    """Great-circle distance."""
    p1, p2 = math.radians(lat1), math.radians(lat2)
    a = (
        math.sin((p2 - p1) / 2) ** 2
        + math.cos(p1) * math.cos(p2) * math.sin(math.radians(lon2 - lon1) / 2) ** 2
    )
    return 6371.0 * 2 * math.asin(math.sqrt(a))


def direction(from_lat, from_lon, to_lat, to_lon):
    """The compass direction, one of eight, in which the second point lies
    as seen from the first."""
    p1, p2 = math.radians(from_lat), math.radians(to_lat)
    d_lon = math.radians(to_lon - from_lon)
    y = math.sin(d_lon) * math.cos(p2)
    x = math.cos(p1) * math.sin(p2) - math.sin(p1) * math.cos(p2) * math.cos(d_lon)
    bearing = (math.degrees(math.atan2(y, x)) + 360) % 360
    return DIRECTIONS[int((bearing + 22.5) // 45) % 8]


def fire_alert(farm, fire):
    """The key and body of the alert one fire gives one farm, or None when
    the fire is further than 5 km from the farm's centre."""
    km = distance_km(farm["lat"], farm["lon"], fire["lat"], fire["lon"])
    if km > FIRE_WATCH_KM:
        return None
    seen = dt.datetime.fromisoformat(fire["detected_at"].replace("Z", "+00:00")).astimezone(IRAQ)
    where = direction(farm["lat"], farm["lon"], fire["lat"], fire["lon"])
    # SORANI NEEDED: these two sentences are English only. Until a native
    # speaker has translated them, `ku` and `action_ku` carry the English.
    text = (
        f"Satellite fire detection about {km:.1f} km to the {where} of your farm, "
        f"seen at {seen:%H:%M}. It has not been checked: it may be a gas flare or a controlled burn."
    )
    action = f"Look towards the {where}. If you see smoke or flames, call civil defence."
    return f"fire:{fire['id']}", {
        "type": "fire",
        "day": seen.date().isoformat(),
        "level": "alarm" if km <= FIRE_ALARM_KM else "watch",
        "confidence": "unsure",
        "ku": text,
        "en": text,
        "action_ku": action,
        "action_en": action,
        "source": FIRE_SOURCE,
    }


def alerts_from_fires(farm, fires):
    found = []
    for fire in fires:
        if fire.get("status") == "out":
            continue
        alert = fire_alert(farm, fire)
        if alert:
            found.append(alert)
    return found


def alerts_from_plan(plan):
    """The (key, body) of every alert in a farm's weekly plan (the JSON of
    BACKEND.md 2.4), keyed `<type>:<day>`.

    The words are the plan's own, in both languages. The action is the plan's
    decision for that type of alert when it has one, otherwise the alert's
    own text. An alert with an unknown type or level, or with a text
    missing, is left out rather than guessed at. A plan says nothing about
    how sure it is; a forecast is `likely`, never `sure`.
    """
    decisions = {d.get("code"): d for d in plan.get("decisions") or []}
    found = {}
    for alert in plan.get("alerts") or []:
        kind, day, level = alert.get("type"), alert.get("day"), alert.get("level")
        ku, en = (alert.get("ku") or "").strip(), (alert.get("en") or "").strip()
        if kind not in PLAN_TYPES or level not in LEVELS or not ku or not en:
            continue
        try:
            dt.date.fromisoformat(day)
        except (TypeError, ValueError):
            continue
        action = next((decisions[c] for c in DECISIONS.get(kind, []) if c in decisions), {})
        body = {
            "type": kind,
            "day": day,
            "level": level,
            "confidence": "likely",
            "ku": ku,
            "en": en,
            "action_ku": (action.get("ku") or "").strip() or ku,
            "action_en": (action.get("en") or "").strip() or en,
            "source": PLAN_SOURCE,
        }
        key = f"{kind}:{day}"
        # Two alerts of one type on one day share a key: the alarm wins.
        if key not in found or (level == "alarm" and found[key]["level"] != "alarm"):
            found[key] = body
    return list(found.items())


def fetch_plan(farm_id):
    """NOT LIVE YET: reads a farm's stored weekly plan for jobs.

    `GET /v1/ingest/farms/{id}/plan` is being built by someone else. Until it
    exists the backend answers 404, which is read as "no plan", so plan
    alerts start by themselves the day the route does. Returns the plan or
    None.
    """
    try:
        return call("GET", f"/ingest/farms/{farm_id}/plan")
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return None
        raise


def main():
    dry_run = "--dry-run" in sys.argv
    if not KEY and not dry_run:
        sys.exit("INGEST__SERVICE_KEY is not set")

    # One run at a time: a run that overlaps the next would only repeat it.
    LOCK.parent.mkdir(parents=True, exist_ok=True)
    lock = open(LOCK, "w")
    try:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError:
        log("another run holds the lock")
        log("done: 0 pushed, 0 skipped, 0 failed")
        return

    farms = call("GET", "/ingest/farms").get("farms", [])
    fires = call("GET", f"/fires?hours={FIRE_HOURS}").get("fires", [])
    log(f"{len(farms)} farms, {len(fires)} fires in the last {FIRE_HOURS} hours")

    pushed = skipped = failed = plans = 0
    for farm in farms:
        alerts = alerts_from_fires(farm, fires)
        try:
            plan = fetch_plan(farm["id"])
        except (urllib.error.URLError, OSError, ValueError) as error:
            log(f"farm {farm['id']}: plan could not be read ({error})")
            failed += 1
            plan = None
        if plan:
            plans += 1
            alerts += alerts_from_plan(plan)
        if not alerts:
            skipped += 1
            continue
        for key, body in alerts:
            if dry_run:
                log(f"farm {farm['id']} {key}: {body['level']} {body['en']}")
                pushed += 1
                continue
            try:
                call("PUT", f"/ingest/farms/{farm['id']}/alerts/{urllib.parse.quote(key, safe=':')}", body)
                pushed += 1
            except (urllib.error.URLError, OSError) as error:
                log(f"farm {farm['id']} {key}: not stored ({error})")
                failed += 1

    log(f"{plans} farms had a plan to read")
    log(f"done: {pushed} {'would be ' if dry_run else ''}pushed, {skipped} skipped, {failed} failed")
    if failed:
        sys.exit(1)


if __name__ == "__main__":
    main()
