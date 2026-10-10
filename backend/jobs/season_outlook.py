#!/usr/bin/env python3
"""Pushes the season outlook from the El Niño / La Niña state known in September.

The team's research (evidence/past_seasons/BACKTEST_RESULTS.md, "Pre-season
test", and backtest/enso_hide_test.py) tested 35 winters, 1991/92 to 2025/26,
of region rain against normal:
  - El Niño (ONI of July to September at or above +0.5): 7 winters, mean 122%
    of normal rain, never below 101%, no drought.
  - La Niña (ONI at or below -0.5): 7 winters, mean 74%, never above 99%.
  - Neutral (21 winters): no call. Most droughts fell in neutral years.
With the winter hidden, the direction was called right in 14 of 14 winters in
which a call was made.

This job turns the current ONI into the call for every district. It is a
REGION signal: every district gets the same call, and the reason text says so.
In a neutral year it pushes nothing.

Environment: FARM_DOCTOR_API, INGEST__SERVICE_KEY (as the other jobs).
Arguments: --season 2026-27 --issued 2026-10 --oni 2.16
"""
import json
import os
import sys
import urllib.request
from pathlib import Path

API = os.environ.get("FARM_DOCTOR_API", "http://127.0.0.1:3000/v1").rstrip("/")
KEY = os.environ.get("INGEST__SERVICE_KEY", "")
HERE = Path(__file__).resolve().parent
EL_NINO_WINTERS, LA_NINA_WINTERS = 7, 7
CALLS_TESTED, CALLS_RIGHT = 14, 14


def put(path, body):
    request = urllib.request.Request(
        f"{API}{path}",
        data=json.dumps(body).encode(),
        method="PUT",
        headers={"content-type": "application/json", "x-service-key": KEY},
    )
    with urllib.request.urlopen(request, timeout=30):
        pass


def arg(name):
    return sys.argv[sys.argv.index(name) + 1]


def main():
    if not KEY:
        sys.exit("INGEST__SERVICE_KEY is not set")
    season, issued, oni = arg("--season"), arg("--issued"), float(arg("--oni"))

    if oni >= 0.5:
        # 7 of 7 with no drought; (7 + 1) / (7 + 2) = 89%, the honest figure
        # for so few winters, rounded down.
        body = {
            "outlook": "good",
            "confidence_pct": 85,
            "reason_en": f"El Niño (ONI {oni:+.2f}). All 7 El Niño winters since 1991 had normal or above-normal "
            "rain here; none was a drought. A region-wide signal, the same for every district.",
            "reason_ku": None,
        }
    elif oni <= -0.5:
        body = {
            "outlook": "bad",
            "confidence_pct": 60,
            "reason_en": f"La Niña (ONI {oni:+.2f}). All 7 La Niña winters since 1991 had below-normal rain here; "
            "3 were droughts. A region-wide signal, the same for every district.",
            "reason_ku": None,
        }
    else:
        print(f"ONI {oni:+.2f} is neutral: no call is made in a neutral year, nothing pushed")
        return

    put(
        f"/ingest/outlooks/{season}/{issued}/run",
        {
            "seasons_tested": CALLS_TESTED,
            "seasons_right": CALLS_RIGHT,
            "method": "El Niño / La Niña (ONI Jul-Sep) against region winter rain, 1991-2025: 14 calls in 35 "
            "winters, direction right 14 of 14; no call in 21 neutral winters",
        },
    )
    districts = json.loads((HERE / "districts.json").read_text())
    pushed = 0
    for district in districts:
        put(f"/ingest/outlooks/{season}/{issued}/zones/{district['slug']}", body)
        pushed += 1
    print(f"done: {pushed} pushed, outlook {body['outlook']} for {season} from ONI {oni:+.2f}")


if __name__ == "__main__":
    main()
