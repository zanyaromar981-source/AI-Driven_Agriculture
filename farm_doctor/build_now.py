"""Sprint 1: run the AIs for all 16 zones for TODAY and write web/now.json for the Ministry dashboard.
Usage: python3 -I build_now.py [YYYY-MM-DD]   (default today; ~5 min, Sentinel-2 fetches run in parallel)"""
import os, sys, json, time, datetime as dt
from concurrent.futures import ThreadPoolExecutor
HERE = os.path.dirname(os.path.abspath(__file__)); ROOT = os.path.join(HERE, '..'); sys.path.insert(0, HERE)
import field_eye, weather_planner, season_check, neighbour_watch, dam_watch, doctor
date = sys.argv[1] if len(sys.argv) > 1 else dt.date.today().isoformat()
s = open(os.path.join(ROOT, 'web', 'data.js')).read(); D = json.loads(s[s.index('{'):s.rindex('}') + 1])
zones = D['zones']; areas = D['areas']
def safe(fn, name):
    t = time.time()
    try: r = fn()
    except Exception as e: r = dict(ai=name, error=str(e)[:160])
    r['_seconds'] = round(time.time() - t, 1); return r
def one(z):
    lon, lat = z['lon'], z['lat']
    return dict(zone=z['name'], ku=z['ku'], area=areas[z['area']]['name'], lon=lon, lat=lat,
                field_eye=safe(lambda: field_eye.measure(lon, lat, date, field_half_km=1.0), 'field_eye'),
                weather=safe(lambda: weather_planner.plan(lon, lat, date if date != dt.date.today().isoformat() else None), 'weather_planner'),
                season=safe(lambda: season_check.check(lon, lat, date), 'season_check'),
                neighbours=safe(lambda: neighbour_watch.nearby(lon, lat, date), 'neighbour_watch'))
t0 = time.time()
with ThreadPoolExecutor(4) as ex: rows = list(ex.map(one, zones))
out = dict(date=date, built_at=dt.datetime.now().isoformat(timespec='seconds'), zones=rows, dams=safe(lambda: dam_watch.latest(date), 'dam_watch'))
# region summary for the doctor
summary = dict(date=date, zones=[dict(zone=r['zone'], area=r['area'], surface=r['field_eye'].get('surface'), greenness_pct_of_normal=r['field_eye'].get('pct_of_own_normal'),
                                      rain_10d_mm=r['weather'].get('rain_next_10_days_mm'), decisions=r['weather'].get('decisions'), season_label=r['season'].get('label'),
                                      rain_since_oct_pct=r['season'].get('rain_since_1_oct_pct_of_normal_zone'), reports_nearby=r['neighbours'].get('reports_nearby')) for r in rows],
               dams=out['dams'], rivers=rows[0]['weather'].get('rivers'), dust_pm10_max=max((r['weather'].get('dust_pm10_max_5_days') or 0) for r in rows),
               el_nino=rows[0]['season'].get('pre_season_note'))
out['summary'] = summary
brief = doctor.ask(summary, 'Write the weekly briefing for the Ministry of Agriculture: the state of the region now, the 3 things to act on this week, and what we cannot tell. Fill "sorani" and "english".')
out['brief'] = brief
json.dump(out, open(os.path.join(ROOT, 'web', 'now.json'), 'w'), indent=1, ensure_ascii=False)
print('wrote web/now.json in', round(time.time() - t0), 's;', sum(1 for r in rows if 'error' in r['field_eye']), 'field-eye errors;', 'brief:', 'ok' if 'english' in brief else brief.get('error', 'failed'))
