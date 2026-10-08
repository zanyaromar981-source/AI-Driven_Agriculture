"""Greenness (NDVI) map of the whole Kurdistan Region, every 16 days, 2000 -> now, 250 m.

Source: NASA MODIS MOD13Q1 v061 (Terra, same product the zone backtest used via ORNL),
        read as Cloud-Optimized GeoTIFF from Microsoft Planetary Computer (keyless; link signed per file).
Area:   one MODIS tile (h21v05) covers the region. We keep a 4 x 5 block of its 512-px internal tiles
        = sinusoidal rows 1024-3071, cols 1536-4095 (covers lon 42.3-46.4, lat 34.3-37.4 with margin).
Output: backtest_data/modis_grid/MOD13Q1_<YYYYDDD>.i16.z  (zlib of int16 little-endian, 2048 rows x 2560 cols,
        NDVI x 10000, nodata -3000) + _grid_info.json (geometry and how to find a lon/lat).
Re-runs skip dates already saved.  Usage: python3 -I fetch_greenness_grid.py [workers]
"""
import json, os, sys, time, zlib, struct, urllib.request, urllib.parse
from concurrent.futures import ThreadPoolExecutor, as_completed

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, '..', 'backtest_data', 'modis_grid')
LOG = os.path.join(HERE, 'fetch_greenness_grid.log')
STAC = 'https://planetarycomputer.microsoft.com/api/stac/v1/search'
SIGN = 'https://planetarycomputer.microsoft.com/api/sas/v1/sign?href='
UA = {'User-Agent': 'Mozilla/5.0'}
TILE_ROWS, TILE_COLS = range(2, 6), range(3, 8)   # internal 512-px tiles kept
TW = 512
X0, Y0, RES, R = 3335851.559, 4447802.078667, 231.65635826395834, 6371007.181  # h21v05 top-left, pixel, sphere


def log(msg):
    line = time.strftime('%H:%M:%S ') + msg
    print(line, flush=True)
    with open(LOG, 'a') as f:
        f.write(line + '\n')


def get(url, rng=None, data=None, tries=6):
    for i in range(tries):
        try:
            h = dict(UA)
            if rng:
                h['Range'] = 'bytes=%d-%d' % rng
            if data is not None:
                h['Content-Type'] = 'application/json'
            return urllib.request.urlopen(urllib.request.Request(url, data=data, headers=h), timeout=120).read()
        except Exception as e:
            if i == tries - 1:
                raise
            time.sleep(min(60, 3 * 2 ** i))


def list_items():
    body = {'collections': ['modis-13Q1-061'], 'bbox': [44.0, 36.0, 44.01, 36.01],
            'datetime': '2000-01-01T00:00:00Z/2030-01-01T00:00:00Z', 'limit': 1000}
    items, nxt = {}, body
    while nxt:
        d = json.loads(get(STAC, data=json.dumps(nxt).encode()))
        for f in d['features']:
            if f['id'].startswith('MOD13Q1.') and '.h21v05.' in f['id']:
                date = f['id'].split('.')[1][1:]          # YYYYDDD
                if date not in items or f['id'] > items[date]['id']:
                    items[date] = {'id': f['id'], 'href': f['assets']['250m_16_days_NDVI']['href']}
        nxt = next((l.get('body') for l in d.get('links', []) if l.get('rel') == 'next'), None)
    return items


def ifd(url):
    b = get(url, (0, 65535))
    off = struct.unpack('<I', b[4:8])[0]
    n = struct.unpack('<H', b[off:off + 2])[0]
    tags = {}
    for i in range(n):
        tag, typ, cnt = struct.unpack('<HHI', b[off + 2 + 12 * i:off + 10 + 12 * i])
        fmt, sz = {3: ('H', 2), 4: ('I', 4), 12: ('d', 8)}.get(typ, ('B', 1))
        if sz * cnt <= 4:
            raw = b[off + 10 + 12 * i:off + 10 + 12 * i + sz * cnt]
        else:
            p = struct.unpack('<I', b[off + 10 + 12 * i:off + 14 + 12 * i])[0]
            raw = b[p:p + sz * cnt] if p + sz * cnt <= len(b) else get(url, (p, p + sz * cnt - 1))
        if typ in (3, 4, 12):
            tags[tag] = struct.unpack('<%d%s' % (cnt, fmt), raw)
    return tags


def fetch(date, item):
    path = os.path.join(OUT, 'MOD13Q1_%s.i16.z' % date)
    if os.path.exists(path):
        return date, 'skip'
    url = json.loads(get(SIGN + urllib.parse.quote(item['href'], safe='')))['href']
    t = ifd(url)
    if (t[256][0], t[257][0], t[258][0], t[259][0], t[322][0]) != (4800, 4800, 16, 8, TW) or t.get(317, (1,))[0] != 1:
        raise IOError('unexpected file layout')
    if abs(t[33922][3] - X0) > 1 or abs(t[33922][4] - Y0) > 1:
        raise IOError('unexpected georeference')
    offs, cnts, across = t[324], t[325], 4800 // TW + 1
    ncols = len(TILE_COLS) * TW
    win = bytearray(len(TILE_ROWS) * TW * ncols * 2)
    for i, tr in enumerate(TILE_ROWS):
        for j, tc in enumerate(TILE_COLS):
            k = tr * across + tc
            buf = zlib.decompress(get(url, (offs[k], offs[k] + cnts[k] - 1)))
            if len(buf) != TW * TW * 2:
                raise IOError('tile size %d' % len(buf))
            for r in range(TW):
                d0 = ((i * TW + r) * ncols + j * TW) * 2
                win[d0:d0 + TW * 2] = buf[r * TW * 2:(r + 1) * TW * 2]
    tmp = path + '.part'
    with open(tmp, 'wb') as f:
        f.write(zlib.compress(bytes(win), 6))
    os.replace(tmp, path)
    return date, 'ok'


def main():
    workers = int(sys.argv[1]) if len(sys.argv) > 1 else 4
    os.makedirs(OUT, exist_ok=True)
    info = {'product': 'MOD13Q1 v061 250m_16_days_NDVI (Terra), tile h21v05', 'source': 'Microsoft Planetary Computer, collection modis-13Q1-061',
            'dtype': 'int16 little-endian, zlib', 'scale': 0.0001, 'nodata': -3000,
            'rows': len(TILE_ROWS) * TW, 'cols': len(TILE_COLS) * TW,
            'row0_in_tile': TILE_ROWS[0] * TW, 'col0_in_tile': TILE_COLS[0] * TW,
            'sinusoidal': {'tile_left_x': X0, 'tile_top_y': Y0, 'pixel_m': RES, 'sphere_radius_m': R},
            'lonlat_to_pixel': 'x = R*rad(lon)*cos(rad(lat)); y = R*rad(lat); col = (x - tile_left_x)/pixel_m - col0_in_tile; row = (tile_top_y - y)/pixel_m - row0_in_tile',
            'file_name': 'MOD13Q1_<YYYYDDD>.i16.z, YYYYDDD = first day of the 16-day composite'}
    with open(os.path.join(OUT, '_grid_info.json'), 'w') as f:
        json.dump(info, f, indent=1)
    items = list_items()
    dates = sorted(items)
    log('START %d dates (%s -> %s), %d workers' % (len(dates), dates[0], dates[-1], workers))
    done = failed = 0
    t0 = time.time()
    with ThreadPoolExecutor(workers) as ex:
        futs = {ex.submit(fetch, d, items[d]): d for d in dates}
        for fu in as_completed(futs):
            d = futs[fu]
            try:
                fu.result()
                done += 1
            except Exception as e:
                failed += 1
                log('FAIL %s %s' % (d, e))
            n = done + failed
            if n % 20 == 0 or n == len(dates):
                el = time.time() - t0
                log('%d/%d done, %d failed, %.0f min elapsed, ~%.0f min left' % (n, len(dates), failed, el / 60, el / n * (len(dates) - n) / 60))
    log('ALL DONE' if failed == 0 else 'DONE WITH %d FAILED (re-run to retry)' % failed)


if __name__ == '__main__':
    main()
