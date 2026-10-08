"""How sure is the warning by mid-April (graze-or-harvest decision)? Uses rain etc. to Apr 15 and satellite composites published by ~Apr 15 (Feb 2 … Apr 6).
Note: the Mar 22–Apr 6 composite is also the first part of the outcome window, so this is a 'confirmation', not a forecast.
Usage: python3 -I april_test.py zones.json ../backtest_data/openmeteo ../backtest_data/modis ../backtest_data/enso/oni.ascii.txt out_deep"""
import sys, os, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import deep_investigation as D
from deep_investigation import *
if __name__ == '__main__':
    ZF, OM, MD, ONIF, OUTD = sys.argv[1:6]
    D.CUTS = {'Apr15': (4, 15)}; D.ONI_AT = {'Apr15': ('JFM', 1)}; D.GREEN_HONEST = {'Apr15': [33, 49, 65, 81]}; D.GREEN_OLD = {}
    rows = D.build(ZF, OM, MD, ONIF)
    f = ['rain_pct', 'soil_pct', 'temp_anom', 'et0_pct', 'last_rain_pct', 'green_now', 'oni']; out = []
    for s in SEASONS:
        tr = [r for r in rows if r['season'] != s and r['label'] is not None]; te = [r for r in rows if r['season'] == s and r['label'] is not None]
        Xtr, mn = matrix(tr, f); Xte, _ = matrix(te, f, mn); m = lr_fit(Xtr, [r['label'] for r in tr]); out += [(r['label'], lr_pred(m, x)) for r, x in zip(te, Xte)]
    ct, fa = catch_at(out); npos = sum(l for l, _ in out); nneg = len(out) - npos
    g = lambda t: (sum(1 for l, p in out if l and p >= t) / npos, sum(1 for l, p in out if not l and p >= t) / nneg, sum(1 for l, p in out if (p >= t) == bool(l)) / len(out))
    a, b = g(0.35), g(0.5)
    print(f'Mid-April (Apr 15): AUC {auc(out):.2f}, catch@15%FA {ct:.0%}; sensitive ≥35%: caught {a[0]:.0%} / false alarms {a[1]:.0%}; cautious ≥50%: caught {b[0]:.0%} / false alarms {b[1]:.0%} / right {b[2]:.0%}')
