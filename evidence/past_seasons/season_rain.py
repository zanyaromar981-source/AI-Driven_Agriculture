import json, statistics as st
names=["Slemani city","Garmiyan/Kalar","Chamchamal","Bazian","Ranya","Penjwen","Sharazur","Qaradagh"]
data=json.load(open("rain_daily_1990_2026.json"))
rows={}
for n,d in zip(names,data):
    for t,p in zip(d["daily"]["time"],d["daily"]["precipitation_sum"]):
        y,m=int(t[:4]),int(t[5:7])
        if m>=9: season=y+1
        elif m<=6: season=y
        else: continue
        r=rows.setdefault((n,season),{"early":0,"full":0})
        if p is None: continue
        if m in (10,11,12): r["early"]+=p
        if m in (10,11,12,1,2,3,4,5): r["full"]+=p
seasons=range(1991,2027)
out=[]
print("season | early(Oct-Dec) avg-of-zones | full(Oct-May) avg | full % of normal | early % of normal")
E={s:st.mean(rows[(n,s)]["early"] for n in names) for s in seasons}
F={s:st.mean(rows[(n,s)]["full"] for n in names) for s in seasons}
base=[s for s in seasons if 1991<=s<=2020]
mE=st.mean(E[s] for s in base); mF=st.mean(F[s] for s in base)
for s in seasons:
    print(f"{s-1}/{str(s)[2:]} | {E[s]:6.0f} | {F[s]:6.0f} | {100*F[s]/mF:5.0f}% | {100*E[s]/mE:5.0f}%")
    out.append({"season":f"{s-1}/{s}","oct_dec_mm":round(E[s]),"oct_may_mm":round(F[s]),"full_pct_normal":round(100*F[s]/mF),"early_pct_normal":round(100*E[s]/mE)})
print(f"normal 1991-2020: early {mE:.0f} mm, full {mF:.0f} mm")
# how well does early predict full? correlation + driest-quarter hits
xs=[E[s] for s in seasons if s<=2026]; ys=[F[s] for s in seasons]
print("correlation early vs full:", round(st.correlation(xs,ys),2))
dryF=sorted(seasons,key=lambda s:F[s])[:9]; dryE=sorted(seasons,key=lambda s:E[s])[:9]
print("9 driest full seasons:", [f"{s-1}/{str(s)[2:]}" for s in sorted(dryF)])
print("9 driest Oct-Dec starts:", [f"{s-1}/{str(s)[2:]}" for s in sorted(dryE)])
print("overlap:", len(set(dryF)&set(dryE)),"of 9")
json.dump(out,open("season_rain_summary.json","w"),indent=1)
