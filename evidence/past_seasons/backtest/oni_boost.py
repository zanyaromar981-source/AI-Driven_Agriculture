"""Does adding the September ENSO state improve the Dec/Jan/Feb/Mar warnings? LOSO, same model."""
import sys, json, csv, math, statistics as st
ONI={}
for line in open(sys.argv[1]).read().splitlines()[1:]:
    seas,yr,tot,anom=line.split(); ONI[(seas,int(yr))]=float(anom)
rows=[r for r in csv.DictReader(open(sys.argv[2])) if r['label']!='']
rain=json.load(open(sys.argv[3])); R={int(r['season'][:4]):r['full_pct_normal'] for r in rain}
for ph,f in [('El Niño',lambda v:v>=0.5),('neutral',lambda v:-0.5<v<0.5),('La Niña',lambda v:v<=-0.5)]:
    ss=[s for s in range(1991,2026) if ('JAS',s) in ONI and f(ONI[('JAS',s)]) and s in R]
    print(f'{ph:8s}: mean season rain {st.mean(R[s] for s in ss):.0f}% of normal, min {min(R[s] for s in ss)}%, max {max(R[s] for s in ss)}%  (n={len(ss)})')
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
FE={'Dec':['rain_pct','soil_pct','temp_anom','et0_pct','last_rain_pct'],'Jan':['rain_pct','soil_pct','temp_anom','et0_pct','last_rain_pct'],
    'Feb':['rain_pct','soil_pct','temp_anom','et0_pct','last_rain_pct','green_now'],'Mar':['rain_pct','soil_pct','temp_anom','et0_pct','last_rain_pct','green_now']}
print('\nLOSO zone-level, threshold 0.35 (sensitive) and 0.5: without ONI → with ONI')
for cut,feats in FE.items():
    data=[r for r in rows if r['cut']==cut and all(r[f]!='' for f in feats) and ('JAS',int(r['season'])) in ONI]
    S=sorted({int(r['season']) for r in data})
    res={}
    for use_oni in (False,True):
        pairs=[]
        for s in S:
            tr=[r for r in data if int(r['season'])!=s]; te=[r for r in data if int(r['season'])==s]
            X=lambda r:[float(r[f]) for f in feats]+([ONI[('JAS',int(r['season']))]] if use_oni else [])
            m=fit([X(r) for r in tr],[int(r['label']) for r in tr])
            for r in te: pairs.append((int(r['label']),predict(m,X(r))))
        out=[]
        for thr in (0.35,0.5):
            tp=sum(1 for l,p in pairs if l and p>=thr); fn=sum(1 for l,p in pairs if l and p<thr); fp=sum(1 for l,p in pairs if not l and p>=thr); tn=sum(1 for l,p in pairs if not l and p<thr)
            out.append(f'@{thr}: {tp/(tp+fn):.0%} caught / {fp/(fp+tn):.0%} FA')
        res[use_oni]=' | '.join(out)
    print(f'  {cut}: {res[False]}   →   {res[True]}')
