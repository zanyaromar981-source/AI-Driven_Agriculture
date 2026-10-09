"""Rebuild the Jutyar Control Room website (web/admin_demo) as a .pen file, measured from the live page.

Usage:  python build_web_pen.py            (needs Microsoft Edge and: pip install websocket-client)
Writes: design/web/jutyar_control_room.pen and design/web/assets/*.png

Each frame is the real page converted by dom2pen.js: every box, text, icon and input at the exact
position, size, colour, font, radius, border and shadow the browser measured. Maps, satellite pictures
and the sun logo are saved as images. Animations are captured by pausing them at a chosen moment.
"""
import base64, json, os, subprocess, sys, time, urllib.request
import websocket

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
PAGE = 'file:///' + os.path.join(REPO, 'web', 'admin_demo', 'index.html').replace('\\', '/')
OUT = os.path.join(HERE, 'jutyar_control_room.pen')
ASSETS = os.path.join(HERE, 'assets')
EDGE = next(p for p in [r'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe', r'C:\Program Files\Microsoft\Edge\Application\msedge.exe'] if os.path.exists(p))
PORT = 9334
W = 1440

PAUSE = "document.getAnimations().forEach(a => a.pause());"
def when(cond, after=0, extra=''):
    """JS: wait until cond is true, wait `after` ms, pause every animation."""
    return f"new Promise(r => {{ const t0 = Date.now(); const i = setInterval(() => {{ if (({cond}) || Date.now() - t0 > 9000) {{ clearInterval(i); setTimeout(() => {{ {extra} {PAUSE} r('ok'); }}, {after}); }} }}, 6); }})"

ROUTES = [('overview', 'Overview'), ('approvals', 'Approvals'), ('farms', 'Farmers and farms'), ('officers', 'Officers and roles'), ('crops', 'Crop register'),
          ('region', 'Region data'), ('alerts', 'Alerts'), ('inbox', 'Inbox'), ('doctor', 'The Doctor (AI)'), ('alwa', 'Alwa market'), ('rules', 'Rules'),
          ('app', 'App control'), ('notify', 'Notifications and SMS'), ('texts', 'Texts and languages'), ('jobs', 'Data jobs'), ('db', 'Database'),
          ('security', 'Security and privacy'), ('audit', 'History'), ('system', 'System settings')]

# (group, name, url query, js to reach the state, full page?, caption)
SPECS = [
    ('A', 'A1 · Preloader · the logo draws itself', '?pre=8', when("true", 520), False,
     'Curtain (2.1) in the deep app green. The Grain Sun app logo draws its 21 rays as gold lines (2.4, SVG stroke), the wordmark starts thin (1.4), the counter starts at 00 (2.3), the green bar at the very top fills with real loading (4.3), the bottom marquee runs (1.5).'),
    ('A', 'A2 · Preloader · words and counter', '?pre=54', when("true", 1450, "const w = document.querySelector('#preSay'); w.textContent = 'Counting 2,031 farms'; w.classList.remove('flick');"), False,
     'The rays fill with gold and slowly turn. Branded words flick by every 0.42 s instead of a spinner (2.2): Reading the fields from space, Counting 2,031 farms, Checking Dukan and Darbandikhan, Asking the weather, Waking the Doctor. Jutyar thickens from weight 200 to 800 (1.4). The marquee speeds up as loading goes on (1.5).'),
    ('A', 'A3 · Preloader · 100', '?pre=100', when("true", 2700, "const w = document.querySelector('#preSay'); w.textContent = 'Ready'; w.classList.remove('flick');"), False,
     'The counter reaches 100 and grows to 1.55 times its size near the end (2.3). Loading waits for the fonts and the data, and never more than 4 seconds. Any key skips it.'),
    ('A', 'A4 · Curtain lift', '', when("document.querySelector('#pre').classList.contains('lift')", 430), False,
     'The whole curtain slides up off the screen in 0.9 s, cubic-bezier(.76, 0, .24, 1) (2.1). Under it, a 6 by 4 grid of green and earth blocks still covers the page.'),
    ('A', 'A5 · Bento blinds open', '', when("document.querySelector('#bento').classList.contains('open')", 330), False,
     'The grid blocks flip open like window blinds, staggered 55 ms per step from the top left (2.5), showing the control room underneath.'),
    ('A', 'A6 · Cascade, typing and scramble', '', when("document.querySelector('#bento').classList.contains('gone')", 40), False,
     'Cards fade in and rise 18 px one after another, 55 ms apart (3.3). The greeting types letter by letter with a gold caret (1.1). Eyebrow and subtitle slide up from behind a mask (1.3). The four numbers decode from random symbols into the real value (1.2).'),
] + [('B', f'B{i + 1:02d} · {t}', f'?static=1#{k}', None, True, '') for i, (k, t) in enumerate(ROUTES)] + [
    ('C', 'C1 · Skeleton screen', '?static=1&skeleton=1#farms', None, False,
     'Every page change first shows grey blocks shaped like the page that is coming (table rows, cards, map), with a light wave moving across them (3.1). The green bar at the top shows the progress (4.3); the live marquee runs faster while loading (1.5).'),
    ('C', 'C2 · Title decrypt', '#farms', "new Promise(r => { const i = setInterval(() => { if (window.Motion && Motion.ready) { clearInterval(i); location.hash = '#rules'; location.hash = '#farms'; const j = setInterval(() => { const h = document.querySelector('#main h1'); if (h && h.classList.contains('scrambling')) { clearInterval(j); setTimeout(() => { " + PAUSE + " r('ok'); }, 110); } }, 5); } }, 20); })", False,
     'The page title arrives as scrambled symbols and settles letter by letter from left to right in 0.65 s (1.2), while its weight grows from 250 to 800 (1.4). The numbers do the same 0.1 s apart.'),
    ('C', 'C3 · Blur-up, low resolution', '?static=1#inbox', "new Promise(r => setTimeout(() => { document.querySelector('#inboxSat .hi').style.opacity = 0; r('ok'); }, 2500))", False,
     'Satellite pictures first show a tiny low-zoom tile, blurred 14 px (3.2)...'),
    ('C', 'C4 · Blur-up, sharp', '?static=1#inbox', "new Promise(r => setTimeout(() => r('ok'), 2500))", False,
     '...then the sharp tiles fade in over 0.9 s once they arrive. The same blur-to-sharp is used on every map tile.'),
    ('C', 'C5 · Cursor glow and magnetic button', '', "new Promise(r => { const i = setInterval(() => { if (window.Motion && Motion.ready && document.querySelector('#heroSoft .hi.ready')) { clearInterval(i); setTimeout(() => { const btn = document.querySelector('.hero .btn.primary').getBoundingClientRect(); const x = btn.right + 26, y = btn.top + btn.height / 2 - 4; let k = 0; const mv = setInterval(() => { const px = x - 60 + k * 6, el = document.elementFromPoint(px, y); el && el.dispatchEvent(new MouseEvent('mousemove', { clientX: px, clientY: y, bubbles: true })); if (++k > 10) { clearInterval(mv); setTimeout(() => { " + PAUSE + " r('ok'); }, 700); } }, 40); }, 1200); } }, 30); })", False,
     'The custom cursor: a gold dot with a ring that follows softly and grows over anything clickable (4.1). Over the satellite banner the picture becomes sharp in a circle around the cursor, the rest stays blurred; cards light up under the cursor. Main buttons pull toward the cursor when it comes within 46 px (4.2).'),
    ('C', 'C6 · Scroll progress and live marquee', '?static=1#farms', "new Promise(r => setTimeout(() => { window.scrollTo(0, 900); window.dispatchEvent(new Event('scroll')); r('ok'); }, 1200))", False,
     'A 2 px gold line at the very top shows how far down the page you are (4.3). The live news strip under the header moves on its own and runs faster while you scroll (1.5).'),
    ('C', 'C7 · Farm drawer', '?static=1#farms', "new Promise(r => setTimeout(() => { A.farm('1207'); r('ok'); }, 900))", False, ''),
    ('C', 'C8 · Show a phone, with a reason', '?static=1#farms', "new Promise(r => setTimeout(() => { A.farm('1207'); A.reveal('1207'); r('ok'); }, 900))", False, ''),
    ('C', 'C9 · Locked for this role', '?static=1#overview', "new Promise(r => setTimeout(() => { A.who(5); location.hash = '#app'; setTimeout(() => r('ok'), 300); }, 900))", False, ''),
    ('C', 'C10 · Search', '?static=1#overview', "new Promise(r => setTimeout(() => { openSearch(); const g = document.querySelector('#gs'); g.value = 'fro'; g.oninput(); r('ok'); }, 1500))", False, ''),
]
CAPTURE_HERO = {'C5 · Cursor glow and magnetic button'}

class Browser:
    def __init__(self):
        prof = os.path.join(os.environ.get('TEMP', HERE), 'jutyar_pen_edge')
        self.p = subprocess.Popen([EDGE, '--headless=new', '--disable-gpu', '--hide-scrollbars', f'--remote-debugging-port={PORT}',
                                   f'--remote-allow-origins=http://127.0.0.1:{PORT}', f'--user-data-dir={prof}', f'--window-size={W},1000', 'about:blank'],
                                  stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        for _ in range(100):
            try:
                tabs = json.load(urllib.request.urlopen(f'http://127.0.0.1:{PORT}/json')); page = next(t for t in tabs if t['type'] == 'page'); break
            except Exception:
                time.sleep(0.2)
        self.ws = websocket.create_connection(page['webSocketDebuggerUrl'], timeout=120); self.n = 0
        self.call('Page.enable'); self.call('Runtime.enable')
    def call(self, method, **params):
        self.n += 1; self.ws.send(json.dumps({'id': self.n, 'method': method, 'params': params}))
        while True:
            m = json.loads(self.ws.recv())
            if m.get('id') == self.n:
                if 'error' in m: raise RuntimeError(f'{method}: {m["error"]}')
                return m.get('result', {})
    def js(self, expr):
        r = self.call('Runtime.evaluate', expression=expr, returnByValue=True, awaitPromise=True)
        if 'exceptionDetails' in r: raise RuntimeError(r['exceptionDetails'].get('exception', {}).get('description', r['exceptionDetails']))
        return r['result'].get('value')
    def size(self, w, h): self.call('Emulation.setDeviceMetricsOverride', width=w, height=h, deviceScaleFactor=1, mobile=False)
    def close(self): self.p.kill()

CAP_CSS = ("html.cap, html.cap body { background: transparent !important; } html.cap * { visibility: hidden !important; } "
           "html.cap [data-cap], html.cap [data-cap] * { visibility: visible !important; }")

def build():
    os.makedirs(ASSETS, exist_ok=True)
    if not sys.argv[1:]:
        for f in os.listdir(ASSETS):
            if f.endswith('.png'): os.remove(os.path.join(ASSETS, f))
    conv = open(os.path.join(HERE, 'dom2pen.js'), encoding='utf-8').read()
    b = Browser(); frames = {}
    try:
        only = sys.argv[1:]
        for group, name, query, prep, full, caption in SPECS:
            if only and not any(name.startswith(o + ' ') for o in only): continue
            b.size(W, 900)
            b.call('Page.navigate', url=PAGE + query); time.sleep(0.4)
            b.js("new Promise(r => { if (document.readyState === 'complete') r(1); else addEventListener('load', () => r(1)); })")
            if prep: b.js(prep)
            else: time.sleep(2.4 if 'static' in query else 0.2)
            if full:
                h = b.js('document.documentElement.scrollHeight'); b.size(W, max(900, h)); time.sleep(1.6)
            b.js(f'window.__viewport = {"false" if full else "true"}')
            if name in CAPTURE_HERO:
                b.js("document.querySelector('.hero').classList.add('cap-hero')")
                conv_run = conv.replace("const RASTER = '", "const RASTER = '.hero, ").replace("#heroSharp, .cursor-dot, .cursor-ring, ", "")
                r = b.js(conv_run + f'; window.dom2pen({json.dumps(name)})')
            else:
                r = b.js(conv + f'; window.dom2pen({json.dumps(name)})')
            node = r['node']
            byid = {}
            def index(n):
                byid[n['id']] = n
                for c in n.get('children', []): index(c)
            index(node)
            slug = name.split(' · ')[0].lower()
            b.call('Emulation.setDefaultBackgroundColorOverride', color={'r': 0, 'g': 0, 'b': 0, 'a': 0})
            for ras in r['rasters']:
                b.js(f"(() => {{ let s = document.getElementById('capcss'); if (!s) {{ s = document.createElement('style'); s.id = 'capcss'; s.textContent = {json.dumps(CAP_CSS)}; document.head.append(s); }} document.documentElement.classList.add('cap'); document.querySelector('[data-capid={ras['key']}]').setAttribute('data-cap', ''); }})()")
                rc = ras['rect']
                d = b.call('Page.captureScreenshot', format='png', captureBeyondViewport=True, clip={'x': rc['x'], 'y': rc['y'], 'width': rc['width'], 'height': rc['height'], 'scale': 2})
                fn = f'{slug}_{ras["key"]}.png'
                open(os.path.join(ASSETS, fn), 'wb').write(base64.b64decode(d['data']))
                byid[ras['node']]['fill']['url'] = 'assets/' + fn
                b.js(f"document.documentElement.classList.remove('cap'); document.querySelector('[data-capid={ras['key']}]').removeAttribute('data-cap')")
            b.call('Emulation.setDefaultBackgroundColorOverride')
            frames[name] = (group, node, caption)
            print(f'{name}: {count(node)} layers, {len(r["rasters"])} images', flush=True)
    finally:
        b.close()
    return frames

def count(n): return 1 + sum(count(c) for c in n.get('children', []))

# ---------- the motion spec sheet, native .pen layers ----------
SPEC_ROWS = [
    ('1.1', 'Letter-by-letter reveal', 'The greeting on Overview', 'Types 42 to 82 ms per letter with a blinking gold caret, which leaves 1.6 s after the last letter', 'A6'),
    ('1.2', 'Line-by-line decrypt', 'Every page title and every big number', 'Random symbols settle into the real text from left to right in 0.65 s; numbers 0.1 s apart', 'A6, C2'),
    ('1.3', 'Text mask reveal', 'Eyebrow and subtitle of each page', 'Each line slides up 0.7 em from behind its own edge in 0.8 s, 90 ms apart', 'A6'),
    ('1.4', 'Variable font weight', 'The Jutyar wordmark and page titles', 'Manrope as a variable font: weight 200 to 800 and spacing .06 to -.02 em in 1.6 s; titles 250 to 800 in 0.9 s', 'A1, A2, C2'),
    ('1.5', 'Kinetic marquee', 'Bottom of the preloader; the live news strip', 'Moves by itself; speed grows with loading (0.6 to 5.6 px a frame) and with scrolling', 'A1, C6'),
    ('2.1', 'Curtain lift', 'The preloader', 'Deep green panel with the logo slides up in 0.9 s, cubic-bezier(.76, 0, .24, 1)', 'A4'),
    ('2.2', 'Branded words', 'Under the wordmark', 'Seven farm words flick up every 0.42 s instead of a spinner', 'A2'),
    ('2.3', 'Counter 00 to 100', 'Bottom right of the preloader', 'Follows real loading (fonts, data, never more than 4 s); grows to 1.55 times near 100', 'A1, A2, A3'),
    ('2.4', 'Line draw (SVG stroke)', 'The Grain Sun app logo', '21 rays and the centre draw as gold lines in 0.9 s, 32 ms apart, then fill and turn slowly', 'A1, A2'),
    ('2.5', 'Bento grid blinds', 'Under the curtain', '24 blocks flip open from the top left, 55 ms per step, 0.75 s each', 'A5'),
    ('3.1', 'Skeleton shimmer', 'Every page change', 'Grey blocks shaped like the coming page (rows, cards, map) with a moving light wave, 0.46 s', 'C1'),
    ('3.2', 'Blur-up images', 'Satellite banner, inbox farm picture, every map tile', 'Low-zoom tile blurred 14 px first, sharp tiles fade in over 0.9 s', 'C3, C4'),
    ('3.3', 'Staggered cascade', 'Cards, rows, filters on each page', 'Fade in and rise 18 px, 55 ms apart, top to bottom', 'A6'),
    ('4.1', 'Mouse-follower glow', 'Custom cursor, satellite banner, cards', 'Gold dot and a soft ring that grows over clickable things; the banner unblurs around the cursor; cards glow', 'C5'),
    ('4.2', 'Magnetic buttons', 'Main buttons and the approvals button', 'Pull up to 28% toward the cursor within 46 px of their edge, spring back when it leaves', 'C5'),
    ('4.3', 'Passive progress bars', 'The very top of the window', 'Green: loading of the app and of each page. Gold, 2 px: how far down the page you are', 'A1, C1, C6'),
]
def T(content, size=14, weight='500', fill='$ink', width=None, font='$font-lat', **kw):
    n = {'type': 'text', 'content': content, 'fontSize': size, 'fontWeight': weight, 'fill': fill, 'fontFamily': font}
    if width is not None: n['textGrowth'] = 'fixed-width'; n['width'] = width
    n.update(kw); return n
def spec_sheet():
    rows = []
    for i, (num, title, where, how, ref) in enumerate(SPEC_ROWS):
        rows.append({'type': 'frame', 'name': 'Row ' + num, 'width': 'fill_container', 'gap': 20, 'padding': [14, 18], 'alignItems': 'center',
                     'fill': '#FFFFFF' if i % 2 == 0 else '#F7F8F4', 'children': [
                         T(num, 14, '800', '$gold', 44), T(title, 15, '700', '$ink', 230), T(where, 14, '500', '$muted', 280),
                         T(how, 14, '500', '$ink', 'fill_container'), T(ref, 13, '700', '$accent', 90)]})
    head = {'type': 'frame', 'name': 'Head', 'width': 'fill_container', 'layout': 'vertical', 'gap': 8, 'children': [
        T('JUTYAR CONTROL ROOM · MOTION', 13, '800', '$gold', letterSpacing=1.6),
        T('Every load and interaction detail, where it lives and how it moves', 34, '800', '$ink', 1200, letterSpacing=-0.5),
        T('Frames A1 to A6 show the opening sequence frozen at key moments, C1 to C6 the effects on each page change and under the cursor. '
          'Everything is real code in web/admin_demo (motion.js, motion.css) and switches off for visitors who ask for reduced motion.', 15, '500', '$muted', 1200, lineHeight=1.5)]}
    hdr = {'type': 'frame', 'name': 'Columns', 'width': 'fill_container', 'gap': 20, 'padding': [10, 18], 'children': [
        T('#', 12, '700', '$muted', 44), T('DETAIL', 12, '700', '$muted', 230), T('WHERE IN JUTYAR', 12, '700', '$muted', 280),
        T('HOW IT MOVES', 12, '700', '$muted', 'fill_container'), T('FRAMES', 12, '700', '$muted', 90)]}
    table = {'type': 'frame', 'name': 'Spec table', 'width': 'fill_container', 'layout': 'vertical', 'cornerRadius': 16, 'clip': True,
             'stroke': '$line', 'strokeWidth': 1, 'children': [hdr] + rows}
    return {'type': 'frame', 'name': 'A0 · Motion spec', 'width': W, 'layout': 'vertical', 'gap': 28, 'padding': [56, 64], 'fill': '$bg', 'children': [head, table]}

def caption(text, width):
    return {'type': 'frame', 'name': 'Caption', 'width': width, 'layout': 'vertical', 'padding': [18, 22], 'fill': '#FFFFFF', 'cornerRadius': 14,
            'stroke': '$line', 'strokeWidth': 1, 'children': [T(text, 17, '500', '$ink', 'fill_container', lineHeight=1.5)]}

def ids(n, seen):
    import random, string
    if 'id' not in n or n['id'] in seen:
        while True:
            s = ''.join(random.choice(string.ascii_letters + string.digits) for _ in range(6))
            if s not in seen: break
        n['id'] = s
    seen.add(n['id'])
    for c in n.get('children', []): ids(c, seen)

def main():
    frames = build()
    doc = json.load(open(OUT, encoding='utf-8')) if os.path.exists(OUT) else {'version': '2.20', 'variables': {}, 'children': []}
    doc['children'] = []
    sheet = spec_sheet(); sheet['x'] = 0; sheet['y'] = 0; doc['children'].append(sheet)
    col = {'A': 0, 'B': 0, 'C': 0}
    for name, (group, node, cap) in frames.items():
        if group == 'A':
            node['x'] = 1640 + col['A'] * 1540; node['y'] = 0; col['A'] += 1
        elif group == 'B':
            k = col['B']; node['x'] = (k % 5) * 1540; node['y'] = 1700 + (k // 5) * 3400; col['B'] += 1
        else:
            k = col['C']; node['x'] = (k % 5) * 1540; node['y'] = 1700 + 4 * 3400 + (k // 5) * 1400; col['C'] += 1
        doc['children'].append(node)
        if cap:
            c = caption(f'{name}. {cap}', W); c['x'] = node['x']; c['y'] = node['y'] + node['height'] + 24; doc['children'].append(c)
    seen = set()
    for n in doc['children']: ids(n, seen)
    json.dump(doc, open(OUT, 'w', encoding='utf-8'), ensure_ascii=False, indent=1)
    print('wrote', OUT, len(doc['children']), 'top-level frames', os.path.getsize(OUT) // 1024, 'kB')

if __name__ == '__main__':
    main()
