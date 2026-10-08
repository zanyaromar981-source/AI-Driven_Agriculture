"""AI 4 — Season Check. Measured, not forecast: rain since 1 Oct vs the 1991-2020 normal for the nearest zone, the honest
alarm lines, and the El Nino note. Reads web/data.js (built by web/build_data.py) and NOAA's ONI file. Numbers and labels only."""
import os, sys, json, math, datetime as dt
HERE = os.path.dirname(os.path.abspath(__file__)); ROOT = os.path.join(HERE, '..')
def load():
    s = open(os.path.join(ROOT, 'web', 'data.js')).read(); return json.loads(s[s.index('{'):s.rindex('}') + 1])
def check(lon, lat, date):
    D = load(); d0 = dt.date.fromisoformat(date)
    season = d0.year if d0.month >= 10 else d0.year - 1
    e = D['seasons'].get(str(season))
    zones = D['zones']; zi = min(range(len(zones)), key=lambda i: (zones[i]['lat'] - lat) ** 2 + ((zones[i]['lon'] - lon) * math.cos(math.radians(lat))) ** 2)
    z = zones[zi]; out = dict(ai='season_check', season=f'{season}/{(season + 1) % 100:02d}', zone=z['name'], area=D['areas'][z['area']]['name'], date=date)
    if not e: out['status'] = 'no data for this season'; return out
    slot = (d0 - dt.date(season, 10, 1)).days
    if d0.month == 2 and d0.day == 29: slot -= 1
    if d0 > dt.date(season + 1, 2, 28): slot -= 1 if (season + 1) % 4 == 0 and d0 > dt.date(season + 1, 2, 29) else 0
    slot = max(0, min(181, slot))
    zr = e.get('zoneRain'); rr = e['rain']
    if slot < len(rr):
        out['rain_since_1_oct_pct_of_normal_region'] = rr[slot]
        if zr and slot < len(zr[zi]): out['rain_since_1_oct_pct_of_normal_zone'] = zr[zi][slot]
    out['days_into_season'] = slot
    out['too_early'] = slot < 45
    cut = D['cutoffs']
    if slot >= 181: line, key = cut['mar'], 'mar'
    elif slot >= 150: line, key = cut['feb'], 'feb'
    elif slot >= 122: line, key = cut['jan'], 'jan'
    else: line, key = None, None
    rp = out.get('rain_since_1_oct_pct_of_normal_region')
    if line and rp is not None:
        out['alarm_line_pct'] = line; out['drought_alarm'] = rp < line
        out['label'] = 'drought' if rp < line else 'dry' if rp < 85 else 'normal' if rp < 115 else 'wet'
    elif slot >= 70 and e.get('watch'):
        out['december_watch'] = e['watch']['on']; out['label'] = 'watch' if e['watch']['on'] else 'normal so far'
    else:
        out['label'] = 'too early to judge the season' if out['too_early'] else 'no check yet (first: 10 Dec)'
    out['reliability'] = 'none before December; drought caught 4/10 by 31 Jan, 6/10 by 28 Feb, 9/10 by 31 Mar (36 winters, 1-2 false alarms)'
    oni = e.get('oni')
    if oni is not None:
        out['el_nino_index_jul_sep'] = oni
        out['pre_season_note'] = ('El Nino: in 35 years no El Nino winter was a drought (7/7); worst case normal' if oni >= 0.5 else
                                  'La Nina: dry-leaning (4/7 droughts); never a wet winter' if oni <= -0.5 else 'neutral: no pre-season signal')
    if e.get('risk') and key: 
        c = {'jan': 'Jan', 'feb': 'Feb', 'mar': 'Mar'}[key]; p = e['risk'][c][zi]; L = D['lights'][c]
        out['zone_crop_risk_pct'] = round(p * 100); out['zone_light'] = 'red' if p >= L['red'] else 'yellow' if p >= L['yellow'] else 'green'
    return out
if __name__ == '__main__':
    print(json.dumps(check(float(sys.argv[1]), float(sys.argv[2]), sys.argv[3]), indent=1))
