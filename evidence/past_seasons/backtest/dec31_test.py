"""Can we detect each drought on 31 December?  (2026-10-08)
Signals known on 31 Dec: rain since 1 Oct (16 zones), the 10 Dec watch, the 7 world seasonal forecasts started 1 Dec (their Jan+Feb part),
El Nino (Sep-Nov), and the Team model's zone crop lights. Linear model and cut-offs learned with each winter hidden (35 winters 1991/92-2025/26).
Usage: python3 -I dec31_test.py   (reads ../../../web/data.js for rain, watch and zone lights; writes dec31_test_output.txt)"""
import os, sys, json, statistics as st
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
import seasonal_models_test as SM
WEB = os.path.join(HERE, '..', '..', '..', 'web', 'data.js')
s = open(WEB).read(); D = json.loads(s[s.index('{'):s.rindex('}') + 1])
fc = SM.load_models(os.path.join(HERE, '..', 'backtest_data', 'seasonal_models'))
TEST = list(range(1991, 2026)); E = {y: D['seasons'][str(y)] for y in TEST}
DR = {y: E[y]['drought'] for y in TEST}; FIN = {y: E[y]['final'] for y in TEST}
rain31 = {y: E[y]['rain'][91] for y in TEST}                      # slot 91 = 31 Dec
def mraw(y):
    out = {}
    for m, d in fc.items():
        f = d.get((y, 12))
        if f and 1 in f and 2 in f: out[m] = f[1] + f[2]        # Jan + Feb forecast from the 1 Dec start
    return out
def mz(y, train):
    zs = []
    for m, v in mraw(y).items():
        tr = [mraw(t).get(m) for t in train]; tr = [x for x in tr if x is not None]
        if len(tr) >= 20: zs.append((v - st.mean(tr)) / st.stdev(tr))
    return st.mean(zs) if zs else 0.0
def fit(X, Y):
    k = len(X[0]) + 1; A = [[0.0] * k for _ in range(k)]; B = [0.0] * k
    for x, yv in zip(X, Y):
        v = [1.0] + x
        for a in range(k):
            B[a] += v[a] * yv
            for b in range(k): A[a][b] += v[a] * v[b] + (0.01 if a == b and a > 0 else 0)
    for c in range(k):
        piv = max(range(c, k), key=lambda r: abs(A[r][c])); A[c], A[piv] = A[piv], A[c]; B[c], B[piv] = B[piv], B[c]
        for r in range(k):
            if r != c and A[c][c]:
                f = A[r][c] / A[c][c]; A[r] = [A[r][j] - f * A[c][j] for j in range(k)]; B[r] -= f * B[c]
    return [B[j] / A[j][j] for j in range(k)]
pred = {}
for y in TEST:
    tr = [t for t in TEST if t != y]
    X = [[rain31[t], mz(t, [u for u in tr if u != t])] for t in tr]; w = fit(X, [FIN[t] for t in tr])
    pred[y] = w[0] + w[1] * rain31[y] + w[2] * mz(y, tr)
def honest(score, K):          # flag if score < cut-off learned from the other winters (K false alarms allowed there)
    out = {}
    for y in TEST:
        normals = sorted(score[t] for t in TEST if t != y and not DR[t]); out[y] = score[y] < normals[K]
    return out
lines = []
def P(x=''): print(x); lines.append(x)
L = D['lights']['Dec']
def red_zones(y):
    e = E[y]
    if not e.get('risk'): return None
    return sum(1 for p in e['risk']['Dec'] if p >= L['red'])
P('31 December check, every winter judged with cut-offs learned without it. 10 droughts in 35 winters.\n')
for lab, sc in (('rain so far only', rain31), ('rain so far + world forecasts (Jan-Feb)', pred)):
    for K in (0, 2, 4):
        f = honest(sc, K); c = [y for y in TEST if DR[y] and f[y]]; fa = [y for y in TEST if not DR[y] and f[y]]
        P(f'{lab:42s} allow {K} false: caught {len(c)}/10 {[str(y)+"/"+str(y+1)[2:] for y in c]} | false {len(fa)} {[str(y)+"/"+str(y+1)[2:] for y in fa]}')
    P()
P('Per drought winter on 31 December:')
P('  winter   final | rain so far (rank of 35, 1 = driest) | 10 Dec watch | model prediction (rank) | red crop zones of 16')
rk = lambda sc, y: 1 + sum(1 for t in TEST if sc[t] < sc[y])
for y in TEST:
    if not DR[y]: continue
    rz = red_zones(y)
    P(f'  {y}/{str(y+1)[2:]}   {FIN[y]:3d}% | {rain31[y]:3d}% (#{rk(rain31,y):2d})                     | {"ON " if E[y]["watch"]["on"] else "off"}          | {pred[y]:4.0f}% (#{rk(pred,y):2d})             | {"–" if rz is None else rz}')
P('\nNormal winters that look dry on 31 Dec (top 6 by model): ' + ', '.join(f'{y}/{str(y+1)[2:]} pred {pred[y]:.0f}% final {FIN[y]}%' for y in sorted([t for t in TEST if not DR[t]], key=lambda t: pred[t])[:6]))
open(os.path.join(HERE, 'dec31_test_output.txt'), 'w').write('\n'.join(lines) + '\n')
