import json, math, re, sys
from shapely.geometry import Polygon, MultiPolygon, mapping
from shapely.ops import unary_union

KML, TOWNS, OUT = sys.argv[1], sys.argv[2], sys.argv[3]

GOV = {
    'Duhok':        {'ku': 'دهۆک',     'color': '#2FB45A'},
    'Erbil':        {'ku': 'هەولێر',   'color': '#F4B223'},
    'Sulaymaniyah': {'ku': 'سلێمانی',  'color': '#1FA2E0'},
    'Halabja':      {'ku': 'هەڵەبجە',  'color': '#EC4C6B'},
}
DIST_KU = {
    'Duhok': 'دهۆک', 'Zakho': 'زاخۆ', 'Sumel': 'سێمێل', 'Amedi': 'ئامێدی', 'Shekhan': 'شێخان', 'Akre': 'ئاکرێ', 'Bardarash': 'بەردەڕەش',
    'Erbil': 'هەولێر', 'Qushtapa': 'قوشتەپە', 'Makhmur': 'مەخموور', 'Koya': 'کۆیە', 'Taqtaq': 'تەقتەق', 'Shaqlawa': 'شەقڵاوە',
    'Harir': 'هەریر', 'Pirmam': 'پیرمام', 'Rawanduz': 'ڕەواندز', 'Soran': 'سۆران', 'Khalifan': 'خەلیفان', 'Chuman': 'چۆمان',
    'Sidakan': 'سیدەکان', 'Mergasor': 'مێرگەسۆر',
    'Sulaymaniyah': 'سلێمانی', 'Chamchamal': 'چەمچەماڵ', 'Darbandikhan': 'دەربەندیخان', 'Dukan': 'دوکان', 'Kalar': 'کەلار',
    'Penjwen': 'پێنجوێن', 'Pshdar': 'پشدەر', 'Ranya': 'ڕانیە', 'Sharbazher': 'شارباژێڕ', 'Sharazur': 'شارەزوور',
    'Halabja': 'هەڵەبجە', 'Khurmal': 'خورماڵ',
    'Kifri': 'کفری', 'Khanaqin': 'خانەقین',
}
# CSO sub-district -> (KRG governorate, KRG district, Sorani sub-district name or None when not certain)
SUB = {
    'Markaz Duhok': ('Duhok', 'Duhok', 'ناوەندی دهۆک'), 'Al-Duski': ('Duhok', 'Duhok', 'مانگێشک'), 'Zawita': ('Duhok', 'Duhok', 'زاویتە'),
    'Markaz Zakho': ('Duhok', 'Zakho', 'ناوەندی زاخۆ'), 'Batifa': ('Duhok', 'Zakho', 'باتیفا'), 'Dercar': ('Duhok', 'Zakho', 'دەرکار'),
    'Sindi': ('Duhok', 'Zakho', 'ڕزگاری'),
    'Markaz Sumail': ('Duhok', 'Sumel', 'ناوەندی سێمێل'), 'Bateel': ('Duhok', 'Sumel', 'باتێل'), 'Fayde': ('Duhok', 'Sumel', 'فایدە'),
    'Markaz Al-Amadiya': ('Duhok', 'Amedi', 'ناوەندی ئامێدی'), 'Barwari Bala': ('Duhok', 'Amedi', 'کانی ماسێ'),
    'Nerwa Rekan': ('Duhok', 'Amedi', 'دێرەلووک و شیلادزێ'), 'Sarsank': ('Duhok', 'Amedi', 'سەرسەنگ'),
    'Markaz Al-Shikhan': ('Duhok', 'Shekhan', 'ناوەندی شێخان'), 'Qasruk': ('Duhok', 'Shekhan', 'قەسرۆک'), 'Atreesh': ('Duhok', 'Shekhan', 'ئەترووش'),
    'Markaz Aqra': ('Duhok', 'Akre', 'ناوەندی ئاکرێ'), 'Dinarta': ('Duhok', 'Akre', 'دینارتێ'), 'Begeel': ('Duhok', 'Akre', 'بجیل'),
    'Kurdsein': ('Duhok', 'Akre', 'گردەسێن'), 'Bardarash': ('Duhok', 'Bardarash', 'بەردەڕەش'),
    'Markaz Erbil': ('Erbil', 'Erbil', 'ناوەندی هەولێر'), 'Qushtappa': ('Erbil', 'Qushtapa', 'قوشتەپە'),
    'Markaz Makhmour': ('Erbil', 'Makhmur', 'ناوەندی مەخموور'), 'Dibaga': ('Erbil', 'Makhmur', 'دیبەگە'), 'Gwyer': ('Erbil', 'Makhmur', 'گوێر'),
    'Qaraj': ('Erbil', 'Makhmur', 'قەراج'),
    'Markaz Koysinjaq': ('Erbil', 'Koya', 'ناوەندی کۆیە'), 'Shorsh': ('Erbil', 'Koya', 'شۆڕش'), 'Taq Taq': ('Erbil', 'Taqtaq', 'تەقتەق'),
    'Khoshnaw': ('Erbil', 'Shaqlawa', 'خۆشناو'), 'Harir': ('Erbil', 'Harir', 'هەریر'), 'Salah Al-Din': ('Erbil', 'Pirmam', 'سەڵاحەدین'),
    'Markaz Rawanduz': ('Erbil', 'Rawanduz', 'ناوەندی ڕەواندز'), 'Diana': ('Erbil', 'Soran', 'دیانا'), 'Khailfan': ('Erbil', 'Khalifan', 'خەلیفان'),
    'Balak': ('Erbil', 'Chuman', 'باڵەکایەتی'), 'Haji Omaran': ('Erbil', 'Chuman', 'حاجی ئۆمەران'), 'Bradost': ('Erbil', 'Sidakan', 'برادۆست'),
    'Mergasur': ('Erbil', 'Mergasor', 'ناوەندی مێرگەسۆر'), 'Barzan': ('Erbil', 'Mergasor', 'بارزان'), 'Mazouri Bala': ('Erbil', 'Mergasor', 'مزووری باڵا'),
    'Bazian': ('Sulaymaniyah', 'Sulaymaniyah', 'بازیان'), 'Qaradagh': ('Sulaymaniyah', 'Sulaymaniyah', 'قەرەداغ'),
    'Sarchnar': ('Sulaymaniyah', 'Sulaymaniyah', 'سەرچنار'),
    'Agjalare': ('Sulaymaniyah', 'Chamchamal', 'ئاغجەلەر'), 'Markaz Chamchamal': ('Sulaymaniyah', 'Chamchamal', 'ناوەندی چەمچەماڵ'),
    'Kadr Karam': ('Sulaymaniyah', 'Chamchamal', 'قادرکەرەم'), 'Cenkaw': ('Sulaymaniyah', 'Chamchamal', 'سەنگاو'),
    'Markaz Derbendikhan': ('Sulaymaniyah', 'Darbandikhan', 'ناوەندی دەربەندیخان'),
    'Gnareen': ('Sulaymaniyah', 'Dukan', None), 'Sourdash': ('Sulaymaniyah', 'Dukan', 'سورداش'),
    'Markaz Kalar': ('Sulaymaniyah', 'Kalar', 'ناوەندی کەلار'), 'Bebaz': ('Sulaymaniyah', 'Kalar', None), 'Tilako': ('Sulaymaniyah', 'Kalar', 'تیلەکۆ'),
    'Karmak': ('Sulaymaniyah', 'Penjwen', None), 'Markaz Panjwin': ('Sulaymaniyah', 'Penjwen', 'ناوەندی پێنجوێن'),
    'Hero': ('Sulaymaniyah', 'Pshdar', 'هێرۆ'), 'Nawdasht': ('Sulaymaniyah', 'Pshdar', 'ناوداشت'), 'Markaz Pshdar': ('Sulaymaniyah', 'Pshdar', 'قەڵادزێ'),
    'Mirka': ('Sulaymaniyah', 'Ranya', None), 'Markaz Rania': ('Sulaymaniyah', 'Ranya', 'ناوەندی ڕانیە'), 'Bitwana': ('Sulaymaniyah', 'Ranya', 'بتوێن'),
    'Saruchik': ('Sulaymaniyah', 'Sharbazher', None), 'Markaz Sharbazher': ('Sulaymaniyah', 'Sharbazher', 'چوارتا'),
    'Siwail': ('Sulaymaniyah', 'Sharbazher', 'سیوەیل'), 'Mawat': ('Sulaymaniyah', 'Sharbazher', 'ماوەت'),
    'Shahrazur': ('Sulaymaniyah', 'Sharazur', 'زەڕایەن'),
    'Markaz Halabja': ('Halabja', 'Halabja', 'هەڵەبجە و سیروان'), 'Beyara': ('Halabja', 'Halabja', 'بیارە'), 'Khourmal': ('Halabja', 'Khurmal', 'خورماڵ'),
    'Al-Saadiya': ('Context', 'Khanaqin', None), 'Markaz Khanaqin': ('Context', 'Khanaqin', None), 'Midan': ('Context', 'Khanaqin', None),
    'Jalawla': ('Context', 'Khanaqin', None), 'Qaratu': ('Context', 'Khanaqin', None),
    'Jabarra': ('Context', 'Kifri', None), 'Markaz Kifri': ('Context', 'Kifri', None), 'Qara Tabe': ('Context', 'Kifri', None),
}
PALETTE = {
    'Duhok':        ['#2FB45A', '#5BC85F', '#1E9E6A', '#86D35A', '#3CBF8A', '#4DAA3F', '#9BDA6C'],
    'Erbil':        ['#F4B223', '#F7C948', '#EE9B1C', '#FAD567', '#F2A93B', '#E8901F', '#F9C23C', '#F6B84F', '#FFD27A',
                     '#EAA226', '#F3BE2E', '#FCCB55', '#E99A30', '#F7AE3F'],
    'Sulaymaniyah': ['#1FA2E0', '#43B8EC', '#1489C9', '#5FCBE6', '#26AEC4', '#7AD0F0', '#1D97B8', '#3FC1D6', '#0F7FC2', '#5AB4F0'],
    'Halabja':      ['#EC4C6B', '#F57A8E'],
    'Context':      ['#C9CED3', '#B9BFC6'],
}

txt = open(KML, encoding='utf-8').read()
subs = []
for m in re.finditer(r'<Placemark><name>([^<]*)</name><description>([^<]*)</description>.*?</Placemark>', txt, re.S):
    name, desc, body = m.group(1), m.group(2), m.group(0)
    if not desc.startswith('Sub-district'):
        continue
    polys = [Polygon([tuple(map(float, p.split(',')[:2])) for p in c.split()])
             for c in re.findall(r'<outerBoundaryIs><LinearRing><coordinates>([^<]*)</coordinates>', body)]
    g = unary_union([p.buffer(0) for p in polys])
    if name not in SUB:
        raise SystemExit('unmapped sub-district: ' + name)
    gov, dist, ku = SUB[name]
    subs.append({'name': name, 'cso_parent': desc.replace('Sub-district of ', ''), 'gov': gov, 'dist': dist, 'ku': ku, 'g': g})
missing = set(SUB) - {s['name'] for s in subs}
if missing:
    raise SystemExit('in table but not in KML: %s' % missing)

def gj(g):
    g = g.buffer(0)
    def r(c):
        return [[round(x, 5), round(y, 5)] for x, y in c]
    polys = list(g.geoms) if isinstance(g, MultiPolygon) else [g]
    polys = [p for p in polys if p.area > 1e-6]
    return {'type': 'MultiPolygon', 'coordinates': [[r(p.exterior.coords)] + [r(i.coords) for i in p.interiors] for p in polys]}

def feat(g, props):
    c = g.representative_point()
    b = g.bounds
    props.update(c=[round(c.y, 5), round(c.x, 5)], bbox=[round(b[1], 5), round(b[0], 5), round(b[3], 5), round(b[2], 5)],
                 area_km2=round(g.area * 111.32 ** 2 * math.cos(math.radians(c.y)), 1))
    return {'type': 'Feature', 'properties': props, 'geometry': gj(g)}

dists = {}
for s in subs:
    dists.setdefault((s['gov'], s['dist']), []).append(s)
dist_feats, dist_color = [], {}
for gov in list(GOV) + ['Context']:
    names = sorted(d for (g, d) in dists if g == gov)
    for i, d in enumerate(names):
        col = PALETTE[gov][i % len(PALETTE[gov])]
        dist_color[(gov, d)] = col
        g = unary_union([s['g'] for s in dists[(gov, d)]])
        dist_feats.append(feat(g, {'level': 'district', 'gov': gov, 'en': d, 'ku': DIST_KU.get(d), 'color': col,
                                   'subs': [s['name'] for s in dists[(gov, d)]]}))
sub_feats = [feat(s['g'], {'level': 'subdistrict', 'gov': s['gov'], 'dist': s['dist'], 'en': s['name'], 'ku': s['ku'],
                           'color': dist_color[(s['gov'], s['dist'])], 'cso_parent': s['cso_parent']}) for s in subs]
gov_feats = []
for gov, meta in GOV.items():
    g = unary_union([s['g'] for s in subs if s['gov'] == gov])
    gov_feats.append(feat(g, {'level': 'governorate', 'gov': gov, 'en': gov, 'ku': meta['ku'], 'color': meta['color'],
                              'districts': sorted(d for (gg, d) in dists if gg == gov)}))

towns = json.load(open(TOWNS, encoding='utf-8'))
sub_by = {s['name']: s for s in subs}
for t in towns:
    s = sub_by[t['sub']]
    t['gov'], t['dist'] = s['gov'], s['dist']
dams = [
    {'en': 'Dukan Dam', 'ku': 'بەنداوی دوکان', 'lat': 35.954167, 'lon': 44.952778, 'gov': 'Sulaymaniyah', 'dist': 'Dukan'},
    {'en': 'Darbandikhan Dam', 'ku': 'بەنداوی دەربەندیخان', 'lat': 35.112778, 'lon': 45.706667, 'gov': 'Sulaymaniyah', 'dist': 'Darbandikhan'},
]
data = {
    'source': 'Boundaries: Iraq CSO 2019 sub-districts via geoBoundaries (kri_borders.kml), regrouped into KRG governorates and districts '
              'following the KRG administrative maps of Duhok (2024), Erbil (2026) and Halabja. Towns: OpenStreetMap via Nominatim, each checked '
              'against its sub-district polygon.',
    'governorates': {'type': 'FeatureCollection', 'features': gov_feats},
    'districts': {'type': 'FeatureCollection', 'features': dist_feats},
    'subdistricts': {'type': 'FeatureCollection', 'features': sub_feats},
    'towns': towns, 'dams': dams,
}
js = 'window.KRI = ' + json.dumps(data, ensure_ascii=False, separators=(',', ':')) + ';\n'
open(OUT, 'w', encoding='utf-8').write(js)
print('governorates', len(gov_feats), 'districts', len(dist_feats), 'subdistricts', len(sub_feats), 'towns', len(towns), 'bytes', len(js.encode()))
for f in gov_feats:
    print(f['properties']['en'], f['properties']['area_km2'], 'km2', len(f['properties']['districts']), 'districts')
