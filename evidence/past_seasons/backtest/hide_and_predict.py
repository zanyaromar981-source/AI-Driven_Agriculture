"""Hide one whole season for every zone, train on the rest, then predict one zone month by month.
Usage: python3 -I hide_and_predict.py out_dir/features.csv Slemani 2020 openmeteo_dir modis_dir
Cutoffs: Sep (pre-season: only last season + summer soil), Dec, Jan, Feb, Mar."""
import csv, sys, json, math, statistics as st, os
F,ZONE,S,OM,MD=sys.argv[1],sys.argv[2],int(sys.argv[3]),sys.argv[4],sys.argv[5]
rows=[r for r in csv.DictReader(open(F))]
for r in rows:
    for k in r:
        if k not in ('zone','cut'): r[k]=float(r[k]) if r[k]!='' else None
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
FE={'Sep':['last_rain_pct','last_green','soil_pct'],'Dec':['rain_pct','soil_pct','temp_anom','et0_pct','last_rain_pct'],
    'Jan':['rain_pct','soil_pct','temp_anom','et0_pct','last_rain_pct'],'Feb':['rain_pct','soil_pct','temp_anom','et0_pct','last_rain_pct','green_now'],
    'Mar':['rain_pct','soil_pct','temp_anom','et0_pct','last_rain_pct','green_now']}
# September pre-season rows: build from the Dec rows but replace soil with end-of-September soil vs normal
om=json.load(open(f'{OM}/{ZONE}.json'))['daily']; idx={t:i for i,t in enumerate(om['time'])}
def soil_sep(z,s):
    d=json.load(open(f'{OM}/{z}.json'))['daily']; ix={t:i for i,t in enumerate(d['time'])}
    b=ix.get(f'{s}-09-30'); 
    if b is None: return None
    v=[x for x in d['soil_moisture_0_to_100cm_mean'][b-6:b+1] if x is not None]; return st.mean(v) if v else None
print(f'\nHidden season: {S}/{str(S+1)[2:]}  zone: {ZONE}\n')
out={}
for cut,feats in FE.items():
    base='Dec' if cut=='Sep' else cut
    data=[r for r in rows if r['cut']==base and r['label'] is not None]
    if cut=='Sep':
        # soil at end of Sept vs the zone's normal
        cache={}
        for r in data+[r for r in rows if r['cut']=='Dec' and r['zone']==ZONE and int(r['season'])==S]:
            z=r['zone']; s=int(r['season'])
            if z not in cache:
                vals={ss:soil_sep(z,ss) for ss in range(1991,2027)}; norm=st.mean(v for ss,v in vals.items() if 1991<=ss<=2020 and v is not None); cache[z]=(vals,norm)
            vals,norm=cache[z]; r['soil_sep_pct']=(vals[s]/norm*100) if vals.get(s) else None
        feats=['last_rain_pct','last_green','soil_sep_pct']
    tr=[r for r in data if int(r['season'])!=S and all(r[f] is not None for f in feats)]
    te=[r for r in rows if r['cut']==base and r['zone']==ZONE and int(r['season'])==S]
    if not te or any(te[0][f] is None for f in feats): print(f'{cut}: no data'); continue
    m=fit([[r[f] for f in feats] for r in tr],[r['label'] for r in tr]); p=predict(m,[te[0][f] for f in feats])
    seen=', '.join(f'{f}={te[0][f]:.0f}' for f in feats)
    out[cut]=p
    print(f'{cut:>3}: risk {p*100:4.0f}%   (trained on {len(tr)} zone-seasons, saw: {seen})')
r=te[0]; print(f'\nREAL outcome {S}/{str(S+1)[2:]} {ZONE}: label={"BAD" if r["label"]==1 else "normal"}  late-spring NDVI={r["peak"]:.3f}')
# real greenness curve of the hidden spring vs the zone median curve
import glob
m={}
for y in range(2000,2026):
    fn=f'{MD}/{ZONE}_{y}.json'
    if os.path.exists(fn):
        for c in json.load(open(fn))['subset']: m.setdefault(y,{})[int(c['modis_date'][5:])]=[v*0.0001 if v>-2000 else None for v in c['data']]
def mean_nd(y,d):
    v=[x for x in m[y].get(d,[]) if x is not None]; return st.mean(v) if v else None
print('\nGreenness of the hidden spring vs. the median of all years:')
for d in (33,49,65,81,97,113,129):
    cur=mean_nd(S+1,d); med=st.median([mean_nd(y,d) for y in m if mean_nd(y,d) is not None])
    print(f'  day {d:3d}: {cur:.2f} vs median {med:.2f}  ({(cur/med-1)*100:+.0f}%)')
json.dump(out,open(f'hide_{ZONE}_{S}.json','w'))
