"""Download everything we can from NASA ORNL (MOD13Q1 250 m, 16-day) in one sequential, resumable queue.
Usage: python3 -I fetch_all.py zones.json data_root
Queue order: (1) remaining zones Feb-May  (2) Dukan + Darbandikhan 40 km boxes, full year 2000-2026
             (3) all zones Jan, then Oct-Dec, then Jun-Sep (full-year curves)  (4) 2026 so far for all zones."""
import json, sys, urllib.request, time, os
zones=json.load(open(sys.argv[1]))['zones']; ROOT=sys.argv[2]
UA={'User-Agent':'Mozilla/5.0 (Macintosh) Jutyar-backtest','Accept':'application/json'}
def url(lat,lon,s,e,km): return (f"https://modis.ornl.gov/rst/api/v1/MOD13Q1/subset?latitude={lat}&longitude={lon}&startDate={s}&endDate={e}"
                                 f"&kmAboveBelow={km}&kmLeftRight={km}&band=250m_16_days_NDVI")
jobs=[]
# (1) remaining zones Feb-May
for z in zones:
    for y in range(2000,2026):
        jobs.append((f"{ROOT}/modis/{z['name']}_{y}.json", url(z['lat'],z['lon'],'A2000049' if y==2000 else f'A{y}033',f'A{y}145',4)))
# (2) reservoirs, full year
RES=[('Dukan',35.95,44.95),('Darbandikhan',35.13,45.70)]
for name,lat,lon in RES:
    for y in range(2000,2027):
        parts=[('A2000049' if y==2000 else f'A{y}001',f'A{y}145','a'),(f'A{y}161',f'A{y}305','b'),(f'A{y}321',f'A{y}353','c')]
        for s,e,tag in parts:
            if y==2026 and tag=='c': continue
            jobs.append((f"{ROOT}/reservoirs/{name}_{y}{tag}.json", url(lat,lon,s,e,20)))
# (4) 2026 so far
for z in zones:
    jobs.append((f"{ROOT}/modis_full/{z['name']}_2026_a.json", url(z['lat'],z['lon'],'A2026001','A2026145',4)))
    jobs.append((f"{ROOT}/modis_full/{z['name']}_2026_b.json", url(z['lat'],z['lon'],'A2026161','A2026289',4)))
# (3) full-year curves for all zones: Jan, then Oct-Dec, then Jun-Sep
for s,e,tag in [('A{y}001','A{y}017','jan'),('A{y}273','A{y}353','autumn'),('A{y}161','A{y}257','summer')]:
    for z in zones:
        for y in range(2001 if tag=='jan' else 2000,2026):
            jobs.append((f"{ROOT}/modis_full/{z['name']}_{y}_{tag}.json", url(z['lat'],z['lon'],s.format(y=y),e.format(y=y),4)))
todo=[j for j in jobs if not (os.path.exists(j[0]) and os.path.getsize(j[0])>1000)]
print(f'queue: {len(jobs)} jobs, {len(todo)} still to download', flush=True)
done=0; fails=0; t0=time.time()
for fn,u in todo:
    ok=False
    for attempt in range(4):
        try:
            raw=urllib.request.urlopen(urllib.request.Request(u,headers=UA),timeout=180).read(); json.loads(raw)
            open(fn,'wb').write(raw); ok=True; break
        except Exception as e:
            err=str(e)[:80]; time.sleep(10*(attempt+1))
    done+=1
    if not ok: fails+=1; print(f'FAIL {os.path.basename(fn)} {err}', flush=True)
    if done%20==0 or done==len(todo):
        el=time.time()-t0; print(f'{done}/{len(todo)} done, {fails} failed, {el/60:.0f} min elapsed, ~{(len(todo)-done)*el/done/60:.0f} min left', flush=True)
    time.sleep(0.5)
print('ALL DONE', flush=True)
