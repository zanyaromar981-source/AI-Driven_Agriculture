#!/usr/bin/env python3
"""Daily brief: an AI agent reads the day's stored values, searches the web a
little, and writes a short brief for the region and for each district that
has farms. Runs on a timer at midnight Baghdad time.

For now the agent is the Codex command-line tool (`codex exec`), run without
a terminal. The same prompt and output shape will later go to an API instead;
only `run_agent` needs to change for that.

What it does:
1. Reads today's values from the backend's public routes (district rain and
   dryness, dams, fires, market prices, season outlook) and the list of farms
   with their centre points from the ingest route.
2. Puts each farm in the district whose centre is nearest (a rough rule: no
   boundaries are used) and tells the backend, so the app can show a farm the
   brief of its district.
3. Asks the agent for one brief for the region and one per district that has
   farms, in Sorani and English, as JSON in a fixed shape.
4. Checks the answer against the backend's limits and pushes each brief.

The agent is told to use only the numbers it is given, to name a source for
anything it found online, to say so when it found nothing, and never to give
pesticide or fertiliser doses or product names.

Needs only the Python standard library, plus the `codex` program, signed in.

Environment:
  FARM_DOCTOR_API       base address, default http://localhost:8790/v1
  INGEST__SERVICE_KEY   the backend's ingest key (required)
  CODEX_BIN             the codex program, default "codex"
  BRIEF_MODEL           model name passed to codex, default: codex's own default
  BRIEF_WEB_SEARCH      "0" to run without web search, default "1"
  BRIEF_MAX_ZONES       most district briefs in one run, default 12
  BRIEF_TIMEOUT_S       longest wait for the agent, default 1500

Run with --dry-run to print the prompt and stop: nothing is asked or pushed.
"""

import datetime as dt
import json
import math
import os
import shutil
import subprocess
import sys
import tempfile
import urllib.error
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
API = os.environ.get("FARM_DOCTOR_API", "http://localhost:8790/v1").rstrip("/")
KEY = os.environ.get("INGEST__SERVICE_KEY", "")
CODEX = os.environ.get("CODEX_BIN", "codex")
MODEL = os.environ.get("BRIEF_MODEL", "")
WEB_SEARCH = os.environ.get("BRIEF_WEB_SEARCH", "1") != "0"
MAX_ZONES = int(os.environ.get("BRIEF_MAX_ZONES", "12"))
TIMEOUT_S = int(os.environ.get("BRIEF_TIMEOUT_S", "1500"))
BAGHDAD = dt.timezone(dt.timedelta(hours=3))

# The backend's limits (briefs slice). Text over a limit is cut, not refused,
# so one long sentence does not lose a whole night's brief.
MAX_HEADLINE, MAX_SUMMARY, MAX_POINT, MAX_POINTS = 120, 2000, 300, 8
MAX_SOURCES, MAX_TITLE, MAX_URL = 12, 200, 500
LEVELS = ("info", "watch", "alarm")

RULES = """You write a short daily brief for farmers and agriculture officers in the
Kurdistan Region of Iraq. Follow every rule:

1. Use only the numbers in the DATA block. Never invent, estimate or round a
   number that is not there. If a value is null or a list is empty, say that
   it is not available yet; do not guess.
2. "dryness" in the data is only yearly rain put on a 0 to 100 scale (50 is
   normal rain, lower is wetter). It is not a soil moisture or crop
   measurement. Describe it as rain against normal, not as crop condition.
3. You may search the web for today's weather warnings, pest and disease
   reports, water and dam news, market news and official announcements for
   the Kurdistan Region and Iraq. Every statement that comes from the web
   must have its page in "sources", with the real address you opened. If you
   found nothing useful, say so in the summary and leave "sources" empty. Do
   not cite a page you did not open.
4. Never give pesticide or fertiliser doses, mixing rates or product names.
   For anything that needs a diagnosis, tell the reader to see the
   agriculture or veterinary office.
5. Plain words, short sentences, no jargon. Sorani text must be Central
   Kurdish in Arabic script, written for farmers, not a word-for-word
   translation of the English.
6. Each brief: a headline of at most 100 characters; a summary of at most
   900 characters; up to 5 points, each at most 250 characters with a level
   of "info", "watch" or "alarm". Use "alarm" only for something a farmer
   should act on today.
7. Write one brief with scope "region", then one for each district listed in
   DISTRICTS_TO_COVER, using that district's slug as the scope. A district
   brief says what is different or specific there; do not repeat the region
   brief.
8. Answer with the JSON object only, in the required shape."""

SCHEMA = {
    "type": "object",
    "additionalProperties": False,
    "required": ["briefs"],
    "properties": {
        "briefs": {
            "type": "array",
            "items": {
                "type": "object",
                "additionalProperties": False,
                "required": [
                    "scope",
                    "headline_en",
                    "headline_ku",
                    "summary_en",
                    "summary_ku",
                    "points",
                    "sources",
                ],
                "properties": {
                    "scope": {"type": "string"},
                    "headline_en": {"type": "string"},
                    "headline_ku": {"type": "string"},
                    "summary_en": {"type": "string"},
                    "summary_ku": {"type": "string"},
                    "points": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "additionalProperties": False,
                            "required": ["level", "text_en", "text_ku"],
                            "properties": {
                                "level": {"type": "string", "enum": list(LEVELS)},
                                "text_en": {"type": "string"},
                                "text_ku": {"type": "string"},
                            },
                        },
                    },
                    "sources": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "additionalProperties": False,
                            "required": ["title", "url"],
                            "properties": {
                                "title": {"type": "string"},
                                "url": {"type": "string"},
                            },
                        },
                    },
                },
            },
        }
    },
}


def log(message):
    print(f"{dt.datetime.now(dt.timezone.utc):%Y-%m-%d %H:%M:%S}Z {message}", flush=True)


def call(method, path, body=None, key=False):
    headers = {"accept": "application/json"}
    data = None
    if body is not None:
        data = json.dumps(body).encode()
        headers["content-type"] = "application/json"
    if key:
        headers["x-service-key"] = KEY
    request = urllib.request.Request(f"{API}{path}", data=data, method=method, headers=headers)
    with urllib.request.urlopen(request, timeout=60) as response:
        raw = response.read()
        return json.loads(raw) if raw else {}


def read(path, key=False):
    """A read that may have nothing to give: a 404 means "no data yet"."""
    try:
        return call("GET", path, key=key)
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return None
        raise


def nearest_district(lat, lon, districts):
    def distance(district):
        d_lat = math.radians(district["lat"] - lat)
        d_lon = math.radians(district["lon"] - lon) * math.cos(math.radians(lat))
        return d_lat * d_lat + d_lon * d_lon

    return min(districts, key=distance)["slug"]


def gather(districts):
    """Everything the agent is allowed to know, and which farm is where."""
    overview = read("/region/overview") or {}
    farms = (read("/ingest/farms", key=True) or {}).get("farms", [])

    farm_zones = {
        farm["id"]: nearest_district(farm["lat"], farm["lon"], districts) for farm in farms
    }
    farms_per_zone = {}
    for slug in farm_zones.values():
        farms_per_zone[slug] = farms_per_zone.get(slug, 0) + 1

    prices = {}
    for market in (read("/alwa/markets") or {}).get("markets", []):
        answer = read(f"/alwa/markets/{market['slug']}/prices")
        if answer and answer.get("prices"):
            prices[market["slug"]] = {"day": answer.get("day"), "prices": answer["prices"]}

    data = {
        "today": dt.datetime.now(BAGHDAD).date().isoformat(),
        "districts": {
            "month": overview.get("month"),
            "summary": overview.get("summary"),
            "by_district": [
                {
                    key: zone.get(key)
                    for key in ("slug", "name_en", "name_ku", "governorate", "dryness", "rank")
                }
                for zone in overview.get("zones", [])
            ],
        },
        "dams": (read("/dams") or {}).get("dams", []),
        "fires_last_24_hours": read("/fires?hours=24") or {},
        "season_outlook": read("/outlooks"),
        "market_prices_iqd_per_kg": prices,
        "farms_per_district": farms_per_zone,
    }
    return data, farm_zones


def districts_to_cover(farms_per_zone):
    """Districts with the most farms first, up to the limit."""
    ranked = sorted(farms_per_zone.items(), key=lambda item: (-item[1], item[0]))
    return [slug for slug, _ in ranked[:MAX_ZONES]]


def build_prompt(data, cover):
    return "\n\n".join(
        [
            RULES,
            "DISTRICTS_TO_COVER: " + (", ".join(cover) if cover else "(none: write the region brief only)"),
            "DATA:\n" + json.dumps(data, ensure_ascii=False, indent=1),
        ]
    )


def run_agent(prompt):
    """Runs the Codex command-line tool once and returns its JSON answer."""
    if shutil.which(CODEX) is None:
        sys.exit(f"'{CODEX}' is not installed or not on the PATH")

    with tempfile.TemporaryDirectory(prefix="daily-brief-") as folder:
        schema_path = Path(folder) / "schema.json"
        answer_path = Path(folder) / "answer.json"
        schema_path.write_text(json.dumps(SCHEMA))

        command = [CODEX]
        if WEB_SEARCH:
            command.append("--search")
        command += [
            "exec",
            "--skip-git-repo-check",
            "--ephemeral",
            "--sandbox",
            "read-only",
            "--cd",
            folder,
            "--output-schema",
            str(schema_path),
            "--output-last-message",
            str(answer_path),
        ]
        if MODEL:
            command += ["--model", MODEL]
        command.append("-")  # the prompt comes on standard input

        log(f"asking the agent ({' '.join(command[:3])} ...), this can take a few minutes")
        result = subprocess.run(
            command,
            input=prompt,
            text=True,
            capture_output=True,
            timeout=TIMEOUT_S,
            cwd=folder,
        )
        if result.returncode != 0:
            tail = (result.stderr or result.stdout or "").strip()[-600:]
            sys.exit(f"the agent failed (exit {result.returncode}): {tail}")
        if not answer_path.exists():
            sys.exit("the agent finished without writing an answer")

        return json.loads(answer_path.read_text())


def cut(text, limit):
    text = " ".join(str(text).split())
    return text if len(text) <= limit else text[: limit - 1].rstrip() + "…"


def clean(brief):
    """Fits one brief to the backend's limits. Returns None if it is unusable."""
    for field in ("headline_en", "headline_ku", "summary_en", "summary_ku"):
        if not str(brief.get(field, "")).strip():
            return None

    points = []
    for point in brief.get("points", [])[:MAX_POINTS]:
        if point.get("level") in LEVELS and point.get("text_en") and point.get("text_ku"):
            points.append(
                {
                    "level": point["level"],
                    "text_en": cut(point["text_en"], MAX_POINT),
                    "text_ku": cut(point["text_ku"], MAX_POINT),
                }
            )

    sources = []
    for source in brief.get("sources", [])[:MAX_SOURCES]:
        url = str(source.get("url", "")).strip()
        title = str(source.get("title", "")).strip()
        if title and url.startswith(("https://", "http://")) and len(url) <= MAX_URL:
            sources.append({"title": cut(title, MAX_TITLE), "url": url})

    return {
        "headline_en": cut(brief["headline_en"], MAX_HEADLINE),
        "headline_ku": cut(brief["headline_ku"], MAX_HEADLINE),
        "summary_en": cut(brief["summary_en"], MAX_SUMMARY),
        "summary_ku": cut(brief["summary_ku"], MAX_SUMMARY),
        "points": points,
        "sources": sources,
    }


def main():
    dry_run = "--dry-run" in sys.argv
    if not KEY:
        sys.exit("INGEST__SERVICE_KEY is not set")

    districts = json.loads((HERE / "districts.json").read_text())
    known = {district["slug"] for district in districts}

    data, farm_zones = gather(districts)
    cover = districts_to_cover(data["farms_per_district"])
    prompt = build_prompt(data, cover)
    log(f"{len(farm_zones)} farms in {len(data['farms_per_district'])} districts; covering {cover or 'the region only'}")

    if dry_run:
        print(prompt)
        return

    if farm_zones:
        mapping = [{"farm_id": farm_id, "zone_slug": slug} for farm_id, slug in farm_zones.items()]
        for start in range(0, len(mapping), 2000):
            call("PUT", "/ingest/briefs/farm-zones", {"farms": mapping[start : start + 2000]}, key=True)
        log(f"recorded the district of {len(mapping)} farms")

    answer = run_agent(prompt)

    day = data["today"]
    author = f"codex-cli {MODEL}".strip()
    generated_at = dt.datetime.now(dt.timezone.utc).isoformat()
    pushed, skipped, failed = 0, 0, 0

    for brief in answer.get("briefs", []):
        scope = str(brief.get("scope", "")).strip()
        body = clean(brief)
        if body is None or not (scope == "region" or scope in known):
            log(f"skipped a brief with scope {scope!r}: unknown district or missing text")
            skipped += 1
            continue
        body.update({"author": author, "generated_at": generated_at})
        try:
            call("PUT", f"/ingest/briefs/{day}/{scope}", body, key=True)
            log(f"{scope}: {body['headline_en']}")
            pushed += 1
        except urllib.error.HTTPError as error:
            log(f"{scope}: backend refused with {error.code}: {error.read()[:200]!r}")
            failed += 1

    log(f"done: {pushed} pushed, {skipped} skipped, {failed} failed")
    if failed or pushed == 0:
        sys.exit(1)


if __name__ == "__main__":
    main()
