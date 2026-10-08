"""NOAA STAR Vegetation Health (VHI/VCI/TCI + SMN/SMT), weekly, province means over CROPLAND only.

Source (keyless): https://www.star.nesdis.noaa.gov/smcd/emb/vci/VH/get_TS_admin.php
  ?provinceID=<id>&country=IRQ&adminVHversion=GC_current&yearlyTag=Weekly&type=Mean&TagCropland=crop&year1=1982&year2=<this year>
  = the "Province-Averaged VH data for CropLand" link on vh_adminMean.php (4 km blended VH, operational GC_current version).
Missing weeks are -1 in the source and are written as empty cells.

Usage: python3 -I fetch_vhi.py RAW_DIR OUT_CSV [YEAR2]
"""
import csv
import os
import re
import sys
import time
import urllib.request

URL = ('https://www.star.nesdis.noaa.gov/smcd/emb/vci/VH/get_TS_admin.php?provinceID=%d&country=IRQ'
       '&adminVHversion=GC_current&yearlyTag=Weekly&type=Mean&TagCropland=crop&year1=1982&year2=%d')
# NOAA STAR province ids for IRQ (getProvinceNames.php?country_code=IRQ)
PROVINCES = [(12, 'Duhok', 'Dihok'), (6, 'Erbil', 'Arbil'), (7, 'Sulaymaniyah', 'As-Sulaymaniyah'),
             (8, 'Kirkuk', "At-Ta'mim"), (16, 'Ninawa', 'Ninawa'), (13, 'Diyala', 'Diyala')]


def get(url, tries=5):
    for k in range(tries):
        try:
            req = urllib.request.Request(url, headers={'User-Agent': 'SmartSuli-backtest/1.0 (python urllib)'})
            with urllib.request.urlopen(req, timeout=180) as r:
                return r.read().decode('utf-8', 'replace')
        except Exception:
            if k == tries - 1:
                raise
            time.sleep(4 * (k + 1))


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    raw_dir, out_csv = sys.argv[1], sys.argv[2]
    year2 = int(sys.argv[3]) if len(sys.argv) > 3 else time.gmtime().tm_year
    os.makedirs(raw_dir, exist_ok=True)
    rows = []
    for pid, name, noaa_name in PROVINCES:
        url = URL % (pid, year2)
        path = os.path.join(raw_dir, 'IRQ_province%02d_%s_cropland_weekly.txt' % (pid, name))
        if not os.path.exists(path):
            txt = get(url)
            with open(path, 'w') as f:
                f.write('# source=%s\n' % url)
                f.write(txt)
        txt = open(path).read()
        if noaa_name not in txt or 'cropland area only' not in txt:
            raise SystemExit('unexpected header in %s' % path)
        n = 0
        for m in re.finditer(r'^\s*(\d{4}),\s*(\d+),\s*([-\d.]+),\s*([-\d.]+),\s*([-\d.]+),\s*([-\d.]+),\s*([-\d.]+),',
                             txt, re.M):
            y, w = int(m.group(1)), int(m.group(2))
            smn, smt, vci, tci, vhi = (float(v) for v in m.group(3, 4, 5, 6, 7))
            if vhi < 0 and vci < 0 and tci < 0:
                continue  # -1 = no data (future weeks, satellite gaps)
            cell = lambda v, nd: '' if v <= nd else v
            rows.append((name, y, w, cell(vci, -0.5), cell(tci, -0.5), cell(vhi, -0.5), cell(smn, -0.5), cell(smt, -0.5)))
            n += 1
        print('%-13s id=%2d  %d valid weeks' % (name, pid, n))
    with open(out_csv, 'w', newline='') as f:
        wr = csv.writer(f)
        wr.writerow(['province', 'year', 'week', 'vci', 'tci', 'vhi', 'smn', 'smt'])
        wr.writerows(rows)
    print('wrote', out_csv, len(rows), 'rows')


if __name__ == '__main__':
    main()
