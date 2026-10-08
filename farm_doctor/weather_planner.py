"""AI 2 — Weather Planner. Free 10-day forecast (Open-Meteo: ECMWF/GraphCast-based) + air quality + river flow,
turned into farm decisions with trial-tested rules. Numbers and labels only."""
import json, sys, urllib.request, urllib.parse, datetime as dt
UA = {'User-Agent': 'Mozilla/5.0'}
RIVERS = {'Greater Zab': (43.325, 35.975), 'Little Zab below Dukan': (44.925, 35.925), 'Sirwan below Darbandikhan': (45.625, 35.025)}

def get(url):
    return json.load(urllib.request.urlopen(urllib.request.Request(url, headers=UA), timeout=60))

def plan(lon, lat, date=None, crop='wheat'):
    p = dict(latitude=lat, longitude=lon, timezone='Asia/Baghdad', forecast_days=10,
             daily='precipitation_sum,temperature_2m_max,temperature_2m_min,wind_speed_10m_max,et0_fao_evapotranspiration,precipitation_probability_max',
             hourly='temperature_2m,relative_humidity_2m,precipitation,wind_speed_10m,soil_moisture_0_to_1cm,soil_moisture_9_to_27cm,soil_moisture_27_to_81cm')
    if date and date != dt.date.today().isoformat():   # replay: historical forecast archive is not free; use the archive of what happened as a stand-in
        p.update(start_date=date, end_date=(dt.date.fromisoformat(date) + dt.timedelta(days=9)).isoformat()); p.pop('forecast_days')
        url = 'https://archive-api.open-meteo.com/v1/archive?' + urllib.parse.urlencode(p)
        p2 = dict(p); p2['daily'] = 'precipitation_sum,temperature_2m_max,temperature_2m_min,wind_speed_10m_max,et0_fao_evapotranspiration'
        p2['hourly'] = 'temperature_2m,relative_humidity_2m,precipitation,wind_speed_10m,soil_moisture_7_to_28cm'
        url = 'https://archive-api.open-meteo.com/v1/archive?' + urllib.parse.urlencode(p2); src = 'ERA5 archive (replay stand-in for the forecast)'
    else:
        url = 'https://api.open-meteo.com/v1/forecast?' + urllib.parse.urlencode(p); src = 'Open-Meteo forecast (ECMWF/GraphCast family)'
    d = get(url); D, Hh = d['daily'], d['hourly']
    days = D['time']; rain = [x or 0 for x in D['precipitation_sum']]; tmax = D['temperature_2m_max']; tmin = D['temperature_2m_min']
    out = dict(ai='weather_planner', source=src, lon=lon, lat=lat, from_date=days[0], days=len(days))
    out['rain_next_10_days_mm'] = round(sum(rain)); out['rain_by_day_mm'] = [round(x, 1) for x in rain]
    big = [(days[i], round(rain[i])) for i in range(len(days)) if rain[i] >= 12]
    out['first_day_with_12mm'] = big[0] if big else None
    # rolling 3-day sums for sowing rule (>=20 mm)
    out['sowing_rain_window'] = next(((days[i], round(sum(rain[i:i+3]))) for i in range(len(days)-2) if sum(rain[i:i+3]) >= 20), None)
    out['frost_nights'] = [(days[i], tmin[i]) for i in range(len(days)) if tmin[i] is not None and tmin[i] <= 0]
    out['hard_frost_nights'] = [(days[i], tmin[i]) for i in range(len(days)) if tmin[i] is not None and tmin[i] <= -2]
    out['heat_days_over_31'] = [(days[i], tmax[i]) for i in range(len(days)) if tmax[i] is not None and tmax[i] >= 31]
    # hourly rules
    ht, hr, hp, hw = Hh['temperature_2m'], Hh['relative_humidity_2m'], Hh['precipitation'], Hh['wind_speed_10m']; htime = Hh['time']
    rust_hours = sum(1 for t, r in zip(ht, hr) if t is not None and r is not None and 6 <= t <= 16 and r >= 90)
    out['rust_weather_hours'] = rust_hours; out['rust_weather'] = 'high' if rust_hours >= 24 else 'some' if rust_hours >= 8 else 'low'
    # spray windows: 6 consecutive hours, no rain, 15-24 C, wind < 15 km/h, daytime
    windows = []
    for i in range(len(htime) - 6):
        blk = range(i, i + 6)
        if all((hp[j] or 0) == 0 and ht[j] is not None and 15 <= ht[j] <= 24 and (hw[j] or 0) < 15 and 7 <= int(htime[j][11:13]) <= 18 for j in blk):
            windows.append(htime[i][:13].replace('T', ' ') + 'h')
    out['spray_windows'] = sorted(set(w[:10] for w in windows))[:5]
    sm_key = next((k for k in Hh if k.startswith('soil_moisture')), None)
    if sm_key and Hh[sm_key][0] is not None:
        sm = Hh[sm_key][0]; out['soil_moisture_now'] = round(sm, 3); out['soil'] = 'dry' if sm < 0.15 else 'moist' if sm < 0.3 else 'wet'
    out['et0_mm_per_day'] = round(sum(x or 0 for x in D['et0_fao_evapotranspiration']) / len(days), 1)
    # sunn pest degree-days since 1 Jan (base 13.3 C): first nymphs ~84, spray between 84 and 223
    try:
        y = days[0][:4]
        a = get('https://archive-api.open-meteo.com/v1/archive?' + urllib.parse.urlencode(dict(latitude=lat, longitude=lon, start_date=f'{y}-01-01', end_date=days[0], daily='temperature_2m_mean', timezone='Asia/Baghdad')))
        dd = sum(max(0, (t or 0) - 13.3) for t in a['daily']['temperature_2m_mean'])
        out['sunn_pest_degree_days'] = round(dd); out['sunn_pest_stage'] = 'before nymphs' if dd < 84 else 'nymphs: count them' if dd < 223 else 'past the spray window'
    except Exception as e:
        out['sunn_pest_degree_days'] = None
    # dust / air quality (forecast only)
    if src.startswith('Open-Meteo'):
        try:
            aq = get('https://air-quality-api.open-meteo.com/v1/air-quality?' + urllib.parse.urlencode(dict(latitude=lat, longitude=lon, hourly='pm10,dust', forecast_days=5, timezone='Asia/Baghdad')))
            pm = [x for x in aq['hourly']['pm10'] if x is not None]; out['dust_pm10_max_5_days'] = round(max(pm)) if pm else None
            out['dust_alert'] = bool(pm) and max(pm) >= 150
        except Exception: pass
        try:
            riv = {}
            for name, (rlon, rlat) in RIVERS.items():
                f = get('https://flood-api.open-meteo.com/v1/flood?' + urllib.parse.urlencode(dict(latitude=rlat, longitude=rlon, daily='river_discharge', forecast_days=10)))
                q = [x for x in f['daily']['river_discharge'] if x is not None]; riv[name] = dict(now_m3s=round(q[0]), max_10_days_m3s=round(max(q)))
            out['rivers'] = riv
        except Exception: pass
    # decisions
    month = int(days[0][5:7]); dec = []
    if month in (10, 11, 12):
        dec.append('SOW: good sowing rain (20 mm or more within 3 days) starts %s' % out['sowing_rain_window'][0] if out['sowing_rain_window'] else 'SOW: no sowing rain (20 mm) in the next 10 days; wait, do not dry-sow')
    if month in (1, 2, 3) and crop == 'wheat':
        dec.append('UREA: spread on dry soil just before %s (%d mm)' % out['first_day_with_12mm'] if out['first_day_with_12mm'] else 'UREA: no rain of 12 mm coming; hold the top-dressing')
    if out['spray_windows']: dec.append('SPRAY: safe days ' + ', '.join(out['spray_windows']))
    if out['hard_frost_nights']: dec.append('FROST: hard frost on ' + ', '.join(d for d, _ in out['hard_frost_nights']) + '; check heads 7-10 days after')
    if out['heat_days_over_31'] and month in (4, 5): dec.append('HEAT: over 31 C on ' + ', '.join(d for d, _ in out['heat_days_over_31']) + ' during flowering')
    if out['rust_weather'] != 'low': dec.append('RUST: %s rust weather (%d cool wet hours); check leaves' % (out['rust_weather'], rust_hours))
    if out.get('sunn_pest_stage') == 'nymphs: count them': dec.append('SUNN PEST: nymphs likely; count with a 0.5 x 0.5 m frame, spray only above 8 per m2')
    if out.get('dust_alert'): dec.append('DUST: PM10 up to %d; delay spraying and harvest, shelter animals' % out['dust_pm10_max_5_days'])
    out['decisions'] = dec
    return out

if __name__ == '__main__':
    lon, lat = float(sys.argv[1]), float(sys.argv[2]); date = sys.argv[3] if len(sys.argv) > 3 else None
    print(json.dumps(plan(lon, lat, date), indent=1, ensure_ascii=False))
