"""Redraw every map in jutyar_dashboard.pen from the real boundaries in web/map_demo/kri_map_data.js.

Usage: python build_kri_maps.py <in.pen> <out.pen>   (also writes kri_map_component.pen next to <out.pen>)
Needs shapely. Each map keeps its meaning: the overview screens get the bright governorate colours of the
map demo, the data screens keep their own colours, carried over by district name. Only the four
governorates are drawn; nothing outside them.
"""
import json, math, random, string, sys
from shapely.geometry import shape, box, Point
from shapely.ops import polylabel, unary_union

SRC, DST = sys.argv[1], sys.argv[2]
DATA = __file__.replace('\\', '/').rsplit('/design/', 1)[0] + '/web/map_demo/kri_map_data.js'
raw = open(DATA, encoding='utf-8').read()
K = json.loads(raw[raw.index('=') + 1:].strip().rstrip(';'))
doc = json.load(open(SRC, encoding='utf-8'))

# ---------- ids ----------
used = set()
def collect(n):
    used.add(n.get('id'))
    for c in n.get('children', []) or []:
        collect(c)
for c in doc['children']:
    collect(c)
rng = random.Random(42)
def nid():
    while True:
        s = ''.join(rng.choice(string.ascii_letters + string.digits) for _ in range(6))
        if s not in used:
            used.add(s)
            return s

def find(pred):
    out = []
    def w(n, parent):
        if pred(n):
            out.append((n, parent))
        for c in n.get('children', []) or []:
            w(c, n)
    for c in doc['children']:
        w(c, None)
    return out
def by_id(i):
    return find(lambda n: n.get('id') == i)[0][0]

# ---------- geometry ----------
def feats(fc, keep=lambda p: True):
    return [(f['properties'], shape(f['geometry']).buffer(0)) for f in fc['features'] if keep(f['properties'])]
GOVS = feats(K['governorates'])
DISTS = feats(K['districts'], lambda p: p['gov'] != 'Context')
SUBS = feats(K['subdistricts'], lambda p: p['gov'] == 'Sulaymaniyah' and p['dist'] == 'Chamchamal')
REGION = unary_union([g for _, g in GOVS])
CHAM = [g for p, g in DISTS if p['en'] == 'Chamchamal'][0]

def merc(lon, lat):
    return lon * math.pi / 180, math.log(math.tan(math.pi / 4 + lat * math.pi / 360))

class View:
    """Web Mercator view: fit lon/lat bounds into a W x H box (optionally blend towards other bounds)."""
    def __init__(self, bounds, W, H, pad):
        x0, y0 = merc(bounds[0], bounds[1]); x1, y1 = merc(bounds[2], bounds[3])
        self.s = min((W - 2 * pad) / (x1 - x0), (H - 2 * pad) / (y1 - y0))
        self.cx, self.cy = (x0 + x1) / 2, (y0 + y1) / 2
        self.W, self.H = W, H
    @staticmethod
    def blend(a, b, t):
        v = View.__new__(View)
        v.W, v.H = a.W, a.H
        v.s = math.exp(math.log(a.s) * (1 - t) + math.log(b.s) * t)
        v.cx, v.cy = a.cx * (1 - t) + b.cx * t, a.cy * (1 - t) + b.cy * t
        return v
    def P(self, lon, lat):
        x, y = merc(lon, lat)
        return self.W / 2 + (x - self.cx) * self.s, self.H / 2 - (y - self.cy) * self.s
    def inv(self, px, py):
        x = (px - self.W / 2) / self.s + self.cx; y = (self.H / 2 - py) / self.s + self.cy
        return x * 180 / math.pi, (2 * math.atan(math.exp(y)) - math.pi / 2) * 180 / math.pi
    def lonlat_box(self, margin=20):
        a = self.inv(-margin, -margin); b = self.inv(self.W + margin, self.H + margin)
        return box(min(a[0], b[0]), min(a[1], b[1]), max(a[0], b[0]), max(a[1], b[1]))

def geom_d(g, v, tol_px=0.5):
    tol = tol_px / v.s * 180 / math.pi
    g = g.simplify(tol, preserve_topology=True)
    parts = []
    for p in (g.geoms if hasattr(g, 'geoms') else [g]):
        if p.is_empty or p.geom_type != 'Polygon':
            continue
        for ring in [p.exterior] + list(p.interiors):
            pts, last = [], None
            for x, y in ring.coords:
                q = v.P(x, y)
                c = (round(q[0], 1), round(q[1], 1))
                if c != last:
                    pts.append(c); last = c
            if len(pts) >= 4:
                parts.append(pts)
    return parts

def path_node(name, g, v, extra, tol_px=0.5):
    parts = geom_d(g, v, tol_px)
    if not parts:
        return None
    xs = [x for p in parts for x, _ in p]; ys = [y for p in parts for _, y in p]
    x0, y0, x1, y1 = min(xs), min(ys), max(xs), max(ys)
    d = ''.join('M' + 'L'.join(f'{x:g} {y:g}' for x, y in p) + 'Z' for p in parts)
    n = {'type': 'path', 'id': nid(), 'name': name, 'x': round(x0, 2), 'y': round(y0, 2),
         'width': round(max(x1 - x0, 0.5), 2), 'height': round(max(y1 - y0, 0.5), 2),
         'geometry': d, 'viewBox': [round(x0, 2), round(y0, 2), round(max(x1 - x0, 0.5), 2), round(max(y1 - y0, 0.5), 2)],
         'strokeLinejoin': 'round'}
    n.update(extra)
    return n

def label_pt(g, v):
    gg = max(g.geoms, key=lambda q: q.area) if hasattr(g, 'geoms') else g
    p = polylabel(gg, 0.001)
    return v.P(p.x, p.y)

# ---------- colours ----------
VARS = {k: v['value'] for k, v in doc.get('variables', {}).items() if v.get('type') == 'color'}
def resolve(c):
    while isinstance(c, str) and c.startswith('$'):
        c = VARS.get(c[1:], '#888888')
    return c
def dark(c):
    c = resolve(c)
    if not isinstance(c, str) or not c.startswith('#') or len(c) < 7:
        return False
    r, g, b = (int(c[i:i + 2], 16) / 255 for i in (1, 3, 5))
    lum = lambda u: u / 12.92 if u <= 0.04045 else ((u + 0.055) / 1.055) ** 2.4
    return 0.2126 * lum(r) + 0.7152 * lum(g) + 0.0722 * lum(b) < 0.22

# new district -> old zone it replaces (for carrying sample data colours over)
ALIAS = {'Qushtapa': 'Dashti Hawler', 'Taqtaq': 'Koya', 'Harir': 'Shaqlawa', 'Pirmam': 'Shaqlawa', 'Khalifan': 'Soran',
         'Chuman': 'Choman', 'Sidakan': 'Choman', 'Khurmal': 'Halabja'}
def old_fills(map_node):
    return {c['name'][7:]: c.get('fill') for c in map_node['children'] if c.get('type') == 'path' and c['name'].startswith('Zone · ')}
def carry(fills, en):
    return fills.get(en) or fills.get(ALIAS.get(en, ''), '#DDD9CB')

GLOW = [{'type': 'shadow', 'shadowType': 'outer', 'offset': {'x': 0, 'y': 0}, 'blur': 5, 'color': '#FFFFFFFF'},
        {'type': 'shadow', 'shadowType': 'outer', 'offset': {'x': 0, 'y': 0}, 'blur': 2, 'color': '#FFFFFFFF'}]

def text_node(name, content, cx, cy, size, weight, fill, font, width=170, spacing=None, glow=True, upper=False):
    t = {'type': 'text', 'id': nid(), 'name': name, 'x': round(cx - width / 2, 1), 'y': round(cy - size * 0.68, 1),
         'textGrowth': 'fixed-width', 'width': width, 'textAlign': 'center', 'fill': fill,
         'content': content.upper() if upper else content, 'fontFamily': font, 'fontSize': size, 'fontWeight': weight}
    if spacing:
        t['letterSpacing'] = spacing
    if glow:
        t['effect'] = GLOW
    return t

def borders(v, scale=1.0, clip=None):
    out = []
    for p, g in GOVS:
        gg = g if clip is None else g.intersection(clip)
        n = path_node('Border casing · ' + p['en'], gg, v, {'stroke': '#14201A59', 'strokeWidth': round(4.5 * scale, 2)})
        if n: out.append(n)
    for p, g in GOVS:
        gg = g if clip is None else g.intersection(clip)
        n = path_node('Border · ' + p['en'], gg, v, {'stroke': '#FFFFFF', 'strokeWidth': round(2.4 * scale, 2)})
        if n: out.append(n)
    return out

def districts(v, colour, scale=1.0, opacity=None, clip=None, skip=()):
    out = []
    for p, g in DISTS:
        if p['en'] in skip:
            continue
        gg = g if clip is None else g.intersection(clip)
        if gg.is_empty or gg.area < 1e-6:
            continue
        extra = {'fill': colour(p), 'stroke': '#FFFFFF', 'strokeWidth': round(1.1 * scale, 2)}
        if opacity is not None:
            extra['opacity'] = opacity
        n = path_node('District · ' + p['en'], gg, v, extra)
        if n: out.append(n)
    return out

def district_labels(v, lang='en', size=10, fills=None, nudge=None):
    out = []
    for p, g in DISTS:
        x, y = label_pt(g, v)
        if nudge and p['en'] in nudge:
            x += nudge[p['en']][0]; y += nudge[p['en']][1]
        f = (fills or {}).get(p['en'], '#22302A')
        white = dark(f)
        content = p['en'] if lang == 'en' else (p['ku'] or p['en'])
        font = '$font-ui' if lang == 'en' else '$font-ku'
        out.append(text_node('Label · ' + p['en'], content, x, y, size if lang == 'en' else size + 1, '600',
                             '#FFFFFF' if white else '#22302A', font, width=120, glow=not white))
    return out

def gov_labels(v, lang='en'):
    out = []
    for p, g in GOVS:
        x, y = label_pt(g, v)
        y += {'Duhok': -26, 'Erbil': -38, 'Sulaymaniyah': -30, 'Halabja': 24}[p['en']]
        x += {'Halabja': 26}.get(p['en'], 0)
        if p['en'] == 'Erbil':
            x, y = 215, 296
        if p['en'] == 'Halabja':
            x, y = 712, 470
        if lang == 'en':
            out.append(text_node('City label · ' + p['en'], p['en'], x, y, 17, '800', '$ink', '$font-display', width=220, spacing=1.6, upper=True))
        else:
            out.append(text_node('City label · ' + p['en'], p['ku'], x, y, 18, '700', '$ink', '$font-ku', width=220))
    return out

def place(node, x, y):
    node['x'] = round(x, 1); node['y'] = round(y, 1)

def strip(map_node, prefixes):
    map_node['children'] = [c for c in map_node['children'] if not any(c.get('name', '').startswith(p) for p in prefixes)]

def put_back(map_node, front, keep_on_top):
    """front = new map nodes (bottom), then the kept overlay nodes in their old order (top)."""
    kept = [c for c in map_node['children'] if c in keep_on_top]
    rest = [c for c in map_node['children'] if c not in keep_on_top]
    map_node['children'] = front + rest + kept

OV = View(REGION.bounds, 760, 732, 22)

def dms_label(v, hemi, step_min):
    d = int(math.floor(v + 1e-9)); m = int(round((v - d) * 60))
    if m == 60: d += 1; m = 0
    return f"{d}°{hemi}" if m == 0 else f"{d}°{m:02d}′{hemi}"

def graticule(v, line_min, tick_min, label_min, avoid=()):
    """Latitude / longitude grid with rulers along the top and left edges (Web Mercator keeps both straight)."""
    W, H = v.W, v.H
    lon0, lat1 = v.inv(0, 0); lon1, lat0 = v.inv(W, H)
    out = []
    def frange(a, b, step):
        k = math.ceil(a / step - 1e-9)
        while k * step <= b + 1e-9:
            yield round(k * step, 6); k += 1
    rect = lambda name, x, y, w, h, fill: {'type': 'rectangle', 'id': nid(), 'name': name, 'x': round(x, 2), 'y': round(y, 2),
                                           'width': round(w, 2), 'height': round(h, 2), 'fill': fill}
    for lat in frange(lat0, lat1, line_min / 60):
        y = v.P(lon0, lat)[1]
        out.append(rect(f'Grid · {lat:.4f}N', 0, y - 0.4, W, 0.8, '#14201A2E'))
    for lon in frange(lon0, lon1, line_min / 60):
        x = v.P(lon, lat0)[0]
        out.append(rect(f'Grid · {lon:.4f}E', x - 0.4, 0, 0.8, H, '#14201A2E'))
    out.append(rect('Ruler · top', 0, 0, W, 14, '#FFFFFFD9'))
    out.append(rect('Ruler · left', 0, 0, 14, H, '#FFFFFFD9'))
    for lon in frange(lon0, lon1, tick_min / 60):
        x = v.P(lon, lat0)[0]; major = abs(lon * 60 / label_min - round(lon * 60 / label_min)) < 1e-3
        out.append(rect('Tick', x - 0.5, 0, 1, 10 if major else 5, '#14201A' if major else '#14201A99'))
        if major and 20 < x < W - 50 and not any(a <= x + 50 and x <= c and b <= 30 and 15 <= d for a, b, c, d in avoid):
            out.append({'type': 'text', 'id': nid(), 'name': 'Ruler label · ' + dms_label(lon, 'E', label_min), 'x': round(x + 3, 1), 'y': 15,
                        'content': dms_label(lon, 'E', label_min), 'fill': '#14201A', 'fontFamily': '$font-ui', 'fontSize': 9.5, 'fontWeight': '700', 'effect': GLOW})
    for lat in frange(lat0, lat1, tick_min / 60):
        y = v.P(lon0, lat)[1]; major = abs(lat * 60 / label_min - round(lat * 60 / label_min)) < 1e-3
        out.append(rect('Tick', 0, y - 0.5, 10 if major else 5, 1, '#14201A' if major else '#14201A99'))
        if major and 20 < y < H - 20 and not any(a <= 70 and 16 <= c and b <= y and y - 14 <= d for a, b, c, d in avoid):
            out.append({'type': 'text', 'id': nid(), 'name': 'Ruler label · ' + dms_label(lat, 'N', label_min), 'x': 16, 'y': round(y - 13, 1),
                        'content': dms_label(lat, 'N', label_min), 'fill': '#14201A', 'fontFamily': '$font-ui', 'fontSize': 9.5, 'fontWeight': '700', 'effect': GLOW})
    return out
TOWN = {t['en']: t for t in K['towns']}
DAM = {d['en']: d for d in K['dams']}
OLD = ['Zone · ', 'Label · ', 'Zones · ', 'Border', 'Zone label · ', 'City label · ', 'District · ']
log = []

# ---------- the map as one master component, screens use instances ----------
# The master holds the geography only (districts, borders, grid and rulers, names). Screens never redraw it:
# each instance overrides colours or names, and screen extras (dams, badges, fires) sit on top of it.
NUDGE = {'Sulaymaniyah': (8, 12), 'Duhok': (0, 10), 'Halabja': (10, -6), 'Sharazur': (-14, -8), 'Khurmal': (12, 2), 'Darbandikhan': (-22, -8)}
BRIGHT = {p['en']: p['color'] for p, _ in DISTS}
KU = {p['en']: p['ku'] or p['en'] for p, _ in DISTS}
GOV_KU = {p['en']: p['ku'] for p, _ in GOVS}

right = max((c.get('x', 0) + (c['width'] if isinstance(c.get('width'), (int, float)) else 1440)) for c in doc['children'])
master_nodes = districts(OV, lambda p: BRIGHT[p['en']]) + borders(OV) + graticule(OV, 30, 10, 60) \
    + district_labels(OV, 'en', 10, BRIGHT, NUDGE) + gov_labels(OV, 'en')
MASTER = {'type': 'frame', 'id': nid(), 'name': 'Component · KRI Map', 'reusable': True, 'x': right + 300, 'y': 0,
          'width': 760, 'height': 732, 'layout': 'none', 'clip': True, 'children': master_nodes,
          'metadata': {'type': 'kri-map', 'source': 'web/map_demo/kri_map_data.js', 'builder': 'design/dashboard/build_kri_maps.py'}}
V6 = View(REGION.bounds, 380, 366, 10)
small_nodes = districts(V6, lambda p: BRIGHT[p['en']], 0.7) + borders(V6, 0.55)
MASTER_S = {'type': 'frame', 'id': nid(), 'name': 'Component · KRI Map small', 'reusable': True, 'x': right + 300, 'y': 832,
            'width': 380, 'height': 366, 'layout': 'none', 'clip': True, 'children': small_nodes,
            'metadata': {'type': 'kri-map', 'source': 'web/map_demo/kri_map_data.js', 'builder': 'design/dashboard/build_kri_maps.py'}}
doc['children'] += [MASTER, MASTER_S]
ID = {n['name']: n['id'] for n in master_nodes}
ID_S = {n['name']: n['id'] for n in small_nodes}

def instance(master, x=0, y=0, over=None, name='KRI Map'):
    r = {'type': 'ref', 'id': nid(), 'ref': master['id'], 'name': name, 'x': x, 'y': y}
    if over:
        r['descendants'] = over
    return r

def colour_overrides(colmap, ids, labels=True):
    over = {}
    for en, col in colmap.items():
        over[ids['District · ' + en]] = {'fill': col}
        if labels and ('Label · ' + en) in ids:
            over[ids['Label · ' + en]] = {'fill': '#FFFFFF', 'effect': []} if dark(col) else {'fill': '#22302A', 'effect': GLOW}
    return over

def place_dams(nodes):
    for d in nodes:
        if d.get('name', '').startswith('Dam · '):
            key = 'Dukan Dam' if 'Dukan' in d['name'] else 'Darbandikhan Dam'
            x, y = OV.P(DAM[key]['lon'], DAM[key]['lat']); place(d, x - 13, y - 12)

# 01 overview (English): the master as it is
m = by_id('TwjaJ')
extras = [c for c in m['children'] if c.get('name', '').startswith('Dam · ')]
place_dams(extras)
m['children'] = [instance(MASTER)] + extras
m['clip'] = True
log.append('TwjaJ: instance of the master + %d dams' % len(extras))

# 07 overview (Sorani): same map, Sorani names
m = by_id('QhxV8')
extras = [c for c in m['children'] if c.get('name', '').startswith('Dam · ')]
place_dams(extras)
over = {ID['Label · ' + en]: {'content': KU[en], 'fontFamily': '$font-ku', 'fontSize': 11} for en in KU}
over.update({ID['City label · ' + en]: {'content': GOV_KU[en], 'fontFamily': '$font-ku', 'fontSize': 18, 'fontWeight': '700', 'letterSpacing': 0}
             for en in GOV_KU})
m['children'] = [instance(MASTER, over=over)] + extras
m['clip'] = True
log.append('QhxV8: instance with Sorani names + %d dams' % len(extras))

# 03, 04, 05 data screens: their own colours on the same map
lp = {p['en']: label_pt(g, OV) for p, g in DISTS}
for map_id in ('xuGjN', 'sOgR8', 'pI9am'):
    m = by_id(map_id)
    fills = old_fills(m)
    colmap = {en: carry(fills, en) for en in BRIGHT}
    extras = [c for c in m['children'] if c.get('type') != 'path' and not c.get('name', '').startswith('Label · ')]
    for o in extras:
        nm = o.get('name', '')
        if nm.startswith('Rank Badge · '):
            x, y = lp[nm.split(' · ')[1]]; place(o, x - 9, y + 8)
        elif nm.startswith('Fire · '):
            x, y = lp[nm.split(' · ')[1]]; place(o, x - 12, y - 52)
    place_dams(extras)
    over = colour_overrides(colmap, ID)
    over.update({ID['City label · ' + en]: {'enabled': False} for en in GOV_KU})
    m['children'] = [instance(MASTER, over=over)] + extras
    m['clip'] = True
    log.append(f'{map_id}: instance with its own colours + {len(extras)} extras')

# 06 compare years: two instances of the small master
for map_id in ('V6nyuc', 'I0vZi'):
    m = by_id(map_id)
    fills = old_fills(m)
    colmap = {en: carry(fills, en) for en in BRIGHT}
    m['children'] = [instance(MASTER_S, over=colour_overrides(colmap, ID_S, labels=False), name='KRI Map small')]
    m['clip'] = True
    log.append(f'{map_id}: instance of the small master')

# standalone copy of both masters, so the map lives on its own and can be reused in other designs
standalone = {'version': doc['version'], 'variables': doc.get('variables', {}),
              'children': [json.loads(json.dumps(dict(MASTER, x=0, y=0))), json.loads(json.dumps(dict(MASTER_S, x=0, y=832)))]}
STANDALONE = DST.replace('\\', '/').rsplit('/', 1)[0] + '/kri_map_component.pen'
json.dump(standalone, open(STANDALONE, 'w', encoding='utf-8'), ensure_ascii=False, indent=2)
log.append('standalone master written: ' + STANDALONE)

# ---------- 02 zoom into Chamchamal ----------
ZV = View(CHAM.bounds, 760, 732, 95)
VIEWBOX = ZV.lonlat_box(30)
m = by_id('jmJ4D')
fills = old_fills(m)
part_fills = {c['name'][7:]: c.get('fill') for c in m['children'] if c.get('name', '').startswith('Part · ')}
labels = {c['name'][13:]: c for c in m['children'] if c.get('name', '').startswith('Part Label · ')}
overlays = [c for c in m['children'] if c.get('name') in ('Breadcrumb', 'Zoom Out Button', 'Zoom Control', 'Mini Map')]
PART = {'Markaz Chamchamal': 'Chamchamal centre', 'Agjalare': 'Aghjalar', 'Cenkaw': 'Sangaw', 'Kadr Karam': 'Qadir Karam'}
new = districts(ZV, lambda p: carry(fills, p['en']), 1.6, opacity=0.32, clip=VIEWBOX, skip=('Chamchamal',))
new += borders(ZV, 1.2, clip=VIEWBOX)
for p, g in SUBS:
    nm = PART[p['en']]
    n = path_node('Part · ' + nm, g, ZV, {'fill': part_fills.get(nm, '$dry-4'), 'stroke': '#FFFFFF', 'strokeWidth': 2})
    new.append(n)
    if nm in labels:
        x, y = label_pt(g, ZV)
        lb = labels[nm]; lb['width'] = 150; lb['alignItems'] = 'center'; place(lb, x - 75, y - 17)
new.append(path_node('Zone Outline · Chamchamal', CHAM, ZV, {'fill': '#FFFFFF00', 'stroke': '$ink', 'strokeWidth': 3}))
ctx_labels = []
for p, g in DISTS:
    if p['en'] == 'Chamchamal':
        continue
    gg = g.intersection(ZV.lonlat_box(-40))
    if gg.is_empty or gg.area < 0.004:
        continue
    x, y = label_pt(gg, ZV)
    if (x < 200 and y > 540) or y < 70 or (x > 690 and y > 600):
        continue
    ctx_labels.append(text_node('Label · ' + p['en'], p['en'], x, y, 12, '600', '$ink-2', '$font-ui', width=130))
# mini map: all four governorates small, Chamchamal coloured, viewport box
mini = [o for o in overlays if o['name'] == 'Mini Map'][0]
MV = View(REGION.bounds, 160, 148, 8)
mini_kept = [c for c in mini['children'] if c.get('type') != 'path' and c.get('name') != 'Viewport']
vp = [c for c in mini['children'] if c.get('name') == 'Viewport'][0]
mini_new = districts(MV, lambda p: '$dry-5' if p['en'] == 'Chamchamal' else '#DDD9CB', 0.4)
a = MV.P(*ZV.inv(0, 0)); b = MV.P(*ZV.inv(760, 732))
vp['x'], vp['y'], vp['width'], vp['height'] = round(a[0], 2), round(a[1], 2), round(b[0] - a[0], 2), round(b[1] - a[1], 2)
mini['children'] = mini_new + mini_kept + [vp]
m['children'] = new + graticule(ZV, 10, 2, 10, avoid=[(0, 0, 330, 56), (590, 0, 760, 56), (0, 556, 190, 732), (690, 610, 760, 732)]) + ctx_labels + list(labels.values()) + overlays
m['clip'] = True
log.append(f'jmJ4D: zoom redrawn with {len(SUBS)} real sub-districts, {len(ctx_labels)} context labels')

# ---------- 02a storyboard thumbs ----------
thumbs = {'C6R9lK': 0.0, 'mEdNt': 0.55, 'PE9yv': 1.0, 'pQisI': 1.0}
T_OV = View(REGION.bounds, 325, 320, 6)
T_Z = View(CHAM.bounds, 325, 320, 8)
for tid, t in thumbs.items():
    m = by_id(tid)
    fills = old_fills(m)
    part_f = {c['name'][7:]: (c.get('fill'), c.get('opacity')) for c in m['children'] if c.get('name', '').startswith('Part · ')}
    cham_old = [c for c in m['children'] if c.get('name') == 'Zone · Chamchamal'][0]
    ctx_op = [c.get('opacity') for c in m['children'] if c.get('name') == 'Zone · Zakho'][0]
    overlays = [c for c in m['children'] if c.get('type') != 'path']
    v = T_OV if t == 0 else T_Z if t == 1 else View.blend(T_OV, T_Z, t)
    clip = v.lonlat_box(30)
    sc = 0.5 if t == 0 else 0.8
    new = districts(v, lambda p: carry(fills, p['en']), sc, opacity=ctx_op, clip=clip, skip=('Chamchamal',))
    new += borders(v, sc * 0.8, clip=clip)
    if part_f:
        for p, g in SUBS:
            nm = PART[p['en']]; f, op = part_f.get(nm, ('$dry-4', None))
            extra = {'fill': f, 'stroke': '#FFFFFF', 'strokeWidth': 1.5}
            if op is not None: extra['opacity'] = op
            new.append(path_node('Part · ' + nm, g, v, extra))
        new.append(path_node('Zone · Chamchamal', CHAM, v, {'fill': '#FFFFFF00', 'stroke': cham_old.get('stroke', '$ink'), 'strokeWidth': cham_old.get('strokeWidth', 2.5)}))
    else:
        new.append(path_node('Zone · Chamchamal', CHAM, v, {'fill': cham_old.get('fill'), 'stroke': cham_old.get('stroke', '$ink'), 'strokeWidth': cham_old.get('strokeWidth', 2.5)}))
    for o in overlays:
        if o.get('name') == 'Cursor':
            x, y = label_pt(CHAM, v); place(o, x - 4, y - 4)
    m['children'] = new + overlays
    log.append(f'{tid}: storyboard step redrawn (t={t})')

# ---------- legends, titles, counts ----------
def set_text(i, s):
    by_id(i)['content'] = s
GOVC = [('Duhok', 'دهۆک', '#2FB45A'), ('Erbil', 'هەولێر', '#F4B223'), ('Sulaymaniyah', 'سلێمانی', '#1FA2E0'), ('Halabja', 'هەڵەبجە', '#EC4C6B')]
# 01 legend: swatch rect ids and range text ids in order 1..5
for (rect, txt), (en, ku, col) in zip([('lYGGB', 'zmkm2'), ('OpZhv', 'G5ZPDq'), ('y2DWL', 'HRgXo'), ('b1cbH', 'VMw2y')], GOVC):
    by_id(rect)['fill'] = col; set_text(txt, en)
by_id('CPH9b')['enabled'] = False
set_text('ci3m7', 'Governorates'); by_id('CRG1e')['enabled'] = False
by_id('A6IGaj')['name'] = 'Governorate Key'
set_text('StBBN', 'Districts shaded inside each governorate · borders: Iraq CSO 2019, KRG grouping')
set_text('PhpGj', '4 governorates · 33 districts · click a district to see its water, dryness and advice')
set_text('Kh2j2', 'Districts'); by_id('YLFrp')['icon'] = 'map'
# 07 legend (right to left order: Swatch 5 .. 1)
for (rect, txt), (en, ku, col) in zip([('Hud3E', 'RFfQ8'), ('F7RiQG', 'l8XxbR'), ('rdOaD', 'MLMXb'), ('MI47p', 'TiAnH')], GOVC):
    by_id(rect)['fill'] = col; set_text(txt, ku)
by_id('mwS6d')['enabled'] = False
set_text('nBQy6', 'پارێزگاکان'); by_id('AJEOC')['enabled'] = False
by_id('Na04N')['name'] = 'Governorate Key'
set_text('u8vQr', 'قەزاکان بە ڕەنگی پارێزگاکەیان · سنوور: ئاماری عێراق ٢٠١٩، دابەشکردنی هەرێم')
set_text('W7CGDk', '٤ پارێزگا · ٣٣ قەزا · کلیک لە قەزایەک بکە بۆ بینینی ئاو، وشکی و ڕاوێژ')
set_text('uInTi', 'قەزاکان'); by_id('kpthO')['icon'] = 'map'
# counts and names that left the map
for i, s in {'AJuuW': 'of 33', 'S1CO8D': 'All 33 districts →', 'c4Ph3D': '3rd driest of 33', 'S0iXd': 'Top 8 of 33',
             'jhXrj': 'Taqtaq', 'j8peu': 'Qushtapa', 'X9dXM': 'Planned for all 33 districts', 'u85zC': 'Near Chuman, oak forest',
             'jYkEN': 'Taqtaq', 'QyS3c': 'Qushtapa', 'YPlHc': 'لە ٣٣', 'USCCt': '← هەموو ٣٣ قەزاکە',
             'spJKW': '3rd driest of the 33 districts', 'BWlDl': 'سێیەم وشکترین قەزا لە ٣٣ قەزا',
             'gxRR2': 'Each zone opens into its sub-districts (78 in total). Clicking a part shows that part\'s figures.'}.items():
    set_text(i, s)

json.dump(doc, open(DST, 'w', encoding='utf-8'), ensure_ascii=False, indent=2)
print('\n'.join(log))
print('ids used', len(used))
