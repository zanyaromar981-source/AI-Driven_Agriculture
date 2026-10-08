"""AI 1 — Field Eye. One field (point + radius) from Sentinel-2 via Microsoft Planetary Computer (keyless).
Returns numbers only: greenness now, vs the same weeks in the last 3 years, vs the 5 km neighbourhood, clear picture date.
Reuses the tested helpers in evidence/past_seasons/backtest/planted_area_test.py."""
import os, sys, math, json, datetime as dt, statistics as st
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, '..', 'evidence', 'past_seasons', 'backtest'))
_argv = sys.argv; sys.argv = [sys.argv[0]]          # planted_area_test parses argv at import
import planted_area_test as P
sys.argv = _argv

def _best_scene(bbox, start, end, max_cloud=60):
    feats = [f for f in P.stac_search(bbox, start, end, max_cloud) if P.contains(f['geometry'], bbox)]
    feats.sort(key=lambda f: f['properties'].get('eo:cloud_cover', 99))
    return feats

def _ndvi_mean(item, bbox, baseline, max_size):
    H, W, nd, valid = P.scene_ndvi(P.http(P.crop_url(item, bbox, max_size)), baseline)
    vals = [v for v in nd if v == v]
    return (st.mean(vals) if vals else None), valid, (H, W, nd)

def measure(lon, lat, date, field_half_km=0.5, hood_half_km=2.5, window_days=20, years_back=3):
    """date: 'YYYY-MM-DD'. Returns a dict of numbers and labels for the doctor."""
    d0 = dt.date.fromisoformat(date)
    fb = P.bbox_of(lon, lat, field_half_km); hb = P.bbox_of(lon, lat, hood_half_km)
    out = dict(ai='field_eye', lon=lon, lat=lat, date=date, field_km2=round((2*field_half_km)**2, 2))
    def pick(start, end):
        for f in _best_scene(fb, start, end)[:6]:
            m = P.item_meta(f)
            v, valid, grid = _ndvi_mean(f['id'], fb, m['baseline'], 120)
            if v is not None and valid >= 0.6: return m, v, valid, grid
        return None, None, None, None
    m, now, valid, grid = pick((d0 - dt.timedelta(days=window_days)).isoformat(), d0.isoformat())
    if m is None:
        out.update(status='no clear picture in the last %d days' % window_days); return out
    out.update(picture_date=m['date'], picture_cloud_pct=m['cloud'], clear_share=round(valid, 2), greenness_now=round(now, 3))
    # same weeks in previous years (own normal)
    hist = []
    for k in range(1, years_back + 1):
        y = dt.date(d0.year - k, d0.month, min(d0.day, 28))
        mh, vh, _, _ = pick((y - dt.timedelta(days=window_days)).isoformat(), (y + dt.timedelta(days=window_days)).isoformat())
        if vh is not None: hist.append((y.year, round(vh, 3)))
    if hist:
        normal = st.mean(v for _, v in hist)
        out.update(greenness_normal=round(normal, 3), history=hist, pct_of_own_normal=round(now / max(normal, 0.05) * 100))
    # neighbourhood on the same picture
    vn, validn, _ = _ndvi_mean(m['id'], hb, m['baseline'], 160)
    if vn is not None:
        out.update(greenness_neighbours=round(vn, 3), pct_of_neighbours=round(now / max(vn, 0.05) * 100))
    # inside the field: share of weak pixels (below 70% of the field's own median) and where they sit
    H, W, nd = grid; vals = [(i, v) for i, v in enumerate(nd) if v == v]
    if vals:
        med = st.median(v for _, v in vals); weak = [i for i, v in vals if v < 0.7 * med and med > 0.15]
        out['weak_share_pct'] = round(100 * len(weak) / len(vals))
        if weak:
            r = st.mean(i // W for i in weak) / H; c = st.mean(i % W for i in weak) / W
            out['weak_where'] = ('north' if r < 0.4 else 'south' if r > 0.6 else 'middle') + '-' + ('west' if c < 0.4 else 'east' if c > 0.6 else 'centre')
    # plain label
    g = out['greenness_now']
    out['surface'] = 'bare soil or stubble' if g < 0.2 else 'sparse crop' if g < 0.35 else 'growing crop' if g < 0.6 else 'dense crop'
    return out

if __name__ == '__main__':
    lon, lat, date = float(sys.argv[1]), float(sys.argv[2]), sys.argv[3]
    print(json.dumps(measure(lon, lat, date), indent=1))
