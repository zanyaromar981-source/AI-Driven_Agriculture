"""Extra ERA5 daily variables per zone (one zone per call, skips done zones, waits on rate limit).
Usage: python3 -I fetch_openmeteo_extra.py zones.json out_dir"""
import json, sys, os, urllib.request, time
zones=json.load(open(sys.argv[1]))['zones']; out=sys.argv[2]
V='temperature_2m_max,temperature_2m_min,snowfall_sum,rain_sum,precipitation_hours,soil_moisture_0_to_7cm_mean,soil_moisture_7_to_28cm_mean,vapour_pressure_deficit_max,shortwave_radiation_sum'
for z in zones:
    fn=f"{out}/{z['name']}.json"
    if os.path.exists(fn) and os.path.getsize(fn)>1000: continue
    url=(f"https://archive-api.open-meteo.com/v1/archive?latitude={z['lat']}&longitude={z['lon']}&start_date=1990-09-01&end_date=2026-10-05&daily={V}&timezone=Asia/Baghdad")
    for attempt in range(30):
        try: d=json.load(urllib.request.urlopen(urllib.request.Request(url,headers={'User-Agent':'Mozilla/5.0'}),timeout=240)); break
        except Exception as e: print(time.strftime('%H:%M'),'wait',z['name'],e,flush=True); time.sleep(300)
    else: sys.exit('gave up '+z['name'])
    json.dump(d,open(fn,'w')); print(time.strftime('%H:%M'),z['name'],len(d['daily']['time']),'days',flush=True); time.sleep(5)
print('EXTRA DONE')
