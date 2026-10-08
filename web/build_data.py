"""Build web/data.js for the Wheat Drought Alarm web app from our tested data (2026-10-08).
Everything shown in the app comes from here: hex map of the region, rain-so-far curves, the honest (hidden-winter)
alarm decisions, the Team model's hidden-season zone risks, the real outcomes and official KRSO wheat.
Usage: python3 -I build_data.py [--refresh-forecast] [--refresh-oni]   (writes data.js next to this file;
       --refresh-forecast first downloads a fresh 16-day forecast for the 16 zones from Open-Meteo, e.g. on demo morning;
       --refresh-oni first downloads NOAA's latest El Nino index file, oni.ascii.txt, published early each month)"""
import os, sys, json, math, csv, statistics as st
WEB = os.path.dirname(os.path.abspath(__file__))
PS = os.path.join(WEB, '..', 'evidence', 'past_seasons'); BT = os.path.join(PS, 'backtest'); DATA = os.path.join(PS, 'backtest_data')
sys.path.insert(0, BT)
from deep_investigation import W, ymd

# ---------------- areas, zones, hex map ----------------
AREAS = [('duhok', 'Duhok', 'دهۆک', ['Al-Amadiya', 'Duhok', 'Sumail', 'Zakho', 'Aqra'], 'Duhok'),
         ('erbil', 'Erbil', 'هەولێر', ['Al-Zibar', 'Erbil', 'Koysinjaq', 'Makhmour', 'Rawanduz', 'Shaqlawa'], 'Erbil'),
         ('slemani', 'Slemani', 'سلێمانی', ['Al-Sulaymaniyah', 'Chamchamal', 'Derbendikhan', 'Dokan', 'Panjwin', 'Pshdar', 'Rania', 'Sharbazher'], 'Sulaymaniyah'),
         ('halabja', 'Halabja', 'هەڵەبجە', ['Halabcha'], 'Halabja'),
         ('garmiyan', 'Garmiyan', 'گەرمیان', ['Kalar'], 'Garmiyan')]
KU = {'Slemani': 'سلێمانی', 'Garmiyan': 'گەرمیان', 'Chamchamal': 'چەمچەماڵ', 'Bazian': 'بازیان', 'Ranya': 'ڕانیە', 'Penjwen': 'پێنجوێن',
      'Sharazur': 'شارەزوور', 'Qaradagh': 'قەرەداغ', 'Erbil': 'هەولێر', 'Duhok': 'دهۆک', 'Zakho': 'زاخۆ', 'Soran': 'سۆران', 'Koya': 'کۆیە',
      'Halabja': 'هەڵەبجە', 'Akre': 'ئاکرێ', 'Makhmour': 'مەخموور'}
zones = json.load(open(os.path.join(BT, 'zones.json')))['zones']
D = json.load(open(os.path.join(DATA, 'boundaries', 'iraq_districts_adm2.geojson')))['features']
def rings(g): return [g['coordinates'][0]] if g['type'] == 'Polygon' else [p[0] for p in g['coordinates']]
def inside(pt, ring):
    x, y = pt; c = False; n = len(ring)
    for i in range(n):
        x1, y1 = ring[i]; x2, y2 = ring[(i + 1) % n]
        if (y1 > y) != (y2 > y) and x < (x2 - x1) * (y - y1) / (y2 - y1) + x1: c = not c
    return c
dist_area = {d: ai for ai, a in enumerate(AREAS) for d in a[3]}
dpolys = [(dist_area[f['properties']['shapeName']], rings(f['geometry'])) for f in D if f['properties']['shapeName'] in dist_area]
def area_of(lon, lat):
    for ai, rs in dpolys:
        if any(inside((lon, lat), r) for r in rs): return ai
    return None
LON0, LAT0 = 44.0, 36.0; KX = math.cos(math.radians(36)) * 111.32; KY = 110.57
def to_km(lon, lat): return ((lon - LON0) * KX, (lat - LAT0) * KY)
def to_ll(x, y): return (LON0 + x / KX, LAT0 + y / KY)
R = 4.6                                                           # hex radius, km (pointy top)
zone_area = [area_of(z['lon'], z['lat']) for z in zones]
assert None not in zone_area, zone_area
zkm = [to_km(z['lon'], z['lat']) for z in zones]
hexes = []
for r in range(-60, 61):
    for q in range(-80, 81):
        x = R * math.sqrt(3) * (q + r / 2); y = -R * 1.5 * r       # r grows southwards
        lon, lat = to_ll(x, y)
        if not (42.2 <= lon <= 46.4 and 34.2 <= lat <= 37.5): continue
        ai = area_of(lon, lat)
        if ai is None: continue
        cand = [i for i in range(len(zones)) if zone_area[i] == ai]
        zi = min(cand, key=lambda i: (zkm[i][0] - x) ** 2 + (zkm[i][1] - y) ** 2)
        hexes.append([q, r, ai, zi])
print(len(hexes), 'hexes')

# ---------------- rain so far (daily) ----------------
SEASONS = list(range(1990, 2027))                                 # 1990/91 .. 2026/27
NDAYS = 182                                                       # 1 Oct .. 31 Mar
def day_keys(s):
    """1 Oct .. 31 Mar as 182 slots; a 29 Feb is folded into 1 Mar so slot 150 = 28 Feb and slot 181 = 31 Mar every year."""
    import datetime as dt
    d0 = dt.date(s, 10, 1); out = []; carry = []
    for i in range(183):
        d = d0 + dt.timedelta(days=i)
        if d.month == 2 and d.day == 29: carry = [d.isoformat()]; continue
        out.append(carry + [d.isoformat()]); carry = []
        if len(out) == NDAYS: break
    return out
zone_pct = {}; zone_pctf = {}; zone_total = {}
for zi, z in enumerate(zones):
    w = W(os.path.join(DATA, 'openmeteo', z['name'] + '.json'))
    cum = {}
    for s in SEASONS:
        acc = 0.0; arr = []
        for slot in day_keys(s):
            idx = [w.ix.get(k) for k in slot]
            if None in idx or any(w.d['precipitation_sum'][i] is None for i in idx): break
            acc += sum(w.d['precipitation_sum'][i] for i in idx); arr.append(acc)
        cum[s] = arr
    norm = [st.mean(cum[s][i] for s in range(1991, 2021)) for i in range(NDAYS)]
    for s in SEASONS:
        zone_pctf[(zi, s)] = [c / max(norm[i], 0.5) * 100 for i, c in enumerate(cum[s])]
        zone_pct[(zi, s)] = [round(min(999, v)) for v in zone_pctf[(zi, s)]]
    tot = {s: w.sum(ymd(s, 10, 1), ymd(s + 1, 5, 31), 'precipitation_sum') for s in range(1990, 2026)}
    nt = st.mean(tot[s] for s in range(1991, 2021))
    for s in tot: zone_total[(zi, s)] = tot[s] / nt * 100
def region_pct(s):
    n = min(len(zone_pctf[(zi, s)]) for zi in range(len(zones)))
    return [round(min(999, st.mean(zone_pctf[(zi, s)][i] for zi in range(len(zones))))) for i in range(n)]
season_rain = {s: st.mean(zone_total[(zi, s)] for zi in range(len(zones))) for s in range(1990, 2026)}
thr = sorted(season_rain.values())[len(season_rain) // 4]
DROUGHT = {s: season_rain[s] <= thr for s in season_rain}

# ---------------- honest alarm decisions (cut-off learned without the judged winter) ----------------
CUT_DAY = {'jan': 122, 'feb': 150, 'mar': 181}                    # day index from 1 Oct: 31 Jan, 28 Feb, 31 Mar
def sofar(s, cut):                                                # same as the rule test: mean of zone % (zone rain so far / zone normal so far)
    return st.mean(zone_pct_exact[(zi, s, cut)] for zi in range(len(zones)))
zone_pct_exact = {}
for zi, z in enumerate(zones):
    w = W(os.path.join(DATA, 'openmeteo', z['name'] + '.json'))
    for cut, (m, d) in (('dec', (12, 31)), ('jan', (1, 31)), ('feb', (2, 28)), ('mar', (3, 31)), ('nov', (11, 30))):
        r = {s: w.sum(ymd(s, 10, 1), ymd(s if m >= 9 else s + 1, m, d), 'precipitation_sum') for s in range(1990, 2026)}
        nr = st.mean(r[s] for s in range(1991, 2021))
        for s in r: zone_pct_exact[(zi, s, cut)] = r[s] / nr * 100
WIN = list(range(1990, 2026))
alarm = {}
for cut, K in (('jan', 0), ('feb', 0), ('mar', 1)):
    for s in WIN:
        normals = sorted(sofar(t, cut) for t in WIN if t != s and not DROUGHT[t])
        T = normals[K]
        alarm[(s, cut)] = dict(value=round(sofar(s, cut)), cutoff=round(T), on=sofar(s, cut) < T)
final_cutoffs = {cut: round(sorted(sofar(t, cut) for t in WIN if not DROUGHT[t])[K]) for cut, K in (('jan', 0), ('feb', 0), ('mar', 1))}

# December watch: rain Oct-Nov + 7 world forecasts (Dec start, Dec-Feb), linear model + watch cut-off, both learned without the judged winter
import seasonal_models_test as SM
fc = SM.load_models(os.path.join(DATA, 'seasonal_models'))
TEST = list(range(1991, 2026))
def mraw(s):
    out = {}
    for m, d in fc.items():
        f = d.get((s, 12))
        if f and all(li in f for li in (0, 1, 2)): out[m] = sum(f[li] for li in (0, 1, 2))
    return out
def mz(s, train):
    zs = []
    for m, v in mraw(s).items():
        tr = [mraw(t).get(m) for t in train]; tr = [x for x in tr if x is not None]
        if len(tr) >= 20: zs.append((v - st.mean(tr)) / st.stdev(tr))
    return st.mean(zs) if zs else None
def fit(X, Y):
    k = len(X[0]) + 1; A = [[0.0] * k for _ in range(k)]; B = [0.0] * k
    for x, y in zip(X, Y):
        v = [1.0] + x
        for a in range(k):
            B[a] += v[a] * y
            for b in range(k): A[a][b] += v[a] * v[b] + (0.01 if a == b and a > 0 else 0)
    for c in range(k):
        piv = max(range(c, k), key=lambda r: abs(A[r][c])); A[c], A[piv] = A[piv], A[c]; B[c], B[piv] = B[piv], B[c]
        for r in range(k):
            if r != c and A[c][c]:
                f = A[r][c] / A[c][c]; A[r] = [A[r][j] - f * A[c][j] for j in range(k)]; B[r] -= f * B[c]
    return [B[j] / A[j][j] for j in range(k)]
def pred_dec(s, train):
    X = [[sofar(t, 'nov'), mz(t, [u for u in train if u != t])] for t in train]; Y = [season_rain[t] for t in train]
    w = fit(X, Y); x = [sofar(s, 'nov'), mz(s, train)]
    return w[0] + w[1] * x[0] + w[2] * x[1]
print('December watch: leave-one-out predictions (slow part)...', flush=True)
decpred = {s: pred_dec(s, [t for t in TEST if t != s]) for s in TEST}
watch = {}
for s in TEST:
    normals = sorted(decpred[t] for t in TEST if t != s and not DROUGHT[t])
    T = normals[min(4, len(normals) - 1)]                           # watch if below the 5th-lowest normal-winter prediction (<= 4 false watches)
    watch[s] = dict(pred=round(decpred[s]), cutoff=round(T), on=decpred[s] < T)

# ---------------- zone crop risk (Team model, hidden-season predictions) ----------------
team = {}
rows = {}
for r in csv.DictReader(open(os.path.join(BT, 'out_deep', 'oof_predictions.csv'))):
    key = None
    if r['cut'] in ('Dec', 'Jan', 'Feb', 'Mar'):
        if r['set'] == '+ El Niño' and r['model'] == 'LR': key = 'lr'
        elif r['set'] == 'all' and r['model'] == 'RF': key = 'rf'
    if key: rows.setdefault((r['cut'], int(r['season']), r['zone']), {})[key] = (float(r['risk']), int(r['label']))
for (cut, s, z), v in rows.items():
    if 'lr' in v and 'rf' in v: team[(cut, s, z)] = ((v['lr'][0] + v['rf'][0]) / 2, v['lr'][1])
lights = {}
for cut in ('Dec', 'Jan', 'Feb', 'Mar'):
    pairs = [(p, l) for (c, s, z), (p, l) in team.items() if c == cut]
    neg = sorted((p for p, l in pairs if not l), reverse=True)
    lights[cut] = dict(red=neg[int(0.08 * len(neg))], yellow=neg[int(0.18 * len(neg))])
    pos = [p for p, l in pairs if l]
    lights[cut]['caught_red'] = round(sum(1 for p in pos if p > lights[cut]['red']) / len(pos) * 100)
    lights[cut]['caught_yellow'] = round(sum(1 for p in pos if p > lights[cut]['yellow']) / len(pos) * 100)
outcome = {}
for r in json.load(open(os.path.join(BT, 'out_deep', 'features_all_v2.json'))):
    if r['cut'] == 'Pre' and r.get('outcome') is not None: outcome[(r['season'], r['zone'])] = r['outcome']

# ---------------- official wheat (KRSO) ----------------
krso = {}
for r in csv.DictReader(open(os.path.join(DATA, 'harvest_stats', 'krso_wheat_barley_governorates.csv'))):
    if r['crop'] == 'wheat' and r['system'] == 'all' and r['production']:
        krso.setdefault(r['governorate'], {})[int(r['harvest_year'])] = dict(t=round(float(r['production'])),
                                                                            y=round(float(r['production']) / float(r['area']) * 1000) if r['area'] else None)
ONIF = os.path.join(DATA, 'enso', 'oni.ascii.txt')
if '--refresh-oni' in sys.argv:
    import urllib.request
    raw = urllib.request.urlopen(urllib.request.Request('https://www.cpc.ncep.noaa.gov/data/indices/oni.ascii.txt', headers={'User-Agent': 'Mozilla/5.0'}), timeout=60).read().decode()
    if 'SEAS' in raw and 'JAS' in raw: open(ONIF, 'w').write(raw)
ONI = {}
for l in open(ONIF).read().splitlines()[1:]:
    a, y, t, an = l.split(); ONI[(a, int(y))] = float(an)
def oct_call(v):
    if v is None: return None
    if v >= 0.5: return 'wet'
    if v <= -0.5: return 'dry'
    return 'none'

# ---------------- this winter (2026/27): world forecasts, 16-day forecast, El Nino history per zone ----------------
import re, glob, datetime as dt, urllib.request
def model_block(title):
    txt = open(os.path.join(BT, 'seasonal_models_test_output.txt')).read()
    blk = txt[txt.index(title):].split('\n')[1:9]
    out = []
    for l in blk:
        m = re.match(r'\s+(\S+)\s+(\d+) mm\s+=\s+(\d+)% of own normal', l)
        if m: out.append(dict(model=m.group(1), pct=int(m.group(3))))
    return out
FDIR = os.path.join(DATA, 'forecast')
if '--refresh-forecast' in sys.argv:
    lat = ','.join(str(z['lat']) for z in zones); lon = ','.join(str(z['lon']) for z in zones)
    url = ('https://api.open-meteo.com/v1/forecast?latitude=' + lat + '&longitude=' + lon +
           '&daily=precipitation_sum,precipitation_probability_max&forecast_days=16&timezone=Asia%2FBaghdad')
    raw = urllib.request.urlopen(urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'}), timeout=60).read()
    open(os.path.join(FDIR, 'forecast16_' + dt.date.today().isoformat() + '.json'), 'wb').write(raw)
fc16f = sorted(glob.glob(os.path.join(FDIR, 'forecast16_*.json')))[-1]
fc16 = json.load(open(fc16f))
def nearest_fc(z):
    return min(fc16, key=lambda f: (f['latitude'] - z['lat']) ** 2 + (f['longitude'] - z['lon']) ** 2)
next16 = []
for z in zones:
    f = nearest_fc(z)['daily']
    next16.append(dict(mm=round(sum(v or 0 for v in f['precipitation_sum'])), days=sum(1 for v in f['precipitation_sum'] if (v or 0) >= 1),
                       first=f['time'][0], last=f['time'][-1]))
nino = [s for s in range(1999, 2025) if (ONI.get(('JAS', s)) or 0) >= 0.5]
nina = [s for s in range(1999, 2025) if (ONI.get(('JAS', s)) or 0) <= -0.5]
def bad_count(zname, seasons): return sum(team[('Mar', s, zname)][1] for s in seasons)
live = dict(models_octdec=model_block('-- start Oct 2026, target Oct-Dec'), models_octmay=model_block('-- start Oct 2026, target Oct-May'),
            next16=next16, next16_made=os.path.basename(fc16f)[11:21],
            nino_seasons=[f'{s}/{(s + 1) % 100:02d}' for s in nino], nina_seasons=[f'{s}/{(s + 1) % 100:02d}' for s in nina],
            nino_bad=[bad_count(z['name'], nino) for z in zones], nina_bad=[bad_count(z['name'], nina) for z in zones],
            strong=[s for s in range(1990, 2026) if (ONI.get(('JAS', s)) or 0) >= 1.0],
            oni_latest=' '.join(map(str, sorted(ONI, key=lambda k: (k[1], ['DJF','JFM','FMA','MAM','AMJ','MJJ','JJA','JAS','ASO','SON','OND','NDJ'].index(k[0])))[-1])))

# ---------------- assemble ----------------
out = dict(
    generated='2026-10-08',
    areas=[dict(id=a[0], name=a[1], ku=a[2], krso=a[4]) for a in AREAS],
    zones=[dict(name=z['name'], ku=KU[z['name']], lat=z['lat'], lon=z['lon'], area=zone_area[i], km=[round(v, 1) for v in zkm[i]]) for i, z in enumerate(zones)],
    hex=dict(R=R, cells=hexes),
    cutoffs=dict(final_cutoffs, watch_note='predicted season rain below the 5th-lowest normal winter'),
    lights={c: dict(red=round(v['red'], 3), yellow=round(v['yellow'], 3), caught_red=v['caught_red'], caught_yellow=v['caught_yellow']) for c, v in lights.items()},
    live=live, seasons={})
for s in range(1990, 2027):
    e = dict(label=f'{s}/{(s + 1) % 100:02d}', oni=ONI.get(('JAS', s)), call=oct_call(ONI.get(('JAS', s))), rain=region_pct(s),
             onim={k: ONI.get((k, s)) for k in ('MAM', 'AMJ', 'MJJ', 'JJA', 'JAS', 'ASO', 'SON', 'OND')})
    e['zoneRain'] = [zone_pct[(zi, s)] for zi in range(len(zones))] if s >= 1999 else None
    if s in season_rain:
        e['final'] = round(season_rain[s]); e['drought'] = DROUGHT[s]
        e['alarms'] = {c: alarm[(s, c)] for c in ('jan', 'feb', 'mar')}
    if s in watch: e['watch'] = watch[s]
    if 1999 <= s <= 2024:
        e['risk'] = {c: [round(team[(c, s, z['name'])][0], 3) for z in zones] for c in ('Dec', 'Jan', 'Feb', 'Mar')}
        e['bad'] = [team[('Mar', s, z['name'])][1] for z in zones]
        e['green'] = [round(outcome[(s, z['name'])]) if (s, z['name']) in outcome else None for z in zones]
    hy = s + 1
    e['wheat'] = {k: krso[g].get(hy) for k, g in [('kri', 'Kurdistan_Region')] + [(a[0], a[4]) for a in AREAS] if g in krso and krso[g].get(hy)}
    prev = [krso['Kurdistan_Region'].get(hy - k) for k in (1, 2, 3)]
    if e['wheat'].get('kri') and all(prev): e['wheatVs3'] = round(e['wheat']['kri']['t'] / st.mean(p['t'] for p in prev) * 100)
    out['seasons'][str(s)] = e
js = 'window.DATA = ' + json.dumps(out, separators=(',', ':'), ensure_ascii=False) + ';\n'
open(os.path.join(WEB, 'data.js'), 'w').write(js)
print('wrote data.js', len(js) // 1024, 'KB; final cut-offs', final_cutoffs, '; lights', {c: (v['caught_yellow'], v['caught_red']) for c, v in lights.items()})
print('watch on:', [s for s in TEST if watch[s]['on']], ' droughts:', [s for s in TEST if DROUGHT[s]])
