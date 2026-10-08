import json, sys, urllib.request, time
zones=json.load(open(sys.argv[1]))['zones']; out=sys.argv[2]
V='precipitation_sum,temperature_2m_mean,et0_fao_evapotranspiration,soil_moisture_0_to_100cm_mean,soil_moisture_28_to_100cm_mean'
for b in range(0,len(zones),8):
    batch=zones[b:b+8]
    url=('https://archive-api.open-meteo.com/v1/archive?latitude='+','.join(str(z['lat']) for z in batch)+'&longitude='+','.join(str(z['lon']) for z in batch)
         +'&start_date=1990-09-01&end_date=2026-10-05&daily='+V+'&timezone=Asia/Baghdad')
    for attempt in range(4):
        try:
            data=json.load(urllib.request.urlopen(url,timeout=180)); break
        except Exception as e:
            print('retry',attempt,e); time.sleep(5*(attempt+1))
    else: sys.exit('failed batch '+str(b))
    if isinstance(data,dict): data=[data]
    for z,d in zip(batch,data):
        json.dump(d,open(f"{out}/{z['name']}.json",'w'))
        print(z['name'],len(d['daily']['time']),'days')
