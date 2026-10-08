"""Can we predict in winter how full Dukan / Darbandikhan will be in summer?
Truth: lake area from space (Landsat 1988–2016, Sentinel-2 2017–2026). Inputs known at each forecast date:
lake area last October (measured from space), El Niño (ONI Jul–Sep), catchment rain so far (CHIRPS, 6 points per catchment incl. Iran).
Leave-one-year-out linear regression vs simple baselines.
Usage: python3 -I dam_forecast_test.py dam_history_landsat.csv dam_water_areas_window_medians.json ../backtest_data/catchment_rain/chirps_catchment_monthly.csv ../backtest_data/enso/oni.ascii.txt"""
import sys, csv, json, statistics as st
sys.path.insert(0, __import__('os').path.dirname(__import__('os').path.abspath(__file__)))
from deep_investigation import solve
LS, S2, CH, ONIF = sys.argv[1:5]
FULL = {'Dukan': 270.0, 'Darbandikhan': 113.0}; PREFIX = {'Dukan': 'Dukan__', 'Darbandikhan': 'Darb__'}
A = {}                                                    # (lake, window, year) -> km²
tmp = {}
for r in csv.DictReader(open(LS)):
    if r['clear'] == '1' and r['water_km2']:
        v = float(r['water_km2'])
        if 10 <= v <= FULL[r['lake']] * 1.1: tmp.setdefault((r['lake'], r['window'], int(r['year'])), []).append(v)
for k, v in tmp.items(): A[k] = st.median(v)
for k, v in json.load(open(S2)).items():
    if k == 'jrc': continue
    lake, y, w = k.split('|'); A[(lake, 'June' if w.startswith('late') else 'October', int(y))] = v
RAIN = {}
for r in csv.DictReader(open(CH)):
    for lake, p in PREFIX.items():
        if r['zone'].startswith(p): RAIN.setdefault((lake, int(r['year']), int(r['month'])), []).append(float(r['precip_mm']))
RAIN = {k: st.mean(v) for k, v in RAIN.items()}
ONI = {}
for l in open(ONIF).read().splitlines()[1:]:
    a, y, t, an = l.split(); ONI[(a, int(y))] = float(an)
def rain_to(lake, y, last_month):       # season Oct (y-1) .. last_month of y
    ms = [(y - 1, 10), (y - 1, 11), (y - 1, 12)] + [(y, m) for m in range(1, last_month + 1)]
    v = [RAIN.get((lake, a, b)) for a, b in ms]; return sum(v) if None not in v else None
def norm(lake, last_month): return st.mean(rain_to(lake, y, last_month) for y in range(1992, 2021))
def loo(rows):
    out = []
    for i, (y, x, t) in enumerate(rows):
        tr = [r for j, r in enumerate(rows) if j != i]; k = len(x) + 1
        M = [[0.0] * k for _ in range(k)]; B = [0.0] * k
        for _, xx, tt in tr:
            v = [1.0] + xx
            for a in range(k):
                B[a] += v[a] * tt
                for b in range(k): M[a][b] += v[a] * v[b] + (1e-6 if a == b else 0)
        w = solve(M, B); out.append((y, w[0] + sum(wi * xi for wi, xi in zip(w[1:], x)), t))
    return out
if __name__ == '__main__':
    for lake in ('Dukan', 'Darbandikhan'):
        for target in ('June', 'October'):
            print(f'\n=== {lake}: predict {target} lake area (full = {FULL[lake]:.0f} km²) ===')
            tm = 6 if target == 'June' else 10
            stages = [('early Nov (last Oct lake + El Niño)', None), ('end of January (+ rain Oct–Jan)', 1), ('end of March (+ rain Oct–Mar)', 3)]
            if target == 'October': stages.append(('end of May (+ rain Oct–May)', 5))
            years = [y for y in range(1989, 2027) if (lake, target, y) in A and (lake, 'October', y - 1) in A and ('JAS', y - 1) in ONI]
            truth = {y: A[(lake, target, y)] for y in years}
            lowline = sorted(truth.values())[len(truth) // 4]
            base_clim = [abs(st.mean(v for yy, v in truth.items() if yy != y) - truth[y]) for y in years]
            base_last = [abs(A[(lake, target, y - 1)] - truth[y]) for y in years if (lake, target, y - 1) in A]
            print(f'  {len(years)} years {years[0]}–{years[-1]}; low-water summer = bottom 25% (≤ {lowline:.0f} km², {lowline/FULL[lake]:.0%} of full)')
            print(f'  baseline "average year": avg error {st.mean(base_clim):.0f} km² | baseline "same as last year": {st.mean(base_last):.0f} km²')
            for lab, m in stages:
                rows = []
                for y in years:
                    x = [A[(lake, 'October', y - 1)], ONI[('JAS', y - 1)]]
                    if m:
                        r = rain_to(lake, y, m)
                        if r is None: break
                        x.append(r / norm(lake, m) * 100)
                    rows.append((y, x, truth[y]))
                if len(rows) < len(years) * 0.8: print(f'  {lab}: not enough rain data'); continue
                o = loo(rows); err = [abs(p - t) for _, p, t in o]
                lows = [(y, p, t) for y, p, t in o if t <= lowline]; caught = sum(1 for _, p, t in lows if p <= lowline * 1.1)
                fa = sum(1 for _, p, t in o if t > lowline and p <= lowline * 1.1)
                r = st.correlation([p for _, p, _ in o], [t for _, _, t in o])
                print(f'  {lab:38s} avg error {st.mean(err):4.0f} km² ({st.mean(err)/FULL[lake]:.0%} of full), r {r:+.2f}, low summers caught {caught}/{len(lows)}, false alarms {fa}/{len(o)-len(lows)}')
                if lab.startswith('end of March') or (target == 'June' and lab.startswith('end of March')):
                    print('     years (true → predicted): ' + ', '.join(f'{y}:{t:.0f}→{p:.0f}' for y, p, t in o if y >= 2017 or t <= lowline))
            # forecast for 2027 from what we know today (Oct 2026 lake + El Niño)
            rows = [(y, [A[(lake, 'October', y - 1)], ONI[('JAS', y - 1)]], truth[y]) for y in years]
            k = 3; M = [[0.0] * k for _ in range(k)]; B = [0.0] * k
            for _, xx, tt in rows:
                v = [1.0] + xx
                for a in range(k):
                    B[a] += v[a] * tt
                    for b in range(k): M[a][b] += v[a] * v[b] + (1e-6 if a == b else 0)
            w = solve(M, B)
            if (lake, 'October', 2026) in A:
                p = w[0] + w[1] * A[(lake, 'October', 2026)] + w[2] * ONI[('JAS', 2026)]
                print(f'  → TODAY\'s forecast for {target} 2027: {p:.0f} km² ({p/FULL[lake]:.0%} of full) [Oct 2026 lake {A[(lake, "October", 2026)]:.0f} km², ONI +2.16; weights: last-Oct lake {w[1]:+.2f}, ONI {w[2]:+.1f} km² per unit]')
