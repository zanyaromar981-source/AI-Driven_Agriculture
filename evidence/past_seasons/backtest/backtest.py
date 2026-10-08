"""Backtest: would the AI have warned about bad seasons before they happened?
Usage: python3 -I backtest.py zones.json ../backtest_data/openmeteo ../backtest_data/modis out_dir
Pure Python (no numpy). Logistic regression, leave-one-season-out (all zones of a season held out together)."""
import json, sys, os, math, statistics as st
from datetime import date, timedelta
zones=[z['name'] for z in json.load(open(sys.argv[1]))['zones']]
OM,MD,OUT=sys.argv[2],sys.argv[3],sys.argv[4]; os.makedirs(OUT,exist_ok=True)
LABEL_MODE=sys.argv[5] if len(sys.argv)>5 else 'ndvi'
LABEL_STAT=sys.argv[6] if len(sys.argv)>6 else 'late'   # 'peak' = max NDVI Mar 6-May 9 ; 'late' = mean NDVI Mar 22-May 9 (grain filling)   # 'ndvi' = MODIS spring peak (real crop outcome); 'rain' = Oct-May rain (fallback / pipeline check)
CUTS={'Dec':(12,31),'Jan':(1,31),'Feb':(2,28),'Mar':(3,31)}
NORMAL_SEASONS=range(1991,2021)      # 1991/92 .. 2020/21
SEASONS=range(1999,2025)             # labelled seasons 1999/00 .. 2024/25 (MODIS spring 2000..2025)

# ---------- weather ----------
def load_om(z):
    d=json.load(open(f'{OM}/{z}.json'))['daily']
    idx={t:i for i,t in enumerate(d['time'])}
    return d,idx
def win(d,idx,s,m2,d2,key):
    """sum of key from Oct 1 of season s to (m2,d2) of s or s+1"""
    y2=s if m2>=10 else s+1
    a=idx.get(f'{s}-10-01'); b=idx.get(f'{y2}-{m2:02d}-{d2:02d}')
    if a is None or b is None: return None
    vals=[v for v in d[key][a:b+1] if v is not None]
    return sum(vals) if vals else None
def mean_win(d,idx,s,m2,d2,key):
    y2=s if m2>=10 else s+1
    a=idx.get(f'{s}-10-01'); b=idx.get(f'{y2}-{m2:02d}-{d2:02d}')
    if a is None or b is None: return None
    vals=[v for v in d[key][a:b+1] if v is not None]
    return st.mean(vals) if vals else None
def at(d,idx,s,m2,d2,key,days=7):
    y2=s if m2>=10 else s+1
    b=idx.get(f'{y2}-{m2:02d}-{d2:02d}')
    if b is None: return None
    vals=[v for v in d[key][max(0,b-days+1):b+1] if v is not None]
    return st.mean(vals) if vals else None

# ---------- MODIS ----------
def load_modis(z):
    """returns {year: {modis_doy: [pixel values or None]}} with NDVI scaled"""
    out={}
    for y in range(2000,2026):
        fn=f'{MD}/{z}_{y}.json'
        if not os.path.exists(fn): continue
        try: j=json.load(open(fn))
        except Exception: continue
        for c in j.get('subset',[]):
            doy=int(c['modis_date'][5:])
            out.setdefault(y,{})[doy]=[v*0.0001 if (v is not None and v>-2000) else None for v in c['data']]
    return out
def zone_series(m):
    """cropland-ish pixel filter, then mean NDVI per (year,doy)"""
    years=sorted(m); npx=max(len(v) for y in years for v in m[y].values())
    peak_by_px={}
    for p in range(npx):
        peaks=[]
        for y in years:
            vals=[m[y][d][p] for d in m[y] if 65<=d<=129 and p<len(m[y][d]) and m[y][d][p] is not None]
            if vals: peaks.append(max(vals))
        if peaks: peak_by_px[p]=st.median(peaks)
    def keep(lo,hi): return [p for p,v in peak_by_px.items() if lo<=v<=hi]
    px=keep(0.30,0.75)
    if len(px)<50: px=keep(0.25,0.80)
    if len(px)<50: px=list(peak_by_px)
    ser={}
    for y in years:
        for d,vals in m[y].items():
            vv=[vals[p] for p in px if p<len(vals) and vals[p] is not None]
            if len(vv)>=max(10,len(px)//4): ser[(y,d)]=st.mean(vv)
    return ser,len(px),npx

rows=[]; info={}
for z in zones:
    d,idx=load_om(z); m=load_modis(z) if LABEL_MODE=='ndvi' else {}
    if LABEL_MODE=='ndvi' and not m: print('no MODIS for',z); continue
    if LABEL_MODE=='ndvi':
        ser,npx_keep,npx=zone_series(m); info[z]={'cropland_px':npx_keep,'box_px':npx}
        if LABEL_STAT=='peak':
            peak={y:max([v for (yy,dd),v in ser.items() if yy==y and 65<=dd<=129] or [None]) for y in range(2000,2026)}
        else:
            peak={}
            for y in range(2000,2026):
                vv=[v for (yy,dd),v in ser.items() if yy==y and 81<=dd<=129]
                peak[y]=st.mean(vv) if len(vv)>=2 else None
    else:
        ser={}; info[z]={'label':'Oct-May rain'}
        peak={y:win(d,idx,y-1,5,31,'precipitation_sum') for y in range(2000,2026)}
    peaks=[v for v in peak.values() if v is not None]
    q25=sorted(peaks)[len(peaks)//4]; med=st.median(peaks)
    info[z]['peak_q25']=round(q25,3); info[z]['peak_median']=round(med,3)
    # normals
    for cut,(m2,d2) in CUTS.items():
        rn=st.mean([v for v in (win(d,idx,s,m2,d2,'precipitation_sum') for s in NORMAL_SEASONS) if v is not None])
        sn=st.mean([v for v in (at(d,idx,s,m2,d2,'soil_moisture_0_to_100cm_mean') for s in NORMAL_SEASONS) if v is not None])
        tn=st.mean([v for v in (mean_win(d,idx,s,m2,d2,'temperature_2m_mean') for s in NORMAL_SEASONS) if v is not None])
        en=st.mean([v for v in (win(d,idx,s,m2,d2,'et0_fao_evapotranspiration') for s in NORMAL_SEASONS) if v is not None])
        fulln=st.mean([v for v in (win(d,idx,s,5,31,'precipitation_sum') for s in NORMAL_SEASONS) if v is not None])
        for s in list(SEASONS)+[2025]:
            r=win(d,idx,s,m2,d2,'precipitation_sum'); so=at(d,idx,s,m2,d2,'soil_moisture_0_to_100cm_mean')
            t=mean_win(d,idx,s,m2,d2,'temperature_2m_mean'); e=win(d,idx,s,m2,d2,'et0_fao_evapotranspiration')
            lastr=win(d,idx,s-1,5,31,'precipitation_sum')
            if None in (r,so,t,e,lastr): continue
            # greenness known by the cutoff (current season), relative to zone median peak
            doys={'Dec':[],'Jan':[],'Feb':[33,49],'Mar':[33,49,65,81]}[cut]
            g=[ser.get((s+1,dd)) for dd in doys]; g=[v for v in g if v is not None]
            green_now=(max(g)/med*100) if g else None
            lastpeak=peak.get(s) ; lastgreen=(lastpeak/med*100) if lastpeak else None
            label=None
            if s<=2024 and peak.get(s+1) is not None: label=1 if peak[s+1]<=q25 else 0
            rows.append(dict(zone=z,season=s,cut=cut,rain_pct=r/rn*100,soil_pct=so/sn*100,temp_anom=t-tn,et0_pct=e/en*100,
                             last_rain_pct=lastr/fulln*100,last_green=lastgreen,green_now=green_now,label=label,
                             peak=peak.get(s+1)))
json.dump(info,open(f'{OUT}/zone_info.json','w'),indent=1)
with open(f'{OUT}/features.csv','w') as f:
    keys=list(rows[0].keys()); f.write(','.join(keys)+'\n')
    for r in rows: f.write(','.join('' if r[k] is None else (f'{r[k]:.3f}' if isinstance(r[k],float) else str(r[k])) for k in keys)+'\n')
print('rows',len(rows),'zones with data',len(info))

# ---------- logistic regression (pure python) ----------
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
def predict(model,x):
    z=sum(model['w'][j]*(x[j]-model['mu'][j])/model['sd'][j] for j in range(len(x)))+model['b']
    return 1/(1+math.exp(-z))

FEATS={'Dec':['rain_pct','soil_pct','temp_anom','et0_pct','last_rain_pct'],
       'Jan':['rain_pct','soil_pct','temp_anom','et0_pct','last_rain_pct'],
       'Feb':['rain_pct','soil_pct','temp_anom','et0_pct','last_rain_pct','green_now'],
       'Mar':['rain_pct','soil_pct','temp_anom','et0_pct','last_rain_pct','green_now']}
def metrics(pairs):  # (label,pred)
    tp=sum(1 for l,p in pairs if l==1 and p==1); fn=sum(1 for l,p in pairs if l==1 and p==0)
    fp=sum(1 for l,p in pairs if l==0 and p==1); tn=sum(1 for l,p in pairs if l==0 and p==0)
    return dict(tp=tp,fn=fn,fp=fp,tn=tn,recall=tp/(tp+fn) if tp+fn else None,far=fp/(fp+tn) if fp+tn else None,
                precision=tp/(tp+fp) if tp+fp else None,acc=(tp+tn)/len(pairs))
results={}
THR=0.5
for cut in CUTS:
    feats=[f for f in FEATS[cut] if not (LABEL_MODE=='rain' and f=='green_now')]
    data=[r for r in rows if r['cut']==cut and r['label'] is not None and all(r[f] is not None for f in feats)]
    seasons=sorted({r['season'] for r in data})
    zone_pairs=[]; zone_pairs_lo=[]; season_rows={}; base_last=[]; base_rain=[]
    for s in seasons:
        tr=[r for r in data if r['season']!=s]; te=[r for r in data if r['season']==s]
        mdl=fit([[r[f] for f in feats] for r in tr],[r['label'] for r in tr])
        for r in te:
            p=predict(mdl,[r[f] for f in feats]); pred=1 if p>=THR else 0
            zone_pairs.append((r['label'],pred)); season_rows.setdefault(s,[]).append((r['label'],pred,p,r['zone']))
            zone_pairs_lo.append((r['label'],1 if p>=0.35 else 0))
            # baselines: last-year-bad (needs last season label) ; rain-only rule (<75% of normal)
            lastlab=next((q['label'] for q in data if q['zone']==r['zone'] and q['season']==s-1),None)
            if lastlab is not None: base_last.append((r['label'],lastlab))
            base_rain.append((r['label'],1 if r['rain_pct']<75 else 0))
    # season level: bad season = >=50% of zones bad; warned = >=50% zones predicted bad
    seas=[]
    for s,lst in season_rows.items():
        frac_bad=sum(l for l,_,_,_ in lst)/len(lst); frac_warn=sum(p for _,p,_,_ in lst)/len(lst)
        seas.append(dict(season=f'{s}/{str(s+1)[2:]}',zones=len(lst),frac_bad=round(frac_bad,2),frac_warn=round(frac_warn,2),
                         mean_p=round(st.mean(x for _,_,x,_ in lst),2),bad=frac_bad>=0.5,warned=frac_warn>=0.5,
                         warned_lo=sum(1 for _,_,x,_ in lst if x>=0.35)/len(lst)>=0.5,
                         bad_zones=[z for l,_,_,z in lst if l],warned_zones=[z for _,p,_,z in lst if p]))
    full=fit([[r[f] for f in feats] for r in data],[r['label'] for r in data])
    results[cut]=dict(n=len(data),seasons=len(seasons),zone=metrics(zone_pairs),zone_thr035=metrics(zone_pairs_lo),baseline_last_year=metrics(base_last),
                      baseline_rain_rule=metrics(base_rain),always_normal_acc=1-st.mean(l for l,_ in zone_pairs),
                      season_table=seas,
                      season_caught=sum(1 for x in seas if x['bad'] and x['warned']),season_bad=sum(1 for x in seas if x['bad']),
                      season_false=sum(1 for x in seas if x['warned'] and not x['bad']),season_normal=sum(1 for x in seas if not x['bad']),
                      season_caught_lo=sum(1 for x in seas if x['bad'] and x['warned_lo']),season_false_lo=sum(1 for x in seas if x['warned_lo'] and not x['bad']),
                      weights={f:round(w,2) for f,w in zip(feats,full['w'])})
    # 2025 test: train <=2023 seasons, predict 2024/25 ; and 2025/26 (unlabelled) for sanity
    tr=[r for r in data if r['season']<=2023]; mdl=fit([[r[f] for f in feats] for r in tr],[r['label'] for r in tr])
    t25=[r for r in rows if r['cut']==cut and r['season']==2024 and all(r[f] is not None for f in feats)]
    t26=[r for r in rows if r['cut']==cut and r['season']==2025 and all(r[f] is not None for f in feats)]
    results[cut]['test_2025']=[dict(zone=r['zone'],p=round(predict(mdl,[r[f] for f in feats]),2),label=r['label'],rain_pct=round(r['rain_pct'])) for r in t25]
    results[cut]['test_2026']=[dict(zone=r['zone'],p=round(predict(mdl,[r[f] for f in feats]),2),rain_pct=round(r['rain_pct'])) for r in t26]
json.dump(results,open(f'{OUT}/results.json','w'),indent=1)
for cut,R in results.items():
    zm=R['zone']; bl=R['baseline_last_year']; br=R['baseline_rain_rule']
    print(f"\n== {cut} == rows {R['n']} seasons {R['seasons']}")
    print(f" zone-level: recall {zm['recall']:.2f} false-alarm {zm['far']:.2f} precision {zm['precision']:.2f} acc {zm['acc']:.2f} | always-normal acc {R['always_normal_acc']:.2f} | last-year recall {bl['recall']:.2f} far {bl['far']:.2f} | rain<75 rule recall {br['recall']:.2f} far {br['far']:.2f}")
    zl=R['zone_thr035']
    print(f" zone-level @0.35: recall {zl['recall']:.2f} false-alarm {zl['far']:.2f} precision {zl['precision']:.2f}")
    print(f" season-level @0.5: caught {R['season_caught']}/{R['season_bad']} bad seasons, false alarms {R['season_false']}/{R['season_normal']} | @0.35: caught {R['season_caught_lo']}/{R['season_bad']}, false alarms {R['season_false_lo']}/{R['season_normal']}")
    print(' seasons:',' '.join(f"{x['season']}{'B' if x['bad'] else '-'}{x['mean_p']:.2f}" for x in R['season_table']))
    print(' weights',R['weights'])
    print(' 2025 test (trained <=2023/24):',' '.join(f"{t['zone']}:{t['p']:.2f}{'*' if t['label'] else ''}" for t in R['test_2025']))
    print(' 2026 check:',' '.join(f"{t['zone']}:{t['p']:.2f}" for t in R['test_2026']))
