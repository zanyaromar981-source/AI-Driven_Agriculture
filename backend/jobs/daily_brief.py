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
import urllib.parse
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

1. Numbers come from the DATA block or from a web page you opened and name
   (rule 3). Never invent, estimate or round a number that is in neither.
   DATA holds only what was measured; a subject that is absent from it was
   not measured by us. NEVER write that something is "not available",
   "not yet available", "missing" or "unknown", and never list what the data
   lacks: the reader wants what is known. For a subject DATA does not cover
   (dam levels, prices, an outlook), either report what your web search
   found, with its date and source, or leave the subject out.
2. "rain_pct_of_normal" is the rain of the last 365 days against the ten
   years before (100 is normal). "dryness" is that same figure on a 0 to 100
   scale (50 is normal rain, lower is wetter); it is not a soil moisture or
   crop measurement. Prefer the rain percentage when you write, and describe
   it as rain against normal, not as crop condition. Each data point carries
   its "source": do not claim more precision than the source has.
3. Before you write, search the web: this is part of the job, not optional.
   Run at least six searches, in English and also in Kurdish or Arabic,
   covering: weather warnings and the forecast for the Kurdistan Region this
   week; Dukan and Darbandikhan dam levels; crop pest and disease reports in
   Iraq; fires in Kurdistan farmland or forest; wheat, barley and vegetable
   prices and government purchase or seed announcements in Iraq; any notice
   from the Kurdistan or Iraqi agriculture or water ministries. Open the
   pages you rely on.
   Put what you learn into the brief, each item with the date of the report
   ("a report of 27 July says ..."), and list its page under "sources" with
   the real address you opened. Prefer the last 7 days; an older report may
   be used if you say how old it is. Numbers from the web are allowed when
   you name the report they come from; they never replace a number in DATA.
   Only if every search finds nothing relevant may you write that no web
   update was found. Do not cite a page you did not open.
4. Never give pesticide or fertiliser doses, mixing rates or product names.
   For anything that needs a diagnosis, tell the reader to see the
   agriculture or veterinary office.
5. Plain words, short sentences, no jargon. Sorani text must be Central
   Kurdish in Arabic script, written for farmers, not a word-for-word
   translation of the English.
6. Each brief: a headline of at most 100 characters; a summary of at most
   900 characters; up to 5 points, each at most 250 characters with a level
   of "info", "watch" or "alarm". Use "alarm" only for something a farmer
   should act on today. Every point tells the reader something they can use
   this week: what the weather of the next days means for sowing, watering,
   spraying, harvest or animals; where the risk is; a price or a notice. A
   point that only explains the data, or says what cannot be told, is not
   written.
7. Write one brief with scope "region", then one for each district listed in
   DISTRICTS_TO_COVER, using that district's slug as the scope. A district
   brief says what is different or specific there; do not repeat the region
   brief.
8. Fire entries are satellite hot spots that nobody has checked on the ground;
   some may be gas flares or controlled burning. Call them "satellite fire
   detections", never confirmed fires, and give them the level "watch".
9. "weather_forecast" is the forecast for the next 7 days at each district's
   centre (Open-Meteo). Lead with it: say on which days rain, heat, cold
   nights or strong wind are expected and where, with the numbers, and what
   that means for field work. A forecast is not a promise: say "is forecast".
   Never look further ahead than the days in DATA.
10. Answer with the JSON object only, in the required shape."""

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


def district_points(overview):
    """Every stored number for every district, from the district route."""
    points = []
    for zone in overview.get("zones", []):
        detail = read(f"/zones/{zone['slug']}") or {}
        reading = detail.get("reading") or {}
        points.append(
            {
                "slug": zone["slug"],
                "name_en": zone.get("name_en"),
                "name_ku": zone.get("name_ku"),
                "governorate": zone.get("governorate"),
                "month": detail.get("month"),
                "rain_pct_of_normal": reading.get("rain_pct_of_normal"),
                "dryness": reading.get("dryness"),
                "band": reading.get("band"),
                "rank_driest_first": reading.get("rank"),
                "ranked_districts": reading.get("rank_of"),
                "change_vs_last_year": zone.get("change_vs_last_year"),
                "greenness_pct_vs_normal": reading.get("greenness_pct_vs_normal"),
                "water_need": reading.get("water_need"),
                "nitrogen_hold": reading.get("nitrogen_hold"),
                "best_crops": reading.get("best_crops"),
                "source": reading.get("source"),
                "updated_at": reading.get("updated_at"),
                "same_month_in_past_years": detail.get("history") or [],
                "sub_districts_with_data": [
                    {"slug": sub["slug"], "dryness": sub["dryness"]}
                    for sub in detail.get("sub_zones", [])
                    if sub.get("dryness") is not None
                ],
            }
        )
    return points


def fire_points():
    """The last 24 hours in full, and the last 72 hours counted per district."""
    day = read("/fires?hours=24") or {}
    three_days = read("/fires?hours=72") or {}
    per_district = {}
    for fire in three_days.get("fires", []):
        slug = fire.get("zone_slug") or "unknown"
        per_district[slug] = per_district.get(slug, 0) + 1
    return {
        "what_these_are": "satellite hot spots inside the districts, gas flare spots removed by a rule; not checked on the ground",
        "last_24_hours_summary": day.get("summary"),
        "last_24_hours": [
            {
                key: fire.get(key)
                for key in (
                    "zone_slug",
                    "place_en",
                    "lat",
                    "lon",
                    "detected_at",
                    "status",
                    "area_ha",
                    "wind_kmh",
                    "wind_direction",
                    "farms_within_5km",
                    "source",
                )
            }
            for fire in day.get("fires", [])[:80]
        ],
        "detections_per_district_last_72_hours": dict(
            sorted(per_district.items(), key=lambda item: -item[1])
        ),
    }


def dam_points():
    """Each dam's latest reading, the one a year before, and the last 90 days."""
    dams = (read("/dams") or {}).get("dams", [])
    since = (dt.date.today() - dt.timedelta(days=90)).isoformat()
    for dam in dams:
        history = read(f"/dams/{dam['slug']}/history?from={since}") or {}
        dam["last_90_days"] = history.get("readings", [])
    return dams


def market_points():
    """Prices staff have entered, what is on sale, and today's deals."""
    markets = {}
    for market in (read("/alwa/markets") or {}).get("markets", []):
        slug = market["slug"]
        prices = read(f"/alwa/markets/{slug}/prices") or {}
        listings = read(f"/alwa/listings?market={slug}&status=open&rows_per_page=100") or {}
        deals = read(f"/alwa/deals?market={slug}") or {}
        on_sale = {}
        for listing in listings.get("listings", []):
            crop = on_sale.setdefault(listing["crop"], {"listings": 0, "kg": 0, "asking_prices": []})
            crop["listings"] += 1
            crop["kg"] += listing.get("quantity_kg") or 0
            crop["asking_prices"].append(listing.get("asking_price_iqd_per_kg"))
        markets[slug] = {
            "name_en": market.get("name_en"),
            "prices_day": prices.get("day"),
            "prices_iqd_per_kg": prices.get("prices") or [],
            "on_sale_now_by_crop": on_sale,
            "deals_today": deals.get("summary"),
        }
    return markets


FORECAST = "https://api.open-meteo.com/v1/forecast"
FORECAST_DAILY = (
    "temperature_2m_max,temperature_2m_min,precipitation_sum,"
    "wind_gusts_10m_max,et0_fao_evapotranspiration"
)


def forecast_points(districts):
    """The next 7 days at each district's centre, from Open-Meteo.

    One call for all districts. A failure leaves the forecast out; the brief
    is still written from the rest.
    """
    query = urllib.parse.urlencode(
        {
            "latitude": ",".join(str(district["lat"]) for district in districts),
            "longitude": ",".join(str(district["lon"]) for district in districts),
            "daily": FORECAST_DAILY,
            "forecast_days": 7,
            "timezone": "Asia/Baghdad",
        }
    )
    try:
        with urllib.request.urlopen(f"{FORECAST}?{query}", timeout=60) as response:
            places = json.loads(response.read())
    except (urllib.error.URLError, OSError, ValueError) as error:
        log(f"no weather forecast: {error}")
        return None

    if isinstance(places, dict):
        places = [places]
    if len(places) != len(districts):
        log("no weather forecast: the answer does not match the districts asked for")
        return None

    by_district = {}
    for district, place in zip(districts, places):
        daily = place.get("daily") or {}
        by_district[district["slug"]] = {
            "rain_mm": daily.get("precipitation_sum"),
            "tmax_c": daily.get("temperature_2m_max"),
            "tmin_c": daily.get("temperature_2m_min"),
            "gust_kmh": daily.get("wind_gusts_10m_max"),
            "et0_mm": daily.get("et0_fao_evapotranspiration"),
        }
    days = (places[0].get("daily") or {}).get("time")
    if not days:
        return None

    return {
        "source": "Open-Meteo forecast at each district's centre; one value per day in 'days'; "
        "et0_mm is the water a well-watered crop would use that day",
        "days": days,
        "districts": by_district,
    }


def known_only(value):
    """Drops nulls and empty lists and objects, so the agent sees what is known."""
    if isinstance(value, dict):
        kept = {key: known_only(item) for key, item in value.items()}
        return {key: item for key, item in kept.items() if item not in (None, {}, [])}
    if isinstance(value, list):
        return [known_only(item) for item in value]
    return value


def measured_only(data):
    """Takes out totals that only add up fields nobody has measured.

    The fire summary sums area, nearby farms and alerted farmers over
    detections where those are null, which gives zeros that read like "no
    damage"; a market with no price, no listing and no deal says nothing; a
    dam with no reading is only its name. They would be read as findings.
    """
    data = dict(data)

    fires = dict(data.get("fires") or {})
    detections = fires.get("last_24_hours") or []
    summary = dict(fires.get("last_24_hours_summary") or {})
    for total in ("area_ha", "farms_within_5km", "farmers_alerted"):
        if not any(fire.get(total) is not None for fire in detections):
            summary.pop(total, None)
    fires["last_24_hours_summary"] = summary
    data["fires"] = fires

    data["markets"] = {
        slug: market
        for slug, market in (data.get("markets") or {}).items()
        if market.get("prices_iqd_per_kg")
        or market.get("on_sale_now_by_crop")
        or (market.get("deals_today") or {}).get("deals")
    }
    data["dams"] = [
        dam
        for dam in data.get("dams") or []
        if dam.get("latest") or dam.get("last_90_days")
    ]

    return known_only(data)


def gather(districts):
    """Everything the agent is allowed to know, and which farm is where.

    Every data point the backend holds goes in: each district's full reading,
    the dams with their recent history, the fires, the season outlook, the
    water plan, the market, the 7-day forecast, and yesterday's brief so
    today's does not simply repeat it. What is empty is left out of the
    prompt (see `known_only`): the brief says what is known.
    """
    overview = read("/region/overview") or {}
    farms = (read("/ingest/farms", key=True) or {}).get("farms", [])

    farm_zones = {
        farm["id"]: nearest_district(farm["lat"], farm["lon"], districts) for farm in farms
    }
    farms_per_zone = {}
    for slug in farm_zones.values():
        farms_per_zone[slug] = farms_per_zone.get(slug, 0) + 1

    topics_seen = {}
    for farm in farms:
        for topic in farm.get("topics", []):
            topics_seen[topic["topic"]] = topics_seen.get(topic["topic"], 0) + 1

    previous = read("/briefs/latest") or {}
    previous_brief = previous.get("brief") or {}

    data = {
        "today": dt.datetime.now(BAGHDAD).date().isoformat(),
        "region_summary": {"month": overview.get("month"), **(overview.get("summary") or {})},
        "districts": district_points(overview),
        "weather_forecast": forecast_points(districts),
        "dams": dam_points(),
        "fires": fire_points(),
        "season_outlook": read("/outlooks"),
        "water_plan": read("/water/plan"),
        "markets": market_points(),
        "farms": {
            "total": len(farms),
            "per_district": farms_per_zone,
            "farms_with_each_kind_of_reading": topics_seen,
        },
        "previous_brief": {
            "day": previous_brief.get("day"),
            "headline_en": previous_brief.get("headline_en"),
        },
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
            "DATA:\n" + json.dumps(measured_only(data), ensure_ascii=False),
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
    cover = districts_to_cover(data["farms"]["per_district"])
    prompt = build_prompt(data, cover)
    log(f"{len(farm_zones)} farms in {len(data['farms']['per_district'])} districts; covering {cover or 'the region only'}")
    log(f"data handed to the agent: {len(json.dumps(data))} characters, {len(data['districts'])} districts, "
        f"{len(data['fires']['last_24_hours'])} fires, {len(data['dams'])} dams, {len(data['markets'])} markets")

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
