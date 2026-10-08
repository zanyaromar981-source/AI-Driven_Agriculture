"""CHIRPS v2.0 monthly rainfall per zone (mean over a +/-0.04 deg box), stdlib only.

Source: UCSB Climate Hazards Center, keyless HTTPS.
  final : https://data.chc.ucsb.edu/products/CHIRPS-2.0/global_monthly/cogs/chirps-v2.0.YYYY.MM.cog
          (cloud-optimised GeoTIFF, 512x512 LZW tiles -> only the one tile over Kurdistan is fetched
           with an HTTP Range request, ~1 MB per month instead of 25 MB)
  prelim: https://data.chc.ucsb.edu/products/CHIRPS-2.0/prelim/global_monthly/tifs/chirps-v2.0.YYYY.MM.tif
          (uncompressed strip TIFF -> only the needed rows are range-read), used for months not yet final.
Grid: 0.05 deg, 7200x2000, top-left corner (-180, 50), PixelIsArea. Values are mm per month, nodata -9999.

Raw: for every month the pixel window over Kurdistan (all zone boxes + margin) is saved, untouched,
     as RAW_DIR/<final|prelim>/chirps-v2.0.YYYY.MM_window.txt (re-runs reuse these files).
Zone value: mean of the pixels overlapping the box [lon+-0.04, lat+-0.04], weighted by overlap area.

Usage: python3 -I fetch_chirps.py ZONES_JSON RAW_DIR OUT_CSV [WORKERS]
"""
import concurrent.futures as cf
import csv
import json
import os
import re
import struct
import sys
import time
import urllib.request

FINAL_DIR = 'https://data.chc.ucsb.edu/products/CHIRPS-2.0/global_monthly/cogs/'
PRELIM_DIR = 'https://data.chc.ucsb.edu/products/CHIRPS-2.0/prelim/global_monthly/tifs/'
RES, X0, Y0 = 0.05, -180.0, 50.0
HALF = 0.04
MARGIN = 3  # pixels around the union of zone boxes
UA = {'User-Agent': 'SmartSuli-backtest/1.0 (python urllib)'}


def http_get(url, start=None, end=None, tries=6, allow_short=False):
    hdr = dict(UA)
    if start is not None:
        hdr['Range'] = 'bytes=%d-%d' % (start, end)
    for k in range(tries):
        try:
            with urllib.request.urlopen(urllib.request.Request(url, headers=hdr), timeout=180) as r:
                data = r.read()
                if start is not None and r.status != 206:
                    raise IOError('server ignored Range (status %s)' % r.status)
                if start is not None and not allow_short and len(data) != end - start + 1:
                    raise IOError('short read %d of %d' % (len(data), end - start + 1))
                return data
        except Exception as e:
            if k == tries - 1:
                raise
            time.sleep(3 * (k + 1))


# ---------------- minimal TIFF reader ----------------
TYPES = {1: ('B', 1), 2: ('s', 1), 3: ('H', 2), 4: ('I', 4), 5: ('II', 8), 11: ('f', 4), 12: ('d', 8), 16: ('Q', 8)}


def read_ifd(url):
    """Return {tag: tuple(values)} of the first IFD (classic little-endian TIFF)."""
    head = http_get(url, 0, 16383, allow_short=True)
    if head[:4] != b'II*\x00':
        raise IOError('not a little-endian classic TIFF: %r' % head[:4])
    off = struct.unpack('<I', head[4:8])[0]
    cache = {0: head}  # offset -> bytes chunks we hold

    def get(o, n):
        for base, buf in cache.items():
            if base <= o and o + n <= base + len(buf):
                return buf[o - base:o - base + n]
        buf = http_get(url, o, o + max(n, 16384) - 1, allow_short=True)
        if len(buf) < n:
            raise IOError('short read at %d' % o)
        cache[o] = buf
        return buf[:n]

    n = struct.unpack('<H', get(off, 2))[0]
    ents = get(off + 2, 12 * n)
    tags = {}
    for i in range(n):
        tag, typ, cnt = struct.unpack('<HHI', ents[12 * i:12 * i + 8])
        fmt, sz = TYPES.get(typ, ('B', 1))
        total = sz * cnt
        raw = ents[12 * i + 8:12 * i + 8 + total] if total <= 4 else get(struct.unpack('<I', ents[12 * i + 8:12 * i + 12])[0], total)
        if typ == 2:
            tags[tag] = raw.decode('latin1')
        else:
            tags[tag] = struct.unpack('<' + fmt * cnt if typ != 5 else '<%dI' % (2 * cnt), raw)
    return tags


def lzw_decode(data, stop_at):
    """TIFF LZW (MSB-first, early change). Stops once stop_at bytes are produced."""
    out = bytearray()
    table = [bytes((i,)) for i in range(256)] + [b'', b'']
    nbits, bitbuf, bitcnt, pos, n = 9, 0, 0, 0, len(data)
    prev = None
    while len(out) < stop_at:
        while bitcnt < nbits:
            if pos >= n:
                return out
            bitbuf = ((bitbuf << 8) | data[pos]) & 0xFFFFFFFF
            pos += 1
            bitcnt += 8
        bitcnt -= nbits
        code = (bitbuf >> bitcnt) & ((1 << nbits) - 1)
        if code == 256:
            del table[258:]
            nbits = 9
            prev = None
            continue
        if code == 257:
            break
        if prev is None:
            entry = table[code]
        else:
            if code < len(table):
                entry = table[code]
                table.append(prev + entry[:1])
            else:
                entry = prev + prev[:1]
                table.append(entry)
            L = len(table)
            nbits = 12 if L >= 2047 else 11 if L >= 1023 else 10 if L >= 511 else 9
        out += entry
        prev = entry
    return out


def read_window(url, r0, r1, c0, c1):
    """Return list of rows (r0..r1 inclusive) of float values for cols c0..c1 inclusive."""
    t = read_ifd(url)
    width, height = t[256][0], t[257][0]
    comp, pred = t[259][0], t.get(317, (1,))[0]
    if t[258][0] != 32 or t.get(339, (1,))[0] != 3 or pred != 1:
        raise IOError('unexpected sample format/predictor')
    scale, tie = t[33550], t[33922]
    if abs(scale[0] - RES) > 1e-6 or abs(tie[3] - X0) > 1e-9 or abs(tie[4] - Y0) > 1e-9:
        raise IOError('unexpected georeference %r %r' % (scale, tie))
    ncol = c1 - c0 + 1
    rows = {}
    if 322 in t:  # tiled (final COG)
        tw, th = t[322][0], t[323][0]
        across = (width + tw - 1) // tw
        offs, cnts = t[324], t[325]
        for tr in range(r0 // th, r1 // th + 1):
            for tc in range(c0 // tw, c1 // tw + 1):
                idx = tr * across + tc
                lr1 = min(r1, tr * th + th - 1) - tr * th
                raw = http_get(url, offs[idx], offs[idx] + cnts[idx] - 1)
                if comp == 5:
                    buf = lzw_decode(raw, (lr1 + 1) * tw * 4)
                elif comp == 1:
                    buf = raw
                else:
                    raise IOError('compression %d not supported' % comp)
                for r in range(max(r0, tr * th), min(r1, tr * th + th - 1) + 1):
                    lr = r - tr * th
                    a, b = max(c0, tc * tw), min(c1, tc * tw + tw - 1)
                    vals = struct.unpack_from('<%df' % (b - a + 1), buf, (lr * tw + (a - tc * tw)) * 4)
                    rows.setdefault(r, [None] * ncol)[a - c0:b - c0 + 1] = vals
    else:  # strips (prelim tif)
        if comp != 1:
            raise IOError('compressed strips not supported')
        rps = t.get(278, (height,))[0]
        offs = t[273]
        if rps != 1:
            raise IOError('rows per strip %d not supported' % rps)
        contiguous = all(offs[r + 1] - offs[r] == width * 4 for r in range(r0, r1))
        if contiguous:
            blob = http_get(url, offs[r0] + c0 * 4, offs[r1] + (c1 + 1) * 4 - 1)
            for r in range(r0, r1 + 1):
                rows[r] = list(struct.unpack_from('<%df' % ncol, blob, (r - r0) * width * 4))
        else:
            for r in range(r0, r1 + 1):
                rows[r] = list(struct.unpack('<%df' % ncol, http_get(url, offs[r] + c0 * 4, offs[r] + (c1 + 1) * 4 - 1)))
    return [rows[r] for r in range(r0, r1 + 1)]


# ---------------- pipeline ----------------
def list_months(url, ext):
    html = http_get(url).decode('utf-8', 'replace')
    return sorted(set(re.findall(r'chirps-v2\.0\.(\d{4})\.(\d{2})\.' + ext + r'"', html)))


def window_for(zones):
    lons = [z['lon'] for z in zones]
    lats = [z['lat'] for z in zones]
    c0 = int((min(lons) - HALF - X0) / RES) - MARGIN
    c1 = int((max(lons) + HALF - X0) / RES) + MARGIN
    r0 = int((Y0 - (max(lats) + HALF)) / RES) - MARGIN
    r1 = int((Y0 - (min(lats) - HALF)) / RES) + MARGIN
    return r0, r1, c0, c1


def raw_path(raw_dir, kind, y, m):
    return os.path.join(raw_dir, kind, 'chirps-v2.0.%s.%s_window.txt' % (y, m))


def fetch_month(args):
    kind, url, path, win = args
    if os.path.exists(path):
        return path, 'cached'
    r0, r1, c0, c1 = win
    grid = read_window(url, r0, r1, c0, c1)
    tmp = path + '.part'
    with open(tmp, 'w') as f:
        f.write('# source=%s\n' % url)
        f.write('# product=CHIRPS v2.0 %s monthly, units=mm/month, nodata=-9999\n' % kind)
        f.write('# window rows %d-%d cols %d-%d of the 7200x2000 global grid; '
                'row r spans lat [%g - 0.05(r+1), %g - 0.05r], col c spans lon [%g + 0.05c, %g + 0.05(c+1)]\n'
                % (r0, r1, c0, c1, Y0, Y0, X0, X0))
        for row in grid:
            f.write(','.join('%.7g' % v for v in row) + '\n')
    os.replace(tmp, path)
    return path, 'downloaded'


def load_window(path):
    meta, rows = {}, []
    with open(path) as f:
        for line in f:
            if line.startswith('#'):
                m = re.search(r'window rows (\d+)-(\d+) cols (\d+)-(\d+)', line)
                if m:
                    meta['win'] = tuple(int(x) for x in m.groups())
                continue
            rows.append([float(v) for v in line.strip().split(',')])
    return meta['win'], rows


def zone_weights(z, win):
    r0, r1, c0, c1 = win
    bx0, bx1 = z['lon'] - HALF, z['lon'] + HALF
    by0, by1 = z['lat'] - HALF, z['lat'] + HALF
    w = []
    for r in range(r0, r1 + 1):
        yt, yb = Y0 - RES * r, Y0 - RES * (r + 1)
        oy = min(yt, by1) - max(yb, by0)
        if oy <= 1e-9:
            continue
        for c in range(c0, c1 + 1):
            xl, xr = X0 + RES * c, X0 + RES * (c + 1)
            ox = min(xr, bx1) - max(xl, bx0)
            if ox > 1e-9:
                w.append((r - r0, c - c0, ox * oy))
    return w


def main():
    if len(sys.argv) < 4:
        sys.exit(__doc__)
    zones = json.load(open(sys.argv[1]))['zones']
    raw_dir, out_csv = sys.argv[2], sys.argv[3]
    workers = int(sys.argv[4]) if len(sys.argv) > 4 else 8
    win = window_for(zones)
    print('window rows %d-%d cols %d-%d' % win)
    for kind in ('final', 'prelim'):
        os.makedirs(os.path.join(raw_dir, kind), exist_ok=True)

    final = list_months(FINAL_DIR, 'cog')
    prelim = [ym for ym in list_months(PRELIM_DIR, 'tif') if ym not in set(final) and ym > final[-1]]
    print('final %s..%s (%d months); prelim-only: %s' % ('-'.join(final[0]), '-'.join(final[-1]), len(final),
                                                          ['-'.join(p) for p in prelim]))
    jobs = [('final', FINAL_DIR + 'chirps-v2.0.%s.%s.cog' % ym, raw_path(raw_dir, 'final', *ym), win) for ym in final]
    jobs += [('prelim', PRELIM_DIR + 'chirps-v2.0.%s.%s.tif' % ym, raw_path(raw_dir, 'prelim', *ym), win) for ym in prelim]

    failed = []
    with cf.ProcessPoolExecutor(workers) as ex:
        futs = {ex.submit(fetch_month, j): j for j in jobs}
        for i, fu in enumerate(cf.as_completed(futs), 1):
            j = futs[fu]
            try:
                fu.result()
            except Exception as e:
                failed.append((j[1], repr(e)))
            if i % 25 == 0 or i == len(jobs):
                print('%d/%d done, %d failed' % (i, len(jobs), len(failed)), flush=True)
    for u, e in failed:
        print('FAILED', u, e)

    weights = {z['name']: zone_weights(z, win) for z in zones}
    out_rows = []
    for kind, url, path, _ in jobs:
        if not os.path.exists(path):
            continue
        w_win, grid = load_window(path)
        if w_win != win:
            raise SystemExit('window mismatch in %s' % path)
        y, m = re.search(r'(\d{4})\.(\d{2})_window', path).groups()
        for z in zones:
            s = sw = 0.0
            for rr, cc, w in weights[z['name']]:
                v = grid[rr][cc]
                if v > -9000:
                    s += v * w
                    sw += w
            out_rows.append((z['name'], int(y), int(m), round(s / sw, 2) if sw else '', kind))
    out_rows.sort(key=lambda r: ([z['name'] for z in zones].index(r[0]), r[1], r[2]))
    with open(out_csv, 'w', newline='') as f:
        wr = csv.writer(f)
        wr.writerow(['zone', 'year', 'month', 'precip_mm', 'status'])
        wr.writerows(out_rows)
    print('wrote', out_csv, len(out_rows), 'rows;', 'pixels per zone:',
          {k: len(v) for k, v in weights.items()})


if __name__ == '__main__':
    main()
