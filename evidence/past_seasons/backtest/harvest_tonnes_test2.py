"""Tonnes = (how much was planted: recent level) × (how good the season is: satellite). Leave-one-year-out.
Usage: python3 -I harvest_tonnes_test2.py zones.json ../backtest_data/modis ../backtest_data/modis_full ../backtest_data/harvest_stats/krso_wheat_barley_governorates.csv out_deep"""
import sys, os, json, math, statistics as st
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import harvest_tonnes_test as H
from deep_investigation import solve
if __name__ == '__main__':
    S, GOV, PROD, zones = H.S, H.GOV, H.PROD, H.zones
    def X(g, y, k):
        return st.mean(S[z][y][k] for z in zones) if g == 'Kurdistan_Region' and all(S[z].get(y, {}).get(k) is not None for z in zones) else (H.gov_val(g, y, k) if g != 'Kurdistan_Region' else None)
    def ols(rows):  # rows: (features list, target)
        k = len(rows[0][0]) + 1; A = [[0.0] * k for _ in range(k)]; B = [0.0] * k
        for f, t in rows:
            v = [1.0] + f
            for a in range(k):
                B[a] += v[a] * t
                for b in range(k): A[a][b] += v[a] * v[b]
        for a in range(1, k): A[a][a] += 1e-6
        return solve(A, B)
    print('Wheat production, year hidden one by one, 2003–2023 (needs 3 earlier years). Error = |estimate − official| / official')
    print('  area             | satellite model: median / mean error | "same as last 3 years": median / mean | "same as last year": median / mean')
    for k, lab in (('april', 'MID-APRIL'), ('late', 'MID-MAY')):
        print(f' {lab}')
        for g in list(GOV) + ['Kurdistan_Region']:
            P = PROD[g]; yrs = [y for y in range(2003, 2024) if y in P and all(yy in P for yy in range(y - 3, y)) and X(g, y, k) is not None and all(X(g, yy, k) is not None for yy in range(y - 3, y))]
            def feat(y):
                lvl = st.mean(math.log(P[yy]) for yy in range(y - 3, y)); gl = st.mean(X(g, yy, k) for yy in range(y - 3, y))
                return lvl, (X(g, y, k) - gl) * 10
            E, B3, B1, out = [], [], [], []
            for y in yrs:
                tr = [(list(feat(t)[1:]), math.log(P[t]) - feat(t)[0]) for t in yrs if t != y]; w = ols(tr)
                lvl, dg = feat(y); est = math.exp(lvl + w[0] + w[1] * dg); out.append((y, est, P[y]))
                E.append(abs(est - P[y]) / P[y]); B3.append(abs(math.exp(lvl) - P[y]) / P[y]); B1.append(abs(P[y - 1] - P[y]) / P[y])
            print(f'  {g:16s} | {st.median(E):4.0%} / {st.mean(E):4.0%}                         | {st.median(B3):4.0%} / {st.mean(B3):4.0%}                    | {st.median(B1):4.0%} / {st.mean(B1):4.0%}')
            if g == 'Kurdistan_Region':
                print('     year: official → satellite estimate: ' + ', '.join(f'{y}: {t/1e3:.0f}k→{e/1e3:.0f}k' for y, e, t in out))
                # big-drop years: did the satellite see them?
                drops = [(y, e, t) for y, e, t in out if t < 0.75 * math.exp(feat(y)[0])]
                print('     years with a big drop (official < 75% of last-3-year level): ' + ', '.join(f'{y} official {t/math.exp(feat(y)[0]):.0%} of normal, satellite said {e/math.exp(feat(y)[0]):.0%}' for y, e, t in drops))
