import json, statistics as st
names=["Slemani city","Garmiyan/Kalar","Chamchamal","Bazian","Ranya","Penjwen","Sharazur","Qaradagh"]
data=json.load(open("rain_daily_1990_2026.json"))
order=[10,11,12,1,2,3,4,5]
cum={}
for n,d in zip(names,data):
    for t,p in zip(d["daily"]["time"],d["daily"]["precipitation_sum"]):
        y,m=int(t[:4]),int(t[5:7])
        if m not in order or p is None: continue
        s=y+1 if m>=10 else y
        for k,mm in enumerate(order):
            if order.index(m)<=k:
                cum[(n,s,mm)]=cum.get((n,s,mm),0)+p
seasons=range(1991,2027)
def avg(s,mm): return st.mean(cum.get((n,s,mm),0) for n in names)
full={s:avg(s,5) for s in seasons}
dry=set(sorted(seasons,key=lambda s:full[s])[:9])   # bottom quarter = "bad season"
print("Bad seasons (bottom 25%):",sorted(f"{s-1}/{str(s)[2:]}" for s in dry))
for upto,label in [(12,"end Dec"),(1,"end Jan"),(2,"end Feb"),(3,"end Mar")]:
    vals={s:avg(s,upto) for s in seasons}
    warn=set(sorted(seasons,key=lambda s:vals[s])[:9])
    hits=len(warn&dry); false=len(warn-dry)
    base=st.mean(vals[s] for s in seasons if s<=2020)
    print(f"{label}: warns on {hits}/9 bad seasons, {false} false alarms | 2024/25 at {100*vals[2025]/base:.0f}% of normal -> {'WARNED' if 2025 in warn else 'missed'}")
