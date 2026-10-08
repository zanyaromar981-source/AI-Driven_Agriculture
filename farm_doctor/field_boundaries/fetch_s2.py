"""Fetch a full-resolution (10 m) Sentinel-2 window (B02,B03,B04,B08,SCL) over a point for the field-boundary test."""
import os, sys, json, struct, ast, array
sys.path.insert(0, '/Users/ranjhamakarim/Documents/SmartSuli_AI_Challenge/evidence/past_seasons/backtest')
_a = sys.argv; sys.argv = [sys.argv[0]]
import planted_area_test as P
sys.argv = _a
name, lon, lat, start, end = sys.argv[1], float(sys.argv[2]), float(sys.argv[3]), sys.argv[4], sys.argv[5]
bbox = P.bbox_of(lon, lat, 1.5)
feats = [f for f in P.stac_search(bbox, start, end, 15) if P.contains(f['geometry'], bbox)]
feats.sort(key=lambda f: f['properties'].get('eo:cloud_cover', 99))
if not feats: raise SystemExit('no clear scene')
f = feats[0]; m = P.item_meta(f); print(name, m)
url = (f"{P.DATA}/item/bbox/{','.join(str(v) for v in bbox)}.npy?collection=sentinel-2-l2a&item={f['id']}&assets=B02&assets=B03&assets=B04&assets=B08&assets=SCL")
b = P.http(url)
open(f'{name}.npy', 'wb').write(b); json.dump(dict(meta=m, bbox=bbox), open(f'{name}.json', 'w'))
shape, a = P.parse_npy(b); print(name, 'shape', shape, len(b)//1024, 'KB')
