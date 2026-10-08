"""Why was 2024/25 missed, and can we catch droughts better?  (2026-10-07)
Part 1 - RAIN DROUGHT ALARM: region-wide Oct-May rain in the bottom 25% of 36 winters (1990/91-2025/26).
         Inputs known at each cutoff: rain so far, soil moisture, rainy days, El Nino index. Each winter hidden in turn.
Part 2 - CROP MODEL: does dropping "last year was wet", or adding the region rain alarm, fix 2024/25 without hurting the rest?
Usage: python3 -I drought_alarm_test.py   (reads zones.json, ../backtest_data/openmeteo, ../backtest_data/enso/oni.ascii.txt, out_deep/features_all_v2.json)
"""
import sys, os, json, statistics as st
from multiprocessing import Pool
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from deep_investigation import W, ymd, CUTS, ONI_AT, lr_fit, lr_pred, auc, catch_at, boot_better, run_cv, rf_fit, rf_pred, matrix

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, '..', 'backtest_data')
NORMAL = range(1991, 2021)
WINTERS = list(range(1990, 2026))          # 1990/91 .. 2025/26
ACUTS = ['Pre', 'Dec', 'Jan', 'Feb', 'Mar']
lines = []
def P(s=''):
    print(s); lines.append(s)

def cdate(s, cut):
    m, d = CUTS[cut]; return ymd(s if m >= 9 else s + 1, m, d)

# ---------------- Part 1 ----------------
def part1():
    zones = [z['name'] for z in json.load(open(os.path.join(HERE, 'zones.json')))['zones']]
    ONI = {}
    for line in open(os.path.join(DATA, 'enso', 'oni.ascii.txt')).read().splitlines()[1:]:
        a, y, t, an = line.split(); ONI[(a, int(y))] = float(an)
    Wz = {z: W(os.path.join(DATA, 'openmeteo', f'{z}.json')) for z in zones}

    def zone_feats(w, s, cut):
        a = ymd(s, 10, 1)
        if cut == 'Pre':
            return dict(soil=st.mean(w.back(ymd(s, 9, 30), 7, 'soil_moisture_0_to_100cm_mean')))
        b = cdate(s, cut); pr = w.rng(a, b, 'precipitation_sum')
        return dict(rain=sum(pr), soil=st.mean(w.back(b, 7, 'soil_moisture_0_to_100cm_mean')), days=sum(1 for x in pr if x >= 1))

    region = {}       # (season, cut) -> dict of region % values
    season_total = {}
    for z in zones:
        w = Wz[z]
        tot = {s: w.sum(ymd(s, 10, 1), ymd(s + 1, 5, 31), 'precipitation_sum') for s in WINTERS}
        nt = st.mean(tot[s] for s in NORMAL)
        for s in WINTERS:
            season_total.setdefault(s, []).append(tot[s] / nt * 100)
        for cut in ACUTS:
            N = [zone_feats(w, s, cut) for s in NORMAL]
            Nm = {k: st.mean(x[k] for x in N) for k in N[0]}
            for s in WINTERS:
                f = zone_feats(w, s, cut)
                d = region.setdefault((s, cut), {})
                for k, v in f.items():
                    d.setdefault(k, []).append(v / Nm[k] * 100)
    rain = {s: st.mean(v) for s, v in season_total.items()}
    thr = sorted(rain.values())[len(rain) // 4]
    drought = {s: 1 if rain[s] <= thr else 0 for s in WINTERS}
    for (s, cut), d in region.items():
        for k in list(d):
            d[k] = st.mean(d[k])
        sname, dy = ONI_AT[cut]; d['oni'] = ONI.get((sname, s + dy))

    P('PART 1 - RAIN DROUGHT ALARM (whole region)')
    P(f'Drought = Oct-May rain, mean of 16 zones, <= {thr:.0f}% of normal (bottom 25%): ' +
      ', '.join(f'{s}/{(s+1)%100:02d} ({rain[s]:.0f}%)' for s in WINTERS if drought[s]))
    ND = sum(drought.values())
    P(f'{ND} drought winters, {len(WINTERS) - ND} normal. Each winter hidden in turn. "caught" = droughts flagged at the stated number of false alarms (normal winters flagged).\n')

    def loo_lr(cut, feats):
        out = {}
        for s in WINTERS:
            tr = [t for t in WINTERS if t != s]
            X = [[region[(t, cut)][f] for f in feats] for t in tr]; y = [drought[t] for t in tr]
            m = lr_fit(X, y, 0.05); out[s] = lr_pred(m, [region[(s, cut)][f] for f in feats])
        return out

    def report(name, score):
        pairs = [(drought[s], score[s]) for s in WINTERS]
        a = auc(pairs)
        neg = sorted((score[s] for s in WINTERS if not drought[s]), reverse=True)
        res = []
        for nfa in (0, 2, 4):
            t = neg[nfa] if nfa < len(neg) else -1e9
            res.append(sum(1 for s in WINTERS if drought[s] and score[s] > t))
        rank25 = 1 + sum(1 for s in WINTERS if score[s] > score[2024])
        P(f'  {name:34s} AUC {a:.2f} | caught with 0 / 2 / 4 false alarms: {res[0]}/{ND}, {res[1]}/{ND}, {res[2]}/{ND} | 2024/25 rank {rank25:2d} of 36')
        return a

    SC = {}
    for cut in ACUTS:
        P(f'== {cut} (data up to {cdate(2024, cut)[5:]}) ==')
        if cut == 'Pre':
            SC[(cut, 'El Nino only')] = {s: -region[(s, cut)]['oni'] for s in WINTERS}
            report('El Nino only (rule)', SC[(cut, 'El Nino only')])
            SC[(cut, 'LR oni+soil')] = loo_lr(cut, ['oni', 'soil'])
            report('model: El Nino + Sept soil', SC[(cut, 'LR oni+soil')])
        else:
            SC[(cut, 'rain')] = {s: -region[(s, cut)]['rain'] for s in WINTERS}
            report('rain so far only (rule)', SC[(cut, 'rain')])
            SC[(cut, 'soil')] = {s: -region[(s, cut)]['soil'] for s in WINTERS}
            report('soil moisture only (rule)', SC[(cut, 'soil')])
            for nm, fs in (('rain+oni', ['rain', 'oni']), ('rain+soil+oni', ['rain', 'soil', 'oni']), ('rain+soil+days+oni', ['rain', 'soil', 'days', 'oni'])):
                SC[(cut, nm)] = loo_lr(cut, fs)
                report('model: ' + nm.replace('oni', 'El Nino').replace('days', 'rainy days'), SC[(cut, nm)])
        P()

    P('Per drought winter, chance of drought from model "rain+soil+El Nino" (El Nino+soil before the season):')
    P('  winter    final rain | Oct   Dec   Jan   Feb   Mar')
    for s in WINTERS:
        if not drought[s] and s not in (2025,):
            continue
        row = [SC[('Pre', 'LR oni+soil')][s]] + [SC[(c, 'rain+soil+oni')][s] for c in ACUTS[1:]]
        P(f'  {s}/{(s+1)%100:02d}   {rain[s]:5.0f}%     | ' + '  '.join(f'{p:4.0%}' for p in row) + ('' if drought[s] else '   (not a drought)'))
    fa = {c: sorted([s for s in WINTERS if not drought[s]], key=lambda s: -SC[(c, 'rain+soil+oni')][s])[:3] for c in ACUTS[1:]}
    P('Highest-scored NORMAL winters (would be false alarms): ' + '; '.join(f'{c}: ' + ', '.join(f'{s}/{(s+1)%100:02d} {SC[(c,"rain+soil+oni")][s]:.0%} (final {rain[s]:.0f}%)' for s in fa[c]) for c in fa))
    P()
    P('Simple rule tables: rain since 1 Oct (% of normal, mean of 16 zones) at the cutoff, driest first. D = drought winter.')
    for c in ('Jan', 'Feb'):
        order = sorted(WINTERS, key=lambda s: region[(s, c)]['rain'])
        P(f'  {c}: ' + ', '.join(f"{s}/{(s+1)%100:02d} {region[(s, c)]['rain']:.0f}%{' D' if drought[s] else ''}" for s in order[:14]))
    P()


# ---------------- Part 2 ----------------
def team_job(job):
    cut, name, feats_lr, feats_rf, rows = job
    out = []
    seasons = sorted(set(r['season'] for r in rows if r['label'] is not None))
    for s in seasons:
        tr = [r for r in rows if r['season'] != s and r['label'] is not None]; te = [r for r in rows if r['season'] == s and r['label'] is not None]
        if not te: continue
        y = [r['label'] for r in tr]
        X1, m1 = matrix(tr, feats_lr); T1, _ = matrix(te, feats_lr, m1); lr = lr_fit(X1, y, 0.05)
        X2, m2 = matrix(tr, feats_rf); T2, _ = matrix(te, feats_rf, m2); F = rf_fit(X2, y, seed=s)
        out += [(s, r['zone'], r['label'], (lr_pred(lr, a) + rf_pred(F, b)) / 2) for r, a, b in zip(te, T1, T2)]
    return (cut, name, out)

if __name__ == '__main__':
    part1()
    rows = json.load(open(os.path.join(HERE, 'out_deep', 'features_all_v2.json')))
    BASE = ['rain_pct', 'soil_pct', 'temp_anom', 'et0_pct', 'last_rain_pct']; TIM = ['first_rain', 'rainy_days_pct', 'dry_spell', 'recent_pct']
    jobs = []
    for c in ('Dec', 'Jan', 'Feb', 'Mar'):
        g = ['green_now'] if c in ('Feb', 'Mar') else []
        cur_lr = BASE + g + ['oni']; cur_rf = BASE + g + ['oni'] + TIM + ['region_rain', 'last_green']
        nolast = lambda fs: [f for f in fs if f not in ('last_rain_pct', 'last_green')]
        R = [r for r in rows if r['cut'] == c]
        jobs += [(c, 'Team now (reference)', cur_lr, cur_rf, R),
                 (c, 'Team without last-year inputs', nolast(cur_lr), nolast(cur_rf), R),
                 (c, 'Team + region rain in LR', cur_lr + ['region_rain'], cur_rf, R),
                 (c, 'Team no last-year + region rain', nolast(cur_lr) + ['region_rain'], nolast(cur_rf), R)]
    with Pool(min(8, os.cpu_count() or 4)) as pool:
        res = pool.map(team_job, jobs)
    OUT = {(c, n): o for c, n, o in res}
    P('PART 2 - CROP MODEL (16 zones x 26 seasons, bad = late-spring greenness in zone bottom 25%; Team = simple regression + random forest)')
    P('"beats now" = share of 1000 season reshuffles where it beats the current Team (>= 0.90 = probably real).')
    for c in ('Dec', 'Jan', 'Feb', 'Mar'):
        P(f'== {c} ==')
        ref = OUT[(c, 'Team now (reference)')]
        for (cc, n), o in OUT.items():
            if cc != c: continue
            pairs = [(l, p) for _, _, l, p in o]; a = auc(pairs); ct, f = catch_at(pairs)
            seas = {}
            for s, z, l, p in o: seas.setdefault(s, []).append(p)
            m = {s: st.mean(v) for s, v in seas.items()}
            rk = 1 + sum(1 for s in m if m[s] > m[2024])
            vs = '' if o is ref else f'beats now {boot_better(o, ref):.2f}'
            P(f'  {n:34s} AUC {a:.3f}  caught@15%FA {ct:4.0%}  2024/25 region risk {m[2024]:4.0%} (rank {rk:2d}/26)  {vs}')
        P()
    # why: regression weights (standardised) of the current LR at Jan, trained on all seasons
    R = [r for r in rows if r['cut'] == 'Jan' and r['label'] is not None]
    fs = BASE + ['oni']; X, _ = matrix(R, fs); m = lr_fit(X, [r['label'] for r in R], 0.05)
    P('Jan regression weights (per 1 standard deviation; + = more risk): ' + ', '.join(f'{f} {w:+.2f}' for f, w in zip(fs, m['w'][1:])))
    r24 = [r for r in R if r['season'] == 2024]
    P('2024/25 Jan inputs (region mean): ' + ', '.join(f'{f} {st.mean(r[f] for r in r24):.1f}' for f in fs))
    open(os.path.join(HERE, 'drought_alarm_test_output.txt'), 'w').write('\n'.join(lines) + '\n')
