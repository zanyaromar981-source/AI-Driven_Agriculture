"""Do Mediterranean / Persian Gulf sea temperatures, known before the winter, predict Kurdistan winter rain?  (2026-10-08)
Sea temperature: NOAA ERSST v5 monthly (2 degree), area mean, via IRI Data Library data.tsv (keyless), Jan 1981 -> latest.
Outcome: Oct-May rain, mean of our 16 zones (ERA5), % of 1991-2020 normal, 36 winters 1990/91-2025/26; drought = bottom 25%.
Same bar as signals_test.py: correlation, and leave-one-winter-out rain forecast El Nino alone vs El Nino + sea.
Usage: python3 -I sea_temp_test.py   (caches to ../backtest_data/sea_temp/, writes sea_temp_test_output.txt)"""
import os, sys, json, time, urllib.request, statistics as st
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from deep_investigation import W, ymd, auc
DATA = os.path.join(HERE, '..', 'backtest_data'); OUT = os.path.join(DATA, 'sea_temp'); os.makedirs(OUT, exist_ok=True)
BOXES = {'East Mediterranean': ('30E', '36E', '31N', '36N'),
         'Whole Mediterranean': ('0E', '36E', '30N', '41N'),
         'Persian Gulf': ('48E', '56E', '24N', '30N')}
Y0 = 1981
def fetch(name, box):
    fn = os.path.join(OUT, name.replace(' ', '_') + '.tsv')
    if not os.path.exists(fn):
        x0, x1, y0, y1 = box
        url = ('https://iridl.ldeo.columbia.edu/SOURCES/.NOAA/.NCDC/.ERSST/.version5/.sst/'
               f'X/({x0})/({x1})/RANGEEDGES/Y/({y0})/({y1})/RANGEEDGES/%5BX/Y%5Daverage/T/(Jan%20{Y0})/(Dec%202026)/RANGEEDGES/data.tsv')
        for i in range(4):
            try:
                b = urllib.request.urlopen(urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'}), timeout=180).read().decode(); break
            except Exception:
                if i == 3: raise
                time.sleep(10)
        open(fn, 'w').write(f'# {url}\n# monthly area-mean SST (deg C) starting Jan {Y0}\n' + b)
    vals = [float(l) for l in open(fn) if l.strip() and not l.startswith('#')]
    return {(Y0 + i // 12, i % 12 + 1): v for i, v in enumerate(vals)}
lines = []
def P(x=''): print(x); lines.append(x)

SST = {n: fetch(n, b) for n, b in BOXES.items()}
ONI = {}
for l in open(os.path.join(DATA, 'enso', 'oni.ascii.txt')).read().splitlines()[1:]:
    a, y, t, an = l.split(); ONI[(a, int(y))] = float(an)
zones = [z['name'] for z in json.load(open(os.path.join(HERE, 'zones.json')))['zones']]
WINTERS = list(range(1990, 2026)); tot = {}
for z in zones:
    w = W(os.path.join(DATA, 'openmeteo', f'{z}.json'))
    t = {s: w.sum(ymd(s, 10, 1), ymd(s + 1, 5, 31), 'precipitation_sum') for s in WINTERS}; n = st.mean(t[s] for s in range(1991, 2021))
    for s in WINTERS: tot.setdefault(s, []).append(t[s] / n * 100)
R = {s: st.mean(v) for s, v in tot.items()}
thr = sorted(R.values())[len(R) // 4]; BAD = {s: R[s] <= thr for s in WINTERS}

def anomaly(d, months, detrend):
    """mean anomaly over `months` (list of (yearoffset, month)) per season start year s; vs 1991-2020 monthly normal; optional linear detrend."""
    norm = {m: st.mean(d[(y, m)] for y in range(1991, 2021)) for m in range(1, 13)}
    a = {}
    for s in range(1982, 2026):
        v = [d.get((s + dy, m)) for dy, m in months]
        if None not in v: a[s] = st.mean(x - norm[m] for x, (dy, m) in zip(v, months))
    if detrend:
        ys = sorted(a); xb = st.mean(ys); yb = st.mean(a[y] for y in ys)
        b = sum((y - xb) * (a[y] - yb) for y in ys) / sum((y - xb) ** 2 for y in ys)
        a = {y: v - b * (y - xb) for y, v in a.items()}
    return a
TIMES = {'Jul-Sep (known early Oct)': [(0, 7), (0, 8), (0, 9)], 'Sep-Nov (known early Dec)': [(0, 9), (0, 10), (0, 11)]}

def loo(feats):
    pts = [(s, [f[s] for f in feats], R[s]) for s in WINTERS if all(s in f for f in feats)]
    preds = []
    for i, (s, x, y) in enumerate(pts):
        tr = [p for j, p in enumerate(pts) if j != i]; k = len(x) + 1
        A = [[0.0] * k for _ in range(k)]; B = [0.0] * k
        for _, xx, yy in tr:
            v = [1.0] + xx
            for a in range(k):
                B[a] += v[a] * yy
                for b in range(k): A[a][b] += v[a] * v[b] + (0.01 if a == b and a > 0 else 0)
        for c in range(k):
            piv = max(range(c, k), key=lambda r: abs(A[r][c])); A[c], A[piv] = A[piv], A[c]; B[c], B[piv] = B[piv], B[c]
            for r in range(k):
                if r != c and A[c][c]:
                    f = A[r][c] / A[c][c]; A[r] = [A[r][j] - f * A[c][j] for j in range(k)]; B[r] -= f * B[c]
        w = [B[j] / A[j][j] for j in range(k)]
        preds.append((s, w[0] + sum(wj * xj for wj, xj in zip(w[1:], x)), y))
    r = st.correlation([p for _, p, _ in preds], [y for _, _, y in preds]); mae = st.mean(abs(p - y) for _, p, y in preds)
    tp = sum(1 for s, p, y in preds if BAD[s] and p < 95); fp = sum(1 for s, p, y in preds if not BAD[s] and p < 95)
    a = auc([(1 if BAD[s] else 0, -p) for s, p, _ in preds])
    return len(preds), r, mae, tp, sum(BAD[s] for s, _, _ in preds), fp, a

P(f'Outcome: 36 winters, drought = Oct-May rain <= {thr:.0f}% of normal ({sum(BAD.values())} droughts). r needed: 0.33 (p<.05); ~0.45 after correcting for the 12 sea tests here.')
P('Sea temperature = anomaly vs 1991-2020 normal; "detrended" removes the warming trend (otherwise a warming sea + a drying trend can fake a link).\n')
P(f'{"sea, months":52s}  r with winter rain   drought AUC (cold=dry? / warm=dry?)   2026 value')
ONIs = {'Jul-Sep (known early Oct)': {s: ONI[('JAS', s)] for s in WINTERS if ('JAS', s) in ONI},
        'Sep-Nov (known early Dec)': {s: ONI[('SON', s)] for s in WINTERS if ('SON', s) in ONI}}
for tl, months in TIMES.items():
    o = ONIs[tl]; ss = [s for s in WINTERS if s in o]
    P(f'  {"El Nino index (reference), " + tl:50s}  r {st.correlation([o[s] for s in ss], [R[s] for s in ss]):+.2f}')
    for name, d in SST.items():
        for dt in (False, True):
            a = anomaly(d, months, dt); ss = [s for s in WINTERS if s in a]
            r = st.correlation([a[s] for s in ss], [R[s] for s in ss])
            au = auc([(1 if BAD[s] else 0, -a[s]) for s in ss])
            now = a.get(2026) if 2026 in a else None
            P(f'  {name + ", " + tl + (" detrended" if dt else ""):50s}  r {r:+.2f}{"*" if abs(r) >= 0.33 else " "}              {au:.2f} / {1-au:.2f}' + (f'      {now:+.2f}' if now is not None else ''))
    P()
P('Leave-one-winter-out rain forecast; warn if predicted < 95% of normal (same rule as the El Nino test):')
P(f'  {"inputs":60s} n   r     avg error  droughts caught  false alarms  drought AUC')
for tl, months in TIMES.items():
    o = ONIs[tl]
    rows = [('El Nino only, ' + tl, [o])]
    for name, d in SST.items():
        rows.append((f'El Nino + {name} (detrended), {tl}', [o, anomaly(d, months, True)]))
        rows.append((f'{name} alone (detrended), {tl}', [anomaly(d, months, True)]))
    for lab, fs in rows:
        n, r, mae, tp, nb, fp, a = loo(fs)
        P(f'  {lab:60s} {n:2d}  {r:+.2f}   {mae:4.0f} pts     {tp}/{nb}             {fp}/{n - nb}         {a:.2f}')
    P()
open(os.path.join(HERE, 'sea_temp_test_output.txt'), 'w').write('\n'.join(lines) + '\n')
