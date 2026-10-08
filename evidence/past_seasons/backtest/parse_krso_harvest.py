"""Wheat + barley area / yield / production per Kurdistan governorate per winter season, from KRSO.

Source PDF (Kurdistan Region Statistics Office, "Summary of agricultural crop data 1969-2023", April 2025):
  https://krso.gov.krd/content/upload/1/root/پوخته‌ى-به‌روبوومه‌-كشتوكاڵيه‌كان1969-2023.pdf
Input is the PDF's text layer, extracted once with:  pdftotext -layout <pdf> <txt>
Each table row is: production (t) | yield (kg/donum) | area (donum) | season (printed right-to-left, e.g. 1970-1969).
1 donum = 2500 m2 = 0.25 ha.  2016-17 (and 2019-20 for Erbil/Duhok) are printed split into rainfed and irrigated;
for those seasons an extra system=all_derived row (sums, yield = production/area) is added.

Usage: python3 -I parse_krso_harvest.py KRSO_TXT OUT_CSV
"""
import csv
import re
import sys

URL = 'https://krso.gov.krd/content/upload/1/root/پوخته‌ى-به‌روبوومه‌-كشتوكاڵيه‌كان1969-2023.pdf'
CROPS = {'گهنم': 'wheat', 'جۆ': 'barley', 'نۆك': None, 'نيسك': None}  # chickpea/lentil tables are skipped
GOVS = [('ههرێمى كوردستان', 'Kurdistan_Region'), ('ههولێر', 'Erbil'), ('سلێمانى', 'Sulaymaniyah'),
        ('دهۆك', 'Duhok'), ('گهرميان', 'Garmiyan'), ('ههڵهبجه', 'Halabja')]
BIDI = dict.fromkeys(map(ord, '‪‫‬‭‮‎‏‌‍'), None)


def num(tok):
    return None if tok == '-' else float(tok.replace(',', ''))


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    lines = open(sys.argv[1], encoding='utf-8').read().splitlines()
    crop = gov = None
    out = []
    for raw in lines:
        line = raw.translate(BIDI)
        if 'رووبهر' in line and 'بهرههمى' in line and 'وهرزى' in line:  # table title
            title_crop = re.search(r'بهرههمى\s+(\S+)\s+له', line)
            crop = CROPS.get(title_crop.group(1)) if title_crop else None
            gov = next((g for k, g in GOVS if k in line), None)
            if crop and not gov:
                raise SystemExit('unknown governorate in title: ' + line)
            continue
        if not crop:
            continue
        m = re.search(r'(\d{4})\s*-\s*(\d{4})', line)
        if not m:
            continue
        y_late, y_early = int(m.group(1)), int(m.group(2))
        if y_late != y_early + 1:
            raise SystemExit('odd season: ' + line)
        system = 'rainfed' if 'دێمى' in line else 'irrigated' if 'ئاوى' in line else 'all'
        toks = re.findall(r'(?<![\d,.])(\d[\d,]*(?:\.\d+)?|-)(?![\d,.])', line[:m.start()])
        if len(toks) != 3:
            raise SystemExit('expected 3 numbers before season: %r' % line)
        prod, yld, area = (num(t) for t in toks)
        out.append([gov, '%d-%d' % (y_early, y_late), y_late, crop, system, area, prod, yld])

    # add summed rows where a season is only given split rainfed/irrigated
    keyed = {}
    for r in out:
        keyed.setdefault(tuple(r[:4]), []).append(r)
    for k, rs in keyed.items():
        systems = {r[4] for r in rs}
        if 'all' not in systems and {'rainfed', 'irrigated'} <= systems:
            a = sum(r[5] or 0 for r in rs)
            p = sum(r[6] or 0 for r in rs)
            out.append(list(k) + ['all_derived', a, p, round(p / a * 1000, 1) if a else None])

    order = [g for _, g in GOVS]
    out.sort(key=lambda r: (r[3], order.index(r[0]), r[2], r[4]))
    bad = []
    with open(sys.argv[2], 'w', newline='') as f:
        w = csv.writer(f)
        w.writerow(['governorate', 'season', 'harvest_year', 'crop', 'system', 'area', 'production', 'yield',
                    'units', 'source_url'])
        for r in out:
            gov, season, hy, crop, system, area, prod, yld = r
            if area and prod is not None and yld:
                implied = prod / area * 1000
                if abs(implied - yld) / yld > 0.15:
                    bad.append('%s %s %s %s: printed yield %g kg/donum vs production/area %.0f'
                               % (gov, crop, season, system, yld, implied))
            fmt = lambda v: '' if v is None else ('%d' % v if float(v).is_integer() else '%g' % v)
            w.writerow([gov, season, hy, crop, system, fmt(area), fmt(prod), fmt(yld),
                        'area=donum(0.25ha);production=t;yield=kg/donum', URL])
    print('wrote', sys.argv[2], len(out), 'rows')
    for c in ('wheat', 'barley'):
        for g in order:
            ys = sorted({r[2] for r in out if r[3] == c and r[0] == g})
            if ys:
                print('  %-6s %-16s %d seasons, harvest years %d-%d' % (c, g, len(ys), ys[0], ys[-1]))
    print('rows where printed yield disagrees with production/area by >15%%: %d' % len(bad))
    for b in bad:
        print('  ', b)


if __name__ == '__main__':
    main()
