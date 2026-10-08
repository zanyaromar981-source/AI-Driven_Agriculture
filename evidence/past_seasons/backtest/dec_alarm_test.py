"""Can we raise the drought alarm in DECEMBER (instead of 31 Jan)?  (2026-10-08)
Alarm date ~10 Dec: rain since 1 Oct up to 30 Nov (16 zones, ERA5) + world seasonal forecasts started 1 Dec for Dec-Feb
(7 models: 6 NMME + ECMWF SEAS5, issued ~5-10 Dec) + El Nino (Sep-Nov). Also ~10 Nov: rain in October + Nov-start forecasts (Nov-Jan).
Outcome: Oct-May rain <= bottom 25% of 36 winters (same 10 droughts as drought_alarm_test.py); models cover 35 winters 1991/92-2025/26.
Each winter hidden in turn; model bias/standardising also learned without the hidden winter.
Usage: python3 -I dec_alarm_test.py   (writes dec_alarm_test_output.txt)"""
import os, sys, json, statistics as st
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
import seasonal_models_test as SM
from deep_investigation import W, ymd, auc
DATA = os.path.join(HERE, '..', 'backtest_data')
fc = SM.load_models(os.path.join(DATA, 'seasonal_models'))
ONI = {}
for l in open(os.path.join(DATA, 'enso', 'oni.ascii.txt')).read().splitlines()[1:]:
    a, y, t, an = l.split(); ONI[(a, int(y))] = float(an)
zones = [z['name'] for z in json.load(open(os.path.join(HERE, 'zones.json')))['zones']]
WIN = list(range(1990, 2026)); tot = {}; sofar = {}
for z in zones:
    w = W(os.path.join(DATA, 'openmeteo', f'{z}.json'))
    t = {s: w.sum(ymd(s, 10, 1), ymd(s + 1, 5, 31), 'precipitation_sum') for s in WIN}; n = st.mean(t[s] for s in range(1991, 2021))
    for s in WIN: tot.setdefault(s, []).append(t[s] / n * 100)
    for lab, end in (('Oct', (10, 31)), ('Nov', (11, 30)), ('Dec', (12, 31))):
        r = {s: w.sum(ymd(s, 10, 1), ymd(s, *end), 'precipitation_sum') for s in WIN}; nr = st.mean(r[s] for s in range(1991, 2021))
        for s in WIN: sofar.setdefault((lab, s), []).append(r[s] / nr * 100)
R = {s: st.mean(v) for s, v in tot.items()}; SOFAR = {k: st.mean(v) for k, v in sofar.items()}
thr = sorted(R.values())[len(R) // 4]; BAD = {s: R[s] <= thr for s in WIN}
TEST = list(range(1991, 2026))
def model_raw(start_m, s):
    """per model: total of the 3 target months from the start month (mm); None if missing."""
    out = {}
    for m, d in fc.items():
        f = d.get((s, start_m))
        if f and all(li in f for li in (0, 1, 2)): out[m] = sum(f[li] for li in (0, 1, 2))
    return out
def model_z(start_m, s, train):
    """mean over models of the standardised forecast; mean/sd from training winters only."""
    zs = []
    for m, v in model_raw(start_m, s).items():
        tr = [model_raw(start_m, t).get(m) for t in train]; tr = [x for x in tr if x is not None]
        if len(tr) >= 20: zs.append((v - st.mean(tr)) / st.stdev(tr))
    return st.mean(zs) if zs else None
def loo(build):
    preds = {}
    for s in TEST:
        train = [t for t in TEST if t != s]
        X = [build(t, train) for t in train]; Y = [R[t] for t in train]
        keep = [(x, y) for x, y in zip(X, Y) if None not in x]; X = [k[0] for k in keep]; Y = [k[1] for k in keep]
        k = len(X[0]) + 1; A = [[0.0] * k for _ in range(k)]; B = [0.0] * k
        for x, y in zip(X, Y):
            v = [1.0] + x
            for a in range(k):
                B[a] += v[a] * y
                for b in range(k): A[a][b] += v[a] * v[b] + (0.01 if a == b and a > 0 else 0)
        w = SM.ols_fit(X, Y) if False else None
        for c in range(k):
            piv = max(range(c, k), key=lambda r: abs(A[r][c])); A[c], A[piv] = A[piv], A[c]; B[c], B[piv] = B[piv], B[c]
            for r in range(k):
                if r != c and A[c][c]:
                    f = A[r][c] / A[c][c]; A[r] = [A[r][j] - f * A[c][j] for j in range(k)]; B[r] -= f * B[c]
        w = [B[j] / A[j][j] for j in range(k)]
        x = build(s, train)
        if None not in x: preds[s] = w[0] + sum(a * b for a, b in zip(w[1:], x))
    return preds
lines = []
def P(x=''): print(x); lines.append(x)
def report(lab, preds):
    ss = sorted(preds); a = auc([(1 if BAD[s] else 0, -preds[s]) for s in ss])
    neg = sorted((-preds[s] for s in ss if not BAD[s]), reverse=True); nb = sum(BAD[s] for s in ss)
    c = []
    for nfa in (0, 1, 2, 4):
        t = neg[nfa]; c.append(sum(1 for s in ss if BAD[s] and -preds[s] > t))
    r = st.correlation([preds[s] for s in ss], [R[s] for s in ss])
    rk = 1 + sum(1 for s in ss if preds[s] < preds[2024])
    P(f'  {lab:58s} n {len(ss)}  r {r:+.2f}  AUC {a:.2f}  caught with 0/1/2/4 false alarms: {c[0]}/{c[1]}/{c[2]}/{c[3]} of {nb}  2024/25 rank {rk}')
    return preds
P(f'Outcome: drought = Oct-May rain <= {thr:.0f}% of normal; {sum(BAD[s] for s in TEST)} droughts in {len(TEST)} winters 1991/92-2025/26. Each winter hidden in turn.\n')
P('~10 November (rain in October + forecasts started 1 Nov for Nov-Jan):')
report('rain in October only', loo(lambda s, tr: [SOFAR[('Oct', s)]]))
report('El Nino (Aug-Oct) only', loo(lambda s, tr: [ONI[('ASO', s)]]))
report('world forecasts only', loo(lambda s, tr: [model_z(11, s, tr)]))
report('rain Oct + El Nino + world forecasts', loo(lambda s, tr: [SOFAR[('Oct', s)], ONI[('ASO', s)], model_z(11, s, tr)]))
P('\n~10 December (rain Oct-Nov + forecasts started 1 Dec for Dec-Feb):')
report('rain Oct-Nov only', loo(lambda s, tr: [SOFAR[('Nov', s)]]))
report('El Nino (Sep-Nov) only', loo(lambda s, tr: [ONI[('SON', s)]]))
report('world forecasts only', loo(lambda s, tr: [model_z(12, s, tr)]))
report('rain Oct-Nov + world forecasts', loo(lambda s, tr: [SOFAR[('Nov', s)], model_z(12, s, tr)]))
dec = report('rain Oct-Nov + El Nino + world forecasts', loo(lambda s, tr: [SOFAR[('Nov', s)], ONI[('SON', s)], model_z(12, s, tr)]))
P('\nReference, 31 December (rain only, already known): ')
report('rain Oct-Dec only', loo(lambda s, tr: [SOFAR[('Dec', s)]]))
P('\nPer drought winter, predicted season rain on ~10 Dec (rain Oct-Nov + El Nino + world forecasts):')
for s in TEST:
    if BAD[s] or dec.get(s, 999) < 85: P(f'  {s}/{(s+1)%100:02d}  predicted {dec[s]:4.0f}%  real {R[s]:4.0f}%' + ('  DROUGHT' if BAD[s] else '  (normal winter)'))
open(os.path.join(HERE, 'dec_alarm_test_output.txt'), 'w').write('\n'.join(lines) + '\n')
