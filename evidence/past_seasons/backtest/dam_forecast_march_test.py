"""Does the lake size at the end of winter (Feb-Mar, from Landsat) fix the June forecast? (2026-10-07 night)
Same truth, rain and test as dam_forecast_test.py (leave-one-year-out linear regression), plus March lake area
(max of clear Landsat measurements 1 Feb - 31 Mar; clouds can only hide water, so the max is the safest).
Usage: python3 -I dam_forecast_march_test.py   (writes dam_forecast_march_test_output.txt)"""
import os, sys, csv, statistics as st
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
sys.argv = [sys.argv[0], os.path.join(HERE, 'dam_history_landsat.csv'), os.path.join(HERE, 'dam_water_areas_window_medians.json'),
            os.path.join(HERE, '..', 'backtest_data', 'catchment_rain', 'chirps_catchment_monthly.csv'), os.path.join(HERE, '..', 'backtest_data', 'enso', 'oni.ascii.txt')]
import dam_forecast_test as F
MAR = {}
for r in csv.DictReader(open(os.path.join(HERE, 'dam_history_march.csv'))):
    if r['clear'] == '1' and r['water_km2']:
        v = float(r['water_km2'])
        if 10 <= v <= F.FULL[r['lake']] * 1.1: MAR[(r['lake'], int(r['year']))] = max(v, MAR.get((r['lake'], int(r['year'])), 0))
lines = []
def P(x=''): print(x); lines.append(x)
for lake in ('Dukan', 'Darbandikhan'):
    years = [y for y in range(1989, 2027) if (lake, 'June', y) in F.A and (lake, 'October', y - 1) in F.A and ('JAS', y - 1) in F.ONI
             and F.rain_to(lake, y, 3) is not None and (lake, y) in MAR]
    truth = {y: F.A[(lake, 'June', y)] for y in years}
    allJune = [F.A[(lake, 'June', y)] for y in range(1989, 2027) if (lake, 'June', y) in F.A]
    low = sorted(allJune)[len(allJune) // 4]
    P(f'=== {lake}: June lake area, forecast at end of March (full {F.FULL[lake]:.0f} km², low summer <= {low:.0f} km²) ===')
    P(f'  {len(years)} years with a clear Feb-Mar picture: {years[0]}-{years[-1]}, {sum(1 for y in years if truth[y] <= low)} of them low summers')
    nr = F.norm(lake, 3)
    sets = {'now: last Oct lake + El Nino + rain Oct-Mar': lambda y: [F.A[(lake, 'October', y - 1)], F.ONI[('JAS', y - 1)], F.rain_to(lake, y, 3) / nr * 100],
            'March lake only': lambda y: [MAR[(lake, y)]],
            'March lake + rain Oct-Mar': lambda y: [MAR[(lake, y)], F.rain_to(lake, y, 3) / nr * 100],
            'now + March lake': lambda y: [F.A[(lake, 'October', y - 1)], F.ONI[('JAS', y - 1)], F.rain_to(lake, y, 3) / nr * 100, MAR[(lake, y)]]}
    for nm, fx in sets.items():
        o = F.loo([(y, fx(y), truth[y]) for y in years]); err = [abs(p - t) for _, p, t in o]
        lows = [(y, p, t) for y, p, t in o if t <= low]; caught = sum(1 for _, p, t in lows if p <= low * 1.1)
        fa = sum(1 for _, p, t in o if t > low and p <= low * 1.1)
        r = st.correlation([p for _, p, _ in o], [t for _, _, t in o])
        y25 = [(p, t) for y, p, t in o if y == 2025]
        P(f'  {nm:44s} avg error {st.mean(err):4.0f} km² ({st.mean(err)/F.FULL[lake]:.0%} of full), r {r:+.2f}, low summers caught {caught}/{len(lows)}, false alarms {fa}'
          + (f' | 2025: predicted {y25[0][0]:.0f}, real {y25[0][1]:.0f}' if y25 else ''))
    P('  2025 inputs: last Oct lake %.0f km², El Nino %+.2f, rain Oct-Mar %.0f%% of normal, March lake %s' % (
        F.A[(lake, 'October', 2024)], F.ONI[('JAS', 2024)], F.rain_to(lake, 2025, 3) / nr * 100, f"{MAR[(lake, 2025)]:.0f} km²" if (lake, 2025) in MAR else 'n/a'))
    P()
open(os.path.join(HERE, 'dam_forecast_march_test_output.txt'), 'w').write('\n'.join(lines) + '\n')
