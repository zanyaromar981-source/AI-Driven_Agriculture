"""Pre-season test: does the El Niño/La Niña state known in September predict a bad season here?
Usage: python3 -I enso_test.py ../backtest_data/enso/oni.ascii.txt ../season_rain_summary.json out_late/features.csv ../backtest_data/openmeteo zones8.json"""
import sys, json, csv, math, statistics as st
ONI={}
for line in open(sys.argv[1]).read().splitlines()[1:]:
    seas,yr,tot,anom=line.split(); ONI[(seas,int(yr))]=float(anom)
def oni_sep(s): return ONI.get(('JAS',s))          # Jul-Aug-Sep value, published early October
def phase(v): return 'La Niña' if v<=-0.5 else ('El Niño' if v>=0.5 else 'neutral')
# ---- A. region level, rain outcome (36 seasons, 8 original zones) ----
rain=json.load(open(sys.argv[2])); R={int(r['season'][:4]):r['full_pct_normal'] for r in rain}
seasons=sorted(s for s in R if 1991<=s<=2025 and oni_sep(s) is not None)
vals=sorted(R[s] for s in seasons); q25=vals[len(vals)//4]
bad={s:R[s]<=q25 for s in seasons}
print(f'A. Rain outcome, {len(seasons)} seasons 1991/92–2025/26, bad = bottom 25% (≤{q25}% of normal) → {sum(bad.values())} bad seasons')
for ph in ['La Niña','neutral','El Niño']:
    ss=[s for s in seasons if phase(oni_sep(s))==ph]
    print(f'   {ph:8s}: {len(ss):2d} seasons, {sum(bad[s] for s in ss)} bad → P(bad)={sum(bad[s] for s in ss)/len(ss):.0%}   seasons: '+' '.join(f"{s}/{str(s+1)[2:]}{'*' if bad[s] else ''}" for s in ss))
tp=sum(1 for s in seasons if bad[s] and phase(oni_sep(s))=='La Niña'); fn=sum(1 for s in seasons if bad[s])-tp
fp=sum(1 for s in seasons if not bad[s] and phase(oni_sep(s))=='La Niña'); tn=sum(1 for s in seasons if not bad[s])-fp
print(f'   Rule "warn if La Niña in September": catches {tp}/{tp+fn} bad seasons, false alarms {fp}/{fp+tn}')
# correlation ONI vs season rain
xs=[oni_sep(s) for s in seasons]; ys=[R[s] for s in seasons]
print(f'   correlation ONI(Sep) vs season rain % of normal: {st.correlation(xs,ys):+.2f}')
print(f'   2026/27 outlook: ONI JAS 2026 = {oni_sep(2026):+.2f} → {phase(oni_sep(2026))}')
# ---- B. zone level, satellite outcome, logistic LOSO with pre-season features ----
rows=[r for r in csv.DictReader(open(sys.argv[3])) if r['cut']=='Dec' and r['label']!='']
OM=sys.argv[4]; zones=[z['name'] for z in json.load(open(sys.argv[5]))['zones']]
def soil_sep_series(z):
    d=json.load(open(f'{OM}/{z}.json'))['daily']; ix={t:i for i,t in enumerate(d['time'])}; out={}
    for s in range(1991,2027):
        b=ix.get(f'{s}-09-30')
        if b is None: continue
        v=[x for x in d['soil_moisture_0_to_100cm_mean'][b-6:b+1] if x is not None]
        if v: out[s]=st.mean(v)
    norm=st.mean(v for s,v in out.items() if 1991<=s<=2020); return {s:v/norm*100 for s,v in out.items()}
SOIL={z:soil_sep_series(z) for z in zones}
data=[]
for r in rows:
    s=int(r['season']); o=oni_sep(s)
    if o is None or s not in SOIL[r['zone']] or r['last_green']=='' : continue
    data.append(dict(zone=r['zone'],season=s,label=int(r['label']),x=[o,float(r['last_rain_pct']),float(r['last_green']),SOIL[r['zone']][s]]))
def fit(X,y,l2=0.05,iters=3000,lr=0.05):
    n,k=len(X),len(X[0]); mu=[st.mean(c) for c in zip(*X)]; sd=[st.pstdev(c) or 1 for c in zip(*X)]
    Z=[[(x[j]-mu[j])/sd[j] for j in range(k)] for x in X]; w=[0.0]*k; b=0.0
    for _ in range(iters):
        gw=[0.0]*k; gb=0.0
        for zi,yi in zip(Z,y):
            p=1/(1+math.exp(-(sum(w[j]*zi[j] for j in range(k))+b))); e=p-yi
            for j in range(k): gw[j]+=e*zi[j]
            gb+=e
        for j in range(k): w[j]-=lr*(gw[j]/n+l2*w[j])
        b-=lr*gb/n
    return dict(w=w,b=b,mu=mu,sd=sd)
def predict(m,x): return 1/(1+math.exp(-(sum(m['w'][j]*(x[j]-m['mu'][j])/m['sd'][j] for j in range(len(x)))+m['b'])))
S=sorted({d['season'] for d in data}); pairs=[]; seasP={}
for s in S:
    tr=[d for d in data if d['season']!=s]; te=[d for d in data if d['season']==s]
    m=fit([d['x'] for d in tr],[d['label'] for d in tr])
    for d in te: p=predict(m,d['x']); pairs.append((d['label'],p)); seasP.setdefault(s,[]).append((d['label'],p))
full=fit([d['x'] for d in data],[d['label'] for d in data])
print(f'\nB. Zone level, satellite outcome, {len(data)} zone-seasons, pre-season model (ONI Sep + last season rain + last spring greenness + Sept soil), leave-one-season-out')
print('   weights: ONI %+.2f, last rain %+.2f, last greenness %+.2f, Sept soil %+.2f' % tuple(full['w']))
for thr in (0.5,0.35,0.3):
    tp=sum(1 for l,p in pairs if l and p>=thr); fn=sum(1 for l,p in pairs if l and p<thr); fp=sum(1 for l,p in pairs if not l and p>=thr); tn=sum(1 for l,p in pairs if not l and p<thr)
    print(f'   threshold {thr}: catches {tp/(tp+fn):.0%} of bad zone-seasons, false alarms {fp/(fp+tn):.0%}, right overall {(tp+tn)/len(pairs):.0%}')
print('   mean pre-season risk per season (B = bad season by satellite): '+' '.join(f"{s}/{str(s+1)[2:]}{'B' if sum(l for l,_ in v)/len(v)>=0.5 else '-'}{st.mean(p for _,p in v):.2f}" for s,v in seasP.items()))
