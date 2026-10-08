"""Daily ERA5 rain/snow/temperature at points in the catchments upstream of Dukan (Little Zab) and Darbandikhan (Sirwan), 1990-09-01 → 2026-10-05.
Usage: python3 -I fetch_catchment_rain.py out_dir"""
import json, sys, os, time, urllib.request
OUT = sys.argv[1]
PTS = {'Dukan': [('Ranya', 36.25, 44.88), ('Qaladiza', 36.18, 45.11), ('Sardasht_IR', 36.15, 45.48), ('Piranshahr_IR', 36.70, 45.14), ('Sidakan', 36.80, 44.67)],
       'Darbandikhan': [('Penjwen', 35.62, 45.94), ('Marivan_IR', 35.52, 46.18), ('Sanandaj_IR', 35.31, 47.00), ('Paveh_IR', 35.04, 46.36), ('Kamyaran_IR', 34.80, 46.94)]}
V = 'precipitation_sum,snowfall_sum,temperature_2m_mean'
for lake, pts in PTS.items():
    fn = f'{OUT}/{lake}_catchment.json'
    if os.path.exists(fn) and os.path.getsize(fn) > 1000: print('have', fn); continue
    url = ('https://archive-api.open-meteo.com/v1/archive?latitude=' + ','.join(str(p[1]) for p in pts) + '&longitude=' + ','.join(str(p[2]) for p in pts)
           + f'&start_date=1990-09-01&end_date=2026-10-05&daily={V}&timezone=Asia/Baghdad')
    for attempt in range(12):
        try: d = json.load(urllib.request.urlopen(urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'}), timeout=240)); break
        except Exception as e: print(time.strftime('%H:%M'), 'wait', lake, e, flush=True); time.sleep(120)
    else: sys.exit('gave up ' + lake)
    json.dump({'points': pts, 'data': d}, open(fn, 'w')); print(lake, 'saved', len(d), 'points', flush=True); time.sleep(5)
