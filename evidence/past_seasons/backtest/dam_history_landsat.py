"""Lake area history 1988–2016 from Landsat (June = start of summer, October = end of summer; --march-only = end of winter), reusing dam_water_test.py.
Usage: python3 -I dam_history_landsat.py --data-dir ../backtest_data/dam_water --csv dam_history_landsat.csv [--workers 4]"""
import sys, os, csv, argparse, concurrent.futures as cf
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import dam_water_test as D
MARCH_CLOUD = 20      # spring is cloudy: allow more scene cloud; clouds over the lake lower the area, so the median of clear dates is used
def platform(y):
    if y <= 2011: return 'landsat-5'
    if y == 2012: return None            # only Landsat 7 with striped gaps → skip
    return 'landsat-8'
def window(job):
    lake, y, lab, s, e = job; plat = platform(y); out = []
    if not plat: return [dict(lake=lake, year=y, window=lab, error='no usable Landsat (2012)')]
    try: feats = D.landsat_dates(lake, f'{y}-{s}', f'{y}-{e}', plat, MARCH_CLOUD if lab == 'March' else 5)
    except Exception as ex: return [dict(lake=lake, year=y, window=lab, error=str(ex)[:120])]
    groups = {}
    for f in feats: groups.setdefault((f['properties']['datetime'][:10], f['id'].split('_')[2][:3]), []).append(f)
    order = sorted(groups.items(), key=lambda kv: max(f['properties']['eo:cloud_cover'] for f in kv[1]))
    got = 0
    for (date, path), fs in order[:5]:
        try: m = D.landsat_measure(lake, fs)
        except Exception as ex: out.append(dict(lake=lake, year=y, window=lab, date=date, error=str(ex)[:120])); continue
        out.append(dict(lake=lake, year=y, window=lab, date=date, source=f'{plat} path {path}', km2=m['ndwi'], valid=m['valid_percent'], clear=m['clear']))
        got += bool(m['clear'])
        if got >= 2: break
    if not feats: out.append(dict(lake=lake, year=y, window=lab, error='no scene <5% cloud'))
    return out
if __name__ == '__main__':
    ap = argparse.ArgumentParser(); ap.add_argument('--data-dir', required=True); ap.add_argument('--csv', required=True)
    ap.add_argument('--workers', type=int, default=4); ap.add_argument('--first-year', type=int, default=1988); ap.add_argument('--last-year', type=int, default=2016)
    ap.add_argument('--march-only', action='store_true', help='only the end-of-winter window (1 Feb - 31 Mar, scene cloud below MARCH_CLOUD percent), e.g. for the June forecast')
    a = ap.parse_args(); D.DATA_DIR = a.data_dir
    jobs = [(lake, y, lab, s, e) for lake in D.LAKES for y in range(a.first_year, a.last_year + 1)
            for lab, s, e in ((('March', '02-01', '03-31'),) if a.march_only else (('June', '05-15', '06-30'), ('October', '09-01', '10-31')))]
    rows = []
    with cf.ThreadPoolExecutor(a.workers) as ex:
        for i, res in enumerate(ex.map(window, jobs)):
            rows += res
            if i % 10 == 0: print(f'{i+1}/{len(jobs)} windows done', flush=True)
    with open(a.csv, 'w', newline='') as fh:
        w = csv.writer(fh); w.writerow(['lake', 'year', 'window', 'date', 'source', 'water_km2', 'valid_percent', 'clear', 'error'])
        for r in rows: w.writerow([r['lake'], r['year'], r['window'], r.get('date', ''), r.get('source', ''), f"{r['km2']:.2f}" if 'km2' in r else '', f"{r['valid']:.1f}" if 'valid' in r else '', int(r.get('clear', 0)), r.get('error', '')])
    print('wrote', a.csv, len(rows), 'rows')
