#!/usr/bin/env python3
"""What is grown around each farm, from MapSPAM 2020 (BACKEND.md 2.17).

Reads the MapSPAM 2020 physical-area CSV (IFPRI, Harvard Dataverse,
DOI 10.7910/DVN/SWPENT). The file is a download behind a guestbook form, so
it is never fetched here: put it on the server and point MAPSPAM_CSV at it.
Either the Iraq-only CSV or the zipped global file
`spam2020V2r2_global_physical_area.csv.zip` works; from the zip only the
rows with iso3 IRQ are kept, and they are saved next to the zip as
`spam2020_physical_area_IRQ.csv` so the next run is quick.

For each farm the job adds up each crop's area over the 5 arc-minute
squares whose centre is within 15 km of the farm centre (and always the
square nearest the centre), keeps the crops with at least 1% of that area,
biggest first, and pushes the topic `crops_grown`. When the file is missing
it stops with a message: it never invents numbers.

  python3 crops_grown.py                          all farms
  python3 crops_grown.py --farm 12                one farm
  python3 crops_grown.py --dry-run --lat 36.74 --lon 43.89   print, push nothing
"""
import csv
import io
import json
import math
import os
import re
import sys
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from farm_analysis import KEY, MEASURES_LIMIT, backend, log, measure, topic_body  # noqa: E402

DEFAULT_CSV = HERE / "data" / "spam2020_physical_area_IRQ.csv"
GLOBAL_ZIP = HERE / "data" / "spam2020V2r2_global_physical_area.csv.zip"
TOPIC = "crops_grown"
SOURCE = "MapSPAM 2020 (IFPRI), crop areas around 2019 to 2021"
AS_OF = "2020-12-31"  # the end of MapSPAM's reference year
RADIUS_KM = 15.0
MIN_SHARE = 0.01
MAX_CROPS = MEASURES_LIMIT // 2  # two measures per crop

# MapSPAM's four-letter codes to our crop codes (the crops table).
OURS = {
    "whea": "wheat",
    "barl": "barley",
    "rice": "rice",
    "pota": "potato",
    "toma": "tomato",
    "onio": "onion",
    "chic": "chickpea",
    "sunf": "sunflower",
    "oliv": "olive",
}
# MapSPAM's crops and groups we have no crop code for keep their own name,
# never a guessed single crop.
GROUPS = {
    "maiz": "maize",
    "mill": "millet",
    "pmil": "pearl_millet",
    "smil": "small_millet",
    "sorg": "sorghum",
    "ocer": "other_cereals",
    "swpo": "sweet_potato",
    "yams": "yams",
    "cass": "cassava",
    "orts": "other_roots",
    "bean": "bean",
    "cowp": "cowpea",
    "pige": "pigeon_pea",
    "lent": "lentil",
    "opul": "other_pulses",
    "soyb": "soybean",
    "grou": "groundnut",
    "cnut": "coconut",
    "oilp": "oil_palm",
    "rape": "rapeseed",
    "sesa": "sesame",
    "ooil": "other_oil_crops",
    "sugc": "sugarcane",
    "sugb": "sugar_beet",
    "cott": "cotton",
    "ofib": "other_fibre_crops",
    "acof": "arabica_coffee",
    "rcof": "robusta_coffee",
    "coco": "cocoa",
    "teas": "tea",
    "toba": "tobacco",
    "bana": "banana",
    "plnt": "plantain",
    "citr": "citrus",
    "trof": "tropical_fruit",
    "temf": "temperate_fruit",
    "vege": "other_vegetables",
    "rubb": "rubber",
    "rest": "other_crops",
}
CROP_COLUMN = re.compile(r"^([a-z]{4})_([air])$")
TECH = {"a": "all", "i": "irrigated", "r": "rainfed"}


def our_code(spam):
    return OURS.get(spam) or GROUPS.get(spam) or f"spam_{spam}"


def name_of(code):
    return code.replace("_", " ")


def km_between(lat1, lon1, lat2, lon2):
    p1, p2 = math.radians(lat1), math.radians(lat2)
    a = math.sin((p2 - p1) / 2) ** 2 + math.cos(p1) * math.cos(p2) * math.sin(math.radians(lon2 - lon1) / 2) ** 2
    return 2 * 6371.0 * math.asin(math.sqrt(a))


def number(text):
    try:
        value = float(text)
    except (TypeError, ValueError):
        return None
    return value if math.isfinite(value) and value > 0 else None


# ---------------------------------------------------------------- reading
def csv_streams(path):
    """Text streams of every CSV in the file (a zip may hold one per technology)."""
    if path.suffix.lower() == ".zip":
        archive = zipfile.ZipFile(path)
        for member in archive.namelist():
            if member.lower().endswith(".csv"):
                yield member, io.TextIOWrapper(archive.open(member), encoding="utf-8", errors="replace", newline="")
    else:
        yield path.name, open(path, encoding="utf-8", errors="replace", newline="")


def is_iraq(row, at):
    if "iso3" in at:
        return row[at["iso3"]].strip().upper() == "IRQ"
    if "fips0" in at:
        return row[at["fips0"]].strip().upper() == "IZ"
    for key in ("adm0_name", "name_cntr"):
        if key in at:
            return row[at[key]].strip().lower() == "iraq"
    return True  # no country column: the file is already one country


def read_cells(path, keep_rows=None):
    """{(lon, lat): {(spam_code, tech): hectares}} for Iraq's squares."""
    cells = {}
    for member, stream in csv_streams(path):
        with stream:
            reader = csv.reader(stream)
            header = next(reader, None)
            if not header:
                continue
            names = [name.strip().lower() for name in header]
            at = {name: index for index, name in enumerate(names)}
            if "x" not in at or "y" not in at:
                log(f"{member}: no x and y columns, skipped")
                continue
            tech_column = at.get("tech_type")
            columns = []
            for index, name in enumerate(names):
                found = CROP_COLUMN.match(name)
                if found:
                    columns.append((index, found.group(1), found.group(2)))
                elif tech_column is not None and (name in OURS or name in GROUPS):
                    columns.append((index, name, None))  # tech comes from tech_type
            if not columns:
                log(f"{member}: no crop columns, skipped")
                continue
            if keep_rows is not None and not keep_rows["header"]:
                keep_rows["header"] = header
            for row in reader:
                if len(row) < len(names) or not is_iraq(row, at):
                    continue
                if keep_rows is not None:
                    keep_rows["rows"].append(row)
                lon, lat = number_or_zero(row[at["x"]]), number_or_zero(row[at["y"]])
                if lon is None or lat is None:
                    continue
                cell = cells.setdefault((lon, lat), {})
                row_tech = row[tech_column].strip().lower()[:1] if tech_column is not None else None
                for index, spam, tech in columns:
                    tech = tech or row_tech
                    if tech not in TECH:
                        continue
                    value = number(row[index])
                    if value is not None:
                        cell[(spam, tech)] = cell.get((spam, tech), 0.0) + value
    return cells


def number_or_zero(text):
    try:
        value = float(text)
    except (TypeError, ValueError):
        return None
    return value if math.isfinite(value) else None


def load(path):
    if not path.exists():
        sys.exit(
            f"MapSPAM file not found: {path}\n"
            "Download spam2020V2r2_global_physical_area.csv.zip from Harvard Dataverse "
            "(DOI 10.7910/DVN/SWPENT, guestbook form first), put it in backend/jobs/data/ "
            "or set MAPSPAM_CSV to its path. No numbers are pushed without it."
        )
    keep = {"header": None, "rows": []} if path.suffix.lower() == ".zip" else None
    log(f"reading {path}")
    cells = read_cells(path, keep)
    if not cells:
        sys.exit(f"{path} has no Iraq rows (iso3 IRQ) with x, y and crop columns; nothing pushed.")
    saved = path.parent / DEFAULT_CSV.name
    if keep and keep["rows"] and not saved.exists():
        with open(saved, "w", newline="") as out:
            writer = csv.writer(out)
            writer.writerow(keep["header"])
            writer.writerows(keep["rows"])
        log(f"saved {len(keep['rows'])} Iraq rows to {saved}")
    log(f"{len(cells)} Iraq squares")
    return cells


def csv_path():
    chosen = os.environ.get("MAPSPAM_CSV")
    if chosen:
        return Path(chosen)
    if DEFAULT_CSV.exists() or not GLOBAL_ZIP.exists():
        return DEFAULT_CSV
    return GLOBAL_ZIP


# ---------------------------------------------------------------- the topic
def area_around(cells, lat, lon):
    near = [key for key in cells if km_between(lat, lon, key[1], key[0]) <= RADIUS_KM]
    if cells:
        nearest = min(cells, key=lambda key: km_between(lat, lon, key[1], key[0]))
        if nearest not in near and km_between(lat, lon, nearest[1], nearest[0]) <= RADIUS_KM + 10:
            near.append(nearest)
    totals = {}
    for key in near:
        for (spam, tech), hectares in cells[key].items():
            code = our_code(spam)
            entry = totals.setdefault(code, {"all": 0.0, "irrigated": 0.0, "rainfed": 0.0, "split": False})
            entry[TECH[tech]] += hectares
            if tech in "ir":
                entry["split"] = True
    for entry in totals.values():
        if entry["all"] <= 0:  # a file with only the I and R columns
            entry["all"] = entry["irrigated"] + entry["rainfed"]
    return len(near), totals


def irrigation_words(pct):
    if pct >= 99:
        return "all irrigated"
    if pct >= 60:
        return "mostly irrigated"
    if pct <= 1:
        return "all rain-fed"
    if pct <= 40:
        return "mostly rain-fed"
    return "partly irrigated"


def join_names(names):
    return names[0] if len(names) == 1 else ", ".join(names[:-1]) + " and " + names[-1]


def crops_topic(cells, lat, lon):
    squares, totals = area_around(cells, lat, lon)
    whole = sum(entry["all"] for entry in totals.values())
    if squares == 0 or whole <= 0:
        return None
    kept = sorted(
        ((code, entry) for code, entry in totals.items() if entry["all"] / whole >= MIN_SHARE),
        key=lambda item: -item[1]["all"],
    )[:MAX_CROPS]
    measures, notes = [], []
    for code, entry in kept:
        label = name_of(code).capitalize()
        measures.append(measure(f"{code}_ha_15km", round(entry["all"]), "ha", f"{label} grown within 15 km"[:60]))
        if entry["split"] and entry["irrigated"] + entry["rainfed"] > 0:
            pct = 100 * entry["irrigated"] / (entry["irrigated"] + entry["rainfed"])
            measures.append(measure(f"{code}_irrigated_pct", round(pct), "%", f"{label}: share irrigated"[:60]))
            if len(notes) < 3 and len(notes) < len(kept[:4]):
                notes.append(f"{label} is {irrigation_words(pct)}.")
    main = [name_of(code) for code, _ in kept[:4]]
    sentences = [f"Around this farm the main crops are {join_names(main)}." if len(main) > 1
                 else f"Around this farm the main crop is {main[0]}."]
    sentences += notes
    sentences.append(f"Modelled crop areas over {round(whole):,} ha within {RADIUS_KM:.0f} km, not seen field by field.")
    return topic_body(AS_OF, SOURCE, "likely", sentences, measures)


def option(name):
    return sys.argv[sys.argv.index(name) + 1] if name in sys.argv else None


def main():
    dry_run = "--dry-run" in sys.argv
    lat, lon = option("--lat"), option("--lon")
    if (lat or lon) and not dry_run:
        sys.exit("--lat and --lon need --dry-run")
    if dry_run and not (lat and lon):
        sys.exit("--dry-run needs --lat and --lon, for example --lat 36.74 --lon 43.89 (Akre)")
    cells = load(csv_path())

    if dry_run:
        body = crops_topic(cells, float(lat), float(lon))
        print(json.dumps(body, ensure_ascii=False, indent=2) if body else "no MapSPAM crop area within 15 km")
        return

    if not KEY:
        sys.exit("INGEST__SERVICE_KEY is not set")
    farms = backend("GET", "/ingest/farms").get("farms", [])
    only = option("--farm")
    if only:
        farms = [farm for farm in farms if str(farm["id"]) == only]
    pushed = skipped = 0
    for farm in farms:
        body = crops_topic(cells, float(farm["lat"]), float(farm["lon"]))
        if body is None:
            skipped += 1
            log(f"farm {farm['id']}: no MapSPAM crop area within {RADIUS_KM:.0f} km, nothing pushed")
            continue
        backend("PUT", f"/ingest/farms/{farm['id']}/insights/{TOPIC}", body)
        pushed += 1
    log(f"{TOPIC}: pushed {pushed}, skipped {skipped}, of {len(farms)} farms")


if __name__ == "__main__":
    main()
