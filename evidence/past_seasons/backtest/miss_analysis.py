"""Where are the errors? Split bad zone-seasons into SEVERE (zone's worst 10%) and MILD (10–25%), and good ones into near-the-line vs clearly good.
Usage: python3 -I miss_analysis.py out_deep"""
import sys, os, json, statistics as st
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from deep_investigation import *
OUTD = sys.argv[1]; rows = json.load(open(f'{OUTD}/features_all.json'))
BASE = ['rain_pct', 'soil_pct', 'temp_anom', 'et0_pct', 'last_rain_pct']
def lr_cv(Rc, feats):
    out = []
    for s in SEASONS:
        tr = [r for r in Rc if r['season'] != s and r['label'] is not None]; te = [r for r in Rc if r['season'] == s and r['label'] is not None]
        Xtr, mn = matrix(tr, feats); Xte, _ = matrix(te, feats, mn); m = lr_fit(Xtr, [r['label'] for r in tr])
        out += [(r, lr_pred(m, x)) for r, x in zip(te, Xte)]
    return out
if __name__ == '__main__':
    pct = {}
    for z in {r['zone'] for r in rows}:
        v = sorted((r['outcome'], r['season']) for r in rows if r['zone'] == z and r['cut'] == 'Dec' and r['label'] is not None)
        for i, (o, s) in enumerate(v): pct[(z, s)] = i / (len(v) - 1) * 100       # 0 = worst season in this zone
    print('Warning at risk ≥35% (sensitive), LR + El Niño, hidden seasons')
    print('month | SEVERE bad (worst 10%) caught | MILD bad (10–25%) caught | false alarms: just above the line (25–50%) | clearly good (top half)')
    for c in ('Dec', 'Jan', 'Feb', 'Mar'):
        f = BASE + (['green_now'] if c in GREEN_HONEST else []) + ['oni']; o = lr_cv([r for r in rows if r['cut'] == c], f)
        grp = lambda lo, hi: [(p >= 0.35) for r, p in o if lo <= pct[(r['zone'], r['season'])] < hi]
        sev, mild, near, good = grp(0, 10), grp(10, 25.1), grp(25.1, 50), grp(50, 101)
        print(f'{c:5s} | {sum(sev)}/{len(sev)} = {sum(sev)/len(sev):4.0%}               | {sum(mild)}/{len(mild)} = {sum(mild)/len(mild):4.0%}          | {sum(near)/len(near):4.0%}                                     | {sum(good)/len(good):4.0%}')
