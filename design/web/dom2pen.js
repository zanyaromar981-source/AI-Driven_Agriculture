/* Converts the live Control Room page into .pen nodes with the browser's exact measurements.
   Run inside the page: dom2pen(name) returns {node, rasters}. Boxes become frames, text becomes text,
   Lucide icons become icon nodes; maps, satellite pictures and the sun logo become image frames
   (their pixels are captured separately and saved next to the .pen file). */
window.dom2pen = function (frameName) {
  let uid = 0;
  const id = () => 'w' + (++uid).toString(36) + Math.random().toString(36).slice(2, 5);
  const R = v => Math.round(v * 100) / 100;
  const RASTER = '.leaflet-container, #heroSoft, #inboxSat, svg.sun, svg.farm-svg';
  const SKIP = '#heroSharp, .cursor-dot, .cursor-ring, .toasts, script, style, link, noscript, .leaflet-control-attribution';
  const ICON_NAMES = { 'alert-triangle': 'triangle-alert', 'bar-chart-3': 'chart-column', 'x-octagon': 'octagon-x', unlock: 'lock-open' };
  const rasters = [];

  function hex(c) {
    const m = c && c.match(/rgba?\(([^)]+)\)/); if (!m) return null;
    const p = m[1].split(/[\s,/]+/).filter(Boolean).map(Number), a = p.length > 3 ? p[3] : 1;
    if (a === 0) return null;
    const h = v => Math.max(0, Math.min(255, Math.round(v))).toString(16).padStart(2, '0').toUpperCase();
    return '#' + h(p[0]) + h(p[1]) + h(p[2]) + (a < 1 ? h(a * 255) : '');
  }
  function splitTop(s) { const out = []; let d = 0, cur = ''; for (const ch of s) { if (ch === '(') d++; if (ch === ')') d--; if (ch === ',' && !d) { out.push(cur.trim()); cur = ''; } else cur += ch; } if (cur.trim()) out.push(cur.trim()); return out; }
  function gradient(bgi) {
    const m = bgi.match(/^(linear|radial)-gradient\((.*)\)$/s); if (!m) return null;
    const parts = splitTop(m[2]); let rot = 180, type = m[1];
    if (type === 'linear' && /deg|to /.test(parts[0])) { const a = parts.shift(); rot = /deg/.test(a) ? parseFloat(a) : ({ 'to right': 90, 'to left': 270, 'to top': 0, 'to bottom': 180 }[a] ?? 180); }
    else if (type === 'radial' && !/rgb/.test(parts[0])) parts.shift();
    const colors = parts.map((p, i, arr) => { const c = p.match(/rgba?\([^)]+\)/); const pos = p.match(/(-?[\d.]+)%\s*$/); return c ? { color: hex(c[0]) || '#00000000', position: pos ? R(parseFloat(pos[1]) / 100) : R(i / Math.max(1, arr.length - 1)) } : null; }).filter(Boolean);
    if (colors.length < 2) return null;
    // CSS 0deg points up and turns clockwise; the .pen rotation is measured the other way from the same start
    return type === 'linear' ? { type: 'gradient', gradientType: 'linear', enabled: true, rotation: R((360 - rot) % 360), size: { height: 1 }, colors }
      : { type: 'gradient', gradientType: 'radial', enabled: true, colors };
  }
  function shadow(bs) {
    if (!bs || bs === 'none') return null;
    const first = splitTop(bs)[0]; const c = first.match(/rgba?\([^)]+\)/); if (!c) return null;
    const n = first.replace(c[0], '').trim().split(/\s+/).map(parseFloat);
    if (first.includes('inset')) return null;
    const col = hex(c[0]); if (!col) return null;
    return { type: 'shadow', shadowType: 'outer', color: col, offset: { x: n[0] || 0, y: n[1] || 0 }, blur: n[2] || 0, spread: n[3] || 0 };
  }
  function radius(cs, w, h) {
    const v = ['borderTopLeftRadius', 'borderTopRightRadius', 'borderBottomRightRadius', 'borderBottomLeftRadius'].map(k => {
      const s = cs[k]; if (!s) return 0; const t = s.split(' ')[0]; return t.endsWith('%') ? Math.min(w, h) * parseFloat(t) / 100 : parseFloat(t) || 0;
    }).map(x => R(Math.min(x, Math.min(w, h) / 2)));
    if (v.every(x => x === v[0])) return v[0] || undefined;
    return v;
  }
  function border(cs) {
    const sides = ['Top', 'Right', 'Bottom', 'Left'].map(s => ({ w: parseFloat(cs['border' + s + 'Width']) || 0, c: hex(cs['border' + s + 'Color']), st: cs['border' + s + 'Style'] }));
    const vis = sides.filter(s => s.w > 0 && s.c && s.st !== 'none');
    if (!vis.length) return null;
    const same = sides.every(s => s.w === sides[0].w && s.c === sides[0].c);
    return { stroke: vis[0].c, strokeWidth: same ? sides[0].w : { top: sides[0].c ? sides[0].w : 0, right: sides[1].c ? sides[1].w : 0, bottom: sides[2].c ? sides[2].w : 0, left: sides[3].c ? sides[3].w : 0 } };
  }
  const fam = f => { const a = f.split(',')[0].replace(/["']/g, '').trim(); return /monospace|consolas|ui-monospace/i.test(f) && !/manrope/i.test(a) ? 'JetBrains Mono' : a === 'system-ui' ? 'Manrope' : a; };
  function nameOf(el) {
    if (el.id) return '#' + el.id;
    const t = (el.innerText || '').trim().split('\n')[0].slice(0, 28);
    const c = typeof el.className === 'string' && el.className.split(' ').filter(Boolean)[0];
    return (c ? c : el.tagName.toLowerCase()) + (t ? ' · ' + t : '');
  }

  function textNode(content, cs, rects, base, extra = {}, scale = 1) {
    const fs = parseFloat(cs.fontSize) * scale, lh = (cs.lineHeight === 'normal' ? parseFloat(cs.fontSize) * 1.36 : parseFloat(cs.lineHeight)) * scale;
    const u = rects.reduce((a, q) => ({ l: Math.min(a.l, q.left), t: Math.min(a.t, q.top), r: Math.max(a.r, q.right), b: Math.max(a.b, q.bottom) }), { l: 1e9, t: 1e9, r: -1e9, b: -1e9 });
    const tops = new Set(rects.map(q => Math.round(q.top / 4))).size;
    if (cs.textTransform === 'uppercase') content = content.toUpperCase();
    const rtl = cs.direction === 'rtl' || /[؀-ۿ]/.test(content);
    // a variable-font weight set by font-variation-settings wins over font-weight
    const fv = (cs.fontVariationSettings || '').match(/["']wght["']\s+([\d.]+)/);
    const weight = fv ? String(Math.round(parseFloat(fv[1]) / 100) * 100) : String(cs.fontWeight);
    const n = { type: 'text', id: id(), name: content.slice(0, 32), content, fill: hex(cs.color) || '#162019', fontFamily: fam(cs.fontFamily), fontSize: R(fs), fontWeight: weight, lineHeight: R(lh / fs) };
    const ls = parseFloat(cs.letterSpacing) * scale; if (ls) n.letterSpacing = R(ls);
    const firstH = rects[0].height;
    n.x = R(u.l - base.left); n.y = R(u.t - base.top - (lh - firstH) / 2);
    if (tops > 1) { n.textGrowth = 'fixed-width'; n.width = R(u.r - u.l + 3); n.textAlign = rtl ? 'right' : (cs.textAlign === 'center' ? 'center' : cs.textAlign === 'right' || cs.textAlign === 'end' ? 'right' : 'left'); }
    else if (rtl) { n.textGrowth = 'fixed-width'; n.width = R(u.r - u.l + 6); n.x = R(n.x - 6); n.textAlign = 'right'; }
    const op = +cs.opacity; if (op < 1) n.opacity = R(op);
    return Object.assign(n, extra);
  }

  function emitText(tn, cs, base, out) {
    const raw = tn.textContent; if (!raw.trim()) return;
    // measure only the visible letters, so a leading space does not shift the text left
    const range = document.createRange(); const s0 = raw.search(/\S/), s1 = raw.length - raw.split('').reverse().join('').search(/\S/);
    range.setStart(tn, s0); range.setEnd(tn, s1);
    const rects = [...range.getClientRects()].filter(q => q.width > 0.5 && q.height > 0.5);
    if (!rects.length) return;
    // text inside a scaled element (the growing counter) is drawn at its on-screen size
    const host = tn.parentElement; let sc = 1;
    for (let e = host; e && e !== document.body; e = e.parentElement) { const t = getComputedStyle(e).transform; const m = t && t.match(/^matrix\(([^,]+),/); if (m) sc *= parseFloat(m[1]); }
    out.push(textNode(raw.replace(/\s+/g, ' ').trim(), cs, rects, base, {}, Math.abs(sc - 1) > 0.01 ? sc : 1));
  }

  function frameFor(el, cs, r, base, extra = {}) {
    const n = { type: 'frame', id: id(), name: nameOf(el), x: R(r.left - base.left), y: R(r.top - base.top), width: R(r.width), height: R(r.height), layout: 'none', children: [] };
    const bg = hex(cs.backgroundColor), g = cs.backgroundImage && cs.backgroundImage !== 'none' ? gradient(cs.backgroundImage) : null;
    if (g && bg) n.fill = [bg, g]; else if (g) n.fill = g; else if (bg) n.fill = bg;
    const rad = radius(cs, r.width, r.height); if (rad) n.cornerRadius = rad;
    const b = border(cs); if (b) Object.assign(n, b);
    const sh = shadow(cs.boxShadow); if (sh) n.effect = sh;
    if (cs.overflow === 'hidden' || cs.overflowX === 'hidden' || cs.overflow === 'auto' && el.scrollHeight <= el.clientHeight + 1) n.clip = true;
    const op = +cs.opacity; if (op < 1) n.opacity = R(op);
    return Object.assign(n, extra);
  }

  function walk(el, base, out) {
    if (el.matches(SKIP)) return;
    const cs = getComputedStyle(el);
    if (cs.display === 'none' || cs.visibility === 'hidden' || +cs.opacity < 0.02) return;
    const r = el.getBoundingClientRect();
    if (vp && cs.position !== 'fixed' && (r.bottom < 0 || r.top > innerHeight) && r.height < innerHeight * 3) return;
    if (el.matches(RASTER)) {
      if (r.width < 1 || r.height < 1) return;
      const key = 'r' + rasters.length; el.setAttribute('data-capid', key);
      const n = { type: 'frame', id: id(), name: nameOf(el) + ' (image)', x: R(r.left - base.left), y: R(r.top - base.top), width: R(r.width), height: R(r.height), layout: 'none', children: [], fill: { type: 'image', enabled: true, url: '', mode: 'stretch' } };
      const rad = radius(getComputedStyle(el.closest('.map, .hero, #inboxSat, .mark') || el), r.width, r.height); if (rad) { n.cornerRadius = rad; n.clip = true; }
      rasters.push({ key, rect: { x: r.left + scrollX, y: r.top + scrollY, width: r.width, height: r.height }, node: n.id });
      out.push(n); return;
    }
    if (el.tagName.toLowerCase() === 'svg' && el.classList.contains('lucide')) {
      const nm = [...el.classList].find(c => c.startsWith('lucide-'));
      if (!nm || r.width < 1) return;
      const name = nm.slice(7);
      out.push({ type: 'icon', id: id(), name: 'Icon ' + name, icon: ICON_NAMES[name] || name, library: 'lucide', x: R(r.left - base.left), y: R(r.top - base.top), width: R(r.width), height: R(r.height), fill: hex(cs.color) || '#5E6E64' });
      return;
    }
    const tag = el.tagName.toLowerCase();
    const hasBox = hex(cs.backgroundColor) || (cs.backgroundImage && cs.backgroundImage !== 'none' && cs.backgroundImage.includes('gradient')) || border(cs) || shadow(cs.boxShadow);
    const clips = (cs.overflow === 'hidden' || cs.overflowX === 'hidden') && r.width > 0;
    const faded = +cs.opacity < 1;
    let target = out, b = base;
    if ((hasBox || clips || faded || ['input', 'select', 'textarea'].includes(tag)) && r.width > 0.5 && r.height > 0.5) {
      const n = frameFor(el, cs, r, base); out.push(n); target = n.children; b = r;
      for (const p of ['::before', '::after']) {
        const ps = getComputedStyle(el, p);
        if (!ps.content || ps.content === 'none' || ps.content === 'normal' || +ps.opacity < 0.02 || ps.display === 'none') continue;
        const pb = hex(ps.backgroundColor); if (!pb) continue;
        const w = parseFloat(ps.width), h = parseFloat(ps.height); if (!(w > 0 && h > 0)) continue;
        const pn = { type: 'frame', id: id(), name: el.className + p, x: R(parseFloat(ps.left) || 0), y: R(parseFloat(ps.top) || 0), width: R(w), height: R(h), layout: 'none', fill: pb, children: [] };
        const rad = radius(ps, w, h); if (rad) pn.cornerRadius = rad;
        const sh = shadow(ps.boxShadow); if (sh) pn.effect = sh;
        target.push(pn);
      }
    }
    if (tag === 'input' || tag === 'textarea' || tag === 'select') {
      const v = tag === 'select' ? (el.options[el.selectedIndex] || {}).text : el.value;
      const shown = v || el.placeholder || '';
      if (shown && el.type !== 'checkbox') {
        const pl = parseFloat(cs.paddingLeft) || 10, pt = parseFloat(cs.paddingTop) || 0, fs = parseFloat(cs.fontSize);
        const lh = cs.lineHeight === 'normal' ? fs * 1.36 : parseFloat(cs.lineHeight);
        const rtl = cs.direction === 'rtl' || /[؀-ۿ]/.test(shown);
        const t = { type: 'text', id: id(), name: shown.slice(0, 30), content: shown, fill: v ? (hex(cs.color) || '#162019') : '#8A978F', fontFamily: fam(cs.fontFamily), fontSize: R(fs), fontWeight: String(cs.fontWeight), lineHeight: R(lh / fs) };
        if (tag === 'textarea' || rtl) { t.textGrowth = 'fixed-width'; t.width = R(r.width - pl * 2); t.textAlign = rtl ? 'right' : (cs.textAlign === 'right' ? 'right' : 'left'); t.x = R(pl); t.y = R(tag === 'textarea' ? pt : (r.height - lh) / 2); }
        else { t.x = R(cs.textAlign === 'right' ? r.width - pl - shown.length * fs * 0.55 : pl); t.y = R((r.height - lh) / 2); }
        target.push(t);
        if (tag === 'select') target.push({ type: 'icon', id: id(), name: 'Icon chevron-down', icon: 'chevron-down', library: 'lucide', x: R(r.width - 22), y: R(r.height / 2 - 7), width: 14, height: 14, fill: '#5E6E64' });
      }
      return;
    }
    for (const c of ordered(el)) {
      if (c.nodeType === 3) emitText(c, cs, b, target);
      else walk(c, b, target);
    }
  }
  // paint order: the browser stacks positioned elements by z-index, the .pen file by order
  function ordered(el) {
    return [...el.childNodes].filter(c => c.nodeType === 3 || c.nodeType === 1).map((c, i) => {
      let z = 0;
      if (c.nodeType === 1) { const s = getComputedStyle(c); if (s.position !== 'static' && s.zIndex !== 'auto') z = parseInt(s.zIndex) || 0; }
      return { c, z, i };
    }).sort((a, b) => a.z - b.z || a.i - b.i).map(o => o.c);
  }

  const vp = !!window.__viewport; // viewport frame: only what is on screen now
  const body = document.body, W = document.documentElement.clientWidth, H = vp ? innerHeight : Math.max(document.documentElement.scrollHeight, innerHeight);
  const root = { type: 'frame', id: id(), name: frameName, width: W, height: H, layout: 'none', clip: true, fill: hex(getComputedStyle(body).backgroundColor) || '#F2F4EF', children: [] };
  const base = vp ? { left: 0, top: 0 } : { left: -scrollX, top: -scrollY };
  for (const c of ordered(body)) if (c.nodeType === 1) walk(c, base, root.children);
  return { node: root, rasters };
};
