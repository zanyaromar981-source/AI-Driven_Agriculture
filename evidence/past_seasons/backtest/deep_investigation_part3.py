"""Part 3: fixed-threshold plain numbers for the final 'Team' model (LR+ElNiño averaged with Random Forest on all inputs).
Usage: python3 -I deep_investigation_part3.py out_deep"""
import sys, os, json, statistics as st
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from deep_investigation_part2 import *
if __name__ == '__main__':
    BY = {c: [r for r in rows if r['cut'] == c] for c in ('Pre', 'Dec', 'Jan', 'Feb', 'Mar')}
    with Pool(12) as pool: rf = pool.map(rf_job, [(c, 'all', sd, BY[c]) for c in BY for sd in (1, 2, 3)])
    RF = {(c, sd): o for c, k, sd, o in rf}; res = {}
    print('month | model | sensitive ≥35%: caught / false alarms | cautious ≥50%: caught / false alarms / right overall | always-normal right')
    for c in BY:
        lr = lr_cv(BY[c], fs(c, 'enso')); rfa = [(x[0], x[1], x[2], st.mean(RF[(c, s)][i][3] for s in (1, 2, 3))) for i, x in enumerate(RF[(c, 1)])]
        team = [(a[0], a[1], a[2], (a[3] + b[3]) / 2) for a, b in zip(lr, rfa)]
        for nm, o in (('LR+ElNiño', lr), ('Team', team)):
            pr = [(l, p) for _, _, l, p in o]; npos = sum(l for l, _ in pr); nneg = len(pr) - npos
            f = lambda t: (sum(1 for l, p in pr if l and p >= t) / npos, sum(1 for l, p in pr if not l and p >= t) / nneg, sum(1 for l, p in pr if (p >= t) == bool(l)) / len(pr))
            a, b = f(0.35), f(0.5); res[f'{c}|{nm}'] = dict(sens=a, caut=b, auc=auc(pr))
            print(f'{c:5s} | {nm:9s} | {a[0]:4.0%} / {a[1]:4.0%} | {b[0]:4.0%} / {b[1]:4.0%} / {b[2]:4.0%} | {nneg/len(pr):4.0%}   AUC {auc(pr):.2f}')
    json.dump(res, open(f'{OUT}/final_fixed_thresholds.json', 'w'), indent=1)
