"""Part 2: stability checks, final model, plain-language numbers, look-alike years, 2026/27 outlook.
Usage: python3 -I deep_investigation_part2.py out_deep"""
import sys, os, json, math, statistics as st
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from deep_investigation import *
from multiprocessing import Pool
OUT = sys.argv[1]; rows = json.load(open(f'{OUT}/features_all.json'))
BASE = ['rain_pct', 'soil_pct', 'temp_anom', 'et0_pct', 'last_rain_pct']; TIM = ['first_rain', 'rainy_days_pct', 'dry_spell', 'recent_pct']
def fs(c, kind):
    b = BASE + (['green_now'] if c in GREEN_HONEST else [])
    if c == 'Pre': return ['oni'] if kind == 'enso' else ['oni', 'last_rain_pct', 'last_green', 'sep_soil_pct']
    return {'cur': b, 'enso': b + ['oni'], 'all': b + ['oni'] + TIM + ['region_rain', 'last_green']}[kind]
def rf_job(a):
    c, kind, seed, R = a; out = []
    for s in SEASONS:
        tr = [r for r in R if r['season'] != s and r['label'] is not None]; te = [r for r in R if r['season'] == s and r['label'] is not None]
        Xtr, mn = matrix(tr, fs(c, kind)); Xte, _ = matrix(te, fs(c, kind), mn)
        F = rf_fit(Xtr, [r['label'] for r in tr], seed=seed * 100 + s); out += [(s, r['zone'], r['label'], rf_pred(F, x)) for r, x in zip(te, Xte)]
    return (c, kind, seed, out)
def lr_cv(R, feats, drop=()):
    out = []
    for s in SEASONS:
        tr = [r for r in R if r['season'] != s and r['label'] is not None and r['zone'] not in drop]
        te = [r for r in R if r['season'] == s and r['label'] is not None and r['zone'] not in drop]
        Xtr, mn = matrix(tr, feats); Xte, _ = matrix(te, feats, mn); m = lr_fit(Xtr, [r['label'] for r in tr])
        out += [(s, r['zone'], r['label'], lr_pred(m, x)) for r, x in zip(te, Xte)]
    return out
P = print
if __name__ == '__main__':
    CUTS_ = ['Pre', 'Dec', 'Jan', 'Feb', 'Mar']; BY = {c: [r for r in rows if r['cut'] == c] for c in CUTS_}
    jobs = [(c, k, sd, BY[c]) for c in CUTS_ for k in ('enso', 'all') for sd in (1, 2, 3)]
    with Pool(12) as pool: rf = pool.map(rf_job, jobs)
    RF = {(c, k, sd): o for c, k, sd, o in rf}
    P('1. Is the Random Forest (tree AI) gain stable across random seeds?  AUC per seed')
    LRE = {c: lr_cv(BY[c], fs(c, 'enso')) for c in CUTS_}; LRC = {c: lr_cv(BY[c], fs(c, 'cur')) for c in CUTS_ if c != 'Pre'}
    for c in CUTS_:
        a = lambda o: auc([(l, p) for _, _, l, p in o])
        P(f'   {c}: LR+ElNiño {a(LRE[c]):.2f} | RF(+ElNiño) ' + ' '.join(f'{a(RF[(c,"enso",s)]):.2f}' for s in (1, 2, 3)) + ' | RF(all) ' + ' '.join(f'{a(RF[(c,"all",s)]):.2f}' for s in (1, 2, 3)))
    P('\n2. Final candidates (LR = simple, explainable; Team = average of LR+ElNiño and RF(all) seed-averaged)')
    FINAL = {}
    for c in CUTS_:
        rfavg = [(x[0], x[1], x[2], st.mean(RF[(c, 'all', s)][i][3] for s in (1, 2, 3))) for i, x in enumerate(RF[(c, 'all', 1)])]
        team = [(a[0], a[1], a[2], (a[3] + b[3]) / 2) for a, b in zip(LRE[c], rfavg)]
        for nm, o in (('LR+ElNiño', LRE[c]), ('RF(all) avg', rfavg), ('Team', team)):
            pr = [(l, p) for _, _, l, p in o]; ct, fa = catch_at(pr)
            vs = boot_better(o, LRC[c]) if c != 'Pre' else None
            P(f'   {c:4s} {nm:12s} AUC {auc(pr):.2f}  catch@15%FA {ct:4.0%}  Brier {brier_skill(pr):+.2f}' + (f'  beats current model in {vs:.0%} of reshuffles' if vs is not None else ''))
            FINAL[(c, nm)] = o
    P('\n3. Plain numbers for LR+ElNiño with FIXED thresholds (what the product would really do; no hindsight)')
    P('   month  | sensitive (≥35% risk): caught / false alarms | cautious (≥50%): caught / false alarms / right overall | "always normal" right')
    for c in CUTS_:
        pr = [(l, p) for _, _, l, p in LRE[c]]; npos = sum(l for l, _ in pr); nneg = len(pr) - npos
        f = lambda t: (sum(1 for l, p in pr if l and p >= t) / npos, sum(1 for l, p in pr if not l and p >= t) / nneg, sum(1 for l, p in pr if (p >= t) == bool(l)) / len(pr))
        s1, s2 = f(0.35), f(0.5)
        P(f'   {c:6s} | {s1[0]:4.0%} / {s1[1]:4.0%}                              | {s2[0]:4.0%} / {s2[1]:4.0%} / {s2[2]:4.0%}                    | {nneg/len(pr):4.0%}')
    P('\n4. Irrigated fields in the Erbil + Makhmour boxes: does removing them help? (LR+ElNiño, AUC)')
    for c in CUTS_:
        o = lr_cv(BY[c], fs(c, 'enso'), drop=('Erbil', 'Makhmour'))
        full_sub = [x for x in LRE[c] if x[1] not in ('Erbil', 'Makhmour')]
        P(f'   {c}: 16 zones {auc([(l,p) for _,_,l,p in LRE[c]]):.2f} | 14 zones (trained without them) {auc([(l,p) for _,_,l,p in o]):.2f} | Erbil+Makhmour alone {auc([(l,p) for _,z,l,p in LRE[c] if z in ("Erbil","Makhmour")]):.2f}')
    P('\n5. Per-zone AUC, March, LR+ElNiño (which places can we predict?)')
    zs = sorted({x[1] for x in LRE['Mar']}); za = {z: auc([(l, p) for _, zz, l, p in LRE['Mar'] if zz == z]) for z in zs}
    P('   ' + '  '.join(f'{z} {v:.2f}' for z, v in sorted(za.items(), key=lambda t: -t[1])))
    P('\n6. What the model learned (LR+ElNiño trained on all 26 seasons; standardized weights, + = more risk)')
    for c in CUTS_:
        R = [r for r in BY[c] if r['label'] is not None]; X, _ = matrix(R, fs(c, 'enso')); m = lr_fit(X, [r['label'] for r in R])
        P(f'   {c}: ' + ', '.join(f'{f} {w:+.2f}' for f, w in sorted(zip(fs(c, 'enso'), m['w'][1:]), key=lambda t: -abs(t[1]))))
    P('\n7. Season ranking by mean risk (LR+ElNiño, hidden season); bad seasons = ≥50% of zones bad')
    for c in CUTS_:
        seas = {}
        for s, z, l, p in LRE[c]: seas.setdefault(s, []).append((l, p))
        rk = sorted(seas, key=lambda s: -st.mean(p for _, p in seas[s])); bad = [s for s in seas if sum(l for l, _ in seas[s]) / len(seas[s]) >= 0.5]
        P(f'   {c}: bad seasons {", ".join(f"{s}/{str(s+1)[2:]}" for s in sorted(bad))} rank ' + ', '.join(str(rk.index(s) + 1) for s in sorted(bad)) + f' of {len(rk)}   (top 5 risk: {", ".join(f"{s}/{str(s+1)[2:]}" for s in rk[:5])})')
    P('\n8. Look-alike years (AI explanation layer): end of February 2025, nearest past seasons in the same zone')
    c = 'Feb'; F = fs(c, 'enso'); R = [r for r in BY[c] if r['label'] is not None and r['season'] != 2024]
    X, mn = matrix(R, F); m = lr_fit(X, [r['label'] for r in R]); wts = [abs(v) for v in m['w'][1:]]
    for z in ('Slemani', 'Garmiyan', 'Duhok', 'Halabja'):
        q = next(r for r in BY[c] if r['zone'] == z and r['season'] == 2024); xq, _ = matrix([q], F, mn); xq = xq[0]
        cand = [(math.sqrt(sum(wt * ((a - b) / s) ** 2 for a, b, s, wt in zip(x, xq, m['sd'], wts))), r) for x, r in zip(X, R) if r['zone'] == z]
        cand.sort(key=lambda t: t[0]); top = cand[:3]
        P(f'   {z:9s} 2024/25 (rain {q["rain_pct"]:.0f}%, green {q["green_now"] or 0:.0f}%, risk {lr_pred(m, xq):.0%}, real outcome: {"BAD" if q["label"] else "ok"} {q["outcome"]:.0f}%) looks like: '
          + '; '.join(f'{r["season"]}/{str(r["season"]+1)[2:]} (rain {r["rain_pct"]:.0f}% → spring {r["outcome"]:.0f}% {"BAD" if r["label"] else "ok"})' for _, r in top))
    P('\n9. 2026/27 pre-season outlook (El Niño only model trained on all 26 seasons; ONI Jul–Sep 2026 = +2.16)')
    R = [r for r in BY['Pre'] if r['label'] is not None]; X, mn = matrix(R, ['oni']); m = lr_fit(X, [r['label'] for r in R])
    for o in (-1.5, -0.5, 0.0, 0.5, 1.0, 2.16): P(f'   ONI {o:+.2f} → chance of a bad spring {lr_pred(m, [o]):.0%}')
    seasons_by_oni = sorted({(r['season'], r['oni']) for r in R}, key=lambda t: -t[1])[:4]
    P('   Look-alike seasons (strongest El Niño Septembers): ' + '; '.join(f'{s}/{str(s+1)[2:]} ONI {o:+.1f} → {sum(r["label"] for r in R if r["season"]==s)}/16 zones bad' for s, o in seasons_by_oni))
