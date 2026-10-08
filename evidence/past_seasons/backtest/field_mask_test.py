"""Does a cleaner 'real crop field' pixel selection make the outcome more predictable?
Current filter: pixels whose median spring peak is 0.30–0.75 (includes pasture, orchards, scrub).
Crop-like filter: also must DRY DOWN by late May (peak − late-May value ≥ 0.12, as harvested/ripening grain does) and be low in early Feb (< 0.40, not evergreen).
Usage: python3 -I field_mask_test.py zones.json ../backtest_data/openmeteo ../backtest_data/modis ../backtest_data/enso/oni.ascii.txt out_deep"""
import sys, os, json, statistics as st
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import deep_investigation as D
from deep_investigation import *
ZF, OM, MD, ONIF, OUTD = sys.argv[1:6]
INFO = {}
def zone_series_crop(m, drop=0.12, feb_max=0.40):
    years = sorted(m); npx = max(len(v) for y in years for v in m[y].values()); keep = []
    for p in range(npx):
        pk, dr, fb = [], [], []
        for y in years:
            g = lambda d: m[y][d][p] if d in m[y] and p < len(m[y][d]) else None
            vals = [g(d) for d in (65, 81, 97, 113, 129) if g(d) is not None]
            if vals:
                pk.append(max(vals))
                if g(145) is not None: dr.append(max(vals) - g(145))
                if g(33) is not None: fb.append(g(33))
        if pk and dr and fb and 0.30 <= st.median(pk) <= 0.75 and st.median(dr) >= drop and st.median(fb) < feb_max: keep.append(p)
    INFO.setdefault('n', []).append((len(keep), npx))
    if len(keep) < 40: return None
    ser = {}
    for y in years:
        for d, vals in m[y].items():
            vv = [vals[p] for p in keep if p < len(vals) and vals[p] is not None]
            if len(vv) >= max(10, len(keep) // 4): ser[(y, d)] = st.mean(vv)
    return ser
def lr_cv(Rc, feats):
    out = []
    for s in SEASONS:
        tr = [r for r in Rc if r['season'] != s and r['label'] is not None]; te = [r for r in Rc if r['season'] == s and r['label'] is not None]
        Xtr, mn = matrix(tr, feats); Xte, _ = matrix(te, feats, mn); m = lr_fit(Xtr, [r['label'] for r in tr])
        out += [(s, r['zone'], r['label'], lr_pred(m, x)) for r, x in zip(te, Xte)]
    return out
if __name__ == '__main__':
    old = json.load(open(f'{OUTD}/features_all.json'))
    orig = D.zone_series
    def patched(m):
        s = zone_series_crop(m); return s if s is not None else orig(m)
    D.zone_series = patched
    new = D.build(ZF, OM, MD, ONIF)
    zones = [z['name'] for z in json.load(open(ZF))['zones']]
    print('crop-like pixels kept per zone (of 1089): ' + ', '.join(f'{z} {n}' for z, (n, t) in zip(zones, INFO['n'])))
    BASE = ['rain_pct', 'soil_pct', 'temp_anom', 'et0_pct', 'last_rain_pct']
    for c in ('Pre', 'Dec', 'Jan', 'Feb', 'Mar'):
        f = ['oni'] if c == 'Pre' else BASE + (['green_now'] if c in GREEN_HONEST else []) + ['oni']
        a = lr_cv([r for r in old if r['cut'] == c], f); b = lr_cv([r for r in new if r['cut'] == c], f)
        A = auc([(l, p) for _, _, l, p in a]); B = auc([(l, p) for _, _, l, p in b])
        ca, _ = catch_at([(l, p) for _, _, l, p in a]); cb, _ = catch_at([(l, p) for _, _, l, p in b])
        print(f'  {c}: current fields AUC {A:.2f} catch {ca:4.0%}  →  crop-like fields AUC {B:.2f} catch {cb:4.0%}')
    # how often do the two outcome definitions agree?
    o = {(r['zone'], r['season']): r['label'] for r in old if r['cut'] == 'Dec' and r['label'] is not None}
    n_ = {(r['zone'], r['season']): r['label'] for r in new if r['cut'] == 'Dec' and r['label'] is not None}
    k = [x for x in o if x in n_]; print(f'  outcome agreement current vs crop-like: {sum(o[x] == n_[x] for x in k) / len(k):.0%} of {len(k)} zone-seasons; bad in both: {sum(o[x] and n_[x] for x in k)}, only current: {sum(o[x] and not n_[x] for x in k)}, only crop-like: {sum(n_[x] and not o[x] for x in k)}')
    json.dump(new, open(f'{OUTD}/features_cropmask.json', 'w'))
