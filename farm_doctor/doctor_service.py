"""The Doctor as a small local web service. The Rust backend checks the farmer and the farm, then posts here (127.0.0.1 only).
This gathers this week's inputs for the farm, asks the Doctor (doctor.py) and returns the answer in the app's shape.

POST /ask  {"farm": {"id", "name", "lat", "lon", "area_m2", "crops"}, "history": {"topics": [...]} | null,
            "question": str | null, "cell": {"e", "n"} | null, "lang": "ku" | "en", "photos": [{"mime", "data" (base64)}]}
  200 {"likely", "confidence": "sure|likely|unsure", "why", "actions_this_week" (max 3), "cannot_tell", "refer_to_officer",
       "ku", "en", "inputs_used"}
  503 {"error": "doctor_not_ready"} when no AI key is set; 502 {"error": "doctor_failed"} when the model fails; 400 bad body.
GET /health  200 {"ok": true, "key": true|false}

Run: python3 -I doctor_service.py   (DOCTOR_PORT, default 8090). Stdlib only. Keys as in doctor.py (farm_doctor/.env)."""
import os, sys, json, time, datetime as dt, concurrent.futures as cf
from http.server import ThreadingHTTPServer, BaseHTTPRequestHandler
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
import field_eye, weather_planner, season_check, dam_watch, doctor

MAX_BODY = 30 * 1024 * 1024
# name -> (call, seconds to wait). Field eye reads Sentinel-2 and is the slow one.
SOURCES = {
    'field_eye': (lambda lon, lat, day: field_eye.measure(lon, lat, day), 35),
    'weather_planner': (lambda lon, lat, day: weather_planner.plan(lon, lat), 25),
    'season_check': (lambda lon, lat, day: season_check.check(lon, lat, day), 10),
    'dam_watch': (lambda lon, lat, day: dam_watch.latest(day), 10),
}
# reports.json is test data, so the Doctor is told reports are not collected yet rather than shown made-up ones.
NO_REPORTS = dict(ai='neighbour_watch', note='Farmer reports are not collected yet. Say nothing about what neighbours reported.')


def gather(lon, lat, day):
    """Runs every source at once; a source that fails or is too slow is passed on as an error, never guessed."""
    out = {}
    def timed(fn):
        t = time.time(); r = fn(lon, lat, day); r['_seconds'] = round(time.time() - t, 1); return r
    ex = cf.ThreadPoolExecutor(max_workers=len(SOURCES))
    t0 = time.time()
    futs = {name: (ex.submit(timed, fn), wait) for name, (fn, wait) in SOURCES.items()}
    for name, (fut, wait) in futs.items():
        try:
            out[name] = fut.result(timeout=max(0.1, wait - (time.time() - t0)))
        except cf.TimeoutError:
            out[name] = dict(ai=name, error=f'no answer within {wait} s')
        except Exception as e:
            out[name] = dict(ai=name, error=str(e)[:160])
    ex.shutdown(wait=False, cancel_futures=True)
    out['neighbour_watch'] = NO_REPORTS
    return out


def slim_history(history):
    """Keeps what the Doctor needs from the farm's 20-year history: each topic's text, source, confidence and numbers."""
    if not isinstance(history, dict):
        return None
    keep = ('topic', 'as_of', 'source', 'confidence', 'summary_en')
    topics = []
    for t in history.get('topics') or []:
        s = {k: t.get(k) for k in keep if t.get(k) is not None}
        s['measures'] = {m.get('code'): m.get('value') for m in t.get('measures') or [] if m.get('code')}
        topics.append(s)
    return topics or None


def shape(a, inputs):
    """The model's JSON in the app's shape. Anything missing becomes empty, never invented."""
    conf = a.get('confidence') if a.get('confidence') in ('sure', 'likely', 'unsure') else 'unsure'
    lst = lambda k: [str(x) for x in (a.get(k) or []) if str(x).strip()] if isinstance(a.get(k), list) else []
    used = [k for k, v in inputs.items() if k in SOURCES and isinstance(v, dict) and 'error' not in v]
    if inputs.get('field_history'):
        used.append('field_history')
    if inputs.get('photos_given'):
        used.append('photos')
    return dict(likely=str(a.get('likely') or ''), confidence=conf, why=lst('why'), actions_this_week=lst('actions_this_week')[:3],
                cannot_tell=lst('cannot_tell'), refer_to_officer=bool(a.get('refer_to_officer')) or conf == 'unsure',
                ku=str(a.get('sorani') or ''), en=str(a.get('english') or ''), inputs_used=used)


def has_key():
    return bool(doctor._env('GEMINI_API_KEY' if doctor._provider() == 'gemini' else 'ANTHROPIC_API_KEY'))


def answer(body):
    """Returns (status, json)."""
    farm = body.get('farm') or {}
    try:
        lon, lat = float(farm['lon']), float(farm['lat'])
    except (KeyError, TypeError, ValueError):
        return 400, dict(error='bad_request', detail='farm.lat and farm.lon are required')
    if not has_key():
        return 503, dict(error='doctor_not_ready')
    photos = [p for p in body.get('photos') or [] if isinstance(p, dict) and p.get('data')][:6]
    question = (body.get('question') or '').strip() or None
    day = dt.date.today().isoformat()
    t0 = time.time()
    inputs = gather(lon, lat, day)
    inputs['farm'] = {k: farm.get(k) for k in ('name', 'area_m2', 'crops') if farm.get(k) is not None}
    inputs['field_history'] = slim_history(body.get('history'))
    if body.get('cell'):
        inputs['farmer_tapped_cell'] = body['cell']
    inputs['photos_given'] = len(photos)
    inputs['farmer_language'] = 'Sorani' if body.get('lang', 'ku') == 'ku' else 'English'
    a = doctor.ask(inputs, question, photos)
    secs = round(time.time() - t0, 1)
    if 'error' in a:
        print(f'doctor error after {secs} s: {a["error"][:200]}', flush=True)
        return (503, dict(error='doctor_not_ready')) if a['error'].startswith('no API key') else (502, dict(error='doctor_failed'))
    if 'raw' in a:
        print(f'doctor answered without JSON after {secs} s', flush=True)
        return 502, dict(error='doctor_failed')
    out = shape(a, inputs)
    print(f'farm {farm.get("id")}: {out["confidence"]}, {len(photos)} photos, {secs} s, used {",".join(out["inputs_used"])}', flush=True)
    try:   # case log for checking answers later; no photos, git-ignored (holds field locations)
        os.makedirs(os.path.join(HERE, 'cases'), exist_ok=True)
        name = dt.datetime.now().strftime('%Y-%m-%d_%H%M%S') + f'_farm{farm.get("id")}.json'
        json.dump(dict(farm=farm, question=question, inputs=inputs, answer=a, seconds=secs),
                  open(os.path.join(HERE, 'cases', name), 'w'), indent=1, ensure_ascii=False)
    except Exception as e:
        print(f'case log failed: {e}', flush=True)
    return 200, out


class Handler(BaseHTTPRequestHandler):
    def _send(self, status, obj):
        data = json.dumps(obj, ensure_ascii=False).encode()
        self.send_response(status)
        self.send_header('content-type', 'application/json; charset=utf-8')
        self.send_header('content-length', str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        if self.path == '/health':
            return self._send(200, dict(ok=True, key=has_key()))
        self._send(404, dict(error='not_found'))

    def do_POST(self):
        if self.path != '/ask':
            return self._send(404, dict(error='not_found'))
        n = int(self.headers.get('content-length') or 0)
        if n <= 0 or n > MAX_BODY:
            return self._send(400, dict(error='bad_request', detail='body missing or too big'))
        try:
            body = json.loads(self.rfile.read(n))
        except Exception:
            return self._send(400, dict(error='bad_request', detail='body is not JSON'))
        try:
            status, out = answer(body)
        except Exception as e:
            print(f'doctor service fault: {e!r}', flush=True)
            status, out = 502, dict(error='doctor_failed')
        self._send(status, out)

    def log_message(self, fmt, *args):   # one line per request, no bodies (they hold photos and questions)
        print(f'{self.command} {self.path} {args[1] if len(args) > 1 else ""}', flush=True)


if __name__ == '__main__':
    port = int(os.environ.get('DOCTOR_PORT', '8090'))
    print(f'doctor service on 127.0.0.1:{port}', flush=True)
    ThreadingHTTPServer(('127.0.0.1', port), Handler).serve_forever()
