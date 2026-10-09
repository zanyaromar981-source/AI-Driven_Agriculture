import json, os, re, sys, time, urllib.parse, urllib.request
from shapely.geometry import Point, shape

KML = sys.argv[1]
txt = open(KML, encoding='utf-8').read()
polys = {}
for m in re.finditer(r'<Placemark><name>([^<]*)</name><description>([^<]*)</description>.*?</Placemark>', txt, re.S):
    name, desc, body = m.group(1), m.group(2), m.group(0)
    if not desc.startswith('Sub-district'):
        continue
    rings = []
    for c in re.findall(r'<outerBoundaryIs><LinearRing><coordinates>([^<]*)</coordinates>', body):
        rings.append([tuple(map(float, p.split(',')[:2])) for p in c.split()])
    from shapely.geometry import Polygon, MultiPolygon
    g = MultiPolygon([Polygon(r) for r in rings]).buffer(0)
    polys[name] = g

# (english, sorani, osm query, sub-district polygon it must fall in)
T = [
 ('Duhok','دهۆک','Duhok','Markaz Duhok'),('Zakho','زاخۆ','Zakho','Markaz Zakho'),('Batifa','باتیفا','Batifa','Batifa'),
 ('Darkar','دەرکار','Darkar Zakho','Dercar'),('Sumel','سێمێل','Sumel','Markaz Sumail'),('Batel','باتێل','Batel Duhok','Bateel'),
 ('Faida','فایدە','Faida Duhok','Fayde'),('Zawita','زاویتە','Zawita','Zawita'),('Mangesh','مانگێشک','Mangesh','Al-Duski'),
 ('Amedi','ئامێدی','Amadiya','Markaz Al-Amadiya'),('Kani Masi','کانی ماسێ','Kani Masi','Barwari Bala'),
 ('Deraluk','دێرەلووک','Deraluk','Nerwa Rekan'),('Shiladze','شیلادزێ','Shiladze','Nerwa Rekan'),('Sarsang','سەرسەنگ','Sarsing','Sarsank'),
 ('Bamarni','بامەڕنێ','Bamarni','Sarsank'),('Shekhan','شێخان','Ain Sifni','Markaz Al-Shikhan'),('Atrush','ئەترووش','Atrush','Atreesh'),
 ('Qasrok','قەسرۆک','Qasrok','Qasruk'),('Baadre','باعەدرێ','Baadra','Markaz Al-Shikhan'),('Akre','ئاکرێ','Akre','Markaz Aqra'),
 ('Dinarta','دینارتێ','Dinarta','Dinarta'),('Bijil','بجیل','Bujeel Akre','Begeel'),('Bardarash','بەردەڕەش','Bardarash','Bardarash'),
 ('Kalak','کەلەک','Kalak Iraq','Bardarash'),
 ('Erbil','هەولێر','Erbil','Markaz Erbil'),('Ankawa','عەنکاوە','Ankawa','Markaz Erbil'),('Khabat','خەبات','Khabat Erbil','Markaz Erbil'),
 ('Qushtapa','قوشتەپە','Qushtapa','Qushtappa'),('Bnaslawa','بنەسڵاوە','Bnaslawa','Markaz Erbil'),('Makhmur','مەخموور','Makhmur','Markaz Makhmour'),
 ('Gwer','گوێر','Gwer','Gwyer'),('Dibaga','دیبەگە','Dibaga','Dibaga'),('Koya','کۆیە','Koya Erbil','Markaz Koysinjaq'),
 ('Taq Taq','تەقتەق','Taq Taq','Taq Taq'),('Shorsh','شۆڕش','Shorsh Koya','Shorsh'),('Shaqlawa','شەقڵاوە','Shaqlawa','Khoshnaw'),
 ('Salahaddin','سەڵاحەدین','Salahaddin Erbil','Salah Al-Din'),('Harir','هەریر','Harir','Harir'),('Soran','سۆران','Soran Erbil','Diana'),
 ('Rawanduz','ڕەواندز','Rawanduz','Markaz Rawanduz'),('Khalifan','خەلیفان','Khalifan','Khailfan'),('Choman','چۆمان','Choman','Balak'),
 ('Haji Omaran','حاجی ئۆمەران','Haji Omaran','Haji Omaran'),('Sidakan','سیدەکان','Sidakan','Bradost'),('Mergasor','مێرگەسۆر','Mergasur','Mergasur'),
 ('Barzan','بارزان','Barzan Iraq','Barzan'),
 ('Sulaymaniyah','سلێمانی','Sulaymaniyah','Sarchnar'),('Chamchamal','چەمچەماڵ','Chamchamal','Markaz Chamchamal'),
 ('Sangaw','سەنگاو','Sangaw','Cenkaw'),('Darbandikhan','دەربەندیخان','Darbandikhan','Markaz Derbendikhan'),
 ('Dukan','دوکان','Dukan Sulaymaniyah','Sourdash'),('Kalar','کەلار','Kalar','Markaz Kalar'),('Penjwen','پێنجوێن','Penjwen','Markaz Panjwin'),
 ('Qaladze','قەڵادزێ','Qaladiza','Markaz Pshdar'),('Ranya','ڕانیە','Ranya','Markaz Rania'),('Chwarta','چوارتا','Chwarta','Markaz Sharbazher'),
 ('Mawat','ماوەت','Mawat','Mawat'),('Qaradagh','قەرەداغ','Qaradagh','Qaradagh'),('Bazian','بازیان','Bazian','Bazian'),
 ('Said Sadiq','سەیدسادق','Said Sadiq','Shahrazur'),('Arbat','عەربەت','Arbat','Sarchnar'),('Hero','هێرۆ','Hero Pshdar','Hero'),
 ('Halabja','هەڵەبجە','Halabja','Markaz Halabja'),('Khurmal','خورماڵ','Khurmal','Khourmal'),('Byara','بیارە','Biyara','Beyara'),
 ('Tawella','تەوێڵە','Tawella','Beyara'),('Sirwan','سیروان','Sirwan Halabja','Markaz Halabja'),
 ('Kifri','کفری','Kifri','Markaz Kifri'),
]
cache = json.load(open('towns_cache.json', encoding='utf-8')) if os.path.exists('towns_cache.json') else {}
out, miss = [], []
ALT = {'Sumel':['Simele','سێمێل'],'Faida':['Faydah','فايدة'],'Baadre':['Baadre','باعدرى'],'Bijil':['Bijeel','بجیل'],
 'Shorsh':['شۆڕش'],'Salahaddin':['Masif Salahaddin','Pirmam','پیرمام'],'Said Sadiq':['Sayid Sadiq','سەیدسادق'],
 'Arbat':['عەربەت','Arbet'],'Byara':['Biara','بیارە'],'Tawella':['Tawela','تەوێڵە'],'Makhmur':['مەخموور'],'Sangaw':['Sengaw']}
def fetch(q):
    if q not in cache:
        url = 'https://nominatim.openstreetmap.org/search?' + urllib.parse.urlencode(
            {'q': q, 'countrycodes': 'iq', 'format': 'jsonv2', 'limit': 8, 'viewbox': '42.2,37.5,46.5,34.3', 'bounded': 1})
        req = urllib.request.Request(url, headers={'User-Agent': 'Jutyar-map-demo/1.0 (hackathon prototype)'})
        try:
            cache[q] = json.load(urllib.request.urlopen(req, timeout=30))
        except Exception as e:
            print('ERR', q, e); cache[q] = []
        json.dump(cache, open('towns_cache.json', 'w', encoding='utf-8'), ensure_ascii=False)
        time.sleep(1.2)
    return cache[q]

OK_TYPES = ('city','town','village','administrative','hamlet','suburb','residential','neighbourhood','quarter','locality')
for en, ku, q0, sub in T:
    g = polys[sub].buffer(0.11)
    pick, seen = None, []
    for q in [q0] + ALT.get(en, []):
        for r in fetch(q):
            seen.append((q, r.get('name'), r.get('type'), r['lat'], r['lon']))
            if r.get('category') == 'place' or r.get('type') in OK_TYPES:
                if g.contains(Point(float(r['lon']), float(r['lat']))):
                    pick = r; break
        if pick:
            break
    if pick:
        out.append({'en': en, 'ku': ku, 'lat': round(float(pick['lat']), 6), 'lon': round(float(pick['lon']), 6),
                    'sub': sub, 'osm': f"{pick.get('osm_type')}/{pick.get('osm_id')}", 'kind': pick.get('type')})
    else:
        miss.append((en, sub, seen[:4]))
json.dump(out, open('towns.json', 'w', encoding='utf-8'), ensure_ascii=False, indent=0)
print('found', len(out), 'of', len(T))
for m in miss:
    print('MISS', m)
