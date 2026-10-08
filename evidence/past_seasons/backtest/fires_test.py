"""FIRMS fire-detection backtest for the Kurdistan Region (stdlib only).

Usage:
  python3 -I fires_test.py fetch   <fires_dir>                      # download FIRMS yearly Iraq CSVs (keyless)
  python3 -I fires_test.py nrt     <fires_dir>                      # download FIRMS near-real-time 7-day files (keyless)
  python3 -I fires_test.py landcover <fires_dir>                    # ESA WorldCover 2021 crop mask (~250 m) for northern Iraq
  python3 -I fires_test.py analyze <fires_dir> <adm1.geojson> <out_csv>
  python3 -I fires_test.py burned  <fires_dir> <adm1.geojson> <out_csv>   # MODIS MCD64A1 burned area via Planetary Computer
  python3 -I fires_test.py burned_crop <fires_dir> <adm1.geojson> <out_csv> 2019,2024   # same, split cropland / not
"""
import csv, json, os, sys, time, urllib.request, urllib.error, concurrent.futures as cf, datetime as dt

UA = {'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 '
                    '(KHTML, like Gecko) Chrome/129.0 Safari/537.36'}
BASE = 'https://firms.modaps.eosdis.nasa.gov/data/country/{s}/{y}/{s}_{y}_Iraq.csv'
# Index of what exists: https://firms.modaps.eosdis.nasa.gov/data/country/yearly_summary_files.txt
# (checked 2026-10-07: modis 2000-2024, viirs-snpp 2012-2024, viirs-jpss1 = NOAA-20 2018-2024; no 2025/2026, no NOAA-21)
SENSORS = {'modis': range(2000, 2027), 'viirs-snpp': range(2012, 2027), 'viirs-jpss1': range(2018, 2027)}
NRT = {  # keyless "last 7 days" files, global (filtered to Iraq bbox after download)
    'nrt_viirs_snpp_7d': 'https://firms.modaps.eosdis.nasa.gov/data/active_fire/suomi-npp-viirs-c2/csv/SUOMI_VIIRS_C2_Global_7d.csv',
    'nrt_viirs_noaa20_7d': 'https://firms.modaps.eosdis.nasa.gov/data/active_fire/noaa-20-viirs-c2/csv/J1_VIIRS_C2_Global_7d.csv',
    'nrt_viirs_noaa21_7d': 'https://firms.modaps.eosdis.nasa.gov/data/active_fire/noaa-21-viirs-c2/csv/J2_VIIRS_C2_Global_7d.csv',
    'nrt_modis_7d': 'https://firms.modaps.eosdis.nasa.gov/data/active_fire/modis-c6.1/csv/MODIS_C6_1_Global_7d.csv',
}
KRI = ['Dohuk', 'Erbil', 'Al-Sulaimaniyah']
REPORT = KRI + ['Kirkuk', 'Ninawa']
IRAQ_BBOX = (38.7, 29.0, 48.8, 37.5)  # lon_min, lat_min, lon_max, lat_max


def get(url, fn, timeout=600):
    """Download url -> fn. Returns (status, bytes, note)."""
    if os.path.exists(fn) and os.path.getsize(fn) > 1000:
        return 'cached', os.path.getsize(fn), ''
    for attempt in range(3):
        try:
            with urllib.request.urlopen(urllib.request.Request(url, headers=UA), timeout=timeout) as r:
                raw = r.read()
            if not raw.startswith(b'latitude'):
                return 'not_csv', len(raw), raw[:120].decode('utf8', 'replace')
            tmp = fn + '.part'
            open(tmp, 'wb').write(raw); os.replace(tmp, fn)
            return 'ok', len(raw), ''
        except urllib.error.HTTPError as e:
            return f'http_{e.code}', 0, str(e.reason)
        except Exception as e:  # network hiccup -> retry
            note = repr(e); time.sleep(3 * (attempt + 1))
    return 'error', 0, note


def cmd_fetch(d):
    jobs = [(s, y, BASE.format(s=s, y=y), os.path.join(d, f'{s}_{y}_Iraq.csv')) for s, ys in SENSORS.items() for y in ys]
    log = {}
    with cf.ThreadPoolExecutor(4) as ex:
        futs = {ex.submit(get, u, fn): (s, y, u) for s, y, u, fn in jobs}
        for f in cf.as_completed(futs):
            s, y, u = futs[f]; st, n, note = f.result()
            log[f'{s}/{y}'] = {'url': u, 'status': st, 'bytes': n, 'note': note}
            print(f'{s:13s} {y} {st:8s} {n:>10d} {note[:80]}', flush=True)
    json.dump(dict(sorted(log.items())), open(os.path.join(d, 'fetch_log.json'), 'w'), indent=1)


def cmd_nrt(d):
    log = {}
    for name, u in NRT.items():
        fn = os.path.join(d, name + '_global.csv')
        if os.path.exists(fn): os.remove(fn)  # always fresh
        st, n, note = get(u, fn)
        log[name] = {'url': u, 'status': st, 'bytes': n, 'note': note,
                     'downloaded_utc': dt.datetime.now(dt.timezone.utc).isoformat(timespec='seconds')}
        print(name, st, n, note[:80], flush=True)
        if st == 'ok':  # keep only the Iraq bbox, drop the big global file
            out = os.path.join(d, name + '_iraqbbox.csv')
            with open(fn, newline='') as fi, open(out, 'w', newline='') as fo:
                r = csv.DictReader(fi); w = csv.DictWriter(fo, r.fieldnames); w.writeheader()
                for row in r:
                    la, lo = float(row['latitude']), float(row['longitude'])
                    if IRAQ_BBOX[1] <= la <= IRAQ_BBOX[3] and IRAQ_BBOX[0] <= lo <= IRAQ_BBOX[2]: w.writerow(row)
            os.remove(fn)
    json.dump(log, open(os.path.join(d, 'nrt_log.json'), 'w'), indent=1)


# ---------------- land cover (ESA WorldCover 2021, 10 m -> ~250 m mode, via Planetary Computer, keyless) -------------
WC_TILES = [(lon, lat) for lat in (33, 36) for lon in (39, 42, 45)]  # 3x3 deg tiles covering 39-48E, 33-39N
WC_N = 1200  # pixels per 3 deg -> 0.0025 deg (~250 m)
WC_URL = ('https://planetarycomputer.microsoft.com/api/data/v1/item/bbox/{x0},{y0},{x1},{y1}/{n}x{n}.npy'
          '?collection=esa-worldcover&item=ESA_WorldCover_10m_2021_v200_N{lat:02d}E{lon:03d}&assets=map&resampling=mode')


def cmd_landcover(d):
    os.makedirs(os.path.join(d, 'worldcover'), exist_ok=True)
    for lon, lat in WC_TILES:
        fn = os.path.join(d, 'worldcover', f'wc2021_N{lat}E{lon:03d}_{WC_N}.npy')
        if os.path.exists(fn): continue
        u = WC_URL.format(x0=lon, y0=lat, x1=lon + 3, y1=lat + 3, n=WC_N, lat=lat, lon=lon)
        raw = urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=300).read()
        assert raw[:6] == b'\x93NUMPY', raw[:200]
        open(fn, 'wb').write(raw); print('ok', fn, len(raw))


def load_landcover(d):
    """Returns f(lon, lat) -> (class at point, n cropland cells in 3x3 window). Classes: 10 tree, 20 shrub,
    30 grass, 40 cropland, 50 built, 60 bare, 80 water, 90 wetland. None if no land-cover files."""
    tiles = {}
    wd = os.path.join(d, 'worldcover')
    if not os.path.isdir(wd): return None
    for lon, lat in WC_TILES:
        fn = os.path.join(wd, f'wc2021_N{lat}E{lon:03d}_{WC_N}.npy')
        if not os.path.exists(fn): continue
        raw = open(fn, 'rb').read(); hl = int.from_bytes(raw[8:10], 'little'); body = raw[10 + hl:]
        tiles[(lon, lat)] = body[:WC_N * WC_N]  # band 1 (class), band 2 is the mask
    step = 3 / WC_N

    def cls(x, y):
        k = (int(x // 3) * 3, int(y // 3) * 3); b = tiles.get(k)
        if b is None: return None
        r = min(WC_N - 1, int((k[1] + 3 - y) / step)); c = min(WC_N - 1, int((x - k[0]) / step))
        return b[r * WC_N + c]

    def f(x, y):
        nb = [cls(x + dx * step, y + dy * step) for dx in (-1, 0, 1) for dy in (-1, 0, 1)]
        return cls(x, y), sum(1 for v in nb if v == 40)
    return f


# ---------------- geometry ----------------
def load_polys(path):
    out = []
    for f in json.load(open(path))['features']:
        g = f['geometry']
        polys = [g['coordinates']] if g['type'] == 'Polygon' else g['coordinates']
        rings = [[(float(x), float(y)) for x, y, *_ in poly[0]] for poly in polys]  # outer rings
        holes = [[(float(x), float(y)) for x, y, *_ in r] for poly in polys for r in poly[1:]]
        xs = [p[0] for r in rings for p in r]; ys = [p[1] for r in rings for p in r]
        out.append({'name': f['properties']['shapeName'], 'rings': rings, 'holes': holes,
                    'bbox': (min(xs), min(ys), max(xs), max(ys))})
    return out


def in_ring(x, y, ring):
    inside = False; j = len(ring) - 1
    for i in range(len(ring)):
        xi, yi = ring[i]; xj, yj = ring[j]
        if (yi > y) != (yj > y) and x < (xj - xi) * (y - yi) / (yj - yi) + xi:
            inside = not inside
        j = i
    return inside


def locate(x, y, polys, cache):
    k = (round(x, 4), round(y, 4))  # ~10 m grid cache; VIIRS pixels repeat often (flares)
    if k in cache: return cache[k]
    hit = None
    for p in polys:
        b = p['bbox']
        if b[0] <= x <= b[2] and b[1] <= y <= b[3] and any(in_ring(x, y, r) for r in p['rings']) \
                and not any(in_ring(x, y, h) for h in p['holes']):
            hit = p['name']; break
    cache[k] = hit
    return hit


def season(date):
    return 'harvest' if 5 <= int(date[5:7]) <= 7 else 'rest'


def cmd_analyze(d, geo, out_csv):
    polys = load_polys(geo); cache = {}; lcf = load_landcover(d)
    lcc = {}      # (sensor, year, gov, season, 'cropland'|'natural'|'other') -> n non-static detections
    counts = {}   # (sensor, year, gov, season, kind) -> n
    frp = {}      # (sensor, year, gov, season) -> sum frp of veg fires
    daily = {}    # (sensor, gov, date) -> n non-static (for 2019 / 2024 checks)
    persistent = {}  # (sensor, round(lat,2), round(lon,2), year) -> set(dates) of vegetation-typed detections
    files = sorted(f for f in os.listdir(d) if f.endswith('_Iraq.csv'))
    for fn in files:
        sensor = fn.split('_')[0]
        with open(os.path.join(d, fn), newline='') as fh:
            for row in csv.DictReader(fh):
                la, lo = float(row['latitude']), float(row['longitude'])
                if not (34.0 <= la <= 37.5 and 41.0 <= lo <= 46.5):  # northern Iraq only (speed)
                    continue
                gov = locate(lo, la, polys, cache)
                if gov not in REPORT: continue
                date = row['acq_date']; y = int(date[:4]); s = season(date)
                t = row.get('type', '').strip()
                conf = row.get('confidence', '').strip().lower()
                low = conf in ('l', 'low') or (conf.isdigit() and int(conf) < 30)
                kind = 'static' if t in ('2', '3', '1') else ('veg_lowconf' if low else 'veg')
                counts[(sensor, y, gov, s, kind)] = counts.get((sensor, y, gov, s, kind), 0) + 1
                if kind != 'static':
                    frp[(sensor, y, gov, s)] = frp.get((sensor, y, gov, s), 0.0) + float(row['frp'] or 0)
                    daily[(sensor, gov, date)] = daily.get((sensor, gov, date), 0) + 1
                    if lcf:
                        c0, ncrop = lcf(lo, la)
                        grp = 'cropland' if (c0 == 40 or ncrop >= 3) else ('natural' if c0 in (10, 20, 30) else 'other')
                        lcc[(sensor, y, gov, s, grp)] = lcc.get((sensor, y, gov, s, grp), 0) + 1
                    pk = (sensor, round(la, 2), round(lo, 2), y)  # ~1 km cell per year
                    persistent.setdefault(pk, set()).add(date)
    # Year-round burners mislabelled as vegetation (new gas flares, refineries, dumps, brick kilns): a ~1 km cell that
    # in some year burns on >=10 distinct days spread over >=4 calendar months. A crop field burns on 1-3 days.
    pers_cells = {(sn, la, lo) for (sn, la, lo, y), ds in persistent.items()
                  if len(ds) >= 10 and len({x[5:7] for x in ds}) >= 4}
    print(f'~1 km cells typed vegetation but burning year-round (excluded as non-crop): {len(pers_cells)}')
    pers_counts = {}
    for fn in files:
        sensor = fn.split('_')[0]
        with open(os.path.join(d, fn), newline='') as fh:
            for row in csv.DictReader(fh):
                la, lo = float(row['latitude']), float(row['longitude'])
                if not (34.0 <= la <= 37.5 and 41.0 <= lo <= 46.5): continue
                if row.get('type', '').strip() in ('1', '2', '3'): continue
                if (sensor, round(la, 2), round(lo, 2)) not in pers_cells: continue
                gov = locate(lo, la, polys, cache)
                if gov not in REPORT: continue
                k = (sensor, int(row['acq_date'][:4]), gov, season(row['acq_date']))
                pers_counts[k] = pers_counts.get(k, 0) + 1
                if lcf:  # keep the land-cover split consistent with crop_fire_estimate
                    c0, ncrop = lcf(lo, la)
                    grp = 'cropland' if (c0 == 40 or ncrop >= 3) else ('natural' if c0 in (10, 20, 30) else 'other')
                    lcc[k + (grp,)] -= 1
                if 5 <= int(row['acq_date'][5:7]) <= 7 and row['acq_date'][:4] in ('2019', '2024'):
                    dk = (sensor, gov, row['acq_date']); daily[dk] = daily.get(dk, 0) - 1
    sensors = sorted({k[0] for k in counts}); years = sorted({k[1] for k in counts})
    rows = []
    for sn in sensors:
        for y in years:
            for g in REPORT:
                for s in ('harvest', 'rest'):
                    c = lambda kind: counts.get((sn, y, g, s, kind), 0)
                    if not any(c(k) for k in ('veg', 'veg_lowconf', 'static')) and not os.path.exists(
                            os.path.join(d, f'{sn}_{y}_Iraq.csv')): continue
                    veg = c('veg') + c('veg_lowconf'); pers = pers_counts.get((sn, y, g, s), 0)
                    rows.append({'sensor': sn, 'year': y, 'governorate': g, 'season': s,
                                 'veg_detections': veg, 'veg_highnom_conf': c('veg'),
                                 'static_type2_excluded': c('static'),
                                 'persistent_cells_in_veg': pers, 'crop_fire_estimate': veg - pers,
                                 'frp_sum_mw': round(frp.get((sn, y, g, s), 0.0), 1),
                                 'on_cropland': lcc.get((sn, y, g, s, 'cropland'), ''),
                                 'on_tree_shrub_grass': lcc.get((sn, y, g, s, 'natural'), ''),
                                 'on_other_landcover': lcc.get((sn, y, g, s, 'other'), '')})
    with open(out_csv, 'w', newline='') as fo:
        w = csv.DictWriter(fo, list(rows[0].keys())); w.writeheader(); w.writerows(rows)
    # daily series for 2019 and 2024 harvest seasons (crop-fire estimate, any confidence)
    dpath = os.path.splitext(out_csv)[0] + '_daily_2019_2024.csv'
    with open(dpath, 'w', newline='') as fo:
        w = csv.writer(fo); w.writerow(['sensor', 'governorate', 'date', 'crop_fire_estimate'])
        for (sn, g, date), n in sorted(daily.items()):
            if date[:4] in ('2019', '2024') and 5 <= int(date[5:7]) <= 7: w.writerow([sn, g, date, n])
    print('wrote', out_csv, dpath)
    # console table: harvest-season crop_fire_estimate per gov per year, per sensor
    for sn in sensors:
        print(f'\n== {sn}: harvest season (May-Jul) crop-fire estimate [type 0, any confidence, minus year-round cells] ==')
        print('year  ' + ' '.join(f'{g[:8]:>9s}' for g in REPORT) + '   KRI-3  | rest-of-yr KRI-3')
        for y in years:
            r = {x['governorate']: x for x in rows if x['sensor'] == sn and x['year'] == y and x['season'] == 'harvest'}
            rr = {x['governorate']: x for x in rows if x['sensor'] == sn and x['year'] == y and x['season'] == 'rest'}
            if not r: continue
            vals = [r[g]['crop_fire_estimate'] if g in r else 0 for g in REPORT]
            kri = sum(r[g]['crop_fire_estimate'] for g in KRI if g in r)
            rk = sum(rr[g]['crop_fire_estimate'] for g in KRI if g in rr)
            print(f'{y}  ' + ' '.join(f'{v:9d}' for v in vals) + f'   {kri:6d}  | {rk:6d}')
        if lcf:
            print(f'-- {sn}: same, only detections on WorldCover cropland (point or >=3/9 neighbours ~250 m) --')
            for y in years:
                r = {x['governorate']: x for x in rows if x['sensor'] == sn and x['year'] == y and x['season'] == 'harvest'}
                rr = {x['governorate']: x for x in rows if x['sensor'] == sn and x['year'] == y and x['season'] == 'rest'}
                if not r: continue
                vals = [r[g]['on_cropland'] or 0 for g in REPORT]
                kri = sum(r[g]['on_cropland'] or 0 for g in KRI if g in r)
                rk = sum(rr[g]['on_cropland'] or 0 for g in KRI if g in rr)
                print(f'{y}  ' + ' '.join(f'{v:9d}' for v in vals) + f'   {kri:6d}  | {rk:6d}')


# ---------------- burned area (MODIS MCD64A1 via Planetary Computer) ----------------
PC_SEARCH = 'https://planetarycomputer.microsoft.com/api/stac/v1/search'
PC_STATS = 'https://planetarycomputer.microsoft.com/api/data/v1/item/statistics'


def post_json(url, body, timeout=300):
    req = urllib.request.Request(url, data=json.dumps(body).encode(), method='POST',
                                 headers={**UA, 'Content-Type': 'application/json', 'Accept': 'application/json'})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.loads(r.read())


def ring_area_km2(ring):
    """Spherical polygon area (km2) of a lon/lat ring."""
    import math
    R = 6371.0088; a = 0.0
    for (x1, y1), (x2, y2) in zip(ring, ring[1:] + ring[:1]):
        a += math.radians(x2 - x1) * (2 + math.sin(math.radians(y1)) + math.sin(math.radians(y2)))
    return abs(a) * R * R / 2


def cmd_burned(d, geo, out_csv, y0=2001, y1=2026):
    """MODIS MCD64A1 (500 m monthly burned area) for May/Jun/Jul per governorate, via Planetary Computer (keyless).
    Burned km2 = (burned pixels / valid pixels in polygon) x polygon area (robust to the server's reprojection)."""
    feats = {f['properties']['shapeName']: f for f in json.load(open(geo))['features']}
    area = {g: sum(ring_area_km2([(float(x), float(y)) for x, y, *_ in poly[0]])
                   for poly in ([feats[g]['geometry']['coordinates']] if feats[g]['geometry']['type'] == 'Polygon'
                                else feats[g]['geometry']['coordinates'])) for g in REPORT}
    print('polygon areas km2:', {g: round(a) for g, a in area.items()})
    items = {}  # (year, month) -> item id (tile h21v05 covers all of northern Iraq)
    for y in range(y0, y1 + 1):
        body = {'collections': ['modis-64A1-061'], 'bbox': [42.0, 35.0, 45.0, 37.0],
                'datetime': f'{y}-05-01T00:00:00Z/{y}-07-31T23:59:59Z', 'limit': 50}
        for f in post_json(PC_SEARCH, body).get('features', []):
            if '.h21v05.' in f['id']:
                items[(y, int(f['properties']['start_datetime'][5:7]))] = f['id']
    missing = [(y, m) for y in range(y0, y1 + 1) for m in (5, 6, 7) if (y, m) not in items]
    print('missing months on Planetary Computer:', missing)
    jobs = [(y, m, g, iid) for (y, m), iid in sorted(items.items()) for g in REPORT]

    def one(job):
        y, m, g, iid = job
        u = f'{PC_STATS}?collection=modis-64A1-061&item={iid}&assets=Burn_Date&categorical=true'
        feat = {'type': 'Feature', 'properties': {}, 'geometry': feats[g]['geometry']}
        for attempt in range(3):
            try:
                st = post_json(u, feat)['properties']['statistics']['Burn_Date_b1']
                cnt, vals = st['histogram']
                h = {int(v): c for c, v in zip(cnt, vals)}
                burned = sum(c for v, c in h.items() if v > 0)
                valid = st['valid_pixels']
                mapped = valid - h.get(-1, 0) - h.get(-2, 0)
                return job, burned, valid, mapped, ''
            except Exception as e:
                err = repr(e)[:200]; time.sleep(5 * (attempt + 1))
        return job, None, None, None, err

    res = {}
    with cf.ThreadPoolExecutor(6) as ex:
        for job, burned, valid, mapped, err in ex.map(one, jobs):
            res[job[:3]] = (burned, valid, mapped, err, job[3])
            if err: print('FAIL', job, err, flush=True)
    rows = []
    for (y, m, g), (burned, valid, mapped, err, iid) in sorted(res.items()):
        km2 = burned / valid * area[g] if burned is not None and valid else None
        rows.append({'year': y, 'month': m, 'governorate': g, 'item': iid, 'burned_pixels': burned,
                     'valid_pixels': valid, 'unmapped_or_water_pixels': None if valid is None else valid - mapped,
                     'burned_km2': None if km2 is None else round(km2, 2),
                     'burned_dunam': None if km2 is None else round(km2 * 400), 'error': err})
    with open(out_csv, 'w', newline='') as fo:
        w = csv.DictWriter(fo, list(rows[0].keys())); w.writeheader(); w.writerows(rows)
    print('wrote', out_csv)
    print('\nMCD64A1 burned area May-Jul, km2 (1 km2 = 400 dunam)')
    print('year  ' + ' '.join(f'{g[:8]:>9s}' for g in REPORT) + '   months')
    for y in range(y0, y1 + 1):
        ms = sorted({r['month'] for r in rows if r['year'] == y})
        if not ms: continue
        vals = [sum(r['burned_km2'] or 0 for r in rows if r['year'] == y and r['governorate'] == g) for g in REPORT]
        print(f'{y}  ' + ' '.join(f'{v:9.1f}' for v in vals) + f'   {ms}')


def cmd_burned_crop(d, geo, out_csv, years):
    """MCD64A1 burned pixels (May-Jul) split by WorldCover cropland vs not, per governorate.
    Reads the Burn_Date raster for the northern-Iraq box at 0.0025 deg via the data API (.npy), tests only burned pixels."""
    import math
    polys = load_polys(geo); cache = {}; lcf = load_landcover(d)
    x0, y0, x1, y1, st = 41.0, 34.0, 46.5, 37.5, 0.0025
    W, H = round((x1 - x0) / st), round((y1 - y0) / st)
    out = []
    os.makedirs(os.path.join(d, 'mcd64a1'), exist_ok=True)
    for y in years:
        body = {'collections': ['modis-64A1-061'], 'bbox': [42.0, 35.0, 45.0, 37.0],
                'datetime': f'{y}-05-01T00:00:00Z/{y}-07-31T23:59:59Z', 'limit': 50}
        items = {int(f['properties']['start_datetime'][5:7]): f['id'] for f in post_json(PC_SEARCH, body)['features']
                 if '.h21v05.' in f['id']}
        for m, iid in sorted(items.items()):
            fn = os.path.join(d, 'mcd64a1', f'{iid}_burndate_{W}x{H}.npy')
            if not os.path.exists(fn):
                u = (f'https://planetarycomputer.microsoft.com/api/data/v1/item/bbox/{x0},{y0},{x1},{y1}/{W}x{H}.npy'
                     f'?collection=modis-64A1-061&item={iid}&assets=Burn_Date&resampling=nearest')
                raw = urllib.request.urlopen(urllib.request.Request(u, headers=UA), timeout=600).read()
                open(fn, 'wb').write(raw)
            raw = open(fn, 'rb').read(); hl = int.from_bytes(raw[8:10], 'little')
            hdr = raw[10:10 + hl].decode(); body_b = raw[10 + hl:]
            assert "'<i2'" in hdr, hdr
            vals = memoryview(body_b).cast('h')[:W * H]
            acc = {}
            for i, v in enumerate(vals):
                if v <= 0: continue
                r, c = divmod(i, W)
                lat = y1 - (r + 0.5) * st; lon = x0 + (c + 0.5) * st
                g = locate(lon, lat, polys, cache)
                if g not in REPORT: continue
                c0, ncrop = lcf(lon, lat)
                grp = 'cropland' if c0 == 40 else ('natural' if c0 in (10, 20, 30) else 'other')
                a = (st * 111.32 * math.cos(math.radians(lat))) * (st * 110.57)
                acc[(g, grp)] = acc.get((g, grp), 0.0) + a
            for g in REPORT:
                out.append({'year': y, 'month': m, 'governorate': g,
                            **{f'burned_km2_{k}': round(acc.get((g, k), 0.0), 2) for k in ('cropland', 'natural', 'other')}})
            print(y, m, {g: round(acc.get((g, 'cropland'), 0), 1) for g in REPORT}, flush=True)
    with open(out_csv, 'w', newline='') as fo:
        w = csv.DictWriter(fo, list(out[0].keys())); w.writeheader(); w.writerows(out)
    print('wrote', out_csv)


if __name__ == '__main__':
    c = sys.argv[1]
    if c == 'fetch': cmd_fetch(sys.argv[2])
    elif c == 'nrt': cmd_nrt(sys.argv[2])
    elif c == 'landcover': cmd_landcover(sys.argv[2])
    elif c == 'analyze': cmd_analyze(sys.argv[2], sys.argv[3], sys.argv[4])
    elif c == 'burned': cmd_burned(sys.argv[2], sys.argv[3], sys.argv[4])
    elif c == 'burned_crop': cmd_burned_crop(sys.argv[2], sys.argv[3], sys.argv[4], [int(x) for x in sys.argv[5].split(',')])
    else: raise SystemExit(__doc__)
