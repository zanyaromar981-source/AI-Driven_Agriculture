"""Build web/green.js: NASA MODIS greenness (NDVI, 250 m, every 16 days) per hex cell of the web map, as % of that cell's normal.
Only farmland-like pixels are used: April greenness 0.30-0.80 and at least 0.15 greener in April than in October (2001-2020 means).
Cells with fewer than 25 such pixels are marked as "little farmland" and use all land pixels.
Input: ../evidence/past_seasons/backtest_data/modis_grid/ (592 dates, see fetch_greenness_grid.py) and data.js (hex cells).
Usage: python3 -I build_greenness.py   (about 5 minutes; writes green.js)"""
import os, sys, json, math, zlib, glob, base64, statistics as st, multiprocessing as mp
from array import array
WEB = os.path.dirname(os.path.abspath(__file__))
GRID = os.path.join(WEB, '..', 'evidence', 'past_seasons', 'backtest_data', 'modis_grid')
NR, NC, ROW0, COL0 = 2048, 2560, 1024, 1536
X0, Y0, RES, RAD = 3335851.559, 4447802.078667, 231.65635826395834, 6371007.181
LON0, LAT0 = 44.0, 36.0; KX = math.cos(math.radians(36)) * 111.32; KY = 110.57     # same projection as build_data.py

def load(path):
    a = array('h'); a.frombytes(zlib.decompress(open(path, 'rb').read())); return a

def pixel_to_cell(cells, R):
    """index of the hex cell that contains each grid pixel, or -1."""
    idx = {(q, r): i for i, (q, r, a, z) in enumerate(cells)}
    out = array('h', [-1]) * (NR * NC)
    for row in range(NR):
        y = Y0 - (row + ROW0 + 0.5) * RES; lat = math.degrees(y / RAD); coslat = math.cos(math.radians(lat))
        yk = (lat - LAT0) * KY; rf = -yk / (1.5 * R)
        base = row * NC
        for col in range(NC):
            x = X0 + (col + COL0 + 0.5) * RES; lon = math.degrees(x / (RAD * coslat))
            qf = (lon - LON0) * KX / (R * math.sqrt(3)) - rf / 2
            # cube rounding
            xq, zq = qf, rf; yq = -xq - zq
            rx, ry, rz = round(xq), round(yq), round(zq)
            dx, dy, dz = abs(rx - xq), abs(ry - yq), abs(rz - zq)
            if dx > dy and dx > dz: rx = -ry - rz
            elif dy <= dz: rz = -rx - ry
            i = idx.get((rx, rz))
            if i is not None: out[base + col] = i
    return out

PIX = None; OWN = None; NCELL = 0
def init(pix, own, ncell):
    global PIX, OWN, NCELL
    PIX, OWN, NCELL = pix, own, ncell

def cell_means(path):
    a = load(path); s = [0] * NCELL; n = [0] * NCELL
    for p, c in zip(PIX, OWN):
        v = a[p]
        if v > -2000: s[c] += v; n[c] += 1
    return os.path.basename(path)[8:15], [s[i] / n[i] / 10000 if n[i] >= 5 else None for i in range(NCELL)]

def main():
    src = open(os.path.join(WEB, 'data.js')).read(); D = json.loads(src[src.index('{'):src.rindex('}') + 1])
    cells, R = D['hex']['cells'], D['hex']['R']
    print('mapping pixels to', len(cells), 'cells...', flush=True)
    owner = pixel_to_cell(cells, R)
    inside = [p for p in range(NR * NC) if owner[p] >= 0]
    print(len(inside), 'pixels inside the map', flush=True)
    # farmland mask from April and October means 2001-2020
    files = {os.path.basename(f)[8:15]: f for f in glob.glob(os.path.join(GRID, 'MOD13Q1_*.i16.z'))}
    apr = [files[k] for k in files if 2001 <= int(k[:4]) <= 2020 and k[4:] in ('097', '113')]
    octo = [files[k] for k in files if 2001 <= int(k[:4]) <= 2020 and k[4:] in ('273', '289')]
    def mean_of(fl):
        s = array('f', [0.0]) * len(inside); n = array('h', [0]) * len(inside)
        for f in fl:
            a = load(f)
            for j, p in enumerate(inside):
                v = a[p]
                if v > -2000: s[j] += v; n[j] += 1
        return [s[j] / n[j] if n[j] else None for j in range(len(inside))]
    print('farmland mask from', len(apr), 'April and', len(octo), 'October maps...', flush=True)
    ma, mo = mean_of(apr), mean_of(octo)
    crop = [ma[j] is not None and mo[j] is not None and 3000 <= ma[j] <= 8000 and ma[j] - mo[j] >= 1500 for j in range(len(inside))]
    per_cell = [0] * len(cells)
    for j, p in enumerate(inside):
        if crop[j]: per_cell[owner[p]] += 1
    little = [n < 25 for n in per_cell]
    use = [p for j, p in enumerate(inside) if crop[j] or little[owner[p]]]
    own = [owner[p] for p in use]
    print(sum(1 for x in little if not x), 'cells with farmland,', sum(little), 'with little farmland;', len(use), 'pixels used', flush=True)
    ctx = mp.get_context('fork')
    with ctx.Pool(min(12, os.cpu_count() or 4), initializer=init, initargs=(array('i', use), array('h', own), len(cells))) as pool:
        res = dict(pool.map(cell_means, [files[k] for k in sorted(files)], chunksize=4))
    print('per-date cell means done', flush=True)
    # normal per day-of-year slot = median of 2001-2020
    doys = sorted(set(k[4:] for k in res))
    normal = {}
    for d in doys:
        normal[d] = []
        for i in range(len(cells)):
            v = [res[k][i] for k in res if k[4:] == d and 2001 <= int(k[:4]) <= 2020 and res[k][i] is not None]
            normal[d].append(st.median(v) if len(v) >= 8 else None)
    # per season: composites from doy 273 (Oct) of year s to doy 145 (late May) of year s+1
    seasons = {}
    for s in range(2000, 2026):
        keys = [k for k in sorted(res) if (int(k[:4]) == s and int(k[4:]) >= 273) or (int(k[:4]) == s + 1 and int(k[4:]) <= 145)]
        comps = []
        for k in keys:
            vals = bytearray()
            for i in range(len(cells)):
                v, n = res[k][i], normal[k[4:]][i]
                pct = 0 if (v is None or not n or n <= 0.05) else max(-99, min(99, round(v / n * 100 - 100)))
                vals.append(pct & 0xFF)
            comps.append([k, base64.b64encode(bytes(vals)).decode()])
        if comps: seasons[str(s)] = comps
    # late-spring (composites doy 81-129 of spring year s+1) per cell, the same window as the crop outcome
    spring = {}
    for s in range(1999, 2025):
        ks = [f'{s + 1}{d:03d}' for d in (81, 97, 113, 129) if f'{s + 1}{d:03d}' in res]
        row = bytearray()
        for i in range(len(cells)):
            v = [res[k][i] for k in ks if res[k][i] is not None]; n = [normal[k[4:]][i] for k in ks if normal[k[4:]][i]]
            pct = 0 if not v or not n else max(-99, min(99, round(st.mean(v) / st.mean(n) * 100 - 100)))
            row.append(pct & 0xFF)
        spring[str(s)] = base64.b64encode(bytes(row)).decode()
    out = dict(note='NASA MODIS MOD13Q1 NDVI per hex cell, % above/below the 2001-2020 normal for the same 16-day period (int8, base64). Farmland pixels only unless little[i].',
               little=[1 if x else 0 for x in little], seasons=seasons, spring=spring)
    js = 'window.GREEN = ' + json.dumps(out, separators=(',', ':')) + ';\n'
    open(os.path.join(WEB, 'green.js'), 'w').write(js)
    print('wrote green.js', len(js) // 1024, 'KB', flush=True)

if __name__ == '__main__':
    main()
