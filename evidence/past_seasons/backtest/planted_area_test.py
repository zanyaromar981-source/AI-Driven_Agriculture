#!/usr/bin/env python3
"""Planted-area test: can free 10 m Sentinel-2 L2A (a) map winter-crop sown area per season
and (b) tell a crop field from fallow at single-field level?

Python stdlib only. Data: Microsoft Planetary Computer (STAC + titiler data API, no key).

Usage (run with python3 -I):
  planted_area_test.py fetch      DATA_DIR
  planted_area_test.py analyze    DATA_DIR KRSO_CSV OUT_DIR
  planted_area_test.py profile    DATA_DIR OUT_DIR [WINDOW] [SEASONS e.g. 2025,2026]
  planted_area_test.py fieldcheck LON LAT HARVEST_YEAR      (timed live-demo style check)

Per-pixel method (per window, per harvest year Y):
  spring composite = per-pixel MAX NDVI over up to 3 clear scenes (Mar 5 - May 5 of Y)
  summer composite = per-pixel MAX NDVI over up to 2 clear scenes (Jul 15 - Aug 31 of Y)
  classes: summer >= 0.25           -> summer-green (orchard / trees / irrigated summer crop)
           spring >= T, summer<0.25  -> winter crop (wheat/barley-like)   [T = 0.45, sens 0.40/0.50]
           spring <  T, summer<0.25  -> fallow / bare / sparse
  Clouds removed with the L2A scene classification (SCL keep 4,5,6,7).
  Processing baseline >= 04.00 (2022-01-25 on) has a +1000 DN offset -> removed before NDVI.
"""
import sys, os, json, time, struct, ast, array, math, zlib, csv, datetime as dt
import urllib.request, urllib.error
import concurrent.futures as cf

UA = ("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 "
      "(KHTML, like Gecko) Chrome/126.0 Safari/537.36")
STAC = "https://planetarycomputer.microsoft.com/api/stac/v1/search"
DATA = "https://planetarycomputer.microsoft.com/api/data/v1"

# name: (lon, lat, governorate for KRSO comparison)
WINDOWS = {
    "erbil_plain": (43.90, 36.10, "Erbil"),
    "koya_plain": (44.60, 36.03, "Erbil"),
    "sumel_plain": (42.80, 36.84, "Duhok"),
}
YEARS = list(range(2018, 2027))
SPRING_SUB = [("03-05", "03-25"), ("03-26", "04-15"), ("04-16", "05-05")]
SUMMER_SUB = [("07-15", "08-07"), ("08-08", "08-31")]
VALID_SCL = {4, 5, 6, 7}
T_CROP = 0.45
T_SUMMER = 0.25
MIN_VALID = 0.60  # scene accepted if >=60% of window is clear (max-NDVI composite tolerates the rest)
NAN = float("nan")


# ----------------------------------------------------------------- helpers
def bbox_of(lon, lat, half_km=2.5):
    dlon = half_km / (111.32 * math.cos(math.radians(lat)))
    dlat = half_km / 110.57
    return (round(lon - dlon, 5), round(lat - dlat, 5), round(lon + dlon, 5), round(lat + dlat, 5))


def http(url, body=None, timeout=240, tries=4):
    err = None
    for k in range(tries):
        try:
            h = {"User-Agent": UA, "Accept": "*/*"}
            data = None
            if body is not None:
                data = json.dumps(body).encode()
                h["Content-Type"] = "application/json"
            with urllib.request.urlopen(urllib.request.Request(url, data=data, headers=h), timeout=timeout) as r:
                return r.read()
        except urllib.error.HTTPError as e:
            if e.code in (400, 404, 422):
                raise
            err = e
        except Exception as e:  # timeouts, resets
            err = e
        time.sleep(3 * (k + 1))
    raise err


def stac_search(bbox, start, end, max_cloud=40):
    body = {"collections": ["sentinel-2-l2a"], "bbox": list(bbox),
            "datetime": f"{start}T00:00:00Z/{end}T23:59:59Z",
            "query": {"eo:cloud_cover": {"lt": max_cloud}}, "limit": 250}
    feats = []
    for _ in range(10):
        d = json.loads(http(STAC, body))
        feats += d.get("features", [])
        nxt = [l for l in d.get("links", []) if l.get("rel") == "next"]
        if not nxt or not nxt[0].get("body"):
            break
        body = nxt[0]["body"]
    return feats


def _pip(x, y, ring):
    inside, j = False, len(ring) - 1
    for i in range(len(ring)):
        xi, yi = ring[i][0], ring[i][1]
        xj, yj = ring[j][0], ring[j][1]
        if (yi > y) != (yj > y) and x < (xj - xi) * (y - yi) / (yj - yi) + xi:
            inside = not inside
        j = i
    return inside


def contains(geom, bbox):
    polys = [geom["coordinates"]] if geom["type"] == "Polygon" else geom["coordinates"]
    corners = [(bbox[0], bbox[1]), (bbox[0], bbox[3]), (bbox[2], bbox[1]), (bbox[2], bbox[3])]
    return all(any(_pip(x, y, p[0]) for p in polys) for x, y in corners)


def item_meta(f):
    p = f["properties"]
    return {"id": f["id"], "date": p["datetime"][:10], "cloud": round(p.get("eo:cloud_cover", -1), 2),
            "tile": p.get("s2:mgrs_tile"), "baseline": float(p.get("s2:processing_baseline", "0") or 0)}


WINDOW_MAX_SIZE = 320  # window crops ~17x21 m px (full 10 m takes ~15 s/scene; 320 px ~3 s)


def crop_url(item, bbox, max_size=None):
    b = ",".join(str(v) for v in bbox)
    return (f"{DATA}/item/bbox/{b}.npy?collection=sentinel-2-l2a&item={item}"
            "&assets=B04&assets=B08&assets=SCL" + (f"&max_size={max_size}" if max_size else ""))


def parse_npy(b):
    if b[:6] != b"\x93NUMPY":
        raise ValueError("not npy: " + b[:80].decode("latin1"))
    if b[6] == 1:
        hl = struct.unpack("<H", b[8:10])[0]; off = 10
    else:
        hl = struct.unpack("<I", b[8:12])[0]; off = 12
    h = ast.literal_eval(b[off:off + hl].decode("latin1"))
    code = {"<u2": "H", "<i2": "h", "|u1": "B", "<f4": "f", "<f8": "d"}[h["descr"]]
    a = array.array(code)
    a.frombytes(b[off + hl:])
    return h["shape"], a


def scene_ndvi(npy_bytes, baseline):
    """-> (H, W, ndvi list with NaN for invalid, valid fraction)"""
    shape, a = parse_npy(npy_bytes)
    nb, H, W = shape
    n = H * W
    off = 1000 if baseline >= 4.0 else 0
    if off:  # a few PC items (seen on S2C 2025-08-01, 2026-06-24) carry no +1000 offset despite baseline>=4:
        # with the offset, B04 DN < 1000 would mean negative red reflectance -> if common, assume no offset
        lowred = sum(1 for i in range(n) if a[2 * n + i] in (4, 5) and a[i] < 1000)
        land = sum(1 for i in range(n) if a[2 * n + i] in (4, 5))
        if land and lowred / land > 0.05:
            off = 0
    out = [NAN] * n
    good = 0
    for i in range(n):
        if nb >= 4 and a[3 * n + i] == 0:
            continue
        if a[2 * n + i] not in VALID_SCL:
            continue
        r = a[i] - off
        nir = a[n + i] - off
        s = nir + r
        if s <= 0:
            continue
        out[i] = max(-1.0, min(1.0, (nir - r) / s))
        good += 1
    return H, W, out, good / n


def write_png(path, W, H, rgb_rows):
    raw = b"".join(b"\x00" + bytes(r) for r in rgb_rows)

    def chunk(t, d):
        return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xffffffff)
    with open(path, "wb") as fh:
        fh.write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", W, H, 8, 2, 0, 0, 0))
                 + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def pearson(x, y):
    n = len(x)
    if n < 3:
        return NAN
    mx, my = sum(x) / n, sum(y) / n
    sxy = sum((a - mx) * (b - my) for a, b in zip(x, y))
    sxx = sum((a - mx) ** 2 for a in x)
    syy = sum((b - my) ** 2 for b in y)
    return sxy / math.sqrt(sxx * syy) if sxx > 0 and syy > 0 else NAN


# ----------------------------------------------------------------- fetch
def pick_tile(bbox):
    feats = stac_search(bbox, "2026-03-01", "2026-05-31", 60)
    cnt = {}
    for f in feats:
        if contains(f["geometry"], bbox):
            t = f["properties"].get("s2:mgrs_tile")
            cnt[t] = cnt.get(t, 0) + 1
    if not cnt:
        raise RuntimeError(f"no tile fully contains {bbox}")
    return max(cnt, key=cnt.get)


def fetch_task(win, bbox, tile, year, phase, subs, wdir):
    """one STAC search per window-year-phase; per sub-window take the clearest scene with >=MIN_VALID clear px."""
    feats = stac_search(bbox, f"{year}-{subs[0][0]}", f"{year}-{subs[-1][1]}", 60)
    allc = [item_meta(f) for f in feats
            if f["properties"].get("s2:mgrs_tile") == tile and contains(f["geometry"], bbox)]
    log = []
    for k, (a, b) in enumerate(subs):
        cands = sorted([m for m in allc if f"{year}-{a}" <= m["date"] <= f"{year}-{b}"], key=lambda m: m["cloud"])
        for m in cands[:4]:
            fn = os.path.join(wdir, f"{year}_{phase}{k}_{m['date']}_{m['id']}.npy")
            t0 = time.time()
            if os.path.exists(fn):
                raw = open(fn, "rb").read()
            else:
                raw = http(crop_url(m["id"], bbox, WINDOW_MAX_SIZE))
                with open(fn, "wb") as fh:
                    fh.write(raw)
            H, W, nd, vf = scene_ndvi(raw, m["baseline"])
            rec = dict(m, window=win, year=year, phase=phase, sub=k, file=os.path.basename(fn),
                       valid_frac=round(vf, 4), shape=[H, W], secs=round(time.time() - t0, 1), used=vf >= MIN_VALID)
            log.append(rec)
            if rec["used"]:
                break
        if not any(r["used"] for r in log if r["sub"] == k):
            log.append({"window": win, "year": year, "phase": phase, "sub": k, "used": False,
                        "error": f"no scene with >=60% clear px ({len(cands)} candidates <60% cloud)"})
    return log


def cmd_fetch(data_dir):
    os.makedirs(data_dir, exist_ok=True)
    tasks = []
    tiles = {}
    for win, (lon, lat, _) in WINDOWS.items():
        bbox = bbox_of(lon, lat)
        tiles[win] = pick_tile(bbox)
        print(win, bbox, "tile", tiles[win], flush=True)
        wdir = os.path.join(data_dir, win)
        os.makedirs(wdir, exist_ok=True)
        for y in YEARS:
            tasks.append((win, bbox, tiles[win], y, "spring", SPRING_SUB, wdir))
            tasks.append((win, bbox, tiles[win], y, "summer", SUMMER_SUB, wdir))
    t0 = time.time()
    allrec = []
    with cf.ThreadPoolExecutor(8) as ex:
        futs = {ex.submit(fetch_task, *t): t for t in tasks}
        for fu in cf.as_completed(futs):
            t = futs[fu]
            try:
                recs = fu.result()
            except Exception as e:
                recs = [{"window": t[0], "year": t[3], "phase": t[4], "sub": -1, "used": False,
                         "error": repr(e)[:300]}]
            allrec += recs
            used = [r["date"] + "(" + str(r["valid_frac"]) + ")" for r in recs if r.get("used")]
            print(f"{t[0]} {t[3]} {t[4]}: used {' '.join(used) or 'NONE'}; tried {len(recs)}"
                  f"  [{time.time() - t0:.0f}s]", flush=True)
    with open(os.path.join(data_dir, "scenes_index.json"), "w") as fh:
        json.dump({"windows": {w: {"bbox": bbox_of(*WINDOWS[w][:2]), "tile": tiles[w]} for w in WINDOWS},
                   "scenes": sorted(allrec, key=lambda r: (r["window"], r["year"], r["phase"], r["sub"]))},
                  fh, indent=1)
    print("fetch done in", round(time.time() - t0), "s")


# ----------------------------------------------------------------- analyze
def composites(data_dir, idx, win, year):
    """spring max / summer max NDVI lists for a window-year."""
    scenes = [s for s in idx["scenes"] if s.get("window") == win and s.get("year") == year and s.get("used")]
    comp = {}
    dates = {"spring": [], "summer": []}
    shape = None
    for s in sorted(scenes, key=lambda s: s["date"]):
        b = open(os.path.join(data_dir, win, s["file"]), "rb").read()
        H, W, nd, vf = scene_ndvi(b, s["baseline"])
        if shape is None:
            shape = (H, W)
        elif shape != (H, W):
            print("  shape mismatch, skipping", s["file"])
            continue
        dates[s["phase"]].append(s["date"])
        c = comp.get(s["phase"])
        if c is None:
            comp[s["phase"]] = nd
        else:
            for i, v in enumerate(nd):
                if v == v and not (c[i] >= v):  # c NaN or smaller
                    c[i] = v
    return shape, comp.get("spring"), comp.get("summer"), dates


def classify(sp, su, t_crop=T_CROP):
    cls = [0] * len(sp)
    for i in range(len(sp)):
        a, b = sp[i], su[i]
        if a != a or b != b:
            continue
        if a < 0 and b < 0:
            cls[i] = 4  # water
        elif b >= T_SUMMER:
            cls[i] = 3
        elif a >= t_crop:
            cls[i] = 1
        else:
            cls[i] = 2
    return cls


def krso_area(path):
    best = {}
    with open(path, newline="", encoding="utf-8") as fh:
        for r in csv.DictReader(fh):
            if r["system"] not in ("all", "all_derived") or not r["area"]:
                continue
            key = (r["governorate"], int(r["harvest_year"]), r["crop"])
            if key in best and best[key][0] == "all":
                continue
            best[key] = (r["system"], float(r["area"]), float(r["production"] or 0))
    tot, prod = {}, {}
    for (g, y, c), (_, a, p) in best.items():
        tot[(g, y)] = tot.get((g, y), 0) + a
        prod[(g, y)] = prod.get((g, y), 0) + p
    return tot, prod


COLORS = {0: (0, 0, 0), 1: (230, 190, 30), 2: (150, 110, 80), 3: (20, 120, 40), 4: (40, 80, 200)}


def cmd_analyze(data_dir, krso_csv, out_dir):
    os.makedirs(out_dir, exist_ok=True)
    idx = json.load(open(os.path.join(data_dir, "scenes_index.json")))
    rows = []
    for win in WINDOWS:
        panels = []
        for y in YEARS:
            shape, sp, su, dates = composites(data_dir, idx, win, y)
            if sp is None or su is None:
                rows.append({"window": win, "harvest_year": y, "note": "missing spring or summer scene",
                             "spring_dates": " ".join(dates["spring"]), "summer_dates": " ".join(dates["summer"])})
                panels.append(None)
                continue
            H, W = shape
            res = {"window": win, "harvest_year": y, "spring_dates": " ".join(dates["spring"]),
                   "summer_dates": " ".join(dates["summer"])}
            for t in (0.40, 0.45, 0.50):
                cls = classify(sp, su, t)
                c = [cls.count(k) for k in range(5)]
                land = c[1] + c[2] + c[3]
                if t == T_CROP:
                    res.update(valid_px=land, valid_frac=round(land / len(cls), 3),
                               frac_winter_crop=round(c[1] / land, 3), frac_fallow_bare=round(c[2] / land, 3),
                               frac_summer_green=round(c[3] / land, 3),
                               sown_share_of_arable=round(c[1] / (c[1] + c[2]), 3))
                    weak = sum(1 for i in range(len(cls)) if cls[i] == 2 and sp[i] >= 0.30)
                    res["frac_weak_green_030_045"] = round(weak / land, 3)
                    cropv = sorted(sp[i] for i in range(len(cls)) if cls[i] == 1)
                    res["median_spring_ndvi_crop_px"] = round(cropv[len(cropv) // 2], 3) if cropv else ""
                    panels.append((H, W, cls))
                else:
                    res[f"frac_winter_crop_T{int(t * 100):02d}"] = round(c[1] / land, 3)
            rows.append(res)
            print(win, y, res.get("frac_winter_crop"), res.get("sown_share_of_arable"),
                  res["spring_dates"], "|", res["summer_dates"], flush=True)
        # montage 3x3, half resolution
        ps = [p for p in panels if p]
        if ps:
            H, W = ps[0][0], ps[0][1]
            h2, w2, gap = H // 2, W // 2, 6
            MW, MH = 3 * w2 + 2 * gap, 3 * h2 + 2 * gap
            img = [bytearray([255] * (MW * 3)) for _ in range(MH)]
            for k, p in enumerate(panels):
                if not p:
                    continue
                oy, ox = (k // 3) * (h2 + gap), (k % 3) * (w2 + gap)
                for r in range(h2):
                    row = img[oy + r]
                    for cc in range(w2):
                        col = COLORS[p[2][(2 * r) * W + 2 * cc]]
                        j = 3 * (ox + cc)
                        row[j:j + 3] = bytes(col)
            write_png(os.path.join(out_dir, f"classmap_{win}_2018-2026.png"), MW, MH, img)
    fields = ["window", "harvest_year", "frac_winter_crop", "frac_winter_crop_T40", "frac_winter_crop_T50",
              "frac_fallow_bare", "frac_weak_green_030_045", "frac_summer_green", "sown_share_of_arable",
              "median_spring_ndvi_crop_px", "valid_px", "valid_frac", "spring_dates", "summer_dates", "note"]
    with open(os.path.join(out_dir, "sown_fraction.csv"), "w", newline="") as fh:
        w = csv.DictWriter(fh, fieldnames=fields, extrasaction="ignore")
        w.writeheader()
        for r in rows:
            w.writerow(r)

    # ---- comparison with KRSO + drought/wet check
    area, prod = krso_area(krso_csv)
    lines = []
    wet, dry = (2019, 2020, 2026), (2021, 2022, 2025)
    for win, (_, _, gov) in WINDOWS.items():
        rr = {r["harvest_year"]: r for r in rows if r["window"] == win and "frac_winter_crop" in r}
        yrs = [y for y in range(2018, 2024) if y in rr and (gov, y) in area]
        for metric in ("frac_winter_crop", "sown_share_of_arable"):
            x = [rr[y][metric] for y in yrs]
            a1 = [area[(gov, y)] for y in yrs]
            a2 = [area[("Kurdistan_Region", y)] for y in yrs]
            dx = [x[i + 1] - x[i] for i in range(len(x) - 1)]
            da = [a1[i + 1] - a1[i] for i in range(len(a1) - 1)]
            agree = sum(1 for p, q in zip(dx, da) if p * q > 0)
            p1 = [prod[(gov, y)] for y in yrs]
            lines.append(f"{win} [{metric}] vs KRSO {gov} wheat+barley area {yrs[0]}-{yrs[-1]} (n={len(yrs)}): "
                         f"r={pearson(x, a1):+.2f}; vs Kurdistan_Region total r={pearson(x, a2):+.2f}; "
                         f"year-to-year direction agree {agree}/{len(dx)}; "
                         f"vs KRSO {gov} PRODUCTION r={pearson(x, p1):+.2f}")
        wv = [rr[y]["frac_winter_crop"] for y in wet if y in rr]
        dv = [rr[y]["frac_winter_crop"] for y in dry if y in rr]
        if wv and dv:
            lines.append(f"{win}: mean winter-crop fraction wet years {wet} = {sum(wv) / len(wv):.3f}; "
                         f"drought years {dry} = {sum(dv) / len(dv):.3f}")
    lines.append("KRSO wheat+barley area (donum) / production (t):")
    for gov in ("Erbil", "Duhok", "Kurdistan_Region"):
        lines.append("  " + gov + " area: " + ", ".join(f"{y}={area[(gov, y)]:,.0f}" for y in range(2018, 2024) if (gov, y) in area))
        lines.append("  " + gov + " prod: " + ", ".join(f"{y}={prod[(gov, y)]:,.0f}" for y in range(2018, 2024) if (gov, y) in prod))
    txt = "\n".join(lines)
    print(txt)
    with open(os.path.join(out_dir, "krso_comparison.txt"), "w") as fh:
        fh.write(txt + "\n")


# ----------------------------------------------------------------- field profiles
def pick_fields(sp, su, H, W, bbox, sp_prev=None):
    """find homogeneous 5x5 px blocks (~90 m at window resolution) for each signature.
    sp_prev = previous season's spring composite (to find fields cropped now but bare last season)."""
    best = {}
    for r0 in range(5, H - 10, 3):
        for c0 in range(5, W - 10, 3):
            a = [sp[(r0 + i) * W + c0 + j] for i in range(5) for j in range(5)]
            b = [su[(r0 + i) * W + c0 + j] for i in range(5) for j in range(5)]
            if any(v != v for v in a + b):
                continue
            ma, mb = sum(a) / 25, sum(b) / 25
            sd = math.sqrt(sum((v - ma) ** 2 for v in a) / 25) + math.sqrt(sum((v - mb) ** 2 for v in b) / 25)
            lab = None
            pv = [sp_prev[(r0 + i) * W + c0 + j] for i in range(5) for j in range(5)] if sp_prev else []
            if min(a) >= 0.6 and max(b) < 0.2 and pv and all(v == v for v in pv) and max(pv) < 0.22:
                lab = "E_crop_now_bare_prev"
            elif min(a) >= 0.6 and max(b) < 0.2 and pv and all(v == v for v in pv) and min(pv) >= 0.6:
                lab = "F_crop_both_years"
            elif min(a) >= 0.6 and max(b) < 0.2:
                lab = "A_crop_strong"
            elif max(a) < 0.2 and max(b) < 0.2:
                lab = "B_fallow_bare"
            elif min(b) >= 0.45 and min(a) >= 0.45:
                lab = "C_summer_green"
            elif 0.30 <= min(a) and max(a) < 0.45 and max(b) < 0.2:
                lab = "D_weak_green"
            if lab and (lab not in best or sd < best[lab][0]):
                lon = bbox[0] + (c0 + 2.5) * (bbox[2] - bbox[0]) / W
                lat = bbox[3] - (r0 + 2.5) * (bbox[3] - bbox[1]) / H
                best[lab] = (sd, round(lon, 5), round(lat, 5), round(ma, 3), round(mb, 3))
    return best


def season_dates(bbox, tile, year, step_days=20, max_cloud=30):
    feats = stac_search(bbox, f"{year - 1}-11-01", f"{year}-07-31", max_cloud)
    ms = [item_meta(f) for f in feats if (tile is None or f["properties"].get("s2:mgrs_tile") == tile)
          and contains(f["geometry"], bbox)]
    if tile is None and ms:  # choose the tile with most items
        cnt = {}
        for m in ms:
            cnt[m["tile"]] = cnt.get(m["tile"], 0) + 1
        tile = max(cnt, key=cnt.get)
        ms = [m for m in ms if m["tile"] == tile]
    start = dt.date(year - 1, 11, 1)
    bins = {}
    for m in ms:
        k = (dt.date.fromisoformat(m["date"]) - start).days // step_days
        if k not in bins or m["cloud"] < bins[k]["cloud"]:
            bins[k] = m
    return [bins[k] for k in sorted(bins)]


def point_ndvi(item, baseline, lon, lat, half_m=25):
    dlon = half_m / (111320 * math.cos(math.radians(lat)))
    dlat = half_m / 110570
    b = http(crop_url(item, (round(lon - dlon, 6), round(lat - dlat, 6), round(lon + dlon, 6), round(lat + dlat, 6))))
    H, W, nd, vf = scene_ndvi(b, baseline)
    v = [x for x in nd if x == x]
    return (sum(v) / len(v) if v and vf >= 0.5 else NAN), vf


def label_profile(prof):
    """prof: list of (date, ndvi). crude rule-based verdict."""
    def vals(m1, m2):
        return [v for d, v in prof if v == v and m1 <= d[5:7] <= m2]
    spring = vals("02", "05")
    summer = vals("07", "07") or vals("06", "06")
    if not spring:
        return "no clear spring observation"
    pk = max(spring)
    late = min(summer) if summer else NAN
    if late == late and late >= T_SUMMER:
        return f"summer-green (orchard/irrigated/perennial): peak {pk:.2f}, Jun-Jul {late:.2f}"
    if pk >= T_CROP:
        return f"winter crop established (peak {pk:.2f} then dry-down {late:.2f})"
    if pk < 0.30:
        return f"no crop canopy detected (peak {pk:.2f}): fallow, or sown-but-failed"
    return f"ambiguous weak green-up (peak {pk:.2f}): poor crop OR natural vegetation"


def cmd_profile(data_dir, out_dir, win="erbil_plain", seasons=(2025, 2026), pick_year=2026):
    os.makedirs(out_dir, exist_ok=True)
    idx = json.load(open(os.path.join(data_dir, "scenes_index.json")))
    bbox = tuple(idx["windows"][win]["bbox"])
    tile = idx["windows"][win]["tile"]
    shape, sp, su, _ = composites(data_dir, idx, win, pick_year)
    H, W = shape
    shp2, sp_prev, _, _ = composites(data_dir, idx, win, pick_year - 1)
    fields = pick_fields(sp, su, H, W, bbox, sp_prev if shp2 == shape else None)
    print("fields picked from", pick_year, "composite:")
    for k, v in sorted(fields.items()):
        print(f"  {k}: lon {v[1]} lat {v[2]}  spring {v[3]} summer {v[4]}")
    out = []
    for yr in seasons:
        dates = season_dates(bbox, tile, yr)
        jobs = [(lab, f, m) for lab, f in sorted(fields.items()) for m in dates]
        t0 = time.time()
        with cf.ThreadPoolExecutor(8) as ex:
            res = list(ex.map(lambda j: (j, point_ndvi(j[2]["id"], j[2]["baseline"], j[1][1], j[1][2])), jobs))
        print(f"season {yr}: {len(dates)} dates x {len(fields)} fields in {time.time() - t0:.1f}s")
        for (lab, f, m), (v, vf) in res:
            out.append({"season": f"{yr - 1}-{yr}", "field": lab, "lon": f[1], "lat": f[2], "date": m["date"],
                        "ndvi": round(v, 3) if v == v else "", "valid_frac": round(vf, 2), "item": m["id"]})
        for lab in sorted(fields):
            prof = [(r["date"], r["ndvi"] if r["ndvi"] != "" else NAN) for r in out
                    if r["field"] == lab and r["season"] == f"{yr - 1}-{yr}"]
            print(f"  {lab:15s} " + " ".join(f"{d[5:]}:{('%.2f' % v) if v == v else ' -- '}" for d, v in prof))
            print(f"  {'':15s} -> {label_profile(prof)}")
    with open(os.path.join(out_dir, "field_profiles.csv"), "w", newline="") as fh:
        w = csv.DictWriter(fh, fieldnames=list(out[0].keys()))
        w.writeheader()
        w.writerows(out)


def cmd_fieldcheck(lon, lat, year):
    t0 = time.time()
    bbox = (lon - 0.0003, lat - 0.0003, lon + 0.0003, lat + 0.0003)
    dates = season_dates(bbox, None, year)
    t1 = time.time()
    with cf.ThreadPoolExecutor(8) as ex:
        res = list(ex.map(lambda m: point_ndvi(m["id"], m["baseline"], lon, lat), dates))
    t2 = time.time()
    prof = [(m["date"], v) for m, (v, _) in zip(dates, res)]
    print(" ".join(f"{d[5:]}:{('%.2f' % v) if v == v else ' -- '}" for d, v in prof))
    print("verdict:", label_profile(prof))
    print(f"timing: STAC search {t1 - t0:.1f}s, {len(dates)} pixel crops (8 parallel) {t2 - t1:.1f}s, total {t2 - t0:.1f}s")


if __name__ == "__main__":
    a = sys.argv[1:]
    if not a:
        print(__doc__); sys.exit(1)
    if a[0] == "fetch":
        cmd_fetch(a[1])
    elif a[0] == "analyze":
        cmd_analyze(a[1], a[2], a[3])
    elif a[0] == "profile":
        cmd_profile(a[1], a[2], a[3] if len(a) > 3 else "erbil_plain",
                    tuple(int(x) for x in a[4].split(",")) if len(a) > 4 else (2025, 2026))
    elif a[0] == "fieldcheck":
        cmd_fieldcheck(float(a[1]), float(a[2]), int(a[3]))
    else:
        print(__doc__); sys.exit(1)
