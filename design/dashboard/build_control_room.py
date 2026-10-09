# Builds the Control Room screens of the Ministry dashboard into a .pen file.
# Usage: python build_control_room.py <in.pen> <out.pen>
# Safe to run again: it removes the screens and nav items it made before and makes them fresh.
# Sample numbers only. Rules come from farm_doctor/weather_planner.py and the backend (zones, farms, alwa, sign in).
import copy, json, math, random, string, sys

SRC, DST = sys.argv[1], sys.argv[2]
doc = json.load(open(SRC, encoding='utf-8'))
random.seed(11)

IDS = set()
def collect(n):
    IDS.add(n.get('id'))
    for c in n.get('children', []):
        collect(c)
for c in doc['children']:
    collect(c)

def nid():
    while True:
        s = ''.join(random.choice(string.ascii_letters + string.digits) for _ in range(6))
        if s not in IDS:
            IDS.add(s)
            return s

PREFIX = 'Control room · '
doc['children'] = [c for c in doc['children'] if PREFIX not in c.get('name', '')]
IDX = {}
def reg(n):
    IDX[n['id']] = n
    for c in n.get('children', []):
        reg(c)
for c in doc['children']:
    reg(c)

# ---------- building blocks ----------
def T(content, size=13, weight='normal', fill='$ink', font='$font-ui', name=None, **kw):
    n = {'type': 'text', 'id': nid(), 'name': name or 'Text', 'fill': fill, 'content': content,
         'fontFamily': font, 'fontSize': size, 'fontWeight': weight}
    if 'width' in kw and 'textGrowth' not in kw:
        n['textGrowth'] = 'fixed-width'
    n.update(kw)
    return n

def KU(content, size=13, weight='normal', fill='$ink', **kw):
    return T(content, size, weight, fill, '$font-ku', **kw)

def F(name, children=(), **kw):
    n = {'type': 'frame', 'id': nid(), 'name': name}
    n.update(kw)
    n['children'] = [c for c in children if c]
    return n

def V(name, children=(), **kw):
    return F(name, children, layout='vertical', **kw)

def I(icon, size=16, fill='$ink-2', name='Icon'):
    return {'type': 'icon', 'id': nid(), 'name': name, 'width': size, 'height': size, 'icon': icon, 'library': 'lucide', 'fill': fill}

def R(name, w, h, fill, **kw):
    n = {'type': 'rectangle', 'id': nid(), 'name': name, 'width': w, 'height': h, 'fill': fill}
    n.update(kw)
    return n

def eyebrow(s, fill='$ink-3'):
    return T(s.upper(), 11, '700', fill, name='Eyebrow', letterSpacing=0.8)

TONE = {'good': ('$good-soft', '$good'), 'warn': ('$warn-soft', '$warn'), 'danger': ('$danger-soft', '$danger'),
        'water': ('$water-soft', '$water'), 'brand': ('$brand-soft', '$brand'), 'none': ('$bg', '$ink-2'),
        'dark': ('$ink', '#FFFFFF')}
def pill(s, tone='none', icon=None, size=11):
    bg, fg = TONE[tone]
    return F('Pill · ' + s, [I(icon, size + 1, fg) if icon else None, T(s, size, '600', fg, name='Label')],
             fill=bg, cornerRadius=999, gap=4, padding=[3, 8], alignItems='center')

def button(s, kind='primary', icon=None, width=None, h=40):
    fill, fg, stroke = {'primary': ('$brand', '#FFFFFF', None), 'secondary': ('$surface', '$ink-2', '$line'),
                        'danger': ('$surface', '$danger', '$danger'), 'dark': ('$ink', '#FFFFFF', None),
                        'off': ('$line-soft', '$ink-3', None)}[kind]
    n = F('Button · ' + s, [I(icon, 15, fg) if icon else None, T(s, 13, '600', fg, name='Label')],
          height=h, fill=fill, cornerRadius=10, gap=7, padding=[0, 14], justifyContent='center', alignItems='center')
    if stroke:
        n['stroke'] = stroke; n['strokeWidth'] = 1
    if width:
        n['width'] = width
    return n

def card(name, children, **kw):
    base = dict(fill='$surface', cornerRadius=14, stroke='$line', strokeWidth=1, padding=16, gap=12)
    base.update(kw)
    return V(name, children, **base)

def dropdown(label, value, width=None, icon='chevron-down'):
    n = F('Filter · ' + label, [T(label, 12, 'normal', '$ink-3', name='Key'), T(value, 12, '600', '$ink', name='Value'),
                               I(icon, 13, '$ink-3')], height=34, fill='$surface', stroke='$line', strokeWidth=1,
          cornerRadius=9, gap=6, padding=[0, 10], alignItems='center')
    if width:
        n['width'] = width
    return n

def search(text, width='fill_container'):
    return F('Search', [I('search', 15, '$ink-3'), T(text, 13, 'normal', '$ink-3', name='Placeholder')], width=width, height=34,
             fill='$surface', stroke='$line', strokeWidth=1, cornerRadius=9, gap=8, padding=[0, 12], alignItems='center')

def stat(label, value, note, tone=None, icon=None):
    return card('Stat · ' + label, [
        F('Top', [I(icon, 15, '$ink-3') if icon else None, T(label, 12, '600', '$ink-2', name='Label')], gap=6, alignItems='center'),
        T(value, 26, '700', TONE[tone][1] if tone else '$ink', '$font-display', name='Value', lineHeight=1),
        T(note, 12, 'normal', '$ink-3', name='Note')], width='fill_container', gap=6, padding=[14, 16])

def table(name, cols, rows, row_h=None, zebra=False, highlight=None, header=True, font=12.5):
    """cols: [(title, width, align)]; rows: list of lists of str or node."""
    out = []
    if header:
        out.append(F('Header Row', [T(t, 11, '600', '$ink-3', name=t or 'Col', width=w, textAlign=a) for t, w, a in cols],
                     width='fill_container', stroke='$line', strokeWidth={'bottom': 1}, gap=10, padding=[8, 12]))
    for i, r in enumerate(rows):
        cells = []
        for (t, w, a), v in zip(cols, r):
            if isinstance(v, dict):
                cells.append(F('Cell', [v], width=w, justifyContent={'right': 'end', 'center': 'center'}.get(a, 'start'), alignItems='center'))
            else:
                cells.append(T(str(v), font, '600' if t == cols[0][0] else 'normal', '$ink', name=t or 'Col', width=w, textAlign=a))
        kw = dict(width='fill_container', stroke='$line-soft', strokeWidth={'bottom': 1}, gap=10, padding=[0, 12], alignItems='center')
        if row_h:
            kw['height'] = row_h
        else:
            kw['padding'] = [9, 12]
        if highlight is not None and i == highlight:
            kw['fill'] = '$brand-soft'; kw['stroke'] = '$brand'; kw['strokeWidth'] = {'left': 3, 'bottom': 1}
        elif zebra and i % 2:
            kw['fill'] = '#FBFAF6'
        out.append(F('Row %d' % (i + 1), cells, **kw))
    return V(name, out, width='fill_container')

def head(eb, title, sub, right=()):
    return F('Head', [V('Title Block', [eyebrow(eb), T(title, 28, '700', '$ink', '$font-display', name='Title'),
                                        T(sub, 13, 'normal', '$ink-2', name='Subtitle')], gap=4, width='fill_container'),
                      F('Actions', list(right), gap=8, alignItems='center')],
             width='fill_container', gap=16, alignItems='end')

def avatar(initials, fill='$brand', size=32):
    return F('Avatar', [T(initials, 12 if size >= 30 else 10, '700', '#FFFFFF', name='Initials')], width=size, height=size,
             fill=fill, cornerRadius=999, justifyContent='center', alignItems='center')

def divider():
    return R('Divider', 'fill_container', 1, '$line')

def kv(k, v, vfill='$ink', vweight='600', font='$font-ui'):
    return F('Fact · ' + k, [T(k, 12, 'normal', '$ink-3', name='Key', width='fill_container'),
                             T(v, 12, vweight, vfill, font, name='Value')], width='fill_container', alignItems='center')

CROP = {'wheat': '#E0B13A', 'barley': '#C8B560', 'tomato': '#D9483B', 'cucumber': '#6DB352', 'potato': '#A9784A',
        'onion': '#B46FA8', 'watermelon': '#EF7C8E', 'grape': '#7E57C2', 'olive': '#7D8B3A', 'sunflower': '#F5C518',
        'chickpea': '#D9B88A', 'empty': '#E4E1D6'}
def crop_chip(c):
    return F('Crop · ' + c, [R('Dot', 9, 9, CROP[c], cornerRadius=3), T(c.capitalize(), 12.5, 'normal', '$ink', name='Crop')],
             gap=6, alignItems='center')

LEVEL = {'normal': ('good', 'Normal'), 'watch': ('warn', 'Watch'), 'alarm': ('danger', 'Alarm'), 'none': ('none', 'No picture yet')}
def level_pill(l):
    tone, s = LEVEL[l]
    return pill(s, tone)

# ---------- top bar and the Control Room menu ----------
TOPBAR = next(n for n in IDX.values() if n.get('name') == 'Top Bar' and any(c.get('name') == 'Nav' for c in n.get('children', [])))

def reid(n):
    n = copy.deepcopy(n)
    def walk(x):
        x['id'] = nid()
        for c in x.get('children', []):
            walk(c)
    walk(n)
    return n

def nav_item(active):
    fg = '$brand' if active else '$ink-2'
    return F('Nav · Control room', [I('shield-check', 16, fg), T('Control room', 13, '600' if active else '500', fg, name='Label')],
             fill='$brand-soft' if active else '#FFFFFF00', cornerRadius=8, gap=8, padding=[8, 9], alignItems='center')

# add the (inactive) entry to every existing dashboard top bar, once, and tighten the bar so it fits
for n in list(IDX.values()):
    if n.get('name') == 'Nav' and n['type'] == 'frame' and len(n.get('children', [])) >= 6:
        n['children'] = [c for c in n['children'] if c.get('name') != 'Nav · Control room'] + [nav_item(False)]
        for it in n['children']:
            it['padding'] = [8, 9]
    if n.get('name') == 'Top Bar' and any(c.get('name') == 'Nav' for c in n.get('children', [])):
        n['gap'] = 12

def topbar():
    t = reid(TOPBAR)
    nav = next(c for c in t['children'] if c.get('name') == 'Nav')
    for it in nav['children']:
        it['fill'] = '#FFFFFF00'
        for c in it['children']:
            c['fill'] = '$ink-2'
            if c['type'] == 'text':
                c['fontWeight'] = '500'
    nav['children'] = [c for c in nav['children'] if c.get('name') != 'Nav · Control room'] + [nav_item(True)]
    for c in t['children']:
        if c.get('name') == 'Updated':
            c['children'][-1]['content'] = '8 Oct 10:46'
    return t

MENU = [('users', 'Farmers and farms', '2,031'), ('wheat', 'Crop register', '18,420 du'), ('bell-ring', 'Send an alert', '1 draft'),
        ('inbox', 'Inbox', '9 new'), ('sliders-horizontal', 'Rules', '1 waiting'), ('store', 'Alwa control', '3 flags'),
        ('activity', 'Data health', '1 issue'), ('history', 'Officers and history', '')]

def menu(active):
    items = []
    for icon, label, badge in MENU:
        on = label == active
        fg = '$brand' if on else '$ink-2'
        warn = badge in ('9 new', '3 flags', '1 issue', '1 waiting')
        items.append(F('Menu · ' + label, [
            I(icon, 16, fg), T(label, 13, '600' if on else '500', '$ink' if on else '$ink-2', name='Label', width='fill_container'),
            (pill(badge, 'danger' if badge in ('1 issue',) else 'warn' if warn else 'none', size=10) if badge else None)],
            width='fill_container', height=38, fill='$brand-soft' if on else '#FFFFFF00', cornerRadius=9, gap=10,
            padding=[0, 10], alignItems='center'))
    officer = F('Officer', [avatar('SA'), V('Who', [T('Shilan Ahmed', 13, '600', '$ink', name='Name'),
                                                    T('District officer · Sulaymaniyah', 11, 'normal', '$ink-3', name='Role')], gap=1, width='fill_container'),
                            I('log-out', 15, '$ink-3')], width='fill_container', gap=10, alignItems='center')
    protected = V('Protected Mode', [
        F('Top', [I('lock', 14, '$good'), T('Protected mode on', 12, '700', '$good', name='Label')], gap=6, alignItems='center'),
        T('Phones and exact fields stay hidden until a farmer asks an officer for help. Every look is written to the history.',
          11, 'normal', '$ink-2', name='Note', width='fill_container', lineHeight=1.4)],
        width='fill_container', fill='$good-soft', cornerRadius=10, padding=12, gap=6)
    return V('Control Room Menu', [
        V('Menu Head', [eyebrow('Control room'), T('Ministry of Agriculture and Water Resources · KRG', 11, 'normal', '$ink-3',
                                                  name='Org', width='fill_container', lineHeight=1.35)], gap=4, width='fill_container', padding=[0, 6, 8, 6]),
        V('Items', items, gap=2, width='fill_container'),
        F('Spacer', [], width='fill_container', height='fill_container'),
        protected, divider(), officer],
        width=244, height='fill_container', fill='$surface', stroke='$line', strokeWidth={'right': 1}, padding=[20, 14], gap=10)

SCREENS = []
def screen(num, title, content):
    body = F('Body', [menu(title), content], width='fill_container', height='fill_container')
    f = V(f'{num} · {PREFIX}{title}', [topbar(), body], width=1440, height=1024, fill='$bg', clip=True)
    SCREENS.append(f)
    return f

def main(children, **kw):
    base = dict(width='fill_container', height='fill_container', padding=[22, 28, 22, 28], gap=16)
    base.update(kw)
    return V('Main', children, **base)

# ---------- the map component ----------
MASTER = next(c for c in doc['children'] if c.get('name') == 'Component · KRI Map')
MID = {c['name']: c['id'] for c in MASTER['children']}
DISTRICTS = [c['name'][len('District · '):] for c in MASTER['children'] if c['name'].startswith('District · ')]
GLOW = [{'type': 'shadow', 'shadowType': 'outer', 'offset': {'x': 0, 'y': 0}, 'blur': 5, 'color': '#FFFFFFFF'},
        {'type': 'shadow', 'shadowType': 'outer', 'offset': {'x': 0, 'y': 0}, 'blur': 2, 'color': '#FFFFFFFF'}]
def dark(hexc):
    h = hexc.lstrip('#')[:6]
    r, g, b = (int(h[i:i + 2], 16) for i in (0, 2, 4))
    return 0.299 * r + 0.587 * g + 0.114 * b < 140

def map_instance(colmap, hide_city=True, label_for=None):
    over = {}
    for en in DISTRICTS:
        col = colmap.get(en, '#ECE9DF')
        over[MID['District · ' + en]] = {'fill': col}
        lab = MID.get('Label · ' + en)
        if lab:
            o = {'fill': '#FFFFFF', 'effect': []} if dark(col) else {'fill': '#22302A', 'effect': GLOW}
            if label_for and en in label_for:
                o['content'] = label_for[en]
            over[lab] = o
    if hide_city:
        for k, v in MID.items():
            if k.startswith('City label · '):
                over[v] = {'enabled': False}
    return {'type': 'ref', 'id': nid(), 'ref': MASTER['id'], 'name': 'KRI Map', 'x': 0, 'y': 0, 'descendants': over}

def legend(title, items):
    return F('Legend', [T(title, 12, '600', '$ink-2', name='Title')] + [
        V('Swatch', [R('Colour', 64, 10, c, cornerRadius=2), T(s, 10, 'normal', '$ink-3', name='Range')], gap=4, alignItems='center')
        for c, s in items], gap=8, alignItems='end')

# =====================================================================================
# 11 Farmers and farms
# =====================================================================================
FARMS = [
    ('1207', 'کێڵگەی سەرەوە', '+964 750 ••• 4567', 'Aghjalar', 11.3, 'wheat', 'watch', '8 Oct 09:12', 'open-r'),
    ('1188', 'زەوی باوکم', '+964 770 ••• 2210', 'Chamchamal centre', 22.4, 'wheat', 'normal', '8 Oct 07:40', '1km'),
    ('1176', 'کێڵگەی تەماتە', '+964 751 ••• 0938', 'Sangaw', 3.1, 'tomato', 'alarm', '7 Oct 18:05', 'open-c'),
    ('1163', 'کێڵگەی جۆ', '+964 750 ••• 7741', 'Qadir Karam', 41.0, 'barley', 'normal', '7 Oct 16:22', '1km'),
    ('1150', 'زەوی ڕووبار', '+964 773 ••• 3306', 'Sangaw', 12.7, 'wheat', 'none', '7 Oct 11:03', '1km'),
    ('1139', 'کێڵگەی نۆک', '+964 750 ••• 5512', 'Aghjalar', 6.2, 'chickpea', 'normal', '6 Oct 20:48', '1km'),
    ('1122', 'خەیار', '+964 771 ••• 8820', 'Chamchamal centre', 0.8, 'cucumber', 'watch', '6 Oct 14:10', '1km'),
    ('1117', 'کێڵگەی خوارەوە', '+964 750 ••• 4567', 'Sangaw', 18.9, 'wheat', 'normal', '5 Oct 09:31', '1km'),
    ('1098', 'زەوی گوند', '+964 770 ••• 6094', 'Qadir Karam', 27.5, 'barley', 'normal', '4 Oct 17:55', '1km'),
    ('1086', 'باخی ترێ', '+964 750 ••• 1450', 'Aghjalar', 2.4, 'grape', 'normal', '3 Oct 12:20', '1km'),
    ('1071', 'کێڵگەی نوێ', '+964 772 ••• 9013', 'Chamchamal centre', 15.3, 'wheat', 'watch', '1 Oct 08:02', '1km'),
    ('1064', 'کێڵگەی پەتاتە', '+964 750 ••• 3187', 'Sangaw', 4.6, 'potato', 'none', '28 Sep 15:44', 'blocked'),
]
def access(a):
    return {'open-r': pill('Open', 'brand', 'lock-open'), 'open-c': pill('Open', 'brand', 'lock-open'),
            '1km': pill('1 km', 'none', 'lock'), 'blocked': pill('Blocked', 'danger', 'ban')}[a]

cols = [('Farm and farmer', 'fill_container', 'left'), ('Sub-district', 118, 'left'), ('Area', 56, 'right'),
        ('Main crop', 88, 'left'), ('Status', 98, 'left'), ('Last sync', 78, 'left'), ('Access', 92, 'left')]
rows = []
for fid, name, phone, sub, area, crop, lvl, sync, acc in FARMS[:10]:
    rows.append([V('Farm', [KU(name, 13, '600', name='Name'), T(f'#{fid} · {phone}', 11, 'normal', '$ink-3', name='Id and phone')], gap=1),
                 sub, f'{area:.1f} du', crop_chip(crop), level_pill(lvl), sync, access(acc)])
farm_table = card('Farms Table', [
    F('Table Head', [T('Chamchamal district', 14, '700', name='Title'), T('214 farms · 1,830 dunam · 163 farmers', 12, 'normal', '$ink-3', name='Count'),
                     F('Spacer', [], width='fill_container', height=1), T('Sorted by last sync', 12, 'normal', '$ink-3', name='Sort'), I('arrow-down-up', 14, '$ink-3')],
      width='fill_container', gap=10, alignItems='center', padding=[0, 4]),
    table('Table', cols, rows, row_h=50, highlight=0),
    F('Pager', [T('Showing 1 to 10 of 214 · Open = the farmer asked an officer for help', 12, 'normal', '$ink-3', name='Range'), F('Spacer', [], width='fill_container', height=1),
                button('Previous', 'secondary', 'chevron-left', h=32), button('Next', 'secondary', 'chevron-right', h=32)],
      width='fill_container', gap=8, alignItems='center', padding=[4, 4, 0, 4])],
    width='fill_container', height='fill_container', padding=[14, 12], gap=10)

# the selected farm, drawn from its walked corners on the 10 m grid
OUT = [(0, 3), (14, 0), (21, 2), (23, 11), (17, 16), (4, 15), (1, 9)]   # corners in cells (10 m)
def inside(px, py, poly):
    c = False
    for i in range(len(poly)):
        (x1, y1), (x2, y2) = poly[i], poly[(i + 1) % len(poly)]
        if (y1 > py) != (y2 > py) and px < (x2 - x1) * (py - y1) / (y2 - y1) + x1:
            c = not c
    return c
S = 11
cells, n_cells, COUNT, WEAK = [], 0, {}, 0
for gy in range(0, 17):
    for gx in range(0, 24):
        if not inside(gx + 0.5, gy + 0.5, OUT):
            continue
        n_cells += 1
        crop = 'tomato' if gx >= 18 else 'empty' if (gx <= 4 and gy >= 12) else 'wheat'
        weak = crop == 'wheat' and gx >= 12 and gy <= 6 and (gx + gy) % 3 != 0
        COUNT[crop] = COUNT.get(crop, 0) + 1; WEAK += weak
        cells.append(R(f'Cell {gx},{gy}', S - 1, S - 1, CROP[crop], x=gx * S + 0.5, y=gy * S + 0.5, cornerRadius=1.5,
                       **({'stroke': '#9A6410', 'strokeWidth': 1.6} if weak else {})))
AREA_M2 = abs(sum(x1 * y2 - x2 * y1 for (x1, y1), (x2, y2) in zip(OUT, OUT[1:] + OUT[:1]))) / 2 * 100
du = lambda m2: f'{m2 / 2500:.1f}'
FACT_AREA = f'{du(AREA_M2)} dunam · {AREA_M2:,.0f} m²'
FACT_CROPS = ' · '.join(f'{c.capitalize()} {du(n * 100)}' for c, n in sorted(COUNT.items(), key=lambda x: -x[1])) + ' du'
FACT_SQ = f'{n_cells} · {WEAK} weak ({100 * WEAK / n_cells:.0f}%)'
d = 'M' + 'L'.join(f'{x * S} {y * S}' for x, y in OUT) + 'Z'
xs = [x * S for x, _ in OUT]; ys = [y * S for _, y in OUT]
outline = {'type': 'path', 'id': nid(), 'name': 'Walked outline', 'x': 0, 'y': 0, 'width': max(xs), 'height': max(ys),
           'geometry': d, 'viewBox': [0, 0, max(xs), max(ys)], 'strokeLinejoin': 'round', 'stroke': '#14201A', 'strokeWidth': 2}
dots = [{'type': 'ellipse', 'id': nid(), 'name': f'Corner {i + 1}', 'x': x * S - 4, 'y': y * S - 4, 'width': 8, 'height': 8,
         'fill': '#FFFFFF', 'stroke': '#14201A', 'strokeWidth': 2} for i, (x, y) in enumerate(OUT)]
drawing = F('Farm Drawing', [F('Grid', cells + [outline] + dots, layout='none', width=24 * S, height=17 * S, x=24, y=14)],
            layout='none', width='fill_container', height=206, fill='#EEF0EA', cornerRadius=10, clip=True)

detail = card('Selected Farm', [
    F('Title', [V('Name', [T('Farm #1207 · Aghjalar', 12, '600', '$ink-3', name='Id'), KU('کێڵگەی سەرەوە', 20, '700', name='Name')], gap=2, width='fill_container'),
                level_pill('watch')], width='fill_container', alignItems='center'),
    F('Access', [I('lock-open', 14, '$brand'), T('Open because the farmer sent report r_3391 (yellow stripes) on 8 Oct. Closes again when the report is closed.',
                                                 11.5, 'normal', '$brand', name='Note', width='fill_container', lineHeight=1.4)],
      width='fill_container', fill='$brand-soft', cornerRadius=9, padding=10, gap=8),
    drawing,
    F('Key', [crop_chip('wheat'), crop_chip('tomato'), crop_chip('empty'),
              F('Weak', [R('Box', 9, 9, '#FFFFFF00', stroke='#9A6410', strokeWidth=1.6, cornerRadius=2), T('Weak square', 12.5, 'normal', name='Label')], gap=6, alignItems='center')],
      gap=12, alignItems='center'),
    V('Facts', [kv('Area inside the outline', FACT_AREA), kv('Squares (10 m)', FACT_SQ),
                kv('Crops', FACT_CROPS), kv('Corners walked', '7 · GPS within 4 to 9 m'),
                kv('Made offline, sent', '6 Oct 14:10 → 18:02'),
                kv('Farmer', '+964 750 ••• 4567 · 2 farms · Sorani')], gap=7, width='fill_container'),
    F('Buttons', [button('Show phone', 'secondary', 'eye', width='fill_container', h=34), button('Open on map', 'secondary', 'map', width='fill_container', h=34)],
      gap=8, width='fill_container'),
    F('Buttons 2', [button('Block', 'danger', 'ban', width='fill_container', h=34),
                    button('Delete on request', 'danger', 'trash-2', width='fill_container', h=34)], gap=8, width='fill_container'),
    T('Officers cannot change a farm outline or its crops: only the farmer can. Showing the phone asks for a reason and is logged.',
      11, 'normal', '$ink-3', name='Rule', width='fill_container', lineHeight=1.4)],
    width=356, height='fill_container', gap=9, padding=[14, 16])

screen('11', 'Farmers and farms', main([
    head('Control room · farmers and farms', '1,284 farmers · 2,031 farms',
         'Everything the app knows about each farm. Phones show only the last 4 digits; fields show at 1 km until the farmer asks for help.',
         [button('Export CSV', 'secondary', 'download')]),
    F('Stats', [stat('Farmers', '1,284', '+37 this week · 61% use Sorani', icon='users'),
                stat('Farms', '2,031', '18,420 dunam · 1.6 farms per farmer', icon='map'),
                stat('Made offline', '412', '20% saved in the field, sent later', icon='wifi-off'),
                stat('Waiting for a picture', '186', 'no satellite reading yet', 'warn', 'satellite')], gap=12, width='fill_container'),
    F('Filters', [search('Farm name, farm id or last 4 digits of the phone', 330), dropdown('Governorate', 'Sulaymaniyah'),
                  dropdown('District', 'Chamchamal'), dropdown('Crop', 'All'), dropdown('Status', 'All'), dropdown('Last sync', 'Any time')],
      gap=8, width='fill_container', alignItems='center'),
    F('Content', [farm_table, detail], gap=14, width='fill_container', height='fill_container')]))

# =====================================================================================
# 12 Crop register
# =====================================================================================
REG = {'Makhmur': 2140, 'Erbil': 1960, 'Chamchamal': 1830, 'Sharazur': 1420, 'Kalar': 1310, 'Qushtapa': 1180, 'Sumel': 900,
       'Shekhan': 750, 'Koya': 760, 'Ranya': 690, 'Halabja': 610, 'Dukan': 540, 'Sulaymaniyah': 520, 'Akre': 480,
       'Darbandikhan': 430, 'Duhok': 380, 'Zakho': 360, 'Bardarash': 330, 'Taqtaq': 300, 'Pshdar': 260, 'Shaqlawa': 240,
       'Khurmal': 210, 'Penjwen': 150, 'Sharbazher': 140, 'Amedi': 120, 'Soran': 110, 'Harir': 90, 'Khalifan': 60,
       'Rawanduz': 50, 'Pirmam': 40, 'Chuman': 30, 'Mergasor': 20, 'Sidakan': 10}
REG['Makhmur'] += 18420 - sum(REG.values())
assert set(REG) == set(DISTRICTS), set(DISTRICTS) ^ set(REG)
SHADES = [(100, '#F3EFD9'), (300, '#E6DA9C'), (700, '#D6BE5A'), (1500, '#B8932A'), (10 ** 9, '#7E5F12')]
shade = lambda v: next(c for lim, c in SHADES if v < lim)
crop_map = F('Map', [map_instance({k: shade(v) for k, v in REG.items()})], layout='none', width=760, height=732, clip=True)
TOT = [('wheat', 9860), ('barley', 3410), ('tomato', 1240), ('potato', 980), ('cucumber', 610), ('onion', 520),
       ('watermelon', 430), ('chickpea', 380), ('grape', 290), ('olive', 260), ('sunflower', 150), ('empty', 290)]
assert sum(v for _, v in TOT) == 18420
bars = []
for c, v in TOT:
    bars.append(F('Bar · ' + c, [F('Name', [crop_chip(c)], width=104),
                                 F('Track', [R('Fill', round(144 * v / 9860, 1) or 1, 10, CROP[c], cornerRadius=3)], width=146, height=10),
                                 T(f'{v:,}', 12.5, '600', name='Dunam', width=54, textAlign='right'),
                                 T(f'{v / 184.2:.1f}%', 12, 'normal', '$ink-3', name='Share', width=44, textAlign='right')],
                  width='fill_container', gap=8, alignItems='center'))
TOPD = [('Makhmur', 2140, 'wheat 71%', '+120'), ('Erbil', 1960, 'wheat 64%', '+85'), ('Chamchamal', 1830, 'wheat 58%', '+44'),
        ('Sharazur', 1420, 'wheat 49%', '+61'), ('Kalar', 1310, 'barley 38%', '+12'), ('Qushtapa', 1180, 'wheat 66%', '+30')]
TOPD[0] = ('Makhmur', REG['Makhmur'], 'wheat 71%', '+120')
dtab = table('By District', [('District', 'fill_container', 'left'), ('Dunam', 60, 'right'), ('Main crop', 88, 'left'), ('Week', 44, 'right')],
             [[a, f'{b:,}', c, d] for a, b, c, d in TOPD], row_h=32, font=12)
crop_panel = card('Crop Panel', [
    F('Title', [V('T', [eyebrow('All registered farms · 8 Oct'), T('18,420 dunam', 30, '700', '$ink', '$font-display', name='Value', lineHeight=1)], gap=6, width='fill_container'),
                pill('+352 du this week', 'good', 'trending-up')], width='fill_container', alignItems='end'),
    V('Bars', bars, gap=7, width='fill_container'),
    divider(), F('DH', [eyebrow('Top districts'), F('S', [], width='fill_container', height=1), T('of 33', 12, 'normal', '$ink-3', name='Count')], width='fill_container'),
    dtab,
    F('Honesty', [I('info', 14, '$warn'), T('Only farms registered in the app, as painted by farmers. Not a census: use it for trends and where to look.',
                                             11.5, 'normal', '$warn', name='Note', width='fill_container', lineHeight=1.4)],
      width='fill_container', fill='$warn-soft', cornerRadius=9, padding=10, gap=8)],
    width=420, height='fill_container', gap=10, padding=[16, 18])
screen('12', 'Crop register', F('Main', [
    V('Map Area', [
        head('Control room · crop register', 'What is planted, where', 'Painted 10 m squares added up per district · darker = more registered dunam',
             [F('Switch', [F('Opt', [T('Dunam', 12, '600', '#FFFFFF', name='Label')], fill='$ink', cornerRadius=7, padding=[7, 12]),
                           F('Opt', [T('Farms', 12, '600', '$ink-2', name='Label')], cornerRadius=7, padding=[7, 12]),
                           F('Opt', [T('Crop', 12, '600', '$ink-2', name='Label'), I('chevron-down', 12, '$ink-3')], cornerRadius=7, padding=[7, 12], gap=4, alignItems='center')],
                      fill='$surface', cornerRadius=10, stroke='$line', strokeWidth=1, gap=2, padding=3)]),
        F('Map Row', [crop_map], width='fill_container', height='fill_container', justifyContent='center', alignItems='center'),
        legend('Registered dunam', [(c, s) for (_, c), s in zip(SHADES, ['under 100', '100–300', '300–700', '700–1,500', '1,500+'])])],
      width='fill_container', height='fill_container', padding=[22, 0, 18, 28], gap=10),
    V('Side', [crop_panel], height='fill_container', padding=[22, 28, 22, 0])], width='fill_container', height='fill_container', gap=16))

# =====================================================================================
# 13 Send an alert
# =====================================================================================
TARGET = ['Penjwen', 'Sharbazher', 'Pshdar']
amap = {d: '#E7E4DA' for d in DISTRICTS}
amap.update({d: '#C2452A' for d in TARGET})
alert_map = F('Map', [map_instance(amap)], layout='none', width=760, height=732, clip=True)

def step(num, title, children, done=True):
    return V('Step ' + num, [F('Head', [F('Num', [T(num, 11, '700', '#FFFFFF' if done else '$ink-2', name='N')], width=20, height=20,
                                         fill='$brand' if done else '$line-soft', cornerRadius=999, justifyContent='center', alignItems='center'),
                                       T(title, 13, '700', name='Title')], gap=8, alignItems='center')] + children,
             width='fill_container', gap=8)

TYPES = [('frost', 'snowflake', True), ('heat', 'thermometer-sun', False), ('heavy_rain', 'cloud-rain', False), ('dust', 'wind', False),
         ('more', 'chevron-down', False)]
type_row = F('Types', [F('Type · ' + t, [I(ic, 13, '#FFFFFF' if on else '$ink-2'), T(t.replace('_', ' '), 12, '600', '#FFFFFF' if on else '$ink-2', name='Label')],
                         fill='$ink' if on else '$surface', stroke='$line', strokeWidth=0 if on else 1, cornerRadius=8, padding=[6, 9], gap=5, alignItems='center')
                       for t, ic, on in TYPES], gap=5, width='fill_container')
ku_msg = 'سەرمای توند: شەوی یەکشەممە پلەی گەرما دادەبەزێت بۆ ٣- پلە. شەممە تەماتە و خەیاری پێگەیشتوو بچنەوە و نەمامەکان دابپۆشە.'
en_msg = 'Hard frost: Sunday night down to -3 °C. On Saturday pick ripe tomatoes and cucumbers and cover young plants.'
def msg_box(lang, text, count, ku=False):
    return V('Message · ' + lang, [F('Top', [T(lang, 11, '700', '$ink-3', name='Lang', letterSpacing=0.6), F('S', [], width='fill_container', height=1),
                                             T(count, 11, 'normal', '$ink-3', name='Count')], width='fill_container'),
                                   (KU(text, 13.5, 'normal', width='fill_container', textAlign='right', lineHeight=1.6) if ku else
                                    T(text, 13, 'normal', width='fill_container', lineHeight=1.45))],
             width='fill_container', fill='$bg', cornerRadius=9, padding=[9, 11], gap=4, stroke='$line', strokeWidth=1)
def push_preview():
    notif = V('Push', [F('App', [F('Icon', [I('sun', 11, '#FFFFFF')], width=18, height=18, fill='#1E7A5A', cornerRadius=5, justifyContent='center', alignItems='center'),
                                 T('JUTYAR', 10, '700', '$ink-2', name='App', letterSpacing=0.5), F('S', [], width='fill_container', height=1), T('now', 10, 'normal', '$ink-3', name='When')],
                         width='fill_container', gap=6, alignItems='center'),
                       KU('سەرمای توند · یەکشەممە', 12.5, '700', width='fill_container', textAlign='right'),
                       KU('شەممە تەماتە و خەیاری پێگەیشتوو بچنەوە', 11.5, 'normal', '$ink-2', width='fill_container', textAlign='right')],
               width='fill_container', fill='#FFFFFFEE', cornerRadius=12, padding=[8, 10], gap=3)
    return V('Push Preview', [T('ON THE FARMER’S LOCK SCREEN', 10, '700', '#FFFFFFB3', name='Label', letterSpacing=0.6), notif],
             width='fill_container', fill='#2C4A3C', cornerRadius=12, padding=[8, 8], gap=6)
PUSH = push_preview()

composer = card('Composer', [
    F('Title', [V('T', [eyebrow('New alert · draft'), T('Frost, Sunday night', 20, '700', '$ink', '$font-display', name='Title')], gap=4, width='fill_container'),
                pill('Alarm · pushed', 'danger', 'bell-ring')], width='fill_container', alignItems='end'),
    step('1', 'Where', [F('Chips', [pill(d, 'danger', 'map-pin') for d in TARGET] + [pill('+ add', 'none')], gap=5),
                        T('Drawn on the map or picked by district / sub-district', 11.5, 'normal', '$ink-3', name='Hint')]),
    step('2', 'What', [type_row, F('Facts', [pill('Day: Sun 11 Oct', 'none', 'calendar'), pill('-3 °C', 'none', 'thermometer'),
                                            pill('How sure: likely', 'warn', 'gauge')], gap=5)]),
    step('3', 'Message', [msg_box('سۆرانی', ku_msg, '118 / 160', ku=True), msg_box('ENGLISH', en_msg, '104 / 160'),
                          F('Check', [I('triangle-alert', 13, '$warn'), T('Sorani text not checked by a native speaker yet', 11.5, '600', '$warn', name='Label')], gap=6, alignItems='center'),
                          PUSH]),
    step('4', 'Who gets it', [V('Reach', [kv('Farms in the area', '318 farms · 241 farmers'), kv('Grow tomato or cucumber', '96 farms'),
                                          kv('Already pushed today', '52 → sent Fri 06:00 instead', '$warn'),
                                          kv('No phone for push yet', '27 → see it in the app only', '$ink-3')], gap=6, width='fill_container')]),
    F('Approve', [avatar('KA', '$water', 26), T('Needs a second officer before it goes out: Karwan Aziz (admin) will be asked.', 11.5, 'normal', '$ink-2', name='Note', width='fill_container', lineHeight=1.35)],
      width='fill_container', fill='$water-soft', cornerRadius=9, padding=10, gap=8, alignItems='center'),
    F('Buttons', [button('Send for approval', 'primary', 'send', width='fill_container'), button('Save draft', 'secondary')], gap=8, width='fill_container')],
    width='fill_container', height='fill_container', gap=12, padding=[16, 18])

screen('13', 'Send an alert', F('Main', [
    V('Map Area', [
        head('Control room · send an alert', 'Send an alert', 'Red = the area that gets it · farms outside get nothing',
             [pill('Max 1 push per farm per day · Alarm only', 'none', 'shield')]),
        F('Map Row', [alert_map], width='fill_container', height='fill_container', justifyContent='center', alignItems='center')],
      width='fill_container', height='fill_container', padding=[22, 0, 18, 28], gap=10),
    V('Side', [composer], width=404, height='fill_container', padding=[22, 28, 22, 0])],
    width='fill_container', height='fill_container', gap=16))

# =====================================================================================
# 14 Inbox
# =====================================================================================
INBOX = [('report', 'Yellow stripes on wheat leaves', 'Aghjalar · farm #1207', '10:31', 'new', True),
         ('case', 'Doctor unsure: rot on tomato stems', 'Sangaw · farm #1176', '09:58', 'new', False),
         ('report', 'Insects on young barley', 'Qadir Karam · farm #1163', '08:12', 'new', False),
         ('report', 'Hail broke tomato plants', 'Sharazur · farm #0884', 'Wed', 'Assigned · Shilan', False),
         ('case', 'Doctor unsure: grape leaves curling', 'Halabja · farm #0731', 'Wed', 'Assigned · Dlovan', False),
         ('report', 'Fire near the field edge', 'Kalar · farm #0952', 'Tue', 'Seen', False),
         ('report', 'Flood after the storm', 'Zakho · farm #0412', 'Mon', 'Closed', False),
         ('report', 'Sheep sick in the village', 'Penjwen · farm #0377', 'Sun', 'Sent to vets', False)]
def inbox_item(kind, title, where, when, st, sel):
    tone = 'danger' if st == 'new' else 'water' if st.startswith('Assigned') else 'none'
    return F('Item · ' + title, [
        F('Kind', [I('message-square-warning' if kind == 'report' else 'stethoscope', 15, '$danger' if kind == 'report' else '$water')],
          width=30, height=30, fill='$danger-soft' if kind == 'report' else '$water-soft', cornerRadius=8, justifyContent='center', alignItems='center'),
        V('Text', [T(title, 13, '600', name='Title', width='fill_container'), T(where, 11.5, 'normal', '$ink-3', name='Where')], gap=2, width='fill_container'),
        V('Side', [T(when, 11, 'normal', '$ink-3', name='When'), pill('New' if st == 'new' else st, tone, size=10)], gap=4, alignItems='end')],
        width='fill_container', padding=[11, 12], gap=10, fill='$brand-soft' if sel else '#FFFFFF00', cornerRadius=10,
        stroke='$brand' if sel else '$line-soft', strokeWidth=1 if sel else {'bottom': 1})
inbox_list = card('Inbox List', [
    F('Tabs', [pill('All 46', 'dark'), pill('Reports 31', 'none'), pill('Doctor 15', 'none'), pill('Not assigned 9', 'warn')], gap=5),
    search('Search inbox', 'fill_container'),
    V('Items', [inbox_item(*x) for x in INBOX], gap=2, width='fill_container')],
    width=388, height='fill_container', gap=10, padding=[14, 12])

photo = F('Photo', [F('Leaf', [R('Stripe', 'fill_container', 14, '#E8C94A', cornerRadius=7, opacity=0.9)], width=180, height=14, rotation=-18, x=40, y=70),
                    F('Tag', [I('image', 13, '#FFFFFF'), T('1 of 3 photos · 8 Oct 10:29', 11, '600', '#FFFFFF', name='Label')], gap=6, alignItems='center',
                      x=10, y=136, fill='#14201A99', cornerRadius=6, padding=[4, 8])],
          layout='none', width='fill_container', height=170, fill='#6E8B3D', cornerRadius=10, clip=True)
WHY = [('Field Eye', f'{WEAK} weak squares in the north-east since 25 Sep (greenness 78% of normal)'),
       ('Weather', '29 hours at 6 to 16 °C with air above 90% wet this week: rust weather high'),
       ('Neighbours', '3 more reports of yellow stripes within 20 km in 14 days')]
case_box = V('Doctor', [F('Head', [I('stethoscope', 14, '$water'), T('What the Doctor thinks', 12.5, '700', '$water', name='Title'), F('S', [], width='fill_container', height=1),
                                   pill('How sure: likely', 'warn', size=10)], width='fill_container', gap=6, alignItems='center'),
                        T('Likely yellow rust. Not sure without a closer look at the leaf.', 13, '600', width='fill_container'),
                        V('Why', [F('Why · ' + a, [T(a, 11.5, '700', '$ink-2', name='From', width=78), T(b, 11.5, 'normal', '$ink-2', name='What', width='fill_container', lineHeight=1.35)],
                                    width='fill_container', gap=8) for a, b in WHY], gap=6, width='fill_container')],
             width='fill_container', fill='$water-soft', cornerRadius=10, padding=12, gap=8)
TL = [('10:29', 'Farmer sent the report with 3 photos'), ('10:31', 'Doctor read the field: likely yellow rust'),
      ('10:31', 'Farm #1207 opened for officers (farmer asked)'), ('10:42', 'Shilan Ahmed opened it')]
timeline = V('Timeline', [F('Ev', [T(a, 11.5, '600', '$ink-3', name='When', width=40), R('Dot', 7, 7, '$brand', cornerRadius=4),
                                   T(b, 11.5, 'normal', '$ink-2', name='What', width='fill_container')], gap=8, alignItems='center', width='fill_container') for a, b in TL],
             gap=7, width='fill_container')
reply = V('Reply', [T('Reply to the farmer (Sorani first)', 11, '700', '$ink-3', name='Label', letterSpacing=0.6),
                    F('Box', [KU('سڵاو، سبەینێ ئەندازیاری کشتوکاڵ سەردانی کێڵگەکەت دەکات.', 13, 'normal', width='fill_container', textAlign='right')],
                      width='fill_container', fill='$bg', stroke='$line', strokeWidth=1, cornerRadius=9, padding=[10, 12])], gap=6, width='fill_container')
inbox_detail = card('Inbox Detail', [
    F('Title', [V('T', [F('Tags', [pill('Report r_3391', 'danger', 'message-square-warning'), pill('Yellow stripes', 'none')], gap=6),
                        T('Yellow stripes on wheat leaves', 22, '700', '$ink', '$font-display', name='Title')], gap=6, width='fill_container'),
                F('Assign', [dropdown('Assign to', 'Shilan Ahmed')], alignItems='center')], width='fill_container', alignItems='end'),
    F('Body', [V('Left', [photo, V('Note', [T('Farmer wrote', 11, '700', '$ink-3', name='Label', letterSpacing=0.6),
                                            KU('گەڵای گەنمەکە زەرد بووە و هێڵی زەردی تێدایە، لە لای باکووری کێڵگەکە.', 13.5, 'normal', width='fill_container', textAlign='right', lineHeight=1.5),
                                            T('“The wheat leaves turned yellow with yellow lines, on the north side of the farm.”', 11.5, 'normal', '$ink-3', name='Translation', width='fill_container', lineHeight=1.4)],
                                       gap=5, width='fill_container'),
                          V('Where', [kv('Farm', '#1207 · Aghjalar, Chamchamal'), kv('Square tapped', 'e 46415 · n 398748'),
                                      kv('Point', '35.5124 N · 44.8417 E (shared by the farmer)'), kv('Phone', '+964 750 ••• 4567 · Show')], gap=6, width='fill_container')],
                 gap=12, width='fill_container'),
               V('Right', [case_box, eyebrow('What happened'), timeline, reply], gap=12, width='fill_container')],
      gap=18, width='fill_container', height='fill_container'),
    F('Buttons', [button('Mark seen', 'secondary', 'eye'), button('Send reply', 'primary', 'send'), button('Send an officer', 'secondary', 'car'),
                  F('S', [], width='fill_container', height=1), button('Close report', 'secondary', 'check-check')], gap=8, width='fill_container'),
    T('The farmer sees “Seen by an officer” and your reply in the app. Closing hides the farm’s exact place again.', 11, 'normal', '$ink-3', name='Rule', width='fill_container')],
    width='fill_container', height='fill_container', gap=14, padding=[18, 20])
screen('14', 'Inbox', main([
    head('Control room · inbox', 'Inbox', 'Farmer reports and the Doctor cases that ask for an officer · 9 new, 4 older than 2 days',
         [dropdown('Area', 'Sulaymaniyah'), button('Export', 'secondary', 'download')]),
    F('Content', [inbox_list, inbox_detail], gap=14, width='fill_container', height='fill_container')]))

# =====================================================================================
# 15 Rules
# =====================================================================================
RULES = [
    ('Weather planner', [('Frost night (watch)', 'night low ≤ 0 °C', 'weather_planner.py', 'v1'),
                         ('Hard frost (alarm)', 'night low ≤ -2 °C', 'weather_planner.py', 'v1'),
                         ('Heat day', 'day high ≥ 31 °C', 'weather_planner.py', 'v1 · change waiting'),
                         ('Big rain day', 'rain ≥ 12 mm in a day', 'weather_planner.py', 'v1'),
                         ('Sowing rain', '≥ 20 mm in 3 days', 'weather_planner.py', 'v1'),
                         ('Rust weather', '6 to 16 °C and air ≥ 90% wet: some ≥ 8 h, high ≥ 24 h', 'weather_planner.py', 'v1'),
                         ('Spray window', '6 h dry, 15 to 24 °C, wind < 15 km/h, daytime', 'weather_planner.py', 'v1'),
                         ('Sunn pest', 'degree-days over 13.3 °C: nymphs at 84, spray until 223', 'weather_planner.py', 'v1'),
                         ('Dust', 'PM10 ≥ 150 µg/m³', 'weather_planner.py', 'v1')]),
    ('Dryness bands', [('Much greener / greener', '0 to 24 / 25 to 44', 'zones · DrynessBand', 'v1'),
                       ('Normal', '45 to 59', 'zones · DrynessBand', 'v1'),
                       ('Dry / very dry', '60 to 79 / 80 to 100', 'zones · DrynessBand', 'v1')]),
    ('Alerts and sign in', [('Pushes', 'max 1 per farm per day, Alarm only', 'BACKEND.md 2.7', 'v1'),
                            ('Weekly plan', 'Sunday 06:00 local, one message', 'BACKEND.md 2.7', 'v1'),
                            ('Sign-in code', '6 digits, 10 min, 5 tries, ask again after 60 s', 'farmers', 'v1'),
                            ('Farm limits', '20 farms per phone, 50,000 squares per farm', 'farms', 'v1')]),
]
rule_rows = []
for group, items in RULES:
    rule_rows.append(F('Group · ' + group, [eyebrow(group, '$brand')], width='fill_container', padding=[12, 12, 4, 12]))
    for name, val, src, ver in items:
        sel = name == 'Heat day'
        rule_rows.append(F('Rule · ' + name, [T(name, 12.5, '600', name='Name', width=170), T(val, 12.5, 'normal', name='Value', width='fill_container'),
                                              T(src, 11.5, 'normal', '$ink-3', name='Source', width=140),
                                              F('Ver', [pill(ver, 'warn' if 'waiting' in ver else 'none', size=10)], width=130),
                                              I('pencil', 14, '$brand' if sel else '$ink-3')],
                           width='fill_container', padding=[8, 12], gap=10, alignItems='center', stroke='$line-soft', strokeWidth={'bottom': 1},
                           **({'fill': '$brand-soft'} if sel else {})))
rules_card = card('Rules Table', [F('HR', [T(h, 11, '600', '$ink-3', name=h, width=w) for h, w in [('Rule', 170), ('Value', 'fill_container'), ('Comes from', 140), ('Version', 130), ('', 14)]],
                                    width='fill_container', padding=[4, 12, 8, 12], gap=10, stroke='$line', strokeWidth={'bottom': 1})] + rule_rows,
                  width='fill_container', height='fill_container', gap=0, padding=[12, 6])
def compare(label, a, b):
    return V('Compare · ' + label, [T(label, 11.5, 'normal', '$ink-3', name='Label'),
                                    F('Vals', [T(a, 22, '700', '$ink-3', '$font-display', name='Old'), I('arrow-right', 16, '$ink-3'),
                                               T(b, 22, '700', '$brand', '$font-display', name='New')], gap=8, alignItems='center')], gap=2)
change = card('Change', [
    eyebrow('Change waiting for a second officer'),
    T('Heat day', 22, '700', '$ink', '$font-display', name='Title'),
    compare('Day high at or above', '31 °C', '33 °C'),
    V('Why', [T('Asked by', 11, '700', '$ink-3', name='Label', letterSpacing=0.6),
              F('Who', [avatar('DO', '$warn', 26), V('T', [T('Dlovan Omar · admin', 12.5, '600', name='Name'), T('Wed 7 Oct 16:20', 11.5, 'normal', '$ink-3', name='When')], gap=1)], gap=8, alignItems='center'),
              T('“Farmers in Makhmur and Kalar get a heat warning almost every day from June to September and stop reading them.”',
                12, 'normal', '$ink-2', name='Reason', width='fill_container', lineHeight=1.45)], gap=8, width='fill_container'),
    V('Impact', [T('If it had been 33 °C last summer (sample)', 11, '700', '$ink-3', name='Label', letterSpacing=0.6),
                 kv('Heat warnings shown', '1,920 → 1,140 (-41%)'), kv('Districts with none', '9 → 14'), kv('Pushes', 'none: heat is a Watch, never pushed')],
           gap=7, width='fill_container', fill='$bg', cornerRadius=10, padding=12),
    V('Source', [T('Research', 11, '700', '$ink-3', name='Label', letterSpacing=0.6),
                 T('Grain number falls above about 31 °C around flowering (reports/Farm_Advice_Research.md). At 33 °C early May heat on flowering wheat would be missed.',
                   12, 'normal', '$ink-2', name='Text', width='fill_container', lineHeight=1.45)], gap=6, width='fill_container'),
    F('S', [], width='fill_container', height='fill_container'),
    F('Buttons', [button('Approve', 'primary', 'check', width='fill_container'), button('Turn down', 'danger', 'x', width='fill_container')], gap=8, width='fill_container'),
    T('The old value stays in the history. The apps get the new rule at the next plan, every 6 hours.', 11, 'normal', '$ink-3', name='Rule', width='fill_container', lineHeight=1.4)],
    width=380, height='fill_container', gap=12, padding=[18, 18])
screen('15', 'Rules', main([
    head('Control room · rules', 'Rules the alerts follow', 'Every number the planner, the dryness map and the pushes use. A change needs two officers and keeps the old value.',
         [button('Rule history', 'secondary', 'history')]),
    F('Content', [rules_card, change], gap=14, width='fill_container', height='fill_container')]))

# =====================================================================================
# 16 Alwa control
# =====================================================================================
PRICES = [('wheat', ['850', '850', '850', '850'], 'Government price', True), ('barley', ['450', '460', '440', '430'], 'Market board', False),
          ('tomato', ['700', '750', '800', '650'], 'Market board', False), ('cucumber', ['600', '575', '650', '550'], 'Market board', False),
          ('potato', ['500', '525', '480', '500'], 'Market board', False), ('onion', ['400', '425', '400', '375'], 'Market board', False),
          ('watermelon', ['300', '325', '275', '250'], 'Market board', False), ('grape', ['1,250', '1,300', '1,400', '1,200'], 'Market board', False),
          ('chickpea', ['1,750', '1,800', '1,750', '1,700'], 'Market board', False)]
pcols = [('Crop', 120, 'left'), ('Sulaymaniyah', 'fill_container', 'right'), ('Erbil', 'fill_container', 'right'), ('Duhok', 'fill_container', 'right'),
         ('Kalar', 'fill_container', 'right'), ('Source', 128, 'left'), ('Fixed', 58, 'center')]
prow = []
for c, ps, src, fixed in PRICES:
    vals = []
    for i, p in enumerate(ps):
        if c == 'tomato' and i == 0:
            vals.append(F('Edited', [T('700', 12.5, '700', '$brand', name='Value'), T('was 750', 10.5, 'normal', '$ink-3', name='Was')], gap=5, alignItems='center',
                          fill='$brand-soft', cornerRadius=6, padding=[3, 6]))
        else:
            vals.append(p)
    prow.append([crop_chip(c)] + vals + [src, I('lock' if fixed else 'lock-open', 14, '$brand' if fixed else '$ink-3')])
prices = card('Prices', [F('Head', [V('T', [eyebrow('Today’s prices · IQD per kg · Thu 8 Oct'), T('The price farmers see in the app', 13, '600', '$ink-2', name='Sub')], gap=4, width='fill_container'),
                                    button('Publish 1 change', 'primary', 'upload', h=34)], width='fill_container', alignItems='end'),
                         table('Price Table', pcols, prow, row_h=38)], width='fill_container', gap=10, padding=[14, 12])
FLAGS = [('Tomato, 40 t at 2,400 IQD/kg', '3.4× the Sulaymaniyah price', 'Seller +964 750 ••• 0021 · listing #518', 'danger'),
         ('Wheat, 120 t, 4 copies', 'Same seller posted the same crop 4 times in 1 hour', 'listing #509 to #512', 'warn'),
         ('Grape, deal #77', 'Buyer says the grade was B, not A', 'Kalar market · disputed 6 Oct', 'water')]
flags = card('Flags', [eyebrow('Needs a look · 3'), V('List', [V('Flag · ' + a, [F('Top', [T(a, 13, '700', name='What', width='fill_container'), pill({'danger': 'Price', 'warn': 'Repeat', 'water': 'Dispute'}[t], t, size=10)],
                                                                           width='fill_container', alignItems='center'),
                                                                         T(b, 12, 'normal', '$ink-2', name='Why', width='fill_container'), T(c, 11.5, 'normal', '$ink-3', name='Who'),
                                                                         F('Buttons', [button('Pause listing' if t != 'water' else 'Open deal', 'secondary', 'pause' if t != 'water' else 'scale', h=30),
                                                                                       button('Remove' if t != 'water' else 'Call both', 'danger' if t != 'water' else 'secondary', 'x' if t != 'water' else 'phone', h=30)], gap=6)],
                                                                        width='fill_container', gap=5, padding=[0, 0, 12, 0], stroke='$line-soft', strokeWidth={'bottom': 1})
                                                                      for a, b, c, t in FLAGS], gap=12, width='fill_container')],
             width=400, gap=12, padding=[16, 16])
HIST_T = [700, 725, 750, 800, 775, 750, 750, 800, 850, 825, 800, 775, 750, 700]
DAYS = ['25', '26', '27', '28', '29', '30', '1', '2', '3', '4', '5', '6', '7', '8']
hist = card('Price History', [F('Head', [eyebrow('Tomato · Sulaymaniyah · last 14 days'), F('S', [], width='fill_container', height=1),
                                         T('IQD per kg · today not published yet', 11.5, 'normal', '$ink-3', name='Unit')], width='fill_container'),
                              F('Bars', [V('Day ' + dd, [T(f'{v}', 10, '600', '$ink-2', name='Value'),
                                                        R('Bar', 'fill_container', round((v - 600) * 0.45), '$brand' if i == 13 else '#D9483B', cornerRadius=3, opacity=1 if i == 13 else 0.55),
                                                        T(dd, 10, 'normal', '$ink-3', name='Day')], gap=4, alignItems='center', width='fill_container', justifyContent='end', height='fill_container')
                                         for i, (v, dd) in enumerate(zip(HIST_T, DAYS))], gap=6, width='fill_container', height='fill_container', alignItems='end')],
            width='fill_container', height='fill_container', gap=10, padding=[14, 16])
mstats = F('Stats', [stat('Open listings', '146', '2,980 t on sale', icon='package'), stat('Offers today', '58', '9 accepted', icon='hand-coins'),
                     stat('Deals this week', '37', '1.2 bn IQD', 'good', 'handshake'), stat('Flags', '3', '1 dispute', 'warn', 'flag')], gap=12, width='fill_container')
screen('16', 'Alwa control', main([
    head('Control room · Alwa market', 'Alwa control', 'Set the official daily price per market, stop bad listings, settle disputes. Phones stay hidden until a deal.',
         [button('Market rules', 'secondary', 'settings-2')]),
    mstats,
    F('Content', [V('Left', [prices, hist], width='fill_container', height='fill_container', gap=14), flags], gap=14, width='fill_container', height='fill_container')]))

# =====================================================================================
# 17 Data health
# =====================================================================================
def strip(pattern):
    col = {'o': '$good', 'x': '$danger', 'p': '$warn', '-': '$line'}
    return F('Strip', [R('Day', 9, 18, col[ch], cornerRadius=2) for ch in pattern], gap=3)
JOBS = [('Satellite (Sentinel-2)', 'satellite', 'daily 05:00', 'Today 05:12', '1,845 of 2,031 farms read · 186 cloudy', 'oooooooooooooo', 'good', 'OK'),
        ('Weather planner', 'cloud-sun', 'every 6 h', 'Today 06:00', '2,031 plans · 3 alarms', 'oooooooooooooo', 'good', 'OK'),
        ('Fires (NASA FIRMS)', 'flame', 'every 3 h', 'Today 09:00', '4 new fires, 1 near farms', 'oooooooooooooo', 'good', 'OK'),
        ('Dams (Dukan, Darbandikhan)', 'waves', 'daily 07:00', 'Yesterday 07:02', 'no reading today: source page did not answer', 'oooooooooooopx', 'danger', 'Late'),
        ('Dryness by district', 'sprout', 'monthly, day 3', '3 Oct 04:40', '33 districts, 72 sub-districts', '-----o-------o', 'good', 'OK'),
        ('Season outlook', 'cloud-rain-wind', 'monthly, day 5', '5 Oct 05:15', '33 districts · track record 64%', '-------o------', 'good', 'OK'),
        ('Alwa prices', 'store', 'daily 08:00', 'Today 08:03', '4 markets · 1 change to publish', 'ooooooopoooooo', 'good', 'OK')]
job_cards = []
for name, icon, every, last, what, pat, tone, st in JOBS:
    job_cards.append(F('Job · ' + name, [
        F('Icon', [I(icon, 16, TONE[tone][1])], width=34, height=34, fill=TONE[tone][0], cornerRadius=9, justifyContent='center', alignItems='center'),
        V('Name', [T(name, 13, '700', name='Name', width='fill_container'), T(every, 11.5, 'normal', '$ink-3', name='Every')], gap=2, width=170),
        V('Last', [T(last, 12.5, '600', name='Last'), T(what, 11.5, 'normal', '$danger' if tone == 'danger' else '$ink-3', name='What', width='fill_container')], gap=2, width='fill_container'),
        V('Days', [strip(pat), T('last 14 days', 10, 'normal', '$ink-3', name='Label')], gap=3),
        F('St', [pill(st, tone)], width=60, justifyContent='end'),
        button('Run now', 'secondary', 'play', h=32)],
        width='fill_container', padding=[11, 14], gap=14, alignItems='center', stroke='$line-soft', strokeWidth={'bottom': 1}))
jobs = card('Jobs', [F('Head', [eyebrow('Data jobs · they push numbers in with the service key'), F('S', [], width='fill_container', height=1),
                                T('Times in Erbil time (UTC+3)', 11.5, 'normal', '$ink-3', name='Tz')], width='fill_container', padding=[0, 4])] + job_cards,
            width='fill_container', gap=2, padding=[14, 8])
warn_box = F('Warning', [I('triangle-alert', 18, '$danger'), V('T', [T('One sign-in code works for every phone', 13, '700', '$danger', name='Title'),
                                                                     T('AUTH__FIXED_SIGN_IN_CODE is set. Fine for the demo, never with real farmers: anyone could open any farm.',
                                                                       12, 'normal', '$danger', name='Text', width='fill_container', lineHeight=1.4)], gap=3, width='fill_container'),
                         button('Turn off', 'danger', 'power', h=34)],
             width='fill_container', fill='$danger-soft', cornerRadius=12, padding=[12, 14], gap=12, alignItems='center', stroke='$danger', strokeWidth=1)
api = card('API', [eyebrow('Server · last 24 h'), kv('Requests', '48,210'), kv('Errors (5xx)', '0.04% · 19'), kv('Slowest 5% answer in', '182 ms'),
                   kv('Database', 'up · 2.1 GB'), kv('Farms still on phones', 'not known here', '$ink-3', 'normal')],
           width='fill_container', gap=8)
keys = card('Keys', [eyebrow('Service keys'), kv('jobs-server-1', 'used 2 min ago'), kv('laptop-arya', 'used 3 days ago', '$warn'),
                     F('B', [button('New key', 'secondary', 'key-round', h=32), button('Revoke old', 'danger', 'x', h=32)], gap=8)],
            width='fill_container', gap=8)
apps = card('Apps', [eyebrow('App versions in use'), V('Bars', [F('V · ' + v, [T(v, 12, '600', name='Ver', width=56), F('Tr', [R('F', w, 8, c, cornerRadius=3)], width=140, height=8),
                                                                       T(p, 12, 'normal', '$ink-3', name='Pct', width=40, textAlign='right')], gap=8, alignItems='center')
                                                                     for v, w, c, p in [('1.0.3', 112, '$good', '80%'), ('1.0.2', 21, '$warn', '15%'), ('1.0.0', 7, '$danger', '5%')]], gap=6),
                     T('Needs the app to send its version with each call (BACKEND.md 2.11).', 11, 'normal', '$ink-3', name='Note', width='fill_container', lineHeight=1.4)],
            width='fill_container', gap=8)
screen('17', 'Data health', main([
    head('Control room · data health', 'Is the data fresh?', 'Every number on the dashboard and in the app comes from these jobs. A late job means old numbers.',
         [pill('6 of 7 on time', 'good', 'circle-check')]),
    warn_box,
    F('Content', [V('Left', [jobs], width='fill_container', height='fill_container'), V('Right', [api, keys, apps], width=330, gap=12)], gap=14, width='fill_container', height='fill_container')]))

# =====================================================================================
# 18 Officers and history
# =====================================================================================
OFF = [('SA', 'Shilan Ahmed', 'District officer', 'Sulaymaniyah', 'now', True, '$brand'),
       ('KA', 'Karwan Aziz', 'Admin', 'All Kurdistan', '12 min ago', True, '$water'),
       ('DO', 'Dlovan Omar', 'Admin', 'All Kurdistan', 'Wed', True, '$warn'),
       ('HM', 'Hevi Mustafa', 'District officer', 'Duhok', 'today 08:40', True, '#7E57C2'),
       ('RS', 'Rebaz Salih', 'District officer', 'Erbil', 'Tue', False, '#A9784A'),
       ('NH', 'Nazdar Hassan', 'Viewer', 'Halabja', 'Mon', True, '#D9483B')]
ocols = [('Officer', 'fill_container', 'left'), ('Role', 112, 'left'), ('Area', 92, 'left'), ('2-step', 44, 'center'), ('Last seen', 76, 'left')]
orows = [[F('O', [avatar(i, c, 26), T(n, 12.5, '600', name='Name')], gap=8, alignItems='center'),
          pill(r, 'dark' if r == 'Admin' else 'brand' if r.startswith('District') else 'none', size=10), a,
          I('shield-check' if tw else 'shield-alert', 15, '$good' if tw else '$danger'), s] for i, n, r, a, s, tw, c in OFF]
roles = V('Roles', [F('Role · ' + a, [pill(a, t, size=10), T(b, 11.5, 'normal', '$ink-2', name='Can', width='fill_container', lineHeight=1.35)], gap=8, width='fill_container')
                    for a, b, t in [('Viewer', 'Sees totals and maps only. No farms, no phones.', 'none'),
                                    ('District officer', 'Own area: farms at 1 km, inbox, draft alerts, show a phone with a reason.', 'brand'),
                                    ('Admin', 'Everything, plus rules, prices, officers. Still needs a second officer to send alerts or change rules.', 'dark')]],
            gap=8, width='fill_container', fill='$bg', cornerRadius=10, padding=12)
officers = card('Officers', [F('Head', [eyebrow('Officers · 6'), F('S', [], width='fill_container', height=1), button('Invite officer', 'primary', 'user-plus', h=32)],
                                width='fill_container', alignItems='center'),
                             table('Table', ocols, orows, row_h=42), roles,
                             F('Warn', [I('shield-alert', 14, '$danger'), T('Rebaz Salih has no 2-step sign-in: blocked from farms until it is on.', 11.5, '600', '$danger', name='Note', width='fill_container')],
                               gap=6, alignItems='center', width='fill_container')],
                width=580, height='fill_container', gap=12, padding=[16, 16])
LOG = [('10:42', 'SA', 'Shilan Ahmed', 'opened farm #1207', 'reason: report r_3391', 'eye', '$brand'),
       ('10:44', 'SA', 'Shilan Ahmed', 'showed the phone of farm #1207', 'reason: “to arrange a field visit”', 'phone', '$warn'),
       ('10:20', 'SA', 'Shilan Ahmed', 'drafted alert “Frost, Sunday night”', 'Penjwen, Sharbazher, Pshdar · 318 farms', 'bell-ring', '$brand'),
       ('09:58', 'KA', 'Karwan Aziz', 'changed tomato price, Sulaymaniyah', '750 → 700 IQD/kg · not published', 'store', '$brand'),
       ('09:15', 'HM', 'Hevi Mustafa', 'assigned report r_3377 to the vets', 'Penjwen · sick sheep', 'inbox', '$brand'),
       ('08:03', '··', 'Job: Alwa prices', 'pushed 36 prices', 'service key jobs-server-1', 'bot', '$ink-3'),
       ('Wed 16:20', 'DO', 'Dlovan Omar', 'asked to change Heat day 31 → 33 °C', 'waiting for a second officer', 'sliders-horizontal', '$warn'),
       ('Wed 11:02', 'KA', 'Karwan Aziz', 'deleted account +964 770 ••• 1182', 'farmer asked by phone call · 2 farms removed', 'trash-2', '$danger'),
       ('Mon 18:30', 'KA', 'Karwan Aziz', 'stopped alert “Heat 31 °C+”, Makhmur', 'reason: same alert sent the day before', 'octagon-x', '$danger')]
log_rows = [F('Log · %d' % i, [T(t, 11.5, '600', '$ink-3', name='When', width=64), F('Ic', [I(ic, 14, c)], width=28, height=28, fill='$bg', cornerRadius=8, justifyContent='center', alignItems='center'),
                               V('T', [F('L', [T(who, 12.5, '700', name='Who'), T(what, 12.5, 'normal', name='What')], gap=5),
                                       T(more, 11.5, 'normal', '$ink-3', name='More')], gap=2, width='fill_container')],
                   width='fill_container', gap=10, alignItems='center', padding=[9, 4], stroke='$line-soft', strokeWidth={'bottom': 1})
            for i, (t, _, who, what, more, ic, c) in enumerate(LOG)]
history = card('History', [F('Head', [eyebrow('History · today and this week'), F('S', [], width='fill_container', height=1), button('Export', 'secondary', 'download', h=32)],
                               width='fill_container', alignItems='center'),
                           F('Filters', [pill('All', 'dark'), pill('Looked at farmers', 'none'), pill('Alerts', 'none'), pill('Rules', 'none'), pill('Prices', 'none'), pill('Deletes', 'none')], gap=5),
                           V('Log', log_rows, width='fill_container'),
                           F('Lock', [I('lock', 13, '$ink-3'), T('History cannot be changed or deleted, also not by admins. Kept 5 years.', 11.5, 'normal', '$ink-3', name='Note')], gap=6, alignItems='center')],
                  width='fill_container', height='fill_container', gap=12, padding=[16, 16])
screen('18', 'Officers and history', main([
    head('Control room · officers and history', 'Who can do what, and who did what', 'Each officer signs in with their phone and a second step. Every look at a farmer and every change is written down.',
         []),
    F('Content', [officers, history], gap=14, width='fill_container', height='fill_container')]))

# ---------- place the screens under the existing ones ----------
bottom = max(c.get('y', 0) + (c['height'] if isinstance(c.get('height'), (int, float)) else 900) for c in doc['children'] if 'Component' not in c.get('name', ''))
top = bottom + 200
for i, f in enumerate(SCREENS):
    f['x'] = (i % 2) * 1540
    f['y'] = top + (i // 2) * 1124
doc['children'] += SCREENS
json.dump(doc, open(DST, 'w', encoding='utf-8'), ensure_ascii=False, indent=2)
print('screens:', [f['name'] for f in SCREENS], 'top', top, 'cells', n_cells)
