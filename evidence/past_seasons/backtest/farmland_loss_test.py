#!/usr/bin/env python3
"""Farmland-loss test (SmartSuli AI Challenge).

Question: can free satellite land-cover data show how much farmland the cities of
Erbil, Slemani and Duhok built over, year by year?

Data: Impact Observatory annual 10 m LULC v02 (2017-2023), collection "io-lulc-annual-v02",
      cross-check with ESA WorldCover (2020 v100, 2021 v200), both via Microsoft
      Planetary Computer (no key needed).

Method
  * Ring = circle of radius R around each city centre, defined in UTM 38N (EPSG:32638,
    the native grid of the IO tiles; all three cities sit in tile 38S).
  * Areas: (a) native 10 m rasters downloaded as PNG through the data API bbox endpoint
    (grid-aligned, no resampling), decoded with stdlib zlib, circle-masked locally;
    (b) the data API /item/statistics endpoint (categorical) with the same circle,
    timed, as the "live demo" query.
  * Pixel-level transitions year-to-year and 2017->2023 from (a).

Usage (stdlib only, Python 3.14):
  python3 -I farmland_loss_test.py --data-dir <raw download folder> --out-dir <output folder>
         [--offline]   # reuse cached rasters/JSON, no network
"""
import argparse
import csv
import json
import math
import os
import struct
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
import zlib
from collections import Counter
from concurrent.futures import ThreadPoolExecutor

UA = ('Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 '
      '(KHTML, like Gecko) Chrome/128.0 Safari/537.36')
STAC = 'https://planetarycomputer.microsoft.com/api/stac/v1'
DATA = 'https://planetarycomputer.microsoft.com/api/data/v1'
IO, WC = 'io-lulc-annual-v02', 'esa-worldcover'
YEARS = list(range(2017, 2024))
# name, lat, lon, ring radius km
CITIES = [('Erbil', 36.19, 44.01, 25), ('Slemani', 35.56, 45.43, 20), ('Duhok', 36.87, 42.99, 20)]
ZONE, EPSG, RES = 38, 'epsg:32638', 10            # IO native grid: 10 m, origin multiple of 10
PIX_KM2 = RES * RES / 1e6
DUNAM_PER_KM2 = 1e6 / 2500.0                       # 1 dunam = 2,500 m2 (Iraqi dunam)
BUILT, CROPS, RANGE, BARE = 7, 5, 11, 8

LOG = []


def log(*a):
    s = ' '.join(str(x) for x in a)
    print(s, flush=True)
    LOG.append(s)


# ---------------------------------------------------------------- HTTP
def http(url, body=None, timeout=300, tries=3):
    hdr = {'User-Agent': UA, 'Accept': '*/*'}
    data = None
    if body is not None:
        data = json.dumps(body).encode()
        hdr['Content-Type'] = 'application/json'
    last = None
    for i in range(tries):
        t = time.time()
        try:
            with urllib.request.urlopen(urllib.request.Request(url, data=data, headers=hdr),
                                        timeout=timeout) as r:
                b = r.read()
            return b, time.time() - t
        except urllib.error.HTTPError as e:
            last = f'HTTP {e.code}: {e.read()[:300]!r}'
            if e.code < 500 and e.code != 429:
                break
        except Exception as e:  # noqa: BLE001 - report any network failure verbatim
            last = f'{type(e).__name__}: {e}'
        time.sleep(3 * (i + 1))
    raise RuntimeError(f'{url[:200]} -> {last}')


def cached_json(path, url, body=None, offline=False):
    if os.path.exists(path) and (offline or body is None):
        with open(path) as f:
            return json.load(f)
    if offline:
        raise RuntimeError(f'offline and missing {path}')
    b, _ = http(url, body)
    with open(path, 'wb') as f:
        f.write(b)
    return json.loads(b)


# ---------------------------------------------------------------- geometry
def ll2utm(lat, lon, zone=ZONE):
    """WGS84 -> UTM northern hemisphere (Snyder series), metres."""
    a, f, k0 = 6378137.0, 1 / 298.257223563, 0.9996
    e2 = f * (2 - f)
    ep2 = e2 / (1 - e2)
    lon0 = math.radians((zone - 1) * 6 - 180 + 3)
    phi, lam = math.radians(lat), math.radians(lon)
    n = a / math.sqrt(1 - e2 * math.sin(phi) ** 2)
    t = math.tan(phi) ** 2
    c = ep2 * math.cos(phi) ** 2
    aa = math.cos(phi) * (lam - lon0)
    m = a * ((1 - e2 / 4 - 3 * e2 ** 2 / 64 - 5 * e2 ** 3 / 256) * phi
             - (3 * e2 / 8 + 3 * e2 ** 2 / 32 + 45 * e2 ** 3 / 1024) * math.sin(2 * phi)
             + (15 * e2 ** 2 / 256 + 45 * e2 ** 3 / 1024) * math.sin(4 * phi)
             - (35 * e2 ** 3 / 3072) * math.sin(6 * phi))
    x = k0 * n * (aa + (1 - t + c) * aa ** 3 / 6
                  + (5 - 18 * t + t * t + 72 * c - 58 * ep2) * aa ** 5 / 120) + 500000
    y = k0 * (m + n * math.tan(phi) * (aa * aa / 2 + (5 - t + 9 * c + 4 * c * c) * aa ** 4 / 24
                                        + (61 - 58 * t + t * t + 600 * c - 330 * ep2) * aa ** 6 / 720))
    return x, y


def city_geom(lat, lon, r_km):
    x, y = ll2utm(lat, lon)
    x0, y0 = round(x / RES) * RES, round(y / RES) * RES   # snap centre to the 10 m grid
    r = int(r_km * 1000)
    nv = 256                                              # divisible by 4 -> bounds = x0+-R, y0+-R
    ring = [[x0 + r * math.sin(2 * math.pi * i / nv), y0 + r * math.cos(2 * math.pi * i / nv)]
            for i in range(nv)]
    ring.append(ring[0])
    feat = {'type': 'Feature', 'properties': {}, 'geometry': {'type': 'Polygon', 'coordinates': [ring]}}
    return x0, y0, r, feat


# ---------------------------------------------------------------- PNG decode (8-bit gray)
def decode_png_gray8(b):
    if b[:8] != b'\x89PNG\r\n\x1a\n':
        raise ValueError('not a PNG: ' + repr(b[:200]))
    i, idat, w, h = 8, [], None, None
    while i < len(b):
        ln, = struct.unpack('>I', b[i:i + 4])
        typ, data = b[i + 4:i + 8], b[i + 8:i + 8 + ln]
        i += 12 + ln
        if typ == b'IHDR':
            w, h, bd, ct, _cm, _fm, il = struct.unpack('>IIBBBBB', data)
            if not (bd == 8 and ct == 0 and il == 0):
                raise ValueError(f'unsupported PNG bitdepth={bd} colortype={ct} interlace={il}')
        elif typ == b'IDAT':
            idat.append(data)
        elif typ == b'IEND':
            break
    raw = zlib.decompress(b''.join(idat))
    out, prev, stride = bytearray(w * h), bytes(w), w + 1
    for r in range(h):
        ft, row = raw[r * stride], raw[r * stride + 1:(r + 1) * stride]
        if ft == 0:
            cur = row
        elif ft == 2:
            cur = bytes([(x + p) & 255 for x, p in zip(row, prev)])
        else:
            c = bytearray(row)
            if ft == 1:
                for k in range(1, w):
                    c[k] = (c[k] + c[k - 1]) & 255
            elif ft == 3:
                c[0] = (c[0] + (prev[0] >> 1)) & 255
                for k in range(1, w):
                    c[k] = (c[k] + ((c[k - 1] + prev[k]) >> 1)) & 255
            elif ft == 4:
                c[0] = (c[0] + prev[0]) & 255
                for k in range(1, w):
                    a, bb, cc = c[k - 1], prev[k], prev[k - 1]
                    p = a + bb - cc
                    pa, pb, pc = abs(p - a), abs(p - bb), abs(p - cc)
                    c[k] = (c[k] + (a if (pa <= pb and pa <= pc) else (bb if pb <= pc else cc))) & 255
            else:
                raise ValueError(f'bad PNG filter {ft}')
            cur = bytes(c)
        out[r * w:(r + 1) * w] = cur
        prev = cur
    return w, h, bytes(out)


# ---------------------------------------------------------------- raster ops (stdlib, whole-buffer)
def circle_mask(px, n, minx, maxy, x0, y0, r):
    """Zero everything whose pixel centre is outside the circle. Returns (masked, n_in_circle)."""
    out, nin = bytearray(n * n), 0
    for row in range(n):
        dy = (maxy - RES * (row + 0.5)) - y0
        if abs(dy) > r:
            continue
        dx = math.sqrt(r * r - dy * dy)
        c0 = max(0, math.ceil((x0 - dx - minx) / RES - 0.5))
        c1 = min(n - 1, math.floor((x0 + dx - minx) / RES - 0.5))
        if c1 >= c0:
            out[row * n + c0:row * n + c1 + 1] = px[row * n + c0:row * n + c1 + 1]
            nin += c1 - c0 + 1
    return bytes(out), nin


SHL4 = bytes(((v << 4) & 0xFF) for v in range(256))


def pair_counts(a, b):
    """Transition matrix {(class_a, class_b): pixels}; both buffers hold codes < 16."""
    s = (int.from_bytes(a.translate(SHL4), 'big') + int.from_bytes(b, 'big')).to_bytes(len(a), 'big')
    return {(k >> 4, k & 15): v for k, v in Counter(s).items()}


def history(imgs, code):
    """Per-pixel 7-bit history: bit k set if pixel == code in YEARS[k]."""
    acc = 0
    for k, y in enumerate(YEARS):
        tab = bytes((1 << k) if v == code else 0 for v in range(256))
        acc += int.from_bytes(imgs[y].translate(tab), 'big')
    return acc.to_bytes(len(imgs[YEARS[0]]), 'big')


MONO = {sum(1 << j for j in range(k, len(YEARS))): k for k in range(len(YEARS))}  # built from YEARS[k] on


def km2(px):
    return px * PIX_KM2


# ---------------------------------------------------------------- main
def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--data-dir', required=True, help='raw downloads (PNG rasters, API JSON)')
    ap.add_argument('--out-dir', required=True, help='CSV/summary outputs')
    ap.add_argument('--offline', action='store_true', help='use cached files only')
    ap.add_argument('--workers', type=int, default=4)
    args = ap.parse_args()
    dd, od = args.data_dir, args.out_dir
    os.makedirs(dd, exist_ok=True)
    os.makedirs(od, exist_ok=True)
    failures = []

    # ---- metadata: class codes + items
    coll = cached_json(os.path.join(dd, 'stac_collection_io-lulc-annual-v02.json'),
                       f'{STAC}/collections/{IO}', offline=args.offline)
    names = {}
    for fv in coll['item_assets']['data']['file:values']:
        for v in fv['values']:
            names[v] = fv['summary']
    log('IO LULC class codes:', names)
    assert names.get(BUILT) == 'Built area' and names.get(CROPS) == 'Crops' and names.get(RANGE) == 'Rangeland'
    io_items = cached_json(os.path.join(dd, 'stac_search_io.json'), f'{STAC}/search',
                           {'collections': [IO], 'bbox': [42.5, 35.2, 46.0, 37.3], 'limit': 50},
                           offline=args.offline)
    item_by_year = {int(f['properties']['start_datetime'][:4]): f['id'] for f in io_items['features']}
    log('IO items:', item_by_year)
    for f in io_items['features']:
        assert f['properties']['proj:epsg'] == 32638 and f['properties']['proj:transform'][0] == RES
    wc_coll = cached_json(os.path.join(dd, 'stac_collection_esa-worldcover.json'),
                          f'{STAC}/collections/{WC}', offline=args.offline)
    wc_names = {c['value']: c['description'] for c in wc_coll['item_assets']['map']['classification:classes']}
    wc_items = cached_json(os.path.join(dd, 'stac_search_worldcover.json'), f'{STAC}/search',
                           {'collections': [WC], 'bbox': [42.5, 35.2, 46.0, 37.3], 'limit': 50},
                           offline=args.offline)

    geoms = {c[0]: city_geom(c[1], c[2], c[3]) for c in CITIES}

    # ---- 1) native 10 m rasters (PNG, grid-aligned bbox, no resampling)
    def raster_job(city, year):
        x0, y0, r, _ = geoms[city]
        n = 2 * r // RES
        path = os.path.join(dd, f'io_lulc_{city}_{year}_10m_r{r // 1000}km.png')
        if not os.path.exists(path):
            if args.offline:
                raise RuntimeError(f'offline and missing {path}')
            url = (f'{DATA}/item/bbox/{x0 - r},{y0 - r},{x0 + r},{y0 + r}/{n}x{n}.png?'
                   + urllib.parse.urlencode({'collection': IO, 'item': item_by_year[year], 'assets': 'data',
                                             'coord_crs': EPSG, 'dst_crs': EPSG, 'return_mask': 'false'}))
            b, dt = http(url)
            with open(path, 'wb') as f:
                f.write(b)
            return city, year, path, dt, len(b)
        return city, year, path, None, os.path.getsize(path)

    t0 = time.time()
    rasters = {}
    with ThreadPoolExecutor(args.workers) as ex:
        futs = [ex.submit(raster_job, c[0], y) for c in CITIES for y in YEARS]
        for fu in futs:
            try:
                city, year, path, dt, size = fu.result()
                rasters[(city, year)] = path
                log(f'raster {city} {year}: {size / 1e6:.2f} MB' + (f' in {dt:.1f}s' if dt else ' (cached)'))
            except Exception as e:  # noqa: BLE001
                failures.append(f'raster download: {e}')
                log('FAIL', e)
    log(f'raster stage {time.time() - t0:.0f}s')

    # ---- 2) statistics endpoint (the live-demo query), sequential so timings are honest
    def stats(coll_id, item, asset, feat, classes, max_size, tag):
        path = os.path.join(dd, f'stats_{tag}.json')
        q = [('collection', coll_id), ('item', item), ('assets', asset), ('categorical', 'true'),
             ('coord_crs', EPSG), ('dst_crs', EPSG), ('max_size', str(max_size))] + [('c', str(c)) for c in classes]
        url = f'{DATA}/item/statistics?' + urllib.parse.urlencode(q)
        dt = None
        if args.offline or (os.path.exists(path) and tag.startswith('wc_')):
            with open(path) as f:
                d = json.load(f)
        else:
            b, dt = http(url, feat)
            with open(path, 'wb') as f:
                f.write(b)
            d = json.loads(b)
        st = next(iter(d['properties']['statistics'].values()))
        counts, vals = st['histogram']
        return {int(v): int(c) for c, v in zip(counts, vals)}, dt

    api = {}
    for city, _, _, rk in CITIES:
        x0, y0, r, feat = geoms[city]
        native = 2 * r // RES
        for year in YEARS:
            row = {}
            try:
                cnt, dt = stats(IO, item_by_year[year], 'data', feat, sorted(names), native,
                                f'io_{city}_{year}_native')
                px_m = 2 * r / native
                row.update({f'api_{k}': cnt.get(k, 0) * px_m * px_m / 1e6 for k in names})
                row['api_s_native'] = dt
                cnt2, dt2 = stats(IO, item_by_year[year], 'data', feat, sorted(names), 512,
                                  f'io_{city}_{year}_512')
                tot2 = sum(v for k, v in cnt2.items() if k != 0)
                row['api512_built_km2'] = cnt2.get(BUILT, 0) / tot2 * math.pi * (r / 1000) ** 2
                row['api_s_512'] = dt2
                log(f'stats {city} {year}: built {row["api_7"]:.1f} km2 (10 m, {dt or 0:.1f}s) | '
                    f'{row["api512_built_km2"]:.1f} km2 (512px, {dt2 or 0:.1f}s)')
            except Exception as e:  # noqa: BLE001
                failures.append(f'statistics {city} {year}: {e}')
                log('FAIL', e)
            api[(city, year)] = row

    # ---- 3) ESA WorldCover cross-check (2020 v100, 2021 v200), summing over 3x3 deg tiles
    wc = {}
    for city, lat, lon, rk in CITIES:
        x0, y0, r, feat = geoms[city]
        dlat, dlon = rk / 110.574, rk / (111.32 * math.cos(math.radians(lat)))
        bb = (lon - dlon, lat - dlat, lon + dlon, lat + dlat)
        for f in wc_items['features']:
            ib = f['bbox']
            if ib[0] >= bb[2] or ib[2] <= bb[0] or ib[1] >= bb[3] or ib[3] <= bb[1]:
                continue
            yr = int(f['id'].split('_')[3])
            try:
                cnt, dt = stats(WC, f['id'], 'map', feat, sorted(wc_names), 2 * r // RES, f'wc_{city}_{f["id"]}')
                acc = wc.setdefault((city, yr), Counter())
                acc.update(cnt)
                log(f'worldcover {city} {f["id"]}: {sum(v for k, v in cnt.items() if k)} px'
                    + (f' {dt:.1f}s' if dt else ' (cached)'))
            except Exception as e:  # noqa: BLE001
                failures.append(f'worldcover {city} {f["id"]}: {e}')
                log('FAIL', e)

    # ---- 4) analysis on the rasters
    rows, summary = [], {}
    keyname = {1: 'water', 2: 'trees', 4: 'flooded_veg', 5: 'crops', 7: 'built', 8: 'bare',
               9: 'snow_ice', 10: 'clouds', 11: 'rangeland'}
    for city, lat, lon, rk in CITIES:
        if not all((city, y) in rasters for y in YEARS):
            failures.append(f'{city}: missing rasters, analysis skipped')
            continue
        x0, y0, r, _ = geoms[city]
        n = 2 * r // RES
        imgs, nin = {}, None
        t = time.time()
        for y in YEARS:
            with open(rasters[(city, y)], 'rb') as f:
                w, h, px = decode_png_gray8(f.read())
            assert w == n and h == n, (w, h, n)
            imgs[y], nin = circle_mask(px, n, x0 - r, y0 + r, x0, y0, r)
            mx = max(set(imgs[y]))
            assert mx <= 11, mx
        log(f'{city}: decoded+masked 7 rasters in {time.time() - t:.0f}s; ring {km2(nin):.1f} km2')
        cls = {y: Counter(imgs[y]) for y in YEARS}
        pairs = {y: pair_counts(imgs[y - 1], imgs[y]) for y in YEARS[1:]}
        p1723 = pair_counts(imgs[YEARS[0]], imgs[YEARS[-1]])
        hb = history(imgs, BUILT)
        hc = history(imgs, CROPS)
        hbc = Counter(hb)
        hcc = Counter(hc)
        # Stable conversions: pixel not built until year k, then built every year to 2023
        # (monotonic built history). Prior class = class in the year just before conversion;
        # 'ever_cropped_before' = labelled crops in at least one year before conversion
        # (robust to crops<->rangeland flips in dry years).
        stable = {}
        for pat, k in MONO.items():
            if k == 0:
                continue
            y = YEARS[k]
            flag = hb.translate(bytes(1 if v == pat else 0 for v in range(256)))
            prior = {a: v for (a, b), v in pair_counts(imgs[YEARS[k - 1]], flag).items() if b == 1}
            cb = hc.translate(bytes(1 if v & ((1 << k) - 1) else 0 for v in range(256)))
            stable[y] = {'total': sum(prior.values()), 'prior': prior,
                         'ever_cropped_before': pair_counts(flag, cb).get((1, 1), 0)}

        for y in YEARS:
            c = cls[y]
            row = {'city': city, 'year': y, 'radius_km': rk, 'ring_km2': round(km2(nin), 2),
                   'nodata_km2': round(km2(nin - sum(v for k, v in c.items() if k)), 3)}
            for code, nm in keyname.items():
                row[f'{nm}_km2'] = round(km2(c.get(code, 0)), 3)
            row['built_dunam'] = round(km2(c.get(BUILT, 0)) * DUNAM_PER_KM2)
            if y in pairs:
                p = pairs[y]
                newb = {a: v for (a, b), v in p.items() if b == BUILT and a not in (BUILT, 0)}
                row['built_net_change_km2'] = round(km2(c.get(BUILT, 0) - cls[y - 1].get(BUILT, 0)), 3)
                row['new_built_gross_km2'] = round(km2(sum(newb.values())), 3)
                row['new_built_from_crops_km2'] = round(km2(newb.get(CROPS, 0)), 3)
                row['new_built_from_rangeland_km2'] = round(km2(newb.get(RANGE, 0)), 3)
                row['new_built_from_bare_km2'] = round(km2(newb.get(BARE, 0)), 3)
                row['new_built_from_other_km2'] = round(km2(sum(v for a, v in newb.items()
                                                                 if a not in (CROPS, RANGE, BARE))), 3)
                row['built_lost_gross_km2'] = round(km2(sum(v for (a, b), v in p.items()
                                                            if a == BUILT and b not in (BUILT, 0))), 3)
                row['crops_to_rangeland_km2'] = round(km2(p.get((CROPS, RANGE), 0)), 3)
                row['rangeland_to_crops_km2'] = round(km2(p.get((RANGE, CROPS), 0)), 3)
                row['crops_to_bare_km2'] = round(km2(p.get((CROPS, BARE), 0)), 3)
                st_ = stable[y]
                row['stable_new_built_km2'] = round(km2(st_['total']), 3)
                row['stable_from_crops_prev_yr_km2'] = round(km2(st_['prior'].get(CROPS, 0)), 3)
                row['stable_from_rangeland_prev_yr_km2'] = round(km2(st_['prior'].get(RANGE, 0)), 3)
                row['stable_from_bare_prev_yr_km2'] = round(km2(st_['prior'].get(BARE, 0)), 3)
                row['stable_from_ever_cropped_km2'] = round(km2(st_['ever_cropped_before']), 3)
            a = api.get((city, y), {})
            if a:
                row['api_built_km2'] = round(a.get('api_7', 0), 3)
                row['api_crops_km2'] = round(a.get('api_5', 0), 3)
                row['api_rangeland_km2'] = round(a.get('api_11', 0), 3)
                row['api_query_s_10m'] = round(a['api_s_native'], 2) if a.get('api_s_native') else ''
                row['api512_built_km2'] = round(a.get('api512_built_km2', 0), 3)
                row['api_query_s_512px'] = round(a['api_s_512'], 2) if a.get('api_s_512') else ''
            rows.append(row)

        # 2017 -> 2023 summary
        b17, b23 = cls[2017].get(BUILT, 0), cls[2023].get(BUILT, 0)
        direct = {a: v for (a, b), v in p1723.items() if b == BUILT and a not in (BUILT, 0)}
        lost = sum(v for (a, b), v in p1723.items() if a == BUILT and b not in (BUILT, 0))
        ever_b = sum(v for k, v in hbc.items() if k)
        mono_b = sum(v for k, v in hbc.items() if k in MONO)
        first_built = {YEARS[MONO[k]]: v for k, v in hbc.items() if k in MONO and MONO[k] > 0}
        # robust conversion: not built 2017 AND 2018, built 2022 AND 2023
        rob_tab = bytes(1 if (v & 0b11) == 0 and (v >> 5) & 0b11 == 0b11 else 0 for v in range(256))
        rob = hb.translate(rob_tab)
        rob17 = {a: v for (a, b), v in pair_counts(imgs[2017], rob).items() if b == 1}
        rob18 = {a: v for (a, b), v in pair_counts(imgs[2018], rob).items() if b == 1}
        # robust & was crops in 2017 or 2018 (handles one dry/odd year)
        crop1718 = hc.translate(bytes(1 if v & 0b11 else 0 for v in range(256)))
        rob_crop_any = pair_counts(rob, crop1718).get((1, 1), 0)
        ever_c = sum(v for k, v in hcc.items() if k)
        always_c = hcc.get((1 << len(YEARS)) - 1, 0)
        s = {
            'ring_km2': km2(nin), 'radius_km': rk,
            'built_2017_km2': km2(b17), 'built_2023_km2': km2(b23),
            'built_net_2017_2023_km2': km2(b23 - b17),
            'built_net_2017_2023_dunam': km2(b23 - b17) * DUNAM_PER_KM2,
            'built_growth_pct': 100 * (b23 - b17) / b17 if b17 else None,
            'direct_new_built_2017_2023_km2_by_2017_class': {keyname.get(a, a): round(km2(v), 3) for a, v in sorted(direct.items())},
            'direct_built_2017_to_nonbuilt_2023_km2': km2(lost),
            'robust_new_built_km2_by_2017_class': {keyname.get(a, a): round(km2(v), 3) for a, v in sorted(rob17.items())},
            'robust_new_built_km2_by_2018_class': {keyname.get(a, a): round(km2(v), 3) for a, v in sorted(rob18.items())},
            'robust_new_built_total_km2': km2(sum(rob17.values())),
            'robust_new_built_from_crops_2017_km2': km2(rob17.get(CROPS, 0)),
            'robust_new_built_from_crops_2017or2018_km2': km2(rob_crop_any),
            'robust_new_built_from_crops_2017or2018_dunam': km2(rob_crop_any) * DUNAM_PER_KM2,
            'robust_new_built_from_rangeland_2017_km2': km2(rob17.get(RANGE, 0)),
            'ever_built_km2': km2(ever_b), 'built_history_monotonic_share': mono_b / ever_b if ever_b else None,
            'first_built_year_km2_monotonic_pixels': {y: round(km2(v), 3) for y, v in sorted(first_built.items())},
            'ever_crops_km2': km2(ever_c), 'always_crops_km2': km2(always_c),
            'crops_by_year_km2': {y: round(km2(cls[y].get(CROPS, 0)), 2) for y in YEARS},
            'stable_new_built_2018_2023_km2': km2(sum(v['total'] for v in stable.values())),
            'stable_new_built_2018_2022_km2_(>=2yr_evidence)': km2(sum(v['total'] for y, v in stable.items() if y <= 2022)),
            'stable_from_crops_prev_yr_km2': km2(sum(v['prior'].get(CROPS, 0) for v in stable.values())),
            'stable_from_rangeland_prev_yr_km2': km2(sum(v['prior'].get(RANGE, 0) for v in stable.values())),
            'stable_from_bare_prev_yr_km2': km2(sum(v['prior'].get(BARE, 0) for v in stable.values())),
            'stable_from_ever_cropped_km2': km2(sum(v['ever_cropped_before'] for v in stable.values())),
            'stable_from_ever_cropped_dunam': km2(sum(v['ever_cropped_before'] for v in stable.values())) * DUNAM_PER_KM2,
        }
        summary[city] = s
        log(f'{city}: built 2017 {km2(b17):.1f} -> 2023 {km2(b23):.1f} km2 (+{km2(b23 - b17):.1f}); '
            f'robust new built {s["robust_new_built_total_km2"]:.1f} km2, from crops(17/18) {km2(rob_crop_any):.1f} km2; '
            f'built history monotonic {100 * s["built_history_monotonic_share"]:.0f}%; '
            f'always-crops {km2(always_c):.0f} of ever-crops {km2(ever_c):.0f} km2')
        del imgs, hb, hc

    # ---- outputs
    cols = []
    for r_ in rows:
        for k in r_:
            if k not in cols:
                cols.append(k)
    with open(os.path.join(od, 'farmland_loss_by_city_year.csv'), 'w', newline='') as f:
        wr = csv.DictWriter(f, fieldnames=cols)
        wr.writeheader()
        wr.writerows(rows)
    wc_rows = []
    wc_key = {40: 'cropland', 50: 'builtup', 30: 'grassland', 20: 'shrubland', 60: 'bare_sparse'}
    for (city, yr), cnt in sorted(wc.items()):
        r = geoms[city][2]
        tot = sum(v for k, v in cnt.items() if k)
        ring = math.pi * (r / 1000) ** 2
        rr = {'city': city, 'year': yr, 'valid_px': tot}
        for code, nm in wc_key.items():
            rr[f'wc_{nm}_km2'] = round(cnt.get(code, 0) / tot * ring, 2) if tot else ''
        io_row = next((x for x in rows if x['city'] == city and x['year'] == yr), {})
        rr['io_built_km2'] = io_row.get('built_km2', '')
        rr['io_crops_km2'] = io_row.get('crops_km2', '')
        rr['io_rangeland_km2'] = io_row.get('rangeland_km2', '')
        wc_rows.append(rr)
        log(f'WorldCover {city} {yr}: built {rr["wc_builtup_km2"]} (IO {rr["io_built_km2"]}), '
            f'cropland {rr["wc_cropland_km2"]} (IO crops {rr["io_crops_km2"]}), grass {rr["wc_grassland_km2"]}')
    if wc_rows:
        with open(os.path.join(od, 'worldcover_crosscheck.csv'), 'w', newline='') as f:
            wr = csv.DictWriter(f, fieldnames=list(wc_rows[0]))
            wr.writeheader()
            wr.writerows(wc_rows)
    with open(os.path.join(od, 'farmland_loss_summary.json'), 'w') as f:
        json.dump({'summary': summary, 'failures': failures, 'io_class_codes': names,
                   'worldcover_class_codes': wc_names}, f, indent=1, default=str)
    if failures:
        log('FAILURES:\n  ' + '\n  '.join(failures))
    with open(os.path.join(od, 'run.log'), 'w') as f:
        f.write('\n'.join(LOG) + '\n')
    return 1 if failures else 0


if __name__ == '__main__':
    sys.exit(main())
