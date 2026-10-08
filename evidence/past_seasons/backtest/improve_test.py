"""Investigation 2: what else raises accuracy? Model upgrades on existing data + new inputs when available.
Usage: python3 -I improve_test.py zones.json ../backtest_data/openmeteo ../backtest_data/modis out_deep [../backtest_data/openmeteo_extra]"""
import sys, os, json, math, statistics as st
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from deep_investigation import *
BASE = ['rain_pct', 'soil_pct', 'temp_anom', 'et0_pct', 'last_rain_pct']
def fs_old(c, kind='enso'): return BASE + (['green_now'] if c in GREEN_HONEST else []) + ['oni']
from multiprocessing import Pool
ZF, OM, MD, OUTD = sys.argv[1:5]; EX = sys.argv[5] if len(sys.argv) > 5 else None
rows = json.load(open(f'{OUTD}/features_all.json')); zones = [z['name'] for z in json.load(open(ZF))['zones']]
R = {(r['zone'], r['season'], r['cut']): r for r in rows}
def Phi(x): return 0.5 * (1 + math.erf(x / math.sqrt(2)))

def add_green_features():
    """green_anom = greenness vs the SAME date in other years (not vs the late-spring median); green_change = Mar 6 minus Feb 2 composite."""
    for z in zones:
        ser = zone_series(load_modis(MD, z))
        clim = {d: st.median(v for (y, dd), v in ser.items() if dd == d) for d in (33, 49, 65)}
        for s in SEASONS + [2025]:
            for cut, doys in (('Feb', [33]), ('Mar', [33, 49, 65])):
                r = R.get((z, s, cut))
                if not r: continue
                g = {d: ser.get((s + 1, d)) for d in doys}
                a = [g[d] / clim[d] * 100 for d in doys if g[d] is not None]
                r['green_anom'] = st.mean(a) if a else None
                r['green_anom_latest'] = g[doys[-1]] / clim[doys[-1]] * 100 if g[doys[-1]] is not None else None
                if cut == 'Mar' and g[65] is not None and g[33] is not None: r['green_change'] = (g[65] - g[33]) * 100
def add_zone_dryness():
    for z in zones:
        w = W(f'{OM}/{z}.json'); mm = st.mean(w.sum(ymd(s, 10, 1), ymd(s + 1, 5, 31), 'precipitation_sum') for s in NORMAL)
        for r in rows:
            if r['zone'] == z:
                r['zone_normal_mm'] = mm
                if r.get('rain_pct') is not None: r['rain_x_dry'] = r['rain_pct'] * (1000 / mm)   # dry zones: rain shortfall hurts more
def add_extra_weather():
    """frost / heat / snow / shallow soil / dryness of air / sunshine, from openmeteo_extra (window Oct 1 → cutoff unless noted)."""
    have = [z for z in zones if os.path.exists(f'{EX}/{z}.json') and os.path.getsize(f'{EX}/{z}.json') > 1000]
    for z in have:
        w = W(f'{EX}/{z}.json')
        def raw(s, cut):
            a, b = ymd(s, 10, 1), cut_date(s, cut)
            tmin = w.rng(ymd(s + 1, 1, 1) if cut in ('Feb', 'Mar') else a, b, 'temperature_2m_min') or []
            tmax = w.rng(a, b, 'temperature_2m_max') or []
            sn = w.sum(a, b, 'snowfall_sum') or 0; rn = w.sum(a, b, 'rain_sum') or 0
            return dict(frost=sum(1 for x in tmin if x < -3), warm=sum(1 for x in tmax if x > 20),
                        snow=sn, snow_share=sn * 7 / max(1, sn * 7 + rn),        # snowfall cm → ~mm water /7
                        sm7=st.mean(w.back(b, 7, 'soil_moisture_0_to_7cm_mean')), sm28=st.mean(w.back(b, 7, 'soil_moisture_7_to_28cm_mean')),
                        vpd=w.mean(a, b, 'vapour_pressure_deficit_max'), sun=w.mean(ymd(s + 1, 1, 1) if cut in ('Feb', 'Mar') else a, b, 'shortwave_radiation_sum'),
                        phours=w.sum(a, b, 'precipitation_hours'))
        for cut in ('Dec', 'Jan', 'Feb', 'Mar'):
            NR = [raw(s, cut) for s in NORMAL]; N = {k: st.mean(x[k] for x in NR) for k in NR[0]}
            for s in SEASONS + [2025]:
                r = R.get((z, s, cut))
                if not r: continue
                x = raw(s, cut)
                r.update(frost_anom=x['frost'] - N['frost'], warm_anom=x['warm'] - N['warm'], snow_share=x['snow_share'] - N['snow_share'],
                         sm7_pct=x['sm7'] / N['sm7'] * 100, sm28_pct=x['sm28'] / N['sm28'] * 100, vpd_pct=x['vpd'] / N['vpd'] * 100,
                         sun_pct=x['sun'] / N['sun'] * 100, phours_pct=x['phours'] / max(1, N['phours']) * 100)
    return len(have)

def lr_cv(Rc, feats):
    out = []
    for s in SEASONS:
        tr = [r for r in Rc if r['season'] != s and r['label'] is not None]; te = [r for r in Rc if r['season'] == s and r['label'] is not None]
        Xtr, mn = matrix(tr, feats); Xte, _ = matrix(te, feats, mn); m = lr_fit(Xtr, [r['label'] for r in tr])
        out += [(s, r['zone'], r['label'], lr_pred(m, x)) for r, x in zip(te, Xte)]
    return out
def ridge_fit(X, y, lam=1.0):
    n, k = len(X), len(X[0]); mu, sd = stdz(X); Z = [[1.0] + [(x[j] - mu[j]) / sd[j] for j in range(k)] for x in X]
    A = [[sum(z[a] * z[b] for z in Z) + (lam if a == b and a > 0 else 0) for b in range(k + 1)] for a in range(k + 1)]
    B = [sum(z[a] * yy for z, yy in zip(Z, y)) for a in range(k + 1)]; w = solve(A, B)
    res = [yy - sum(a * b for a, b in zip(w, z)) for z, yy in zip(Z, y)]
    return dict(w=w, mu=mu, sd=sd, sig=st.pstdev(res))
def ridge_pred(m, x): return m['w'][0] + sum(m['w'][j + 1] * (x[j] - m['mu'][j]) / m['sd'][j] for j in range(len(x)))
THR = {}
for r in rows:
    if r['label'] == 1 and r['outcome'] is not None: THR[r['zone']] = max(THR.get(r['zone'], 0), r['outcome'])
def reg_cv(Rc, feats):
    """predict the size of the harvest greenness (% of zone median), then risk = chance it falls below the zone's bad line."""
    out = []
    for s in SEASONS:
        tr = [r for r in Rc if r['season'] != s and r['outcome'] is not None]; te = [r for r in Rc if r['season'] == s and r['label'] is not None]
        Xtr, mn = matrix(tr, feats); Xte, _ = matrix(te, feats, mn); m = ridge_fit(Xtr, [r['outcome'] for r in tr], lam=len(tr) * 0.05)
        out += [(s, r['zone'], r['label'], Phi((THR[r['zone']] - ridge_pred(m, x)) / m['sig'])) for r, x in zip(te, Xte)]
    return out
def score(o, ref=None):
    pr = [(l, p) for _, _, l, p in o]; ct, fa = catch_at(pr)
    return f'AUC {auc(pr):.2f}  catch@15%FA {ct:4.0%}  Brier {brier_skill(pr):+.2f}' + (f'  beats LR+ElNiño in {boot_better(o, ref):.0%}' if ref else '')

if __name__ == '__main__':
    add_green_features(); add_zone_dryness(); nex = add_extra_weather() if EX else 0
    print(f'extra weather available for {nex}/16 zones' + (' (extra-weather rows use only these zones; reference recomputed on the same zones)' if 0 < nex < 16 else ''))
    TIMEX = ['frost_anom', 'warm_anom', 'snow_share', 'sm7_pct', 'sm28_pct', 'vpd_pct', 'sun_pct', 'phours_pct']
    for c in ('Dec', 'Jan', 'Feb', 'Mar'):
        Rc = [r for r in rows if r['cut'] == c]; base = fs_old(c, 'enso'); ref = lr_cv(Rc, base)
        print(f'\n=== {c} ===   reference LR+ElNiño: {score(ref)}')
        tests = []
        if c in ('Feb', 'Mar'):
            gA = [f for f in base if f != 'green_now'] + ['green_anom']
            tests += [('satellite vs same date (anomaly) instead of vs late median', gA, 'LR'),
                      ('both satellite measures', base + ['green_anom'], 'LR'),
                      ('latest image anomaly only', [f for f in base if f != 'green_now'] + ['green_anom_latest'], 'LR')]
            if c == 'Mar': tests.append(('+ greening speed Feb→Mar', base + ['green_change'], 'LR'))
        tests += [('+ zone dryness × rain', base + ['zone_normal_mm', 'rain_x_dry'], 'LR'),
                  ('predict harvest size (regression), same inputs', base, 'REG')]
        if c in ('Feb', 'Mar'): tests.append(('predict harvest size + satellite anomaly', base + ['green_anom'], 'REG'))
        for nm, f, kind in tests:
            o = lr_cv(Rc, f) if kind == 'LR' else reg_cv(Rc, f); print(f'  {nm:58s} {score(o, ref)}')
        if nex:
            Rz = [r for r in Rc if r.get('sm7_pct') is not None]; refz = lr_cv(Rz, base)
            print(f'  -- new weather inputs ({nex} zones; reference on same zones: {score(refz)})')
            for nm, f in [('+ frost days', ['frost_anom']), ('+ warm days (>20°C)', ['warm_anom']), ('+ snow share', ['snow_share']),
                          ('+ shallow soil 0–7 & 7–28 cm', ['sm7_pct', 'sm28_pct']), ('+ air dryness (VPD)', ['vpd_pct']),
                          ('+ sunshine', ['sun_pct']), ('+ rain hours', ['phours_pct']), ('+ all new weather', TIMEX)]:
                o = lr_cv(Rz, base + f); print(f'     {nm:55s} {score(o, refz)}')
    json.dump(rows, open(f'{OUTD}/features_all_v2.json', 'w'))
