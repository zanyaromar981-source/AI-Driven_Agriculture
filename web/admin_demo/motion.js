'use strict';
/* Jutyar Control Room motion: the preloader, text effects, skeletons, cascade, blur-up images,
   cursor, magnetic buttons, marquees and progress bars. Off when the visitor asks for reduced motion.
   Test hooks: ?pre=40 freezes the preloader at 40%, ?skeleton=farms freezes a skeleton screen. */
(function () {
  const q = new URLSearchParams(location.search);
  const still = q.has('static');
  if (still) document.documentElement.classList.add('static');
  const reduce = still || matchMedia('(prefers-reduced-motion: reduce)').matches;
  const fine = matchMedia('(pointer: fine)').matches;
  const $ = s => document.querySelector(s);
  const M = { reduce, ready: false, appReady: false, isLoading: true };

  // ---------- Grain Sun (the app logo: 21 grain rays around a centre, app/lib/widgets/grain_sun.dart) ----------
  M.sun = (size, mode = 'solid') => {
    let rays = '';
    for (let i = 0; i < 21; i++) rays += `<path class="ray" style="--i:${i}" d="M50 30A30 30 0 0 1 50 7A30 30 0 0 1 50 30Z" transform="rotate(${(i * 360 / 21).toFixed(3)} 50 50)"/>`;
    return `<svg class="sun ${mode}" width="${size}" height="${size}" viewBox="0 0 100 100"><g class="rays">${rays}</g><circle class="core" cx="50" cy="50" r="14"/></svg>`;
  };

  // ---------- passive progress bars ----------
  let loadP = 0;
  M.progress = v => { loadP = v; const b = $('#loadBar'); if (b) { b.style.width = v + '%'; b.style.opacity = v >= 100 ? 0 : 1; } };
  M.loading = on => {
    M.isLoading = on;
    if (on) { M.progress(8); clearInterval(M._li); M._li = setInterval(() => M.progress(Math.min(90, loadP + (90 - loadP) * 0.18)), 60); }
    else { clearInterval(M._li); M.progress(100); setTimeout(() => M.progress(0), 450); }
  };
  addEventListener('scroll', () => {
    const h = document.documentElement.scrollHeight - innerHeight;
    $('#scrollBar').style.width = (h > 0 ? scrollY / h * 100 : 0) + '%';
  }, { passive: true });

  // ---------- marquees ----------
  const tracks = [];
  let scrollBoost = 0, lastY = scrollY;
  addEventListener('scroll', () => { scrollBoost = Math.min(14, scrollBoost + Math.abs(scrollY - lastY) * 0.08); lastY = scrollY; }, { passive: true });
  function marquee(el, html, speedFn) {
    el.innerHTML = html + html; const t = { el, x: 0, speedFn }; tracks.push(t); return t;
  }
  function tick() {
    for (const t of tracks) {
      const w = t.el.scrollWidth / 2; if (!w) continue;
      t.x -= t.speedFn(); if (-t.x >= w) t.x += w;
      t.el.style.transform = `translate3d(${t.x}px,0,0)`;
    }
    scrollBoost *= 0.92;
    requestAnimationFrame(tick);
  }
  if (!reduce) requestAnimationFrame(tick);
  M.ticker = items => {
    const html = items.map(([ic, txt]) => `<span class="mq-item"><span class="sep">${ic}</span>${txt}</span>`).join('');
    marquee($('#ticker'), html, () => 0.5 + scrollBoost + (M.isLoading ? 3 : 0));
  };

  // ---------- preloader ----------
  const WORDS = ['Waking up', 'Reading the fields from space', 'Counting 2,031 farms', 'Checking Dukan and Darbandikhan', 'Asking the weather for 10 days', 'Waking the Doctor', 'Opening the control room'];
  M.preload = () => new Promise(resolve => {
    const pre = $('#pre'), bento = $('#bento');
    const finish = () => { pre.classList.add('gone'); bento.classList.add('gone'); document.body.classList.remove('loading'); M.ready = true; M.isLoading = false; resolve(); };
    if (reduce) { M.progress(0); finish(); return; }
    $('#preSun').innerHTML = M.sun(148, 'draw');
    bento.innerHTML = Array.from({ length: 24 }, (_, i) => `<i style="--d:${(i % 6) + Math.floor(i / 6)}"></i>`).join('');
    const word = 'JUTYAR <span style="color:#E8C77A">·</span> جوتیار <span style="color:#E8C77A">·</span> FIELDS · WATER · WHEAT · BARLEY · DAMS · WEATHER · ';
    let p = 0;
    marquee($('#preMarquee'), `<span>${word}</span>`, () => 0.6 + p / 100 * 5);
    const freeze = q.get('pre');
    let w = 0; const say = $('#preSay');
    const wi = setInterval(() => { w = (w + 1) % WORDS.length; say.textContent = WORDS[w]; say.classList.remove('flick'); void say.offsetWidth; say.classList.add('flick'); }, 420);
    let fontsOk = false; document.fonts && document.fonts.ready.then(() => { fontsOk = true; });
    const t0 = performance.now(), count = $('#preCount');
    let skip = false; const onSkip = () => { skip = true; };
    addEventListener('keydown', onSkip, { once: true }); pre.addEventListener('click', onSkip, { once: true });
    // a timer, not requestAnimationFrame, so loading never stalls in a hidden tab
    const iv = setInterval(() => step(performance.now()), 16);
    function step(now) {
      const t = Math.min(1, (now - t0) / 2600);
      let target = 100 * (1 - Math.pow(1 - t, 2.2));
      const late = now - t0 > 4000; // never wait longer than 4 s for fonts or data
      if (!fontsOk && !late) target = Math.min(target, 72);
      if (!M.appReady && !late) target = Math.min(target, 88);
      if (late && !freeze) target = 100;
      if (skip) target = 100;
      if (freeze && !isNaN(+freeze)) target = +freeze;
      const dt = now - (step.last || now); step.last = now; // follow time, not ticks: throttled timers still finish
      p += (target - p) * (1 - Math.exp(-dt / (skip ? 40 : 140))); if (target - p < 0.4) p = target;
      count.textContent = String(Math.floor(p)).padStart(2, '0');
      count.parentElement.style.transform = `scale(${1 + Math.pow(p / 100, 3) * 0.55})`;
      M.progress(p);
      if (freeze && !isNaN(+freeze)) return;
      if (p < 100) return;
      clearInterval(iv); clearInterval(wi); say.textContent = 'Ready'; M.progress(100); setTimeout(() => M.progress(0), 500);
      setTimeout(() => {
        pre.classList.add('lift');
        setTimeout(() => { bento.classList.add('open'); setTimeout(resolve, 250); M.ready = true; M.isLoading = false; document.body.classList.remove('loading'); }, 620);
        setTimeout(() => { pre.classList.add('gone'); bento.classList.add('gone'); }, 1900);
      }, 260);
    }
  });

  // ---------- skeleton screens that mirror each layout ----------
  const lines = (n, ws = ['w90', 'w60', 'w80']) => Array.from({ length: n }, (_, i) => `<i class="sk ${ws[i % ws.length]}"></i>`).join('');
  const headSk = `<div class="page-head"><div class="t"><i class="sk w20"></i><i class="sk h28 w40"></i><i class="sk w60"></i></div></div>`;
  const kpiSk = `<div class="grid g4" style="margin-bottom:14px">${'<div class="card"><i class="sk w40"></i><i class="sk h28 w30"></i><i class="sk w60"></i></div>'.repeat(4)}</div>`;
  const rowsSk = n => `<div class="card">${Array.from({ length: n }, () => `<div class="sk-row"><i class="sk circle"></i><i class="sk h14"></i><i class="sk"></i><i class="sk" style="flex:.5"></i><i class="sk" style="flex:.4"></i></div>`).join('')}</div>`;
  M.skeleton = kind => {
    if (kind === 'overview') return `<div class="sk block" style="min-height:230px;margin-bottom:14px"></div>${kpiSk}<div class="grid g-main-l"><div class="card">${lines(9)}</div><div class="card"><i class="sk block" style="min-height:360px"></i></div></div>`;
    if (kind === 'table') return headSk + `<div class="row" style="margin-bottom:12px"><i class="sk h28" style="flex:1"></i><i class="sk h28 w20"></i><i class="sk h28 w20"></i></div>` + rowsSk(10);
    if (kind === 'map') return headSk + `<div class="grid g-main"><div class="card"><i class="sk block"></i></div><div class="card">${lines(12)}</div></div>`;
    return headSk + `<div class="grid g2">${('<div class="card">' + lines(6) + '</div>').repeat(4)}</div>`;
  };

  // ---------- text effects ----------
  const GLYPHS = 'ABCDEFGHJKLMNPRSTUVWXYZ0123456789#%&*+/<>=?';
  M.scramble = (el, dur = 700) => {
    if (reduce || el.children.length) return;
    const text = el.textContent; const t0 = performance.now(); const order = [...text].map((_, i) => i / text.length * 0.7 + Math.random() * 0.3);
    el.classList.add('scrambling');
    (function f(now) {
      const k = Math.min(1, (now - t0) / dur);
      el.textContent = [...text].map((c, i) => (c === ' ' || k >= order[i]) ? c : GLYPHS[Math.floor(Math.random() * GLYPHS.length)]).join('');
      if (k < 1) requestAnimationFrame(f); else { el.textContent = text; el.classList.remove('scrambling'); }
    })(t0);
  };
  M.type = (el, speed = 42) => {
    if (reduce) return;
    const text = el.dataset.type || el.textContent; el.dataset.type = text; el.textContent = '';
    const caret = document.createElement('span'); caret.className = 'caret';
    const node = document.createTextNode(''); el.append(node, caret);
    let i = 0;
    (function f() { node.textContent = text.slice(0, ++i); if (i < text.length) setTimeout(f, speed + Math.random() * 40); else setTimeout(() => caret.remove(), 1600); })();
  };

  // ---------- staggered cascade + text reveal when a page arrives ----------
  M.enter = root => {
    refreshMagnets();
    if (reduce || !root) return;
    let n = 0;
    const add = el => { if (n > 22 || el.classList.contains('cascade')) return; el.classList.add('cascade'); el.style.setProperty('--n', n++); };
    root.querySelectorAll('.hero, .page-head .actions, .tabs, .filters, .chips, .grid > *, .stack > *, :scope > .card, :scope > .note').forEach(add);
    root.querySelectorAll('tbody tr, table.t tr, .list-item').forEach((el, i) => { if (i < 16) { el.classList.add('cascade'); el.style.setProperty('--n', 4 + i * 0.6); } });
    root.querySelectorAll('.page-head .eyebrow, .page-head .sub, .hero .sub, .hero .eyebrow').forEach((el, i) => { el.classList.add('mask-up'); el.style.animationDelay = (i * 90) + 'ms'; });
    root.querySelectorAll('h1').forEach(h => { if (h.dataset.type !== undefined) M.type(h); else { h.classList.remove('wght-in'); void h.offsetWidth; h.classList.add('wght-in'); M.scramble(h, 650); } });
    root.querySelectorAll('.kpi .v').forEach((v, i) => setTimeout(() => M.scramble(v, 600), 120 + i * 90));
  };

  // ---------- blur-up satellite tiles (Esri World Imagery): a low-zoom blurred copy first, then the sharp tiles ----------
  const tile = (z, x, y) => `https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/${z}/${y}/${x}`;
  const lonlat2tile = (lat, lon, z) => { const n = 2 ** z, r = lat * Math.PI / 180; return [Math.floor((lon + 180) / 360 * n), Math.floor((1 - Math.log(Math.tan(r) + 1 / Math.cos(r)) / Math.PI) / 2 * n)]; };
  M.satTiles = (el, lat, lon, z, cols, rows, dz = 4, opts = {}) => {
    if (typeof el === 'string') el = $(el); if (!el) return;
    const [cx, cy] = lonlat2tile(lat, lon, z), x0 = cx - Math.floor(cols / 2), y0 = cy - Math.floor(rows / 2), f = 2 ** dz;
    const W = cols * 256, H = rows * 256;
    const s = Math.max(el.clientWidth / W, el.clientHeight / H);
    const grid = cls => `<div class="${cls}" style="width:${W}px;height:${H}px;left:50%;top:50%;transform:translate(-50%,-50%) scale(${s});transform-origin:center;display:grid;grid-template-columns:repeat(${cols},256px)">`;
    let lo = '', hi = '';
    for (let y = y0; y < y0 + rows; y++) for (let x = x0; x < x0 + cols; x++) {
      lo += `<div style="width:256px;height:256px;background:url(${tile(z - dz, Math.floor(x / f), Math.floor(y / f))}) ${-(x % f) * 256}px ${-(y % f) * 256}px / ${f * 256}px ${f * 256}px"></div>`;
      hi += `<img src="${tile(z, x, y)}" width="256" height="256" alt="" draggable="false">`;
    }
    el.classList.add('blurup');
    el.innerHTML = (opts.noLow ? '' : grid('lo') + lo + '</div>') + grid('hi') + hi + '</div>' + (opts.tag ? `<span class="tag">${opts.tag}</span>` : '');
    const imgs = [...el.querySelectorAll('.hi img')]; let done = 0;
    const ready = () => { if (++done >= imgs.length) setTimeout(() => el.querySelector('.hi').classList.add('ready'), opts.delay || 250); };
    imgs.forEach(im => im.complete ? ready() : (im.onload = im.onerror = ready));
  };

  // ---------- cursor follower, card glow, hero unblur, magnetic buttons ----------
  let mx = -100, my = -100, rx = -100, ry = -100, magnets = [];
  function refreshMagnets() { magnets = [...document.querySelectorAll('.magnetic, .btn.primary')]; magnets.forEach(m => m.classList.add('magnetic')); }
  M.refreshMagnets = refreshMagnets;
  if (fine && !reduce) {
    document.body.classList.add('has-cursor');
    const dot = $('#cursorDot'), ring = $('#cursorRing');
    addEventListener('mousemove', e => {
      mx = e.clientX; my = e.clientY;
      dot.style.transform = `translate(${mx}px,${my}px)`;
      const hot = e.target.closest && e.target.closest('a, button, .click, .list-item, .chip, label.switch, select');
      ring.classList.toggle('big', !!hot);
      const card = e.target.closest && e.target.closest('.card');
      if (card) { const r = card.getBoundingClientRect(); card.style.setProperty('--mx', (mx - r.left) + 'px'); card.style.setProperty('--my', (my - r.top) + 'px'); }
      const hero = e.target.closest && e.target.closest('.hero');
      if (hero) { const r = hero.getBoundingClientRect(); hero.style.setProperty('--hx', (mx - r.left) + 'px'); hero.style.setProperty('--hy', (my - r.top) + 'px'); }
      for (const m of magnets) {
        if (!m.isConnected) continue;
        const r = m.getBoundingClientRect(), cx = r.left + r.width / 2, cy = r.top + r.height / 2, dx = mx - cx, dy = my - cy;
        const reach = Math.max(r.width, r.height) / 2 + 46;
        if (Math.hypot(dx, dy) < reach) { m.style.transform = `translate(${dx * 0.28}px,${dy * 0.38}px)`; m._pulled = true; }
        else if (m._pulled) { m.style.transform = ''; m._pulled = false; }
      }
    }, { passive: true });
    document.addEventListener('mouseleave', () => { mx = my = -100; });
    (function follow() { rx += (mx - rx) * 0.18; ry += (my - ry) * 0.18; ring.style.transform = `translate(${rx}px,${ry}px)`; requestAnimationFrame(follow); })();
  }

  M.freezeSkeleton = q.get('skeleton');
  window.Motion = M;
})();
