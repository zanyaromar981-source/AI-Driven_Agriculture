"""Do December-start seasonal model forecasts (issued ~Dec 5–10) improve the end-of-December and end-of-January crop warnings?
Feature = model forecast of Jan–Mar rain (leads 1–3 of the December start), standardized per model, averaged over models.
Usage: python3 -I seasonal_models_crop_test.py ../backtest_data/seasonal_models out_deep"""
import sys, os, json, statistics as st
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from seasonal_models_test import load_models, ALL_MODELS
from deep_investigation import lr_fit, lr_pred, matrix, auc, catch_at, brier_skill, boot_better, SEASONS
MDIR, OUTD = sys.argv[1:3]
fc = load_models(MDIR)
def zser(model, leads):
    v = {}
    for (y, m), f in fc.get(model, {}).items():
        if m == 12 and all(l in f for l in leads): v[y] = sum(f[l] for l in leads)
    if len(v) < 20: return {}
    mu, sd = st.mean(v.values()), st.pstdev(v.values()); return {y: (x - mu) / sd for y, x in v.items()}
FEAT = {}
for name, models in (('all 7 models', ALL_MODELS), ('best 3 (CanSIPS, ECMWF, NASA)', ['CanSIPS-IC4', 'ECMWF-SEAS5', 'NASA-GEOSS2S'])):
    for leads, lab in (([1, 2, 3], 'Jan–Mar'), ([0, 1, 2], 'Dec–Feb')):
        Z = [zser(m, leads) for m in models]; Z = [z for z in Z if z]
        FEAT[f'{name}, {lab}'] = {y: st.mean(z[y] for z in Z if y in z) for y in range(1991, 2027) if any(y in z for z in Z)}
BASE = ['rain_pct', 'soil_pct', 'temp_anom', 'et0_pct', 'last_rain_pct', 'oni']
def lr_cv(Rc, feats, seasons, key='label'):
    out = []
    for s in seasons:
        tr = [r for r in Rc if r['season'] != s and r['season'] in seasons and r.get(key) is not None]; te = [r for r in Rc if r['season'] == s and r.get(key) is not None]
        if not te: continue
        Xtr, mn = matrix(tr, feats); Xte, _ = matrix(te, feats, mn); m = lr_fit(Xtr, [r[key] for r in tr])
        out += [(s, r['zone'], r[key], lr_pred(m, x)) for r, x in zip(te, Xte)]
    return out
def sc(o, ref=None):
    pr = [(l, p) for _, _, l, p in o]; ct, _ = catch_at(pr)
    return f'AUC {auc(pr):.2f} catch@15%FA {ct:4.0%} Brier {brier_skill(pr):+.2f}' + (f'  beats ref in {boot_better(o, ref):.0%}' if ref else '')
if __name__ == '__main__':
    rows = json.load(open(f'{OUTD}/features_all.json'))
    print('A. Satellite-greenness outcome, 16 zones, 1999/00–2024/25')
    for c in ('Dec', 'Jan'):
        Rc = [r for r in rows if r['cut'] == c]; ref = lr_cv(Rc, BASE, SEASONS); print(f'  {c}: weather + El Niño {sc(ref)}')
        for k, v in FEAT.items():
            for r in Rc: r['sm'] = v.get(r['season'])
            print(f'     + {k:38s} {sc(lr_cv(Rc, BASE + ["sm"], SEASONS), ref)}')
    G = json.load(open(f'{OUTD}/governorate_rows.json')); S91 = list(range(1991, 2023))
    print('\nB. Official wheat harvest bad (governorate, 1991/92–2022/23)')
    for c in ('Dec', 'Jan'):
        Rc = [v for k, v in G.items() if v['cut'] == c]; ref = lr_cv(Rc, BASE, S91, 'wheat_bad'); print(f'  {c}: weather + El Niño {sc(ref)}')
        for k, v in FEAT.items():
            for r in Rc: r['sm'] = v.get(r['season'])
            print(f'     + {k:38s} {sc(lr_cv(Rc, BASE + ["sm"], S91, "wheat_bad"), ref)}')
    print('\n2026/27 is not testable yet (December 2026 forecasts come out ~Dec 8).')
