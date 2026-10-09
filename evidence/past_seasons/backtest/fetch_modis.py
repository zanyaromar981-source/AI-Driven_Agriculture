import json, sys, urllib.request, time, os
from concurrent.futures import ThreadPoolExecutor
zones=json.load(open(sys.argv[1]))['zones']; out=sys.argv[2]
def get(z,y):
    fn=f"{out}/{z['name']}_{y}.json"
    if os.path.exists(fn) and os.path.getsize(fn)>1000: return 'skip'
    s='A2000049' if y==2000 else f'A{y}033'
    url=(f"https://modis.ornl.gov/rst/api/v1/MOD13Q1/subset?latitude={z['lat']}&longitude={z['lon']}&startDate={s}&endDate=A{y}145&kmAboveBelow=4&kmLeftRight=4&band=250m_16_days_NDVI")
    for attempt in range(4):
        try:
            req=urllib.request.Request(url,headers={'User-Agent':'Mozilla/5.0 (Macintosh) Jutyar-backtest','Accept':'application/json'})
            raw=urllib.request.urlopen(req,timeout=120).read(); json.loads(raw)
            open(fn,'wb').write(raw); return 'ok'
        except Exception as e:
            err=str(e); time.sleep(8*(attempt+1))
    return 'FAIL '+err
jobs=[(z,y) for z in zones for y in range(2000,2026)]
done=0
with ThreadPoolExecutor(max_workers=int(sys.argv[3]) if len(sys.argv)>3 else 1) as ex:
    for (z,y),r in zip(jobs, ex.map(lambda j:get(*j), jobs)):
        done+=1
        if r.startswith('FAIL') or done%40==0: print(done,'/',len(jobs),z['name'],y,r,flush=True)
print('DONE')
