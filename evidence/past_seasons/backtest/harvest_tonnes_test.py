"""Can the satellite estimate Kurdistan's wheat harvest in TONNES (per governorate and region), weeks before harvest?
Truth: KRSO official wheat production 2000–2023. Satellite: MODIS late-spring greenness (Mar 22–May 9, known ~mid-May)
and the mid-April version (pictures to Apr 6). Leave-one-year-out linear regression. Also estimates 2024, 2025, 2026 (no official numbers yet).
Usage: python3 -I harvest_tonnes_test.py zones.json ../backtest_data/modis ../backtest_data/modis_full ../backtest_data/harvest_stats/krso_wheat_barley_governorates.csv out_deep"""
import sys, os, json, csv, statistics as st
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from deep_investigation import load_modis
ZF, MD, MF, HVF, OUTD = sys.argv[1:6]
zones = [z['name'] for z in json.load(open(ZF))['zones']]
GOV = {'Sulaymaniyah': ['Slemani', 'Bazian', 'Chamchamal', 'Penjwen', 'Sharazur', 'Qaradagh', 'Ranya', 'Halabja', 'Garmiyan'],
       'Erbil': ['Erbil', 'Koya', 'Soran', 'Makhmour'], 'Duhok': ['Duhok', 'Zakho', 'Akre']}
PROD = {}
for r in csv.DictReader(open(HVF)):
    if r['crop'] == 'wheat' and r['system'] in ('all', 'all_derived'):
        try: PROD.setdefault(r['governorate'], {})[int(r['harvest_year'])] = float(r['production'])
        except ValueError: pass
# Sulaymaniyah lost Garmiyan (2013+) and Halabja (2017+) in the KRSO table → add them back so the area is constant
for y in range(2013, 2024):
    for extra in ('Garmiyan', 'Halabja'):
        if y in PROD.get(extra, {}) and y in PROD['Sulaymaniyah'] and not (extra == 'Halabja' and y < 2017): PROD['Sulaymaniyah'][y] += PROD[extra][y]
def series(z):
    m = load_modis(MD, z)
    j = f'{MF}/{z}_2026_a.json'
    if os.path.exists(j):
        for c in json.load(open(j)).get('subset', []):
            m.setdefault(2026, {})[int(c['modis_date'][5:])] = [v * 0.0001 if (v is not None and v > -2000) else None for v in c['data']]
    years = sorted(y for y in m if y <= 2025); npx = max(len(v) for y in years for v in m[y].values()); pk = {}
    for p in range(npx):
        peaks = [max(v) for y in years for v in [[m[y][d][p] for d in m[y] if 65 <= d <= 129 and p < len(m[y][d]) and m[y][d][p] is not None]] if v]
        if peaks: pk[p] = st.median(peaks)
    px = [p for p, v in pk.items() if 0.30 <= v <= 0.75] or list(pk)
    out = {}
    for y in m:
        def mean_doys(ds):
            vals = []
            for d in ds:
                if d in m[y]:
                    vv = [m[y][d][p] for p in px if p < len(m[y][d]) and m[y][d][p] is not None]
                    if len(vv) >= max(10, len(px) // 4): vals.append(st.mean(vv))
            return st.mean(vals) if len(vals) >= 2 else None
        out[y] = dict(late=mean_doys([81, 97, 113, 129]), april=mean_doys([49, 65, 81]))
    return out
S = {z: series(z) for z in zones}
def gov_val(g, y, k):
    v = [S[z][y][k] for z in GOV[g] if y in S[z] and S[z][y][k] is not None]; return st.mean(v) if len(v) == len(GOV[g]) else None
def loo(pts):
    """pts = [(year, x, y)], y = tonnes. Leave-one-year-out simple linear regression. Returns [(year, pred, true)]."""
    out = []
    for i, (yr, x, y) in enumerate(pts):
        tr = [p for j, p in enumerate(pts) if j != i]; mx = st.mean(p[1] for p in tr); my = st.mean(p[2] for p in tr)
        b = sum((p[1] - mx) * (p[2] - my) for p in tr) / sum((p[1] - mx) ** 2 for p in tr); out.append((yr, my + b * (x - mx), y))
    return out
def fit_all(pts):
    mx = st.mean(p[1] for p in pts); my = st.mean(p[2] for p in pts); b = sum((p[1] - mx) * (p[2] - my) for p in pts) / sum((p[1] - mx) ** 2 for p in pts)
    return lambda x: my + b * (x - mx)
if __name__ == '__main__':
    YRS = range(2000, 2024); res = {}
    for k, lab in (('april', 'mid-April (pictures to Apr 6, ~5 weeks before harvest)'), ('late', 'mid-May (full spring)')):
        print(f'\n=== {lab} ===')
        print('  area           | r     | avg error (LOO) | baseline "average of last 5 years" error | 2024 est | 2025 est | 2026 est')
        tot_pred = {}
        for g in list(GOV) + ['Kurdistan_Region']:
            if g == 'Kurdistan_Region': pts = [(y, st.mean(S[z][y][k] for z in zones), PROD[g][y]) for y in YRS if y in PROD[g] and all(S[z].get(y, {}).get(k) is not None for z in zones)]
            else: pts = [(y, gov_val(g, y, k), PROD[g][y]) for y in YRS if y in PROD[g] and gov_val(g, y, k) is not None]
            o = loo(pts); r = st.correlation([p[1] for p in pts], [p[2] for p in pts])
            err = st.mean(abs(p - t) / t for _, p, t in o); allp = {y: t for y, _, t in pts}
            base = [abs(st.mean(PROD[g][yy] for yy in range(y - 5, y) if yy in PROD[g]) - allp[y]) / allp[y] for y in allp if sum(1 for yy in range(y - 5, y) if yy in PROD[g]) >= 3]
            f = fit_all(pts)
            est = {}
            for y in (2024, 2025, 2026):
                x = st.mean(S[z][y][k] for z in zones) if g == 'Kurdistan_Region' and all(S[z].get(y, {}).get(k) is not None for z in zones) else (gov_val(g, y, k) if g != 'Kurdistan_Region' else None)
                est[y] = f(x) if x is not None else None
            res[f'{k}|{g}'] = dict(r=r, err=err, base=st.mean(base), est=est, loo=o)
            print(f'  {g:15s}| {r:+.2f} | {err:5.0%}            | {st.mean(base):5.0%}                                    | ' + ' | '.join(f'{est[y]/1000:6.0f}k' if est[y] else '   n/a ' for y in (2024, 2025, 2026)))
        o = res[f'{k}|Kurdistan_Region']['loo']
        print('  Kurdistan, year by year (official → satellite estimate, year hidden): ' + ', '.join(f'{y}: {t/1000:.0f}k→{p/1000:.0f}k' for y, p, t in o))
    json.dump({k: {kk: vv for kk, vv in v.items() if kk != 'loo'} for k, v in res.items()}, open(f'{OUTD}/harvest_tonnes.json', 'w'), indent=1, default=str)
