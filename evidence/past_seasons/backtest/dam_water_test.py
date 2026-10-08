"""Dam water backtest: can free satellite data measure Dukan / Darbandikhan lake area over time?

Method
  * Sentinel-2 L2A (2017-2026) and Landsat C2 L2 (2008/2013/2019) from Microsoft Planetary Computer.
  * Each lake = one rectangle in UTM 38N (EPSG:32638) that contains the whole reservoir (checked
    against JRC Global Surface Water max extent; the river below each dam is cut off).
  * The Planetary Computer /item/statistics endpoint is called with coord_crs=dst_crs=EPSG:32638 and
    an explicit width/height, so every output pixel is exactly 20 m x 20 m (S2) / 30 m (Landsat):
    area = water_pixel_count * pixel_area. No max_size guessing.
  * Sentinel-2 tiles overlap by 9.8 km, so each lake rectangle is cut along the middle of the tile
    overlaps into non-overlapping pieces, each read from its own MGRS tile of the same datatake
    (same satellite pass). Pieces are summed.
  * Water = NDWI (B03-B08)/(B03+B08) > 0 (reported as water_km2). MNDWI (B03-B11) and the SCL "water"
    class are kept as cross-checks: MNDWI was unstable date-to-date (e.g. Dukan 2026-09-22 Sentinel-2C
    164 km2 vs 247-250 on 09-24/09-27, while NDWI stayed 249-251), SCL water is far noisier still. The 0 threshold is unaffected by the +1000 DN offset of processing baseline >= 04.00
    (it only changes the denominator, not the sign).
  * Cloud check inside the rectangle from SCL (classes 3, 8, 9, 10); a date is "clear" if those cover
    < 0.5 % of the rectangle and every piece is >= 99 % valid (inside the satellite swath).

Usage
  python3 -I dam_water_test.py --data-dir <raw json cache dir> --csv <out csv> [--workers 4]
Every API response is cached in --data-dir, so re-runs are offline.
"""
import argparse
import concurrent.futures as cf
import csv
import datetime as dt
import hashlib
import json
import math
import os
import sys
import time
import urllib.parse
import urllib.request

UA = ("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 "
      "(KHTML, like Gecko) Chrome/128.0 Safari/537.36")
STAC = "https://planetarycomputer.microsoft.com/api/stac/v1/search"
STATS = "https://planetarycomputer.microsoft.com/api/data/v1/item/statistics"

# Lake rectangles in UTM 38N metres (x0, y0, x1, y1) and their pieces -> MGRS tile.
# S2 tile columns: M = x 399960-509760, N = x 499980-609780; rows: D = y 3790200-3900000,
# E = y 3890220-4000020, F = y 3990240-4100040. Cuts at x=505000, y=3995000, y=3895000.
LAKES = {
    "Dukan": {
        # relative orbit 135 sees the whole lake mid-swath; orbit 092's western swath edge cuts the NW
        # corner of the box and gave +10-35 km2 of near-zero MNDWI "water" on the same day -> excluded
        "orbits": {135},
        "rect": (471000, 3977000, 522000, 4020000),
        "pieces": {
            "38SME": (471000, 3977000, 505000, 3995000),
            "38SMF": (471000, 3995000, 505000, 4020000),
            "38SNE": (505000, 3977000, 522000, 3995000),
            "38SNF": (505000, 3995000, 522000, 4020000),
        },
    },
    "Darbandikhan": {
        "orbits": {92},  # only orbit 092 covers the lake
        "rect": (545000, 3885000, 595000, 3925000),
        "pieces": {
            "38SND": (545000, 3885000, 595000, 3895000),
            "38SNE": (545000, 3895000, 595000, 3925000),
        },
    },
}
JRC_ITEM = "40E_40Nv1_3_2020"
S2_EXPR = "(B03-B11)/(B03+B11);(B03-B08)/(B03+B08);SCL"
# shared histogram edges: fine bins in [-1, 1] for the indices, then one bin per SCL class group
IDX_EDGES = [-1.0001, -0.5, -0.4, -0.3, -0.25, -0.2, -0.15, -0.1, -0.05, 0, 0.05, 0.1, 0.15, 0.2, 0.3, 0.5, 1.0001]
BINS = IDX_EDGES + [2.5, 3.5, 5.5, 6.5, 7.5, 10.5, 11.5]


def bin_of(value):
    for i in range(len(BINS) - 1):
        if BINS[i] <= value < BINS[i + 1]:
            return i
    raise ValueError(value)


def km2_above(hist, thr, pk):
    """pixels in index bins whose left edge >= thr (thr must be an edge)."""
    return sum(hist[i] for i in range(len(IDX_EDGES) - 1) if IDX_EDGES[i] >= thr - 1e-9) * pk


SCL_WATER, SCL_SHADOW, SCL_CLOUD = bin_of(6), bin_of(3), bin_of(9)
# Landsat C2 L2: SR = DN*2.75e-5 - 0.2  ->  normalized difference = (a-b)/(a+b-14545.45) in DN
LS_EXPR = "(green-swir16)/(green+swir16-14545.4545);(green-nir08)/(green+nir08-14545.4545)"
LS_BINS = [-100, -0.3, -0.1, 0, 0.1, 0.3, 100]

DATA_DIR = None
TIMINGS = []  # seconds per live statistics request


# ---------------------------------------------------------------- geometry
def utm2ll(E, N, zone=38):
    """Inverse UTM (Krueger series), WGS84 -> (lat, lon). Only used for STAC search boxes."""
    a = 6378137.0; f = 1 / 298.257223563; k0 = 0.9996
    n = f / (2 - f); A = a / (1 + n) * (1 + n**2 / 4 + n**4 / 64)
    be = [n / 2 - 2 / 3 * n**2 + 37 / 96 * n**3, 1 / 48 * n**2 + 1 / 15 * n**3, 17 / 480 * n**3]
    de = [2 * n - 2 / 3 * n**2 - 2 * n**3, 7 / 3 * n**2 - 8 / 5 * n**3, 56 / 15 * n**3]
    xi = N / (k0 * A); eta = (E - 500000) / (k0 * A)
    xi_ = xi - sum(be[j] * math.sin(2 * (j + 1) * xi) * math.cosh(2 * (j + 1) * eta) for j in range(3))
    eta_ = eta - sum(be[j] * math.cos(2 * (j + 1) * xi) * math.sinh(2 * (j + 1) * eta) for j in range(3))
    chi = math.asin(math.sin(xi_) / math.cosh(eta_))
    lat = chi + sum(de[j] * math.sin(2 * (j + 1) * chi) for j in range(3))
    lon = math.radians(zone * 6 - 183) + math.atan2(math.sinh(eta_), math.cos(xi_))
    return math.degrees(lat), math.degrees(lon)


def rect_ll_polygon(rect):
    x0, y0, x1, y1 = rect
    pts = [utm2ll(x, y) for x, y in ((x0, y0), (x1, y0), (x1, y1), (x0, y1))]
    lats = [p[0] for p in pts]; lons = [p[1] for p in pts]
    a, b, c, d = min(lons) - 0.01, min(lats) - 0.01, max(lons) + 0.01, max(lats) + 0.01
    return {"type": "Polygon", "coordinates": [[[a, b], [c, b], [c, d], [a, d], [a, b]]]}


def utm_feature(rect):
    x0, y0, x1, y1 = rect
    return {"type": "Feature", "properties": {}, "geometry": {
        "type": "Polygon", "coordinates": [[[x0, y0], [x1, y0], [x1, y1], [x0, y1], [x0, y0]]]}}


# ---------------------------------------------------------------- http + cache
def post_json(url, body, timeout=300):
    key = hashlib.sha1((url + json.dumps(body, sort_keys=True)).encode()).hexdigest()
    path = os.path.join(DATA_DIR, key + ".json")
    if os.path.exists(path):
        with open(path) as fh:
            return json.load(fh), 0.0, True
    last = None
    for attempt in range(5):
        req = urllib.request.Request(url, data=json.dumps(body).encode(),
                                     headers={"User-Agent": UA, "Content-Type": "application/json"})
        t = time.time()
        try:
            with urllib.request.urlopen(req, timeout=timeout) as r:
                d = json.loads(r.read())
            el = time.time() - t
            with open(path, "w") as fh:
                json.dump({"_request": {"url": url, "body": body}, **d} if isinstance(d, dict) else d, fh)
            return d, el, False
        except Exception as e:  # 504s happen; retry with backoff
            last = e
            time.sleep(2 + 3 * attempt)
    raise RuntimeError(f"request failed after retries: {url[:160]} -> {last}")


def stac_search(collection, intersects, datetime_, query=None):
    body = {"collections": [collection], "intersects": intersects, "datetime": datetime_, "limit": 250}
    if query:
        body["query"] = query
    out = []
    while True:
        d, _, _ = post_json(STAC, body)
        out += d.get("features", [])
        nxt = [l for l in d.get("links", []) if l.get("rel") == "next"]
        if not nxt or not d.get("features"):
            break
        body = nxt[0].get("body") or body
        if not nxt[0].get("body"):
            break
    return out


def item_stats(collection, item, params, rect, px):
    x0, y0, x1, y1 = rect
    W = int(round((x1 - x0) / px)); H = int(round((y1 - y0) / px))
    q = dict(collection=collection, item=item, coord_crs="EPSG:32638", dst_crs="EPSG:32638",
             width=W, height=H, **params)
    url = STATS + "?" + urllib.parse.urlencode(q)
    d, el, cached = post_json(url, utm_feature(rect))
    if not cached:
        TIMINGS.append(el)
    st = d["properties"]["statistics"]
    pix_km2 = ((x1 - x0) / W) * ((y1 - y0) / H) / 1e6
    return st, pix_km2, W * H


# ---------------------------------------------------------------- JRC reference
def jrc_reference(lake):
    rect = LAKES[lake]["rect"]
    st, pk, _ = item_stats("jrc-gsw", JRC_ITEM, dict(assets="occurrence", histogram_bins="0.5,25.5,50.5,75.5,100.5"),
                           rect, 30)
    h = list(st.values())[0]["histogram"][0]
    return {"max_extent_km2": sum(h) * pk, "occ_ge50_km2": (h[2] + h[3]) * pk,
            "occ_ge25_km2": (h[1] + h[2] + h[3]) * pk}


# ---------------------------------------------------------------- Sentinel-2
def s2_datatakes(lake, start, end, max_cloud=20):
    pieces = LAKES[lake]["pieces"]
    feats = stac_search("sentinel-2-l2a", rect_ll_polygon(LAKES[lake]["rect"]), f"{start}/{end}",
                        {"eo:cloud_cover": {"lt": max_cloud}})
    groups = {}
    for f in feats:
        p = f["properties"]
        tile = p.get("s2:mgrs_tile")
        if tile not in pieces:
            continue
        key = (p["datetime"][:10], p.get("sat:relative_orbit"), p.get("platform"))
        g = groups.setdefault(key, {})
        # keep the newest processing if duplicates
        if tile not in g or f["id"] > g[tile]["id"]:
            g[tile] = {"id": f["id"], "cloud": p.get("eo:cloud_cover", 100),
                       "nodata": p.get("s2:nodata_pixel_percentage")}
    full = []
    for key, g in groups.items():
        if all(t in g for t in pieces) and key[1] in LAKES[lake]["orbits"]:
            full.append({"date": key[0], "orbit": key[1], "platform": key[2], "tiles": g,
                         "max_cloud": max(v["cloud"] for v in g.values())})
    return full


def s2_measure(lake, dtake):
    pieces = LAKES[lake]["pieces"]
    tot = {"mndwi": 0.0, "mndwi_m01": 0.0, "ndwi": 0.0, "scl_water": 0.0, "cloud": 0.0, "pix": 0, "valid": 0}
    minvalid = 100.0
    t0 = time.time()
    # all pieces of one date are fetched in parallel (this is the live-demo latency)
    with cf.ThreadPoolExecutor(len(pieces)) as ex:
        res = list(ex.map(lambda tr: item_stats(
            "sentinel-2-l2a", dtake["tiles"][tr[0]]["id"],
            dict(expression=S2_EXPR, asset_as_band="true", histogram_bins=",".join(str(b) for b in BINS)),
            tr[1], 20), pieces.items()))
    for st, pk, npx in res:
        keys = list(st.keys())
        mn, nd, scl = st[keys[0]], st[keys[1]], st[keys[2]]
        hm, hn, hs = mn["histogram"][0], nd["histogram"][0], scl["histogram"][0]
        tot["mndwi"] += km2_above(hm, 0, pk)
        tot["mndwi_m01"] += km2_above(hm, -0.1, pk)       # threshold sensitivity
        tot["ndwi"] += km2_above(hn, 0, pk)
        tot["scl_water"] += hs[SCL_WATER] * pk            # SCL class 6
        tot["cloud"] += (hs[SCL_SHADOW] + hs[SCL_CLOUD]) * pk  # class 3 shadow, 8-10 cloud
        tot["pix"] += npx * pk
        tot["valid"] += mn["valid_pixels"] * pk
        minvalid = min(minvalid, mn["valid_percent"])
    tot["elapsed"] = time.time() - t0
    tot["minvalid"] = minvalid
    tot["valid_percent"] = 100 * tot["valid"] / tot["pix"]
    tot["cloud_pct"] = 100 * tot["cloud"] / tot["pix"]
    tot["clear"] = tot["cloud_pct"] < 0.5 and minvalid >= 99.5
    return tot


def s2_window(lake, year, label, start, end, target, want=3, max_eval=8):
    try:
        cands = s2_datatakes(lake, f"{year}-{start}", f"{year}-{end}")
    except Exception as e:
        return [{"lake": lake, "year": year, "window": label, "error": str(e)}]
    tgt = dt.date(year, *map(int, target.split("-")))
    cands.sort(key=lambda c: (0 if c["max_cloud"] < 2 else 1 if c["max_cloud"] < 10 else 2,
                              abs((dt.date.fromisoformat(c["date"]) - tgt).days)))
    out, n_eval = [], 0
    for c in cands:
        if len([o for o in out if o.get("clear")]) >= want or n_eval >= max_eval:
            break
        n_eval += 1
        try:
            m = s2_measure(lake, c)
        except Exception as e:
            out.append({"lake": lake, "year": year, "window": label, "date": c["date"], "error": str(e)})
            continue
        out.append({"lake": lake, "year": year, "window": label, "date": c["date"],
                    "source": f"Sentinel-2 {c['platform'][-2:]} R{c['orbit']:03d}",
                    "items": "|".join(v["id"] for v in c["tiles"].values()), **m})
    if not cands:
        out.append({"lake": lake, "year": year, "window": label, "error": "no datatake with all tiles and tile cloud<20%"})
    return out


# ---------------------------------------------------------------- Landsat
def landsat_dates(lake, start, end, platform, max_cloud=5):
    feats = stac_search("landsat-c2-l2", rect_ll_polygon(LAKES[lake]["rect"]), f"{start}/{end}",
                        {"eo:cloud_cover": {"lt": max_cloud}, "platform": {"eq": platform}})
    feats.sort(key=lambda f: f["properties"]["eo:cloud_cover"])
    return feats


def landsat_measure(lake, feats):
    """feats: scenes of one date and WRS path (rows 35/36). Rect is cut into 10 km bands; each band is
    read from the first scene that covers it >= 99.5 %."""
    x0, y0, x1, y1 = LAKES[lake]["rect"]
    t0 = time.time()
    tot = {"mndwi": 0.0, "ndwi": 0.0, "pix": 0.0, "valid": 0.0}
    used = set()
    y = y0
    while y < y1:
        band = (x0, y, x1, min(y + 10000, y1))
        best = None
        for f in feats:
            st, pk, npx = item_stats("landsat-c2-l2", f["id"],
                                     dict(expression=LS_EXPR, asset_as_band="true", nodata=0,
                                          histogram_bins=",".join(str(b) for b in LS_BINS)), band, 30)
            mn = list(st.values())[0]
            if best is None or mn["valid_percent"] > best[0]:
                best = (mn["valid_percent"], st, pk, npx, f["id"])
            if mn["valid_percent"] >= 99.5:
                break
        vp, st, pk, npx, fid = best
        used.add(fid)
        mn, nd = list(st.values())
        tot["mndwi"] += sum(mn["histogram"][0][3:]) * pk
        tot["ndwi"] += sum(nd["histogram"][0][3:]) * pk
        tot["pix"] += npx * pk
        tot["valid"] += mn["valid_pixels"] * pk
        y += 10000
    vp = 100 * tot["valid"] / tot["pix"]
    return {"mndwi": tot["mndwi"], "ndwi": tot["ndwi"], "mndwi_m01": None, "valid_percent": vp,
            "minvalid": vp, "elapsed": time.time() - t0,
            "cloud_pct": max(f["properties"]["eo:cloud_cover"] for f in feats),
            "scl_water": None, "clear": vp >= 99.0, "items": "|".join(sorted(used))}


# ---------------------------------------------------------------- usable dates per year
def usable_per_year(lake, year):
    cands = s2_datatakes(lake, f"{year}-01-01", f"{year}-12-31", max_cloud=30)
    dates1 = {c["date"] for c in cands if c["max_cloud"] < 1}
    dates5 = {c["date"] for c in cands if c["max_cloud"] < 5}
    return len(dates1), len(dates5)


# ---------------------------------------------------------------- main
def main():
    global DATA_DIR
    ap = argparse.ArgumentParser()
    ap.add_argument("--data-dir", required=True)
    ap.add_argument("--csv", required=True)
    ap.add_argument("--workers", type=int, default=3)
    ap.add_argument("--first-year", type=int, default=2017)
    ap.add_argument("--last-year", type=int, default=2026)
    args = ap.parse_args()
    DATA_DIR = args.data_dir
    os.makedirs(DATA_DIR, exist_ok=True)

    ref = {lake: jrc_reference(lake) for lake in LAKES}
    for lake, r in ref.items():
        print(f"JRC GSW {lake}: max extent 1984-2021 {r['max_extent_km2']:.1f} km2, "
              f"water >=50% of time {r['occ_ge50_km2']:.1f} km2")

    jobs = []
    for lake in LAKES:
        for y in range(args.first_year, args.last_year + 1):
            jobs.append((lake, y, "late-May/Jun", "05-20", "06-30", "06-01"))
            jobs.append((lake, y, "Sep/Oct", "09-01", "10-31", "10-01"))
    rows = []
    with cf.ThreadPoolExecutor(args.workers) as ex:
        for res in ex.map(lambda j: s2_window(*j), jobs):
            rows += res

    ls_jobs = [("2008-05-15", "2008-06-30", "landsat-5"), ("2013-05-15", "2013-06-30", "landsat-8"),
               ("2019-05-20", "2019-06-20", "landsat-8")]
    for lake in LAKES:
        for s, e, plat in ls_jobs:
            try:
                feats = landsat_dates(lake, s, e, plat)
            except Exception as ex_:
                rows.append({"lake": lake, "year": int(s[:4]), "window": "Landsat", "error": str(ex_)})
                continue
            groups = {}
            for f in feats:  # group scenes by date + WRS path
                groups.setdefault((f["properties"]["datetime"][:10], f["id"].split("_")[2][:3]), []).append(f)
            order = sorted(groups.items(), key=lambda kv: max(f["properties"]["eo:cloud_cover"] for f in kv[1]))
            got = 0
            for (date, path), fs in order[:4]:
                try:
                    m = landsat_measure(lake, fs)
                except Exception as ex_:
                    rows.append({"lake": lake, "year": int(s[:4]), "window": "Landsat", "date": date,
                                 "error": str(ex_)})
                    continue
                rows.append({"lake": lake, "year": int(s[:4]), "window": "Landsat", "date": date,
                             "source": f"{plat} path {path}", **m})
                got += bool(m["clear"])
                if got >= 2:
                    break
            if not feats:
                rows.append({"lake": lake, "year": int(s[:4]), "window": "Landsat", "error": "no scene <10% cloud"})

    rows.sort(key=lambda r: (r["lake"], r.get("date") or str(r["year"])))
    with open(args.csv, "w", newline="") as fh:
        w = csv.writer(fh)
        w.writerow(["lake", "date", "source", "water_km2", "valid_percent", "window", "clear",
                    "mndwi_km2", "scl_water_km2", "mndwi_gt_minus0.1_km2", "cloud_pct", "pct_of_jrc_max",
                    "seconds", "items"])
        for r in rows:
            if "error" in r:
                w.writerow([r["lake"], r.get("date", ""), "ERROR", "", "", r["window"], "", "", "", "", "", "", "",
                            r["error"]])
                continue
            mx = ref[r["lake"]]["max_extent_km2"]
            w.writerow([r["lake"], r["date"], r["source"], f"{r['ndwi']:.2f}", f"{r['valid_percent']:.2f}",
                        r["window"], int(bool(r["clear"])), f"{r['mndwi']:.2f}",
                        "" if r["scl_water"] is None else f"{r['scl_water']:.2f}",
                        "" if r.get("mndwi_m01") is None else f"{r['mndwi_m01']:.2f}", f"{r['cloud_pct']:.2f}",
                        f"{100 * r['ndwi'] / mx:.1f}", f"{r['elapsed']:.1f}", r["items"]])
    print(f"wrote {args.csv} ({len(rows)} rows)")

    print("\nKey table (clear dates only; first number = NDWI>0 km2):")
    for r in rows:
        if "error" in r:
            print(f"  {r['lake']:12s} {r['year']} {r['window']:12s} ERROR {r['error'][:100]}")
        elif r["clear"]:
            print(f"  {r['lake']:12s} {r['date']} {r['window']:12s} {r['ndwi']:7.1f} km2 "
                  f"(MNDWI {r['mndwi']:6.1f}, SCL {r['scl_water'] if r['scl_water'] is not None else float('nan'):6.1f}, "
                  f"cloud {r['cloud_pct']:.2f}%, valid {r['valid_percent']:.1f}%) {r['source']}")
    summary = {}
    for r in rows:
        if "error" not in r and r["clear"]:
            k = (r["lake"], r["year"], r["window"])
            summary.setdefault(k, []).append(r["ndwi"])
    print("\nPer-window median of clear dates (km2) [n dates, min-max]:")
    med = {}
    for k in sorted(summary):
        v = sorted(summary[k]); m = v[len(v) // 2] if len(v) % 2 else (v[len(v)//2 - 1] + v[len(v)//2]) / 2
        med[k] = m
        print(f"  {k[0]:12s} {k[1]} {k[2]:12s} {m:7.1f}  [n={len(v)}, {v[0]:.1f}-{v[-1]:.1f}]")
    with open(os.path.splitext(args.csv)[0] + "_window_medians.json", "w") as fh:
        json.dump({f"{a}|{b}|{c}": v for (a, b, c), v in med.items()} | {"jrc": ref}, fh, indent=1)
    if TIMINGS:
        TIMINGS.sort()
        print(f"\nlive statistics requests: n={len(TIMINGS)}, median {TIMINGS[len(TIMINGS)//2]:.1f}s, "
              f"max {TIMINGS[-1]:.1f}s")

    print("\nUsable Sentinel-2 dates per year (all lake tiles, tile cloud <1% / <5%):")
    for lake in LAKES:
        for y in (2018, 2019, 2021, 2023, 2024, 2025):
            try:
                a, b = usable_per_year(lake, y)
                print(f"  {lake:12s} {y}: {a} / {b}")
            except Exception as e:
                print(f"  {lake:12s} {y}: ERROR {e}")


if __name__ == "__main__":
    main()
