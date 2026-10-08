"""If we knew the FUTURE weather perfectly (after the cutoff, until the harvest window), how much better would the warning be?
This is the most any forecast could add. Usage: python3 -I future_value_test.py zones.json ../backtest_data/openmeteo ../backtest_data/modis out_deep"""
import sys, os, json, statistics as st
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from deep_investigation import *
from improve_test import lr_cv, score, fs_old
ZF, OM, MD_, OUTD = sys.argv[1:5]; rows = json.load(open(f'{OUTD}/features_all_v2.json')); zones = [z['name'] for z in json.load(open(ZF))['zones']]
R = {(r['zone'], r['season'], r['cut']): r for r in rows}
NEXT = {'Dec': [(1, 1, 1, 31), (2, 1, 2, 28), (3, 1, 3, 31), (4, 1, 4, 30)], 'Jan': [(2, 1, 2, 28), (3, 1, 3, 31), (4, 1, 4, 30)],
        'Feb': [(3, 1, 3, 31), (4, 1, 4, 30)], 'Mar': [(4, 1, 4, 30)]}
for z in zones:
    w = W(f'{OM}/{z}.json')
    for cut, wins in NEXT.items():
        for k, (m1, d1, m2, d2) in enumerate(wins):
            f = lambda s: (w.sum(ymd(s + 1, m1, d1), ymd(s + 1, m2, d2), 'precipitation_sum'), w.mean(ymd(s + 1, m1, d1), ymd(s + 1, m2, d2), 'temperature_2m_mean'))
            NR = [f(s) for s in NORMAL]; nr = st.mean(x[0] for x in NR); nt = st.mean(x[1] for x in NR)
            for s in SEASONS:
                r = R.get((z, s, cut))
                if r: x = f(s); r[f'fut_rain_{k}'] = x[0] / nr * 100; r[f'fut_temp_{k}'] = x[1] - nt
if __name__ == '__main__':
    print('Perfect knowledge of the coming weather (rain + temperature, month by month, up to Apr 30):')
    for cut, wins in NEXT.items():
        Rc = [r for r in rows if r['cut'] == cut]; base = fs_old(cut); ref = lr_cv(Rc, base)
        one = lr_cv(Rc, base + ['fut_rain_0', 'fut_temp_0'])
        allf = lr_cv(Rc, base + [f'fut_rain_{k}' for k in range(len(wins))] + [f'fut_temp_{k}' for k in range(len(wins))])
        print(f'  {cut}: today {score(ref)}\n       + perfect next month       {score(one, ref)}\n       + perfect rest until Apr 30 {score(allf, ref)}')
