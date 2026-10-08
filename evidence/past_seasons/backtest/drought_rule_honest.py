"""Honest test of the rain-drought alarm rule: the cut-off is chosen WITHOUT the winter being judged.
Rule: on the cutoff date, alarm if rain since 1 Oct (mean of 16 zones, % of 1991-2020 normal) < T.
For each winter: T is picked from the other 35 winters (largest T with at most K false alarms there), then applied to the hidden winter.
Usage: python3 -I drought_rule_honest.py   (writes drought_rule_honest_output.txt)"""
import os, sys, json, statistics as st
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from deep_investigation import W, ymd, CUTS

HERE = os.path.dirname(os.path.abspath(__file__)); DATA = os.path.join(HERE, '..', 'backtest_data')
NORMAL = range(1991, 2021); WINTERS = list(range(1990, 2026))
def cdate(s, cut): m, d = CUTS[cut]; return ymd(s if m >= 9 else s + 1, m, d)
zones = [z['name'] for z in json.load(open(os.path.join(HERE, 'zones.json')))['zones']]
so_far = {}; total = {}
for z in zones:
    w = W(os.path.join(DATA, 'openmeteo', f'{z}.json'))
    tot = {s: w.sum(ymd(s, 10, 1), ymd(s + 1, 5, 31), 'precipitation_sum') for s in WINTERS}; nt = st.mean(tot[s] for s in NORMAL)
    for s in WINTERS: total.setdefault(s, []).append(tot[s] / nt * 100)
    for c in ('Dec', 'Jan', 'Feb', 'Mar'):
        r = {s: w.sum(ymd(s, 10, 1), cdate(s, c), 'precipitation_sum') for s in WINTERS}; nr = st.mean(r[s] for s in NORMAL)
        for s in WINTERS: so_far.setdefault((c, s), []).append(r[s] / nr * 100)
rain = {s: st.mean(v) for s, v in total.items()}; so_far = {k: st.mean(v) for k, v in so_far.items()}
thr = sorted(rain.values())[len(rain) // 4]; D = {s: rain[s] <= thr for s in WINTERS}
lines = []
def P(x=''): print(x); lines.append(x)
P(f'Drought = Oct-May rain <= {thr:.0f}% of normal: {sum(D.values())} of {len(WINTERS)} winters. Each winter judged with a cut-off learned from the other 35.\n')
for c in ('Dec', 'Jan', 'Feb', 'Mar'):
    for K in (0, 1, 2):
        caught = []; false = []; Ts = []
        for s in WINTERS:
            tr = [t for t in WINTERS if t != s]
            normals = sorted(so_far[(c, t)] for t in tr if not D[t])
            T = normals[K]                                   # largest cut-off with K false alarms in training ("< T")
            Ts.append(T)
            if so_far[(c, s)] < T: (caught if D[s] else false).append(s)
        P(f'{c} | learned with <= {K} false alarm(s): cut-off {min(Ts):.0f}-{max(Ts):.0f}% | hidden-winter result: caught {len(caught)}/{sum(D.values())} droughts, '
          f'{len(false)} false alarm(s) in 26 normal winters | caught: ' + ', '.join(f'{s}/{(s+1)%100:02d}' for s in caught) +
          (' | false: ' + ', '.join(f'{s}/{(s+1)%100:02d}' for s in false) if false else ''))
    P()
open(os.path.join(HERE, 'drought_rule_honest_output.txt'), 'w').write('\n'.join(lines) + '\n')
