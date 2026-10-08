"""Dam Watch. Latest measured lake areas from the tested Sentinel-2 method (dam_water_areas.csv); no new fetch here."""
import os, csv, json, sys
HERE = os.path.dirname(os.path.abspath(__file__)); F = os.path.join(HERE, '..', 'evidence', 'past_seasons', 'backtest', 'dam_water_areas.csv')
FULL = {'Dukan': 270.0, 'Darbandikhan': 113.0}
def latest(before=None):
    rows = [r for r in csv.DictReader(open(F)) if r['clear'] == '1' and r['water_km2'] and (not before or r['date'] <= before)]
    out = dict(ai='dam_watch')
    for lake in FULL:
        rr = [r for r in rows if r['lake'] == lake]
        if rr:
            r = max(rr, key=lambda r: r['date']); out[lake] = dict(date=r['date'], km2=round(float(r['water_km2'])), pct_of_full=round(float(r['water_km2']) / FULL[lake] * 100))
    return out
if __name__ == '__main__':
    print(json.dumps(latest(sys.argv[1] if len(sys.argv) > 1 else None), indent=1))
