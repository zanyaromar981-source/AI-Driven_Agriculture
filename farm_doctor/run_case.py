"""One end-to-end case: a field (lon lat) on a date -> all AIs -> the Doctor. Writes cases/<date>_<lon>_<lat>.json (the case log).
Usage: python3 -I run_case.py <lon> <lat> <YYYY-MM-DD> ["farmer question"] [photo.jpg ...]"""
import os, sys, json, time, datetime as dt
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
import field_eye, weather_planner, season_check, neighbour_watch, dam_watch, doctor
def run(lon, lat, date, question=None, photos=None):
    t0 = time.time(); inputs = {}
    for name, fn in [('field_eye', lambda: field_eye.measure(lon, lat, date)), ('weather_planner', lambda: weather_planner.plan(lon, lat, date)),
                     ('season_check', lambda: season_check.check(lon, lat, date)), ('neighbour_watch', lambda: neighbour_watch.nearby(lon, lat, date)),
                     ('dam_watch', lambda: dam_watch.latest(date))]:
        t = time.time()
        try: inputs[name] = fn()
        except Exception as e: inputs[name] = dict(ai=name, error=str(e)[:160])
        inputs[name]['_seconds'] = round(time.time() - t, 1); print(f'{name:16s} {inputs[name]["_seconds"]:5.1f} s', flush=True)
    inputs['photos_given'] = len(photos or [])
    answer = doctor.ask(inputs, question, photos); print(f'doctor           {round(time.time() - t0, 1):5.1f} s total', flush=True)
    case = dict(lon=lon, lat=lat, date=date, question=question, inputs=inputs, answer=answer, run_at=dt.datetime.now().isoformat(timespec='seconds'))
    os.makedirs(os.path.join(HERE, 'cases'), exist_ok=True)
    out = os.path.join(HERE, 'cases', f'{date}_{lon}_{lat}.json'); json.dump(case, open(out, 'w'), indent=1, ensure_ascii=False)
    return case
if __name__ == '__main__':
    lon, lat, date = float(sys.argv[1]), float(sys.argv[2]), sys.argv[3]
    q = sys.argv[4] if len(sys.argv) > 4 else None; photos = sys.argv[5:] or None
    c = run(lon, lat, date, q, photos)
    print(json.dumps(c['answer'], indent=1, ensure_ascii=False))
