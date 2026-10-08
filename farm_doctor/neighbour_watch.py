"""AI 5 — Neighbour Watch. Farmers' geotagged reports (reports.json). Returns similar reports within 20 km in the last 14 days.
reports.json is TEST DATA for now (marked), until the app collects real ones."""
import os, sys, json, math, datetime as dt
HERE = os.path.dirname(os.path.abspath(__file__))
def km(lon1, lat1, lon2, lat2):
    return math.hypot((lon2 - lon1) * 111.32 * math.cos(math.radians(lat1)), (lat2 - lat1) * 110.57)
def nearby(lon, lat, date, radius_km=20, days=14):
    f = os.path.join(HERE, 'reports.json'); R = json.load(open(f)) if os.path.exists(f) else []
    d0 = dt.date.fromisoformat(date); hits = []
    for r in R:
        rd = dt.date.fromisoformat(r['date'])
        if 0 <= (d0 - rd).days <= days and km(lon, lat, r['lon'], r['lat']) <= radius_km:
            hits.append(dict(type=r['type'], crop=r.get('crop'), km=round(km(lon, lat, r['lon'], r['lat']), 1), days_ago=(d0 - rd).days))
    types = {}
    for h in hits: types[h['type']] = types.get(h['type'], 0) + 1
    return dict(ai='neighbour_watch', reports_nearby=len(hits), by_type=types, closest=sorted(hits, key=lambda h: h['km'])[:3], test_data=True, radius_km=radius_km, days=days)
if __name__ == '__main__':
    print(json.dumps(nearby(float(sys.argv[1]), float(sys.argv[2]), sys.argv[3]), indent=1))
