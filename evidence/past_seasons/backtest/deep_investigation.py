"""Deep investigation: which inputs, combined, and which AI model give the best spring-crop warning?
Outcome (same as BACKTEST_RESULTS): late-spring greenness (MODIS mean of composites day 81–129) in the zone's bottom 25%.
Test: leave-one-season-out (all 16 zones of the held-out season hidden together), seasons 1999/00–2024/25.
Usage: python3 -I deep_investigation.py zones.json ../backtest_data/openmeteo ../backtest_data/modis ../backtest_data/enso/oni.ascii.txt out_deep"""
import sys, json, os, math, random, statistics as st
from multiprocessing import Pool

NORMAL = range(1991, 2021); SEASONS = list(range(1999, 2025))
CUTS = {'Pre': (9, 30), 'Dec': (12, 31), 'Jan': (1, 31), 'Feb': (2, 28), 'Mar': (3, 31), 'Ceiling': (4, 30)}
ONI_AT = {'Pre': ('JAS', 0), 'Dec': ('SON', 0), 'Jan': ('OND', 0), 'Feb': ('NDJ', 0), 'Mar': ('DJF', 1), 'Ceiling': ('JFM', 1)}
GREEN_HONEST = {'Feb': [33], 'Mar': [33, 49, 65]}            # composites really available by the cutoff
GREEN_OLD = {'Feb': [33, 49], 'Mar': [33, 49, 65, 81]}        # what the first backtest used (81 overlaps the outcome)

# ---------------- data ----------------
def ymd(y, m, d): return f'{y}-{m:02d}-{d:02d}'
def cut_date(s, cut): m, d = CUTS[cut]; return ymd(s if m >= 9 else s + 1, m, d)
class W:
    def __init__(s, fn):
        d = json.load(open(fn))['daily']; s.d = d; s.ix = {t: i for i, t in enumerate(d['time'])}
    def rng(s, a, b, key):
        i, j = s.ix.get(a), s.ix.get(b)
        if i is None or j is None or j < i: return None
        return [v for v in s.d[key][i:j + 1] if v is not None]
    def sum(s, a, b, key): v = s.rng(a, b, key); return sum(v) if v else (0.0 if v == [] else None)
    def mean(s, a, b, key): v = s.rng(a, b, key); return st.mean(v) if v else None
    def back(s, b, n, key):
        j = s.ix.get(b)
        if j is None: return None
        v = [x for x in s.d[key][max(0, j - n + 1):j + 1] if x is not None]; return v
def load_modis(MD, z):
    out = {}
    for y in range(2000, 2026):
        fn = f'{MD}/{z}_{y}.json'
        if not os.path.exists(fn): continue
        try: j = json.load(open(fn))
        except Exception: continue
        for c in j.get('subset', []):
            out.setdefault(y, {})[int(c['modis_date'][5:])] = [v * 0.0001 if (v is not None and v > -2000) else None for v in c['data']]
    return out
def zone_series(m):
    years = sorted(m); npx = max(len(v) for y in years for v in m[y].values()); pk = {}
    for p in range(npx):
        peaks = []
        for y in years:
            vals = [m[y][d][p] for d in m[y] if 65 <= d <= 129 and p < len(m[y][d]) and m[y][d][p] is not None]
            if vals: peaks.append(max(vals))
        if peaks: pk[p] = st.median(peaks)
    keep = lambda lo, hi: [p for p, v in pk.items() if lo <= v <= hi]
    px = keep(0.30, 0.75)
    if len(px) < 50: px = keep(0.25, 0.80)
    if len(px) < 50: px = list(pk)
    ser = {}
    for y in years:
        for d, vals in m[y].items():
            vv = [vals[p] for p in px if p < len(vals) and vals[p] is not None]
            if len(vv) >= max(10, len(px) // 4): ser[(y, d)] = st.mean(vv)
    return ser
def longest_dry(v):
    best = cur = 0
    for x in v:
        cur = cur + 1 if x < 1 else 0; best = max(best, cur)
    return best
def first_rain_days(w, s, cutd, thr=25):
    v = w.rng(ymd(s, 9, 1), cutd, 'precipitation_sum') or []
    c = 0
    for i, x in enumerate(v):
        c += x
        if c >= thr: return i
    return len(v) + 15                                           # not yet: censored
def build(ZF, OM, MD, ONIF):
    zones = [z['name'] for z in json.load(open(ZF))['zones']]
    ONI = {}
    for line in open(ONIF).read().splitlines()[1:]:
        a, y, t, an = line.split(); ONI[(a, int(y))] = float(an)
    rows = []
    for z in zones:
        w = W(f'{OM}/{z}.json'); ser = zone_series(load_modis(MD, z))
        late = {}
        for y in range(2000, 2026):
            vv = [v for (yy, dd), v in ser.items() if yy == y and 81 <= dd <= 129]
            late[y] = st.mean(vv) if len(vv) >= 2 else None
        vals = [v for v in late.values() if v is not None]; q25 = sorted(vals)[len(vals) // 4]; med = st.median(vals)
        fulln = st.mean(w.sum(ymd(s, 10, 1), ymd(s + 1, 5, 31), 'precipitation_sum') for s in NORMAL)
        sepn = st.mean(st.mean(w.back(ymd(s, 9, 30), 7, 'soil_moisture_0_to_100cm_mean')) for s in NORMAL)
        for cut in CUTS:
            if cut == 'Pre':
                N = None
            else:
                def feats_raw(s):
                    a, b = ymd(s, 10, 1), cut_date(s, cut)
                    pr = w.rng(a, b, 'precipitation_sum')
                    return dict(r=w.sum(a, b, 'precipitation_sum'), so=st.mean(w.back(b, 7, 'soil_moisture_0_to_100cm_mean')),
                                t=w.mean(a, b, 'temperature_2m_mean'), e=w.sum(a, b, 'et0_fao_evapotranspiration'),
                                rd=sum(1 for x in pr if x >= 1), dry=longest_dry(w.rng(ymd(s, 11, 15), b, 'precipitation_sum') or []),
                                rec=sum(w.back(b, 45, 'precipitation_sum')), fr=first_rain_days(w, s, b))
                NR = [feats_raw(s) for s in NORMAL]
                N = {k: st.mean(x[k] for x in NR) for k in NR[0]}; N['fr_med'] = st.median(x['fr'] for x in NR)
            for s in SEASONS + [2025, 2026]:
                if cut != 'Pre' and s == 2026: continue
                lab = None
                if s + 1 in late and late.get(s + 1) is not None and s <= 2024: lab = 1 if late[s + 1] <= q25 else 0
                lr = w.sum(ymd(s - 1, 10, 1), ymd(s, 5, 31), 'precipitation_sum')
                r = dict(zone=z, season=s, cut=cut, label=lab, outcome=(late[s + 1] / med * 100) if late.get(s + 1) else None,
                         last_rain_pct=lr / fulln * 100, last_green=(late[s] / med * 100) if late.get(s) else None)
                sname, dy = ONI_AT[cut]; r['oni'] = ONI.get((sname, s + dy))
                if cut == 'Pre':
                    r['sep_soil_pct'] = st.mean(w.back(ymd(s, 9, 30), 7, 'soil_moisture_0_to_100cm_mean')) / sepn * 100
                else:
                    f = feats_raw(s)
                    r.update(rain_pct=f['r'] / N['r'] * 100, soil_pct=f['so'] / N['so'] * 100, temp_anom=f['t'] - N['t'],
                             et0_pct=f['e'] / N['e'] * 100, rainy_days_pct=f['rd'] / N['rd'] * 100, dry_spell=f['dry'] - N['dry'],
                             recent_pct=f['rec'] / max(N['rec'], 1) * 100, first_rain=f['fr'] - N['fr_med'])
                    for nm, D in (('green_now', GREEN_HONEST), ('green_old', GREEN_OLD)):
                        if cut in D:
                            g = [ser.get((s + 1, dd)) for dd in D[cut]]; g = [v for v in g if v is not None]
                            r[nm] = max(g) / med * 100 if g else None
                rows.append(r)
    # region-wide rain so far (mean over zones)
    for cut in CUTS:
        if cut == 'Pre': continue
        for s in SEASONS + [2025]:
            rr = [r for r in rows if r['cut'] == cut and r['season'] == s]
            m = st.mean(r['rain_pct'] for r in rr)
            for r in rr: r['region_rain'] = m
    return rows

# ---------------- models (pure python) ----------------
def solve(A, b):
    n = len(b); M = [row[:] + [b[i]] for i, row in enumerate(A)]
    for c in range(n):
        p = max(range(c, n), key=lambda r: abs(M[r][c])); M[c], M[p] = M[p], M[c]
        for r in range(n):
            if r != c and M[c][c]:
                f = M[r][c] / M[c][c]
                if f: M[r] = [a - f * bb for a, bb in zip(M[r], M[c])]
    return [M[i][n] / M[i][i] for i in range(n)]
def stdz(X):
    mu = [st.mean(c) for c in zip(*X)]; sd = [st.pstdev(c) or 1 for c in zip(*X)]; return mu, sd
def lr_fit(X, y, l2=0.05):
    n, k = len(X), len(X[0]); mu, sd = stdz(X)
    Z = [[1.0] + [(x[j] - mu[j]) / sd[j] for j in range(k)] for x in X]; w = [0.0] * (k + 1); lam = n * l2
    for _ in range(30):
        H = [[0.0] * (k + 1) for _ in range(k + 1)]; g = [0.0] * (k + 1)
        for zi, yi in zip(Z, y):
            eta = sum(a * b for a, b in zip(w, zi)); eta = max(-30, min(30, eta)); p = 1 / (1 + math.exp(-eta))
            q = p * (1 - p); e = p - yi
            for a in range(k + 1):
                g[a] += e * zi[a]; za = zi[a] * q; Ha = H[a]
                for b in range(a, k + 1): Ha[b] += za * zi[b]
        for a in range(k + 1):
            for b in range(a): H[a][b] = H[b][a]
        for a in range(1, k + 1): H[a][a] += lam; g[a] += lam * w[a]
        stp = solve(H, g); w = [a - b for a, b in zip(w, stp)]
        if max(abs(x) for x in stp) < 1e-7: break
    return dict(w=w, mu=mu, sd=sd)
def lr_pred(m, x):
    eta = m['w'][0] + sum(m['w'][j + 1] * (x[j] - m['mu'][j]) / m['sd'][j] for j in range(len(x)))
    return 1 / (1 + math.exp(-max(-30, min(30, eta))))
def knn_pred(Xtr, ytr, ztr, x, z, wts, mu, sd, k, same_zone=False):
    d = []
    for xi, yi, zi in zip(Xtr, ytr, ztr):
        if same_zone and zi != z: continue
        d.append((math.sqrt(sum(wt * ((a - b) / s) ** 2 for a, b, s, wt in zip(xi, x, sd, wts))), yi))
    d.sort(); nb = d[:k]
    return (sum(y for _, y in nb) + 0.25) / (len(nb) + 1)        # 1 pseudo-neighbour at the base rate
def tree(idx, X, y, depth, mtry, rnd, minleaf):
    ys = [y[i] for i in idx]; p = sum(ys) / len(ys)
    if depth == 0 or len(idx) < 2 * minleaf or p in (0.0, 1.0): return p
    k = len(X[0]); best = None; tot = sum(ys); n = len(idx)
    for f in rnd.sample(range(k), mtry):
        srt = sorted(idx, key=lambda i: X[i][f]); left = 0
        for pos in range(1, n):
            left += y[srt[pos - 1]]
            if pos < minleaf or n - pos < minleaf: continue
            a, b = X[srt[pos - 1]][f], X[srt[pos]][f]
            if a == b: continue
            pl = left / pos; pr = (tot - left) / (n - pos); gini = pos * pl * (1 - pl) + (n - pos) * pr * (1 - pr)
            if best is None or gini < best[0]: best = (gini, f, (a + b) / 2, srt[:pos], srt[pos:])
    if best is None: return p
    _, f, thr, L, R = best
    return (f, thr, tree(L, X, y, depth - 1, mtry, rnd, minleaf), tree(R, X, y, depth - 1, mtry, rnd, minleaf))
def tree_pred(t, x):
    while isinstance(t, tuple): t = t[2] if x[t[0]] <= t[1] else t[3]
    return t
def rf_fit(X, y, ntree=120, depth=4, minleaf=8, seed=1):
    rnd = random.Random(seed); n = len(X); mtry = max(1, round(math.sqrt(len(X[0])))); forest = []
    for _ in range(ntree):
        idx = [rnd.randrange(n) for _ in range(n)]; forest.append(tree(idx, X, y, depth, mtry, rnd, minleaf))
    return forest
def rf_pred(F, x): return sum(tree_pred(t, x) for t in F) / len(F)

# ---------------- evaluation ----------------
def matrix(rows, feats, means=None):
    if means is None: means = {f: st.mean(r[f] for r in rows if r.get(f) is not None) for f in feats}
    return [[r[f] if r.get(f) is not None else means[f] for f in feats] for r in rows], means
def run_cv(job):
    """job = (cut, name, feats, model, rows). Returns list of (season, zone, label, risk)."""
    cut, name, feats, model, rows = job
    out = []
    for s in SEASONS:
        tr = [r for r in rows if r['season'] != s and r['label'] is not None]; te = [r for r in rows if r['season'] == s and r['label'] is not None]
        if not te: continue
        Xtr, means = matrix(tr, feats); ytr = [r['label'] for r in tr]; Xte, _ = matrix(te, feats, means)
        if model.startswith('LR'):
            m = lr_fit(Xtr, ytr, 0.5 if model == 'LR-strong' else 0.05); P = [lr_pred(m, x) for x in Xte]
        elif model.startswith('kNN'):
            m = lr_fit(Xtr, ytr, 0.05); wts = [abs(v) for v in m['w'][1:]]; mu, sd = m['mu'], m['sd']; zt = [r['zone'] for r in tr]
            P = [knn_pred(Xtr, ytr, zt, x, r['zone'], wts, mu, sd, 4 if model == 'kNN-zone' else 20, model == 'kNN-zone') for x, r in zip(Xte, te)]
        elif model == 'RF':
            F = rf_fit(Xtr, ytr, seed=s); P = [rf_pred(F, x) for x in Xte]
        out += [(s, r['zone'], r['label'], p) for r, p in zip(te, P)]
    return (cut, name, model, out)
def auc(pairs):
    srt = sorted(pairs, key=lambda t: t[1]); ranks = [0.0] * len(srt); i = 0
    while i < len(srt):
        j = i
        while j + 1 < len(srt) and srt[j + 1][1] == srt[i][1]: j += 1
        for k in range(i, j + 1): ranks[k] = (i + j) / 2 + 1
        i = j + 1
    npos = sum(l for l, _ in srt); nneg = len(srt) - npos
    return (sum(rk for rk, (l, _) in zip(ranks, srt) if l) - npos * (npos + 1) / 2) / (npos * nneg)
def catch_at(pairs, fa=0.15):
    neg = sorted((p for l, p in pairs if not l), reverse=True); thr = neg[int(fa * len(neg))]
    pos = [p for l, p in pairs if l]; return sum(1 for p in pos if p > thr) / len(pos), sum(1 for p in neg if p > thr) / len(neg)
def brier_skill(pairs):
    base = sum(l for l, _ in pairs) / len(pairs)
    return 1 - sum((p - l) ** 2 for l, p in pairs) / sum((base - l) ** 2 for l, _ in pairs)
def boot_better(outA, outB, n=1000, seed=7):
    """P(AUC_A > AUC_B) when seasons are resampled (paired)."""
    rnd = random.Random(seed); byA = {}; byB = {}
    for s, z, l, p in outA: byA.setdefault(s, []).append((l, p))
    for s, z, l, p in outB: byB.setdefault(s, []).append((l, p))
    S = sorted(set(byA) & set(byB)); win = 0; done = 0
    for _ in range(n):
        pick = [rnd.choice(S) for _ in S]; a = [x for s in pick for x in byA[s]]; b = [x for s in pick for x in byB[s]]
        if 0 < sum(l for l, _ in a) < len(a): done += 1; win += auc(a) > auc(b)
    return win / done

if __name__ == '__main__':
    ZF, OM, MD, ONIF, OUT = sys.argv[1:6]; os.makedirs(OUT, exist_ok=True)
    rows = build(ZF, OM, MD, ONIF)
    json.dump(rows, open(f'{OUT}/features_all.json', 'w'))
    BASE = ['rain_pct', 'soil_pct', 'temp_anom', 'et0_pct', 'last_rain_pct']; TIM = ['first_rain', 'rainy_days_pct', 'dry_spell', 'recent_pct']
    SETS = {}
    SETS['Pre'] = {'last year only': ['last_rain_pct', 'last_green', 'sep_soil_pct'], 'El Niño only': ['oni'],
                   'all': ['oni', 'last_rain_pct', 'last_green', 'sep_soil_pct']}
    for c in ('Dec', 'Jan', 'Feb', 'Mar'):
        b = BASE + (['green_now'] if c in GREEN_HONEST else [])
        SETS[c] = {'current model': b, '+ El Niño': b + ['oni'], '+ rain timing': b + TIM, '+ region & last spring': b + ['region_rain', 'last_green'],
                   'all': b + ['oni'] + TIM + ['region_rain', 'last_green']}
        if c in GREEN_OLD: SETS[c]['current model, old leaky green'] = BASE + ['green_old']
    SETS['Ceiling'] = {'perfect weather to Apr 30': BASE, 'perfect weather + timing': BASE + TIM}
    jobs = []
    for c, sets in SETS.items():
        R = [r for r in rows if r['cut'] == c]
        for nm, fs in sets.items():
            jobs.append((c, nm, fs, 'LR', R))
            if nm == 'all':
                for m in ('LR-strong', 'kNN', 'kNN-zone', 'RF'): jobs.append((c, nm, fs, m, R))
    with Pool(min(8, os.cpu_count() or 4)) as pool: res = pool.map(run_cv, jobs)
    OUTS = {(c, nm, m): o for c, nm, m, o in res}
    # ensemble of LR, kNN, RF on 'all'
    for c in SETS:
        if (c, 'all', 'RF') in OUTS:
            parts = [OUTS[(c, 'all', m)] for m in ('LR', 'kNN', 'RF')]
            OUTS[(c, 'all', 'Ensemble')] = [(a[0], a[1], a[2], (a[3] + b[3] + d[3]) / 3) for a, b, d in zip(*parts)]
    lines = []; P = lambda s='': (print(s), lines.append(s))
    P('Score guide: AUC = chance the model gives a bad zone-season a higher risk than a good one (0.50 = coin flip, 1.00 = perfect).')
    P('catch@15%FA = share of bad zone-seasons caught when false alarms are held at 15%.  Brier skill > 0 = better than always saying 25%.')
    P('"vs current" = in how many of 1000 reshuffles of the seasons this beats the current model (≥ 0.90 means probably real).\n')
    summ = {}
    for c in SETS:
        P(f'=== {c} ===')
        ref = OUTS.get((c, 'current model', 'LR')) or OUTS.get((c, 'last year only', 'LR')) or OUTS.get((c, 'perfect weather to Apr 30', 'LR'))
        for (cc, nm, m), o in OUTS.items():
            if cc != c: continue
            pairs = [(l, p) for _, _, l, p in o]; a = auc(pairs); ct, fa = catch_at(pairs); bs = brier_skill(pairs)
            vs = boot_better(o, ref) if o is not ref else None
            seas = {}
            for s, z, l, p in o: seas.setdefault(s, []).append((l, p))
            sp = [(1 if sum(l for l, _ in v) / len(v) >= 0.5 else 0, st.mean(p for _, p in v)) for v in seas.values()]
            sa = auc(sp) if 0 < sum(l for l, _ in sp) < len(sp) else None
            P(f'  {nm:34s} {m:10s} AUC {a:.2f}  catch@15%FA {ct:4.0%}  Brier skill {bs:+.2f}  season AUC {sa:.2f}  ' + (f'vs current {vs:.2f}' if vs is not None else '(reference)'))
            summ[f'{c}|{nm}|{m}'] = dict(auc=round(a, 3), catch15=round(ct, 3), brier_skill=round(bs, 3), season_auc=round(sa, 3) if sa else None, vs_ref=vs)
        P()
    json.dump(summ, open(f'{OUT}/summary.json', 'w'), indent=1)
    with open(f'{OUT}/oof_predictions.csv', 'w') as f:
        f.write('cut,set,model,season,zone,label,risk\n')
        for (c, nm, m), o in OUTS.items():
            for s, z, l, p in o: f.write(f'{c},{nm},{m},{s},{z},{l},{p:.4f}\n')
    open(f'{OUT}/report.txt', 'w').write('\n'.join(lines))
