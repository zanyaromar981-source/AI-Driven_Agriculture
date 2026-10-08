"""Investigation 4: do December / January satellite pictures (did the crop come up?) improve the Dec/Jan/Feb warnings?
Composites used only if published by the cutoff: Dec 31 → Nov 17–Dec 2 (321) + Dec 3–18 (337); Jan 31 → + Dec 19–Jan 3 (353) + Jan 1–16 (001); Feb 28 → + Jan 17–Feb 1 (017).
Usage: python3 -I winter_green_test.py zones.json ../backtest_data/modis ../backtest_data/modis_full out_deep"""
import sys, os, json, statistics as st
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from deep_investigation import *
ZF, MD, MF, OUTD = sys.argv[1:5]
zones = [z['name'] for z in json.load(open(ZF))['zones']]; rows = json.load(open(f'{OUTD}/features_all.json'))
R = {(r['zone'], r['season'], r['cut']): r for r in rows}
def crop_pixels(m):
    years = sorted(m); npx = max(len(v) for y in years for v in m[y].values()); pk = {}
    for p in range(npx):
        peaks = [max(v) for y in years for v in [[m[y][d][p] for d in m[y] if 65 <= d <= 129 and p < len(m[y][d]) and m[y][d][p] is not None]] if v]
        if peaks: pk[p] = st.median(peaks)
    px = [p for p, v in pk.items() if 0.30 <= v <= 0.75]
    return px if len(px) >= 50 else list(pk)
def load_full(z):
    out = {}
    for fn in os.listdir(MF):
        if not fn.startswith(z + '_') or not (fn.endswith('_jan.json') or fn.endswith('_autumn.json')): continue
        try: j = json.load(open(f'{MF}/{fn}'))
        except Exception: continue
        for c in j.get('subset', []):
            out[(int(c['modis_date'][1:5]), int(c['modis_date'][5:]))] = [v * 0.0001 if (v is not None and v > -2000) else None for v in c['data']]
    return out
CUTDOYS = {'Dec': [(0, 321), (0, 337)], 'Jan': [(0, 321), (0, 337), (0, 353), (1, 1)], 'Feb': [(0, 337), (0, 353), (1, 1), (1, 17)]}
if __name__ == '__main__':
    cov = {}
    for z in zones:
        px = crop_pixels(load_modis(MD, z)); F = load_full(z); ser = {}
        for k, vals in F.items():
            vv = [vals[p] for p in px if p < len(vals) and vals[p] is not None]
            if len(vv) >= max(10, len(px) // 4): ser[k] = st.mean(vv)
        clim = {d: st.median([v for (y, dd), v in ser.items() if dd == d]) for d in (289, 305, 321, 337, 353, 1, 17) if any(dd == d for (y, dd) in ser)}
        cov[z] = len(ser)
        for s in SEASONS + [2025]:
            for cut, doys in CUTDOYS.items():
                r = R.get((z, s, cut))
                if not r: continue
                a = [ser[(s + yo, d)] / clim[d] * 100 for yo, d in doys if (s + yo, d) in ser and d in clim]
                r['wgreen_anom'] = st.mean(a) if a else None
                last = [(yo, d) for yo, d in doys if (s + yo, d) in ser and d in clim]
                r['wgreen_latest'] = ser[(s + last[-1][0], last[-1][1])] / clim[last[-1][1]] * 100 if last else None
                if (s, 305) in ser and last: r['wgreen_rise'] = (ser[(s + last[-1][0], last[-1][1])] - ser[(s, 305)]) * 100   # emergence since early Nov
    print('winter composites per zone: ' + ', '.join(f'{z} {n}' for z, n in cov.items()))
    BASE = ['rain_pct', 'soil_pct', 'temp_anom', 'et0_pct', 'last_rain_pct']
    def lr_cv(Rc, feats):
        out = []
        for s in SEASONS:
            tr = [r for r in Rc if r['season'] != s and r['label'] is not None]; te = [r for r in Rc if r['season'] == s and r['label'] is not None]
            Xtr, mn = matrix(tr, feats); Xte, _ = matrix(te, feats, mn); m = lr_fit(Xtr, [r['label'] for r in tr])
            out += [(s, r['zone'], r['label'], lr_pred(m, x)) for r, x in zip(te, Xte)]
        return out
    def sc(o, ref=None):
        pr = [(l, p) for _, _, l, p in o]; ct, _ = catch_at(pr)
        return f'AUC {auc(pr):.2f} catch@15%FA {ct:4.0%} Brier {brier_skill(pr):+.2f}' + (f'  beats ref in {boot_better(o, ref):.0%}' if ref else '')
    for c in ('Dec', 'Jan', 'Feb'):
        Rc = [r for r in rows if r['cut'] == c]; b = BASE + (['green_now'] if c == 'Feb' else []) + ['oni']; ref = lr_cv(Rc, b)
        n = sum(1 for r in Rc if r.get('wgreen_anom') is not None and r['label'] is not None)
        print(f'{c}: reference LR+ElNiño {sc(ref)}   (rows with winter pictures: {n})')
        for nm, f in [('+ winter greenness vs same date', ['wgreen_anom']), ('+ latest winter picture', ['wgreen_latest']),
                      ('+ crop emergence since early Nov', ['wgreen_rise']), ('+ all three', ['wgreen_anom', 'wgreen_latest', 'wgreen_rise'])]:
            print(f'    {nm:36s} {sc(lr_cv(Rc, b + f), ref)}')
    json.dump(rows, open(f'{OUTD}/features_all_winter.json', 'w'))
