"""Investigation 3: CHIRPS satellite rain, NOAA crop-health index (VHI), and REAL harvest statistics (KRSO).
Usage: python3 -I new_sources_test.py zones.json ../backtest_data/openmeteo ../backtest_data/modis ../backtest_data/enso/oni.ascii.txt out_deep ../backtest_data/chirps/chirps_monthly_zones.csv ../backtest_data/vhi/vhi_weekly_provinces.csv ../backtest_data/harvest_stats/krso_wheat_barley_governorates.csv"""
import sys, os, json, csv, math, random, statistics as st
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import deep_investigation as D
from deep_investigation import lr_fit, lr_pred, matrix, auc, catch_at, brier_skill, boot_better, NORMAL
ZF, OM, MD, ONIF, OUTD, CHF, VHF, HVF = sys.argv[1:9]
GOV = {'Sulaymaniyah': ['Slemani', 'Bazian', 'Chamchamal', 'Penjwen', 'Sharazur', 'Qaradagh', 'Ranya', 'Halabja', 'Garmiyan'],
       'Erbil': ['Erbil', 'Koya', 'Soran', 'Makhmour'], 'Duhok': ['Duhok', 'Zakho', 'Akre']}
Z2G = {z: g for g, zs in GOV.items() for z in zs}
CUTM = {'Dec': (0, 12), 'Jan': (1, 1), 'Feb': (1, 2), 'Mar': (1, 3)}          # (year offset, month)
WEEKS = {'Dec': (0, 49, 52), 'Jan': (1, 1, 4), 'Feb': (1, 5, 8), 'Mar': (1, 9, 13)}
BASE = ['rain_pct', 'soil_pct', 'temp_anom', 'et0_pct', 'last_rain_pct']
def P(*a): print(*a, flush=True)
# ---------- load ----------
CH = {}
for r in csv.DictReader(open(CHF)): CH[(r['zone'], int(r['year']), int(r['month']))] = float(r['precip_mm'])
def ch_sum(z, s, cut):
    yo, m = CUTM[cut]; months = [(s, 10), (s, 11), (s, 12)] + [(s + 1, k) for k in range(1, m + 1)] if yo else [(s, k) for k in range(10, m + 1)]
    v = [CH.get((z, y, mm)) for y, mm in months]; return sum(v) if None not in v else None
VH = {}
for r in csv.DictReader(open(VHF)): VH[(r['province'], int(r['year']), int(r['week']))] = (float(r['vci']), float(r['tci']), float(r['vhi']))
def vh_at(g, s, cut):
    yo, w1, w2 = WEEKS[cut]; v = [VH.get((g, s + yo, w)) for w in range(w1, w2 + 1)]; v = [x for x in v if x]
    return tuple(st.mean(x[i] for x in v) for i in range(3)) if v else None
HV = {}
for r in csv.DictReader(open(HVF)):
    if r['system'] not in ('all', 'all_derived') or r['governorate'] not in GOV: continue
    try: a, p = float(r['area']), float(r['production'])
    except ValueError: continue
    if a > 0: HV.setdefault((r['governorate'], r['crop']), {})[int(r['harvest_year'])] = p / a * 1000   # kg per donum
if __name__ == '__main__':
    D.SEASONS = list(range(1991, 2025))                    # weather features back to 1991/92 for the harvest test
    rows = D.build(ZF, OM, MD, ONIF)
    for r in rows:
        if r['cut'] in CUTM:
            c = ch_sum(r['zone'], r['season'], r['cut']); r['_ch'] = c
            vh = vh_at(Z2G[r['zone']], r['season'], r['cut'])
            if vh: r['vci'], r['tci'], r['vhi'] = vh
    for z in Z2G:
        for cut in CUTM:
            n = st.mean(ch_sum(z, s, cut) for s in NORMAL)
            for r in rows:
                if r['zone'] == z and r['cut'] == cut and r.get('_ch') is not None: r['chirps_pct'] = r['_ch'] / n * 100; r['rain_avg2'] = (r['chirps_pct'] + r['rain_pct']) / 2
    SAT = [s for s in range(1999, 2025)]
    def lr_cv(Rc, feats, seasons, key='label'):
        out = []
        for s in seasons:
            tr = [r for r in Rc if r['season'] != s and r['season'] in seasons and r.get(key) is not None]
            te = [r for r in Rc if r['season'] == s and r.get(key) is not None]
            if not te: continue
            Xtr, mn = matrix(tr, feats); Xte, _ = matrix(te, feats, mn); m = lr_fit(Xtr, [r[key] for r in tr])
            out += [(s, r['zone'], r[key], lr_pred(m, x)) for r, x in zip(te, Xte)]
        return out
    def sc(o, ref=None):
        pr = [(l, p) for _, _, l, p in o]; ct, _ = catch_at(pr)
        return f'AUC {auc(pr):.2f} catch@15%FA {ct:4.0%}' + (f'  beats ref in {boot_better(o, ref):.0%}' if ref else '')
    P('A+B. Satellite-greenness outcome (16 zones, 1999/00–2024/25). Reference = LR + El Niño')
    for c in CUTM:
        Rc = [r for r in rows if r['cut'] == c]; b = BASE + (['green_now'] if c in ('Feb', 'Mar') else []) + ['oni']
        ref = lr_cv(Rc, b, SAT); P(f'  {c}: reference {sc(ref)}')
        for nm, f in [('+ CHIRPS rain', b + ['chirps_pct']), ('CHIRPS instead of ERA5 rain', [x if x != 'rain_pct' else 'chirps_pct' for x in b]),
                      ('average of both rains', [x if x != 'rain_pct' else 'rain_avg2' for x in b]), ('+ NOAA crop health (VCI, TCI)', b + ['vci', 'tci']),
                      ('+ NOAA VHI only', b + ['vhi']), ('+ CHIRPS + VCI/TCI', b + ['chirps_pct', 'vci', 'tci'])]:
            P(f'     {nm:32s} {sc(lr_cv(Rc, f, SAT), ref)}')
    # ---------- C. real harvest ----------
    P('\nC. REAL HARVEST (KRSO wheat & barley yield per governorate)')
    G = {}                                                  # governorate-season rows = mean of its zones
    for g, zs in GOV.items():
        for c in list(CUTM) + ['Pre']:
            for s in range(1991, 2025):
                rr = [r for r in rows if r['zone'] in zs and r['cut'] == c and r['season'] == s]
                if not rr: continue
                x = dict(zone=g, season=s, cut=c)
                for k in BASE + ['oni', 'green_now', 'chirps_pct', 'vci', 'tci', 'vhi', 'outcome', 'last_green', 'sep_soil_pct']:
                    v = [r.get(k) for r in rr if r.get(k) is not None]; x[k] = st.mean(v) if v else None
                G[(g, c, s)] = x
    for crop in ('wheat', 'barley'):
        lab = {}
        for g in GOV:
            y = HV.get((g, crop), {}); yrs = [t for t in range(1992, 2025) if t in y]
            # remove the slow technology trend: linear fit over the years
            mx = st.mean(yrs); my = st.mean(y[t] for t in yrs); b1 = sum((t - mx) * (y[t] - my) for t in yrs) / sum((t - mx) ** 2 for t in yrs)
            rel = {t: y[t] / (my + b1 * (t - mx)) * 100 for t in yrs}; q = sorted(rel.values())[len(rel) // 4]
            for t in yrs:
                for c in list(CUTM) + ['Pre']:
                    if (g, c, t - 1) in G: G[(g, c, t - 1)][f'{crop}_rel'] = rel[t]; G[(g, c, t - 1)][f'{crop}_bad'] = 1 if rel[t] <= q else 0
        # does the satellite outcome track the real harvest?
        pts = [(G[(g, 'Mar', s)]['outcome'], G[(g, 'Mar', s)].get(f'{crop}_rel'), g, s) for g in GOV for s in range(1999, 2024) if (g, 'Mar', s) in G and G[(g, 'Mar', s)].get('outcome') and G[(g, 'Mar', s)].get(f'{crop}_rel')]
        P(f'\n  {crop.upper()}: satellite late-spring greenness vs real yield (trend removed), 2000–2023: r = {st.correlation([p[0] for p in pts], [p[1] for p in pts]):+.2f} (n={len(pts)}); per governorate: ' +
          ', '.join(f"{g} {st.correlation([p[0] for p in pts if p[2]==g], [p[1] for p in pts if p[2]==g]):+.2f}" for g in GOV))
        vh = [(G[(g, 'Mar', s)]['vhi'], G[(g, 'Mar', s)].get(f'{crop}_rel')) for g in GOV for s in range(1991, 2024) if (g, 'Mar', s) in G and G[(g, 'Mar', s)].get('vhi') is not None and G[(g, 'Mar', s)].get(f'{crop}_rel')]
        P(f'  {crop.upper()}: NOAA VHI in March vs real yield, 1992–2023: r = {st.correlation([p[0] for p in vh], [p[1] for p in vh]):+.2f} (n={len(vh)})')
        P(f'  Predicting a bad {crop} harvest (governorate yield in its bottom 25%, trend removed), seasons hidden one by one:')
        key = f'{crop}_bad'; L91 = list(range(1991, 2024)); L99 = list(range(1999, 2023))
        for c in ['Pre'] + list(CUTM):
            Rc = [x for (g, cc, s), x in G.items() if cc == c]
            if c == 'Pre':
                lines = [('El Niño only, 1991–2023', ['oni'], L91)]
            else:
                w = BASE + ['oni']
                lines = [('weather + El Niño, 1991–2023 (33 seasons)', w, L91), ('+ NOAA crop health, 1991–2023', w + ['vci', 'tci'], L91),
                         ('+ CHIRPS rain, 1991–2023', w + ['chirps_pct'], L91), ('weather + El Niño, 1999–2022 only', w, L99)]
                if c in ('Feb', 'Mar'): lines.append(('+ MODIS greenness (our system), 1999–2022', w + ['green_now'], L99))
            ref = None
            for nm, f, S in lines:
                o = lr_cv(Rc, f, S, key)
                if ref is None or nm.startswith('weather + El Niño, 1999'): r0 = None if ref is None else (ref if S == L91 else None)
                txt = sc(o, ref if (ref and S == L91) else None); P(f'     {c:4s} {nm:44s} {txt}')
                if ref is None: ref = o
                if nm.startswith('weather + El Niño, 1999'): ref99 = o
                if 'MODIS' in nm: P(f'          (vs weather + El Niño on the same years: beats in {boot_better(o, ref99):.0%})')
    json.dump({f'{k[0]}|{k[1]}|{k[2]}': v for k, v in G.items()}, open(f'{OUTD}/governorate_rows.json', 'w'))
