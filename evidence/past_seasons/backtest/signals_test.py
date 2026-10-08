"""Which large-scale climate signals, known before the season, predict Kurdistan winter rain?
Same bar as the El Niño test: 35 real seasons 1991/92–2025/26, region rain % of normal (ERA5, 8 zones).
Usage: python3 -I signals_test.py ../backtest_data/climate_indices ../backtest_data/enso/oni.ascii.txt ../season_rain_summary.json"""
import sys, json, math, statistics as st, os
IDX=sys.argv[1]
def psl(fn, miss=90):
    """NOAA PSL monthly table: 'year v1..v12'. Missing if |v|>=miss."""
    out={}
    for line in open(os.path.join(IDX,fn)):
        t=line.split()
        if len(t)==13 and t[0].isdigit() and len(t[0])==4:
            for m,v in enumerate(t[1:],1):
                try: v=float(v)
                except: continue
                if abs(v)<miss: out[(int(t[0]),m)]=v
    return out
def cpc_tele(col):
    out={}
    for line in open(os.path.join(IDX,'cpc_teleconnections.txt')):
        t=line.replace('-99.90',' -99.90 ').split()
        if len(t)>=12 and t[0].isdigit() and len(t[0])==4 and t[1].isdigit():
            try: v=float(t[col])
            except: continue
            if v>-90: out[(int(t[0]),int(t[1]))]=v
    return out
def rutgers():
    out={}
    for line in open(os.path.join(IDX,'eurasia_snow_rutgers.txt')):
        t=line.split()
        if len(t)==3 and t[0].isdigit(): out[(int(t[0]),int(t[1]))]=float(t[2])/1e6  # million km²
    # anomaly per calendar month vs 1991-2020
    for m in range(1,13):
        base=[out[(y,m)] for y in range(1991,2021) if (y,m) in out]; mu=st.mean(base)
        for y in range(1966,2027):
            if (y,m) in out: out[(y,m)]-=mu
    return out
ONI={}
for line in open(sys.argv[2]).read().splitlines()[1:]:
    seas,yr,tot,anom=line.split(); ONI[(seas,int(yr))]=float(anom)
# ---- predictors: value known by end of September (mean Jul–Sep), or Oct where noted ----
SIG={
 'ENSO (ONI, Jul–Sep)':            {('oni',s):ONI[('JAS',s)] for s in range(1950,2027) if ('JAS',s) in ONI},
 'Indian Ocean Dipole (DMI)':      psl('iod_dmi.txt'),
 'Pacific Decadal Osc. (PDO)':     psl('pdo_ersst5.txt'),
 'Atlantic Multidecadal (AMO)':    psl('amo_unsmoothed.txt'),
 'Tropical N. Atlantic SST (TNA)': psl('tna.txt'),
 'Stratosphere wind (QBO 30hPa)':  psl('qbo.txt'),
 'Southern Osc. Index (SOI)':      psl('soi.txt'),
 'NAO (Sep, atmosphere)':          psl('nao.txt'),
 'Arctic Osc. (Sep, atmosphere)':  psl('ao.txt'),
 'EA/West Russia (Sep, atmos.)':   cpc_tele(7),
 'Scandinavia pattern (Sep)':      cpc_tele(8),
 'Eurasia snow (Oct, anomaly)':    rutgers(),
}
def pre(name, s):
    d=SIG[name]
    if name.startswith('ENSO'): return d.get(('oni',s))
    if 'Oct' in name: return d.get((s,10))
    if '(Sep' in name: return d.get((s,9))
    v=[d.get((s,m)) for m in (7,8,9)]; v=[x for x in v if x is not None]
    return st.mean(v) if len(v)==3 else None
rain=json.load(open(sys.argv[3])); R={int(r['season'][:4]):r['full_pct_normal'] for r in rain}; E={int(r['season'][:4]):r['early_pct_normal'] for r in rain}
seasons=[s for s in range(1991,2026) if s in R]
q25=sorted(R[s] for s in seasons)[len(seasons)//4]; bad={s:R[s]<=q25 for s in seasons}
print(f'{len(seasons)} seasons, bad = bottom 25% (≤{q25}% of normal) → {sum(bad.values())} bad seasons.  r needed: 0.33 (p<.05), 0.43 (p<.01), 0.46 (after correcting for testing 12 signals)\n')
print(f'{"signal (state known in September)":34s} {"n":>2s} {"r full":>7s} {"r Oct–Dec":>9s}  P(bad) low-third / high-third   2026 state')
res={}
for name in SIG:
    pts=[(pre(name,s),R[s],E[s],s) for s in seasons if pre(name,s) is not None]
    if len(pts)<20: print(f'{name:34s} {len(pts):2d}  not enough data'); continue
    xs=[p[0] for p in pts]; r_full=st.correlation(xs,[p[1] for p in pts]); r_early=st.correlation(xs,[p[2] for p in pts])
    srt=sorted(pts); k=len(pts)//3; lo=srt[:k]; hi=srt[-k:]
    plo=sum(bad[p[3]] for p in lo)/k; phi=sum(bad[p[3]] for p in hi)/k
    now=pre(name,2026); res[name]=dict(r=r_full,n=len(pts))
    flag='**' if abs(r_full)>=0.46 else ('* ' if abs(r_full)>=0.33 else '  ')
    print(f'{name:34s} {len(pts):2d} {r_full:+7.2f}{flag}{r_early:+8.2f}     {plo:4.0%} / {phi:4.0%}                 {now if now is None else f"{now:+.2f}"}')
# ---- concurrent winter atmosphere (Dec–Feb mean) — explains, does not predict ----
print('\nSame-winter atmosphere (Dec–Feb mean) vs season rain — tells us WHY, cannot be known in September:')
for name,d in [('NAO',psl('nao.txt')),('Arctic Osc.',psl('ao.txt')),('EA/West Russia',cpc_tele(7)),('Scandinavia',cpc_tele(8)),('East Atlantic',cpc_tele(3))]:
    pts=[]
    for s in seasons:
        v=[d.get((s,12)),d.get((s+1,1)),d.get((s+1,2))]
        if all(x is not None for x in v): pts.append((st.mean(v),R[s]))
    print(f'   {name:16s} n={len(pts):2d}  r={st.correlation([p[0] for p in pts],[p[1] for p in pts]):+.2f}')
# ---- does adding a second signal to ENSO help? leave-one-season-out linear prediction of rain % ----
def loo(names):
    pts=[(s,[pre(n,s) for n in names],R[s]) for s in seasons]; pts=[p for p in pts if all(x is not None for x in p[1])]
    preds=[]
    for i,(s,x,y) in enumerate(pts):
        tr=[p for j,p in enumerate(pts) if j!=i]
        X=[[1.0]+p[1] for p in tr]; Y=[p[2] for p in tr]; k=len(X[0])
        # normal equations with tiny ridge
        A=[[sum(X[r][a]*X[r][b] for r in range(len(X)))+(0.01 if a==b and a>0 else 0) for b in range(k)] for a in range(k)]
        B=[sum(X[r][a]*Y[r] for r in range(len(X))) for a in range(k)]
        for c in range(k):  # gauss
            piv=max(range(c,k),key=lambda r:abs(A[r][c])); A[c],A[piv]=A[piv],A[c]; B[c],B[piv]=B[piv],B[c]
            for r in range(k):
                if r!=c and A[c][c]:
                    f=A[r][c]/A[c][c]; A[r]=[A[r][j]-f*A[c][j] for j in range(k)]; B[r]-=f*B[c]
        w=[B[j]/A[j][j] for j in range(k)]
        preds.append((s,w[0]+sum(w[j+1]*x[j] for j in range(len(x))),y))
    r=st.correlation([p[1] for p in preds],[p[2] for p in preds]); mae=st.mean(abs(p[1]-p[2]) for p in preds)
    # call: "below normal" if predicted <95
    tp=sum(1 for s,p,y in preds if bad[s] and p<95); fn=sum(1 for s,p,y in preds if bad[s] and p>=95); fp=sum(1 for s,p,y in preds if not bad[s] and p<95); tn=sum(1 for s,p,y in preds if not bad[s] and p>=95)
    return len(preds),r,mae,tp,fn,fp,tn
print('\nPre-season rain forecast (leave-one-season-out), warn if predicted <95% of normal:')
print(f'   {"signals used":52s} n  r(pred,actual)  avg error   bad seasons caught   false alarms')
base='ENSO (ONI, Jul–Sep)'
for names in [[base]]+[[base,n] for n in SIG if n!=base and n in res]:
    n,r,mae,tp,fn,fp,tn=loo(names)
    print(f'   {" + ".join(x.split(" (")[0] for x in names):52s} {n:2d}   {r:+.2f}        {mae:4.0f} pts     {tp}/{tp+fn}                {fp}/{fp+tn}')
