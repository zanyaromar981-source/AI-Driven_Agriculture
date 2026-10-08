'use strict';
/* SmartSuli Control Room demo. Sample data only: nothing is saved or sent.
   Every section the Ministry needs to run the app, the data and the process.
   Real district borders come from ../map_demo/kri_map_data.js. */

// ---------- sample data ----------
const KRI = window.KRI;
const GOVS = ['Duhok', 'Erbil', 'Sulaymaniyah', 'Halabja'];
const DISTS = KRI.districts.features.map(f => f.properties).filter(p => GOVS.includes(p.gov));
const SUBS = KRI.subdistricts.features.map(f => f.properties).filter(p => GOVS.includes(p.gov));
const NOW = new Date('2026-10-09T10:46:00+03:00');
let seed = 20261009;
const rnd = () => { seed |= 0; seed = seed + 0x6D2B79F5 | 0; let t = Math.imul(seed ^ seed >>> 15, 1 | seed); t = t + Math.imul(t ^ t >>> 7, 61 | t) ^ t; return ((t ^ t >>> 14) >>> 0) / 4294967296; };
const pick = (arr, w) => { if (!w) return arr[Math.floor(rnd() * arr.length)]; let s = w.reduce((a, b) => a + b, 0), r = rnd() * s; for (let i = 0; i < arr.length; i++) { r -= w[i]; if (r <= 0) return arr[i]; } return arr[arr.length - 1]; };

const CROPS = {
  wheat: ['#E0B13A', 'گەنم', '🌾'], barley: ['#C8B560', 'جۆ', '🌿'], tomato: ['#D9483B', 'تەماتە', '🍅'], cucumber: ['#6DB352', 'خەیار', '🥒'],
  potato: ['#A9784A', 'پەتاتە', '🥔'], onion: ['#B46FA8', 'پیاز', '🧅'], watermelon: ['#EF7C8E', 'شووتی', '🍉'], grape: ['#7E57C2', 'ترێ', '🍇'],
  olive: ['#7D8B3A', 'زەیتوون', '🫒'], sunflower: ['#F5C518', 'گوڵەبەڕۆژە', '🌻'], chickpea: ['#D9B88A', 'نۆک', '🫘'], empty: ['#D9D6CC', 'بەتاڵ', '⬜'],
};
const CROP_KEYS = Object.keys(CROPS).filter(c => c !== 'empty');
const CROP_W = [45, 17, 7, 4, 5, 3, 3, 3, 3, 1, 3];
const DIST_W = { Makhmur: 9, Erbil: 8, Chamchamal: 8, Sharazur: 6, Kalar: 6, Qushtapa: 5, Sumel: 5, Shekhan: 4, Koya: 4, Ranya: 3, Halabja: 3, Dukan: 3,
  Sulaymaniyah: 3, Akre: 2.5, Darbandikhan: 2, Duhok: 2, Zakho: 2, Bardarash: 2, Taqtaq: 1.5, Pshdar: 1.5, Shaqlawa: 1.2, Khurmal: 1 };
const FARM_NAMES = ['کێڵگەی سەرەوە', 'زەوی باوکم', 'کێڵگەی تەماتە', 'کێڵگەی جۆ', 'زەوی ڕووبار', 'کێڵگەی نۆک', 'کێڵگەی خوارەوە', 'زەوی گوند', 'باخی ترێ',
  'کێڵگەی نوێ', 'کێڵگەی پەتاتە', 'باخی زەیتوون', 'زەوی گەورە', 'کێڵگەی بچووک', 'زەوی کانی', 'کێڵگەی گەنم', 'زەوی دایکم', 'کێڵگەی شووتی'];
const PREFIX = ['750', '770', '751', '773', '771', '772', '780', '782'];
const LEVELS = { normal: ['good', 'Normal'], watch: ['warn', 'Watch'], alarm: ['danger', 'Alarm'], none: ['', 'No picture yet'] };

const FARMERS = [];
for (let i = 0; i < 1284; i++) {
  const ph = '+964' + pick(PREFIX) + String(Math.floor(rnd() * 1e7)).padStart(7, '0');
  FARMERS.push({ id: 'p' + (1000 + i), phone: ph, lang: pick(['ku', 'en', 'kmr', 'ar'], [61, 8, 22, 9]), farms: [], joined: daysAgo(Math.floor(rnd() * 120) + 1),
    app: pick(['1.0.3', '1.0.2', '1.0.0'], [80, 15, 5]), push: rnd() > 0.09, blocked: false });
}
const distW = DISTS.map(d => (DIST_W[d.en] || 0.4));
const FARMS = [];
for (let i = 0; i < 2031; i++) {
  const d = pick(DISTS, distW), subs = SUBS.filter(s => s.dist === d.en), s = subs.length ? pick(subs) : null;
  const area = Math.min(400, Math.max(0.3, Math.exp(1.9 + 0.95 * gauss())));
  const main = pick(CROP_KEYS, CROP_W);
  const crops = [[main, area * (0.55 + rnd() * 0.45)]];
  if (rnd() < 0.3) crops.push([pick(CROP_KEYS.filter(c => c !== main), CROP_W.filter((_, k) => CROP_KEYS[k] !== main)), area - crops[0][1]]);
  else crops[0][1] = area * (0.92 + rnd() * 0.08);
  const farmer = FARMERS[i < 1284 ? i : Math.floor(rnd() * 1284)];
  const c = (s || d).c;
  const f = { id: String(2400 - i), name: pick(FARM_NAMES), farmer, gov: d.gov, dist: d.en, sub: s ? s.en : d.en, area, crops, main,
    level: pick(['normal', 'watch', 'alarm', 'none'], [70, 14, 3, 13]), sync: minutesAgo(Math.floor(7 + Math.pow(rnd(), 1.4) * 60 * 24 * 30)),
    offline: rnd() < 0.2, access: '1km', reason: null, lat: c[0] + (rnd() - 0.5) * 0.12, lon: c[1] + (rnd() - 0.5) * 0.12, corners: 4 + Math.floor(rnd() * 9),
    gps: [3 + Math.floor(rnd() * 3), 6 + Math.floor(rnd() * 8)], seed: Math.floor(rnd() * 1e9) };
  farmer.farms.push(f); FARMS.push(f);
}
function gauss() { let u = 0, v = 0; while (!u) u = rnd(); while (!v) v = rnd(); return Math.sqrt(-2 * Math.log(u)) * Math.cos(2 * Math.PI * v); }
function daysAgo(n) { return new Date(NOW - n * 864e5); }
function minutesAgo(n) { return new Date(NOW - n * 6e4); }
// farms opened because their farmer asked an officer for help
const F1207 = FARMS.find(f => f.dist === 'Chamchamal'); Object.assign(F1207, { id: '1207', name: 'کێڵگەی سەرەوە', sub: 'Aghjalar', level: 'watch', area: 11.3, crops: [['wheat', 9.4], ['tomato', 1.9]], main: 'wheat', access: 'open', reason: 'report r_3391' });
FARMS.filter(f => f.dist === 'Chamchamal' && f !== F1207).slice(0, 1).forEach(f => Object.assign(f, { id: '1176', access: 'open', reason: 'Doctor case c_812', main: 'tomato', level: 'alarm' }));
FARMS.filter(f => f.level === 'alarm').slice(2, 6).forEach((f, k) => Object.assign(f, { access: 'open', reason: 'report r_33' + (60 + k) }));
FARMS.slice(40, 42).forEach(f => { f.access = 'blocked'; f.farmer.blocked = true; });

const OFFICERS = [
  { id: 'KA', name: 'Karwan Aziz', role: 'admin', areas: ['All Kurdistan'], twofa: true, last: '12 min ago', color: '#2A6DB5' },
  { id: 'SA', name: 'Shilan Ahmed', role: 'district', areas: ['Sulaymaniyah'], twofa: true, last: 'now', color: '#0F5B4B' },
  { id: 'DO', name: 'Dlovan Omar', role: 'admin', areas: ['All Kurdistan'], twofa: true, last: 'Wed', color: '#9A6410' },
  { id: 'HM', name: 'Hevi Mustafa', role: 'district', areas: ['Duhok'], twofa: true, last: 'today 08:40', color: '#7E57C2' },
  { id: 'RS', name: 'Rebaz Salih', role: 'district', areas: ['Erbil'], twofa: false, last: 'Tue', color: '#A9784A' },
  { id: 'NH', name: 'Nazdar Hassan', role: 'viewer', areas: ['Halabja'], twofa: true, last: 'Mon', color: '#D9483B' },
];
const ROLE = { viewer: [0, 'Viewer'], district: [1, 'District officer'], admin: [2, 'Admin'] };
let ME = OFFICERS[0];
const rank = () => ROLE[ME.role][0];
const inArea = f => ME.areas.includes('All Kurdistan') || ME.areas.includes(f.gov);

const S = {
  approvals: [
    { id: 'ap1', kind: 'alert', need: 1, title: 'Alert: Frost, Sunday night', detail: 'Penjwen, Sharbazher, Pshdar · ' + FARMS.filter(f => ['Penjwen', 'Sharbazher', 'Pshdar'].includes(f.dist)).length + ' farms · Alarm', by: 'Shilan Ahmed', at: 'today 10:20' },
    { id: 'ap2', kind: 'rule', need: 2, title: 'Rule: Heat day 31 → 33 °C', detail: 'Asked because farmers in Makhmur and Kalar stop reading daily heat warnings', by: 'Dlovan Omar', at: 'Wed 16:20' },
    { id: 'ap3', kind: 'price', need: 2, title: 'Alwa: publish 1 price change', detail: 'Tomato, Sulaymaniyah market: 750 → 700 IQD/kg', by: 'Karwan Aziz', at: 'today 09:58' },
    { id: 'ap4', kind: 'plan', need: 2, title: 'Water plan, season 2026/27', detail: '1.46 bn m³ for 33 districts · reserve 0.94 bn m³', by: 'Dlovan Omar', at: 'Tue 11:30' },
    { id: 'ap5', kind: 'delete', need: 2, title: 'Delete account +964 770 ••• 1182', detail: 'The farmer asked by phone call on 7 Oct · 2 farms, 1 report', by: 'Hevi Mustafa', at: 'Wed 11:02' },
  ],
  audit: [
    ['10:44', 'Shilan Ahmed', 'showed the phone of farm #1207', 'reason: “to arrange a field visit”', 'phone', 'warn', 'farmers'],
    ['10:42', 'Shilan Ahmed', 'opened farm #1207', 'open because of report r_3391', 'eye', 'brand', 'farmers'],
    ['10:20', 'Shilan Ahmed', 'drafted alert “Frost, Sunday night”', 'Penjwen, Sharbazher, Pshdar · ' + FARMS.filter(f => ['Penjwen', 'Sharbazher', 'Pshdar'].includes(f.dist)).length + ' farms', 'bell-ring', 'brand', 'alerts'],
    ['09:58', 'Karwan Aziz', 'changed tomato price, Sulaymaniyah', '750 → 700 IQD/kg · not published', 'store', 'brand', 'prices'],
    ['09:15', 'Hevi Mustafa', 'sent report r_3377 to the vets', 'Penjwen · sick sheep', 'inbox', 'brand', 'inbox'],
    ['09:00', 'Job: Fires', 'pushed 4 fire detections', 'service key jobs-server-1', 'bot', '', 'jobs'],
    ['08:03', 'Job: Alwa prices', 'pushed 36 prices', 'service key jobs-server-1', 'bot', '', 'jobs'],
    ['06:00', 'Job: Weather planner', 'stored 2,031 plans, 3 alarms', 'service key jobs-server-1', 'bot', '', 'jobs'],
    ['Wed 16:20', 'Dlovan Omar', 'asked to change Heat day 31 → 33 °C', 'waiting for a second officer', 'sliders-horizontal', 'warn', 'rules'],
    ['Wed 11:02', 'Hevi Mustafa', 'asked to delete account +964 770 ••• 1182', 'farmer asked by phone · waiting for an admin', 'trash-2', 'danger', 'deletes'],
    ['Mon 18:30', 'Karwan Aziz', 'stopped alert “Heat 31 °C+”, Makhmur', 'reason: same alert sent the day before', 'x-octagon', 'danger', 'alerts'],
    ['Mon 09:12', 'Karwan Aziz', 'invited Nazdar Hassan as Viewer', 'area Halabja', 'user-plus', 'brand', 'officers'],
  ],
  inbox: [
    { id: 'r_3391', kind: 'report', type: 'yellow_stripes', title: 'Yellow stripes on wheat leaves', farm: '1207', where: 'Aghjalar, Chamchamal', gov: 'Sulaymaniyah', at: '10:31', state: 'new', assigned: null, photos: 3,
      note: 'گەڵای گەنمەکە زەرد بووە و هێڵی زەردی تێدایە، لە لای باکووری کێڵگەکە.', en: 'The wheat leaves turned yellow with yellow lines, on the north side of the farm.',
      doctor: { likely: 'Likely yellow rust. Not sure without a closer look at the leaf.', conf: 'likely', why: [['Field Eye', '26 weak squares in the north-east since 25 Sep (78% of normal)'], ['Weather', '29 h at 6 to 16 °C with air above 90% wet: rust weather high'], ['Neighbours', '3 more yellow-stripe reports within 20 km in 14 days']] } },
    { id: 'c_812', kind: 'case', type: 'doctor', title: 'Doctor unsure: rot on tomato stems', farm: '1176', where: 'Sangaw, Chamchamal', gov: 'Sulaymaniyah', at: '09:58', state: 'new', assigned: null, photos: 2,
      note: 'قەدی تەماتەکان ڕەش بوون و دەڕزێن.', en: 'The tomato stems turned black and are rotting.', doctor: { likely: 'Cannot tell: could be stem rot or frost damage. Refer to an officer.', conf: 'unsure', why: [['Field Eye', 'tomato plot 81% of normal'], ['Weather', 'two nights at 1 °C last week'], ['Neighbours', 'no similar reports']] } },
    { id: 'r_3388', kind: 'report', type: 'insects', title: 'Insects on young barley', farm: '1163', where: 'Qadir Karam, Chamchamal', gov: 'Sulaymaniyah', at: '08:12', state: 'new', assigned: null, photos: 1,
      note: 'مێرووی بچووک لەسەر جۆکە هەیە.', en: 'Small insects on the barley.', doctor: null },
    { id: 'r_3380', kind: 'report', type: 'hail', title: 'Hail broke tomato plants', farm: '0884', where: 'Sharazur', gov: 'Sulaymaniyah', at: 'Wed', state: 'assigned', assigned: 'Shilan Ahmed', photos: 4, note: 'تەرزە تەماتەکانی شکاند.', en: 'Hail broke the tomatoes.', doctor: null },
    { id: 'c_805', kind: 'case', type: 'doctor', title: 'Doctor unsure: grape leaves curling', farm: '0731', where: 'Halabja', gov: 'Halabja', at: 'Wed', state: 'assigned', assigned: 'Dlovan Omar', photos: 3, note: 'گەڵای ترێ دەپێچێتەوە.', en: 'The grape leaves are curling.', doctor: { likely: 'Unsure: leafroll virus or water stress.', conf: 'unsure', why: [['Weather', 'dry 21 days'], ['Field Eye', '72% of normal']] } },
    { id: 'r_3371', kind: 'report', type: 'fire', title: 'Fire near the field edge', farm: '0952', where: 'Kalar', gov: 'Sulaymaniyah', at: 'Tue', state: 'seen', assigned: 'Shilan Ahmed', photos: 1, note: 'ئاگر لە نزیک کێڵگەکە.', en: 'Fire near the farm.', doctor: null },
    { id: 'r_3377', kind: 'report', type: 'animal_disease', title: 'Sheep sick in the village', farm: '0377', where: 'Penjwen', gov: 'Sulaymaniyah', at: 'Sun', state: 'forwarded', assigned: 'Vets, Sulaymaniyah', photos: 2, note: 'مەڕەکان نەخۆشن.', en: 'The sheep are sick.', doctor: null },
    { id: 'r_3352', kind: 'report', type: 'flood', title: 'Flood after the storm', farm: '0412', where: 'Zakho', gov: 'Duhok', at: 'Mon', state: 'closed', assigned: 'Hevi Mustafa', photos: 2, note: 'لافاو کێڵگەکەی گرت.', en: 'The flood covered the farm.', doctor: null },
  ],
  alerts: [
    { id: 'a_41', title: 'Frost, Sunday night', type: 'frost', area: 'Penjwen, Sharbazher, Pshdar', farms: FARMS.filter(f => ['Penjwen', 'Sharbazher', 'Pshdar'].includes(f.dist)).length, state: 'waiting', by: 'Shilan Ahmed', ok: null, opened: null, day: 'Sun 11 Oct' },
    { id: 'a_40', title: 'Dust, PM10 above 150', type: 'dust', area: 'Kalar, Darbandikhan', farms: 142, state: 'sent', by: 'Shilan Ahmed', ok: 'Karwan Aziz', opened: 71, day: 'Tue 6 Oct' },
    { id: 'a_39', title: 'Heavy rain 14 mm', type: 'heavy_rain', area: 'Zakho, Amedi', farms: 88, state: 'sent', by: 'Hevi Mustafa', ok: 'Dlovan Omar', opened: 64, day: 'Fri 2 Oct' },
    { id: 'a_38', title: 'Heat 31 °C+', type: 'heat', area: 'Makhmur', farms: 206, state: 'stopped', by: 'Rebaz Salih', ok: null, opened: null, day: 'Mon 28 Sep' },
    { id: 'a_37', title: 'Good sowing rain from Thursday', type: 'sowing_rain', area: 'Erbil, Qushtapa, Makhmur', farms: 512, state: 'sent', by: 'Rebaz Salih', ok: 'Karwan Aziz', opened: 58, day: 'Thu 24 Sep' },
  ],
  flags: { fixedCode: true, maintenance: false, features: { add_farm: true, walk_mode: true, paint: true, satellite: true, doctor: true, reports: true, alwa: true, plan: true, push: true } },
  revealed: new Set(),
  tab: {},
  sel: {},
  farmFilter: { q: '', gov: '', dist: '', crop: '', level: '', access: '', page: 0 },
  alertSel: new Set(['Penjwen', 'Sharbazher', 'Pshdar']),
  cropView: 'all',
};

// ---------- helpers ----------
const $ = s => document.querySelector(s);
const esc = s => String(s).replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const fmt = (n, d = 0) => Number(n).toLocaleString('en-US', { minimumFractionDigits: d, maximumFractionDigits: d });
const I = n => `<i data-lucide="${n}"></i>`;
const pill = (t, tone = '', icon) => `<span class="pill ${tone}">${icon ? I(icon) : ''}${t}</span>`;
const mask = p => `${p.slice(0, 4)} ${p.slice(4, 7)} ••• ${p.slice(-4)}`;
const full = p => `${p.slice(0, 4)} ${p.slice(4, 7)} ${p.slice(7, 10)} ${p.slice(10)}`;
const when = d => { const m = Math.round((NOW - d) / 6e4); if (m < 60) return m + ' min ago'; if (m < 1440) return d.toTimeString().slice(0, 5); if (m < 2880) return 'yesterday'; return d.getDate() + ' ' + d.toLocaleString('en', { month: 'short' }); };
const levelPill = l => pill(LEVELS[l][1], LEVELS[l][0]);
const cropTag = c => `<span class="nowrap"><span class="dotc" style="background:${CROPS[c][0]}"></span>${c[0].toUpperCase() + c.slice(1)}</span>`;
const kpi = (l, v, n, tone = '', icon = '') => `<div class="card kpi ${tone}"><div class="l">${icon ? I(icon) : ''}${l}</div><div class="v">${v}</div><div class="n">${n}</div></div>`;
const head = (eb, t, sub, actions = '') => `<div class="page-head"><div class="t"><div class="eyebrow">${eb}</div><h1>${t}</h1><div class="sub">${sub}</div></div><div class="actions">${actions}</div></div>`;
const sw = (on, act, dis) => `<label class="switch"><input type="checkbox" ${on ? 'checked' : ''} ${dis ? 'disabled' : ''} ${act ? `onchange="${act}"` : ''}><span></span></label>`;
const set = (t, s, ctl) => `<div class="set"><div class="txt"><b>${t}</b><small>${s}</small></div><div class="ctl">${ctl}</div></div>`;
const locked = t => `${pill('Locked', 'dark', 'lock')}<span class="muted small">${t}</span>`;
const strip = p => `<span class="strip">${[...p].map(c => `<i class="${c === '-' ? '' : c}"></i>`).join('')}</span>`;
function toast(msg, tone = '') { const t = document.createElement('div'); t.className = 'toast ' + tone; t.innerHTML = msg; $('#toasts').append(t); setTimeout(() => t.remove(), 3200); }
function log(what, more, icon = 'circle', tone = 'brand', kind = 'other') { S.audit.unshift(['just now', ME.name, what, more, icon, tone, kind]); }
function modal(html) { $('#modal').innerHTML = html; $('#modalBg').hidden = false; refreshIcons(); }
function closeModal() { $('#modalBg').hidden = true; }
function drawer(html) { $('#drawer').innerHTML = html; $('#drawer').hidden = false; $('#drawerBg').hidden = false; refreshIcons(); }
function closeDrawer() { $('#drawer').hidden = true; $('#drawerBg').hidden = true; MAPS.drawer && (MAPS.drawer.remove(), MAPS.drawer = null); }
function refreshIcons() { window.lucide && lucide.createIcons(); }
function need(r, what) { if (rank() >= r) return true; toast(`${ROLE[ME.role][1]}s cannot ${what}.`, 'warn'); return false; }
function csv(name) { toast(`${I('download')} ${name}.csv exported (demo)`); log('exported ' + name, 'CSV download', 'download', '', 'exports'); }

// ---------- map ----------
const MAPS = {};
function districtMap(el, style, onClick, key = 'main') {
  if (MAPS[key]) MAPS[key].remove();
  const m = L.map(el, { zoomSnap: 0.25, attributionControl: true, scrollWheelZoom: false });
  L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', { maxZoom: 18, opacity: 0.35, attribution: '© OpenStreetMap' }).addTo(m);
  const layer = L.geoJSON(KRI.districts, {
    style: f => GOVS.includes(f.properties.gov) ? Object.assign({ color: '#fff', weight: 1, fillOpacity: 0.85 }, style(f.properties)) : { color: '#bbb', weight: 0.5, fillColor: '#ddd', fillOpacity: 0.3 },
    onEachFeature: (f, l) => {
      if (!GOVS.includes(f.properties.gov)) return;
      l.bindTooltip(f.properties.en, { permanent: true, direction: 'center', className: 'map-label' });
      if (onClick) l.on('click', () => onClick(f.properties, l));
    },
  }).addTo(m);
  L.geoJSON(KRI.governorates, { style: { color: '#14201A', weight: 2, opacity: 0.55, fill: false }, interactive: false }).addTo(m);
  m.fitBounds(L.geoJSON(KRI.governorates).getBounds(), { padding: [10, 10] });
  MAPS[key] = m; m._dl = layer;
  return m;
}
const ramp = (v, stops, cols) => cols[stops.findIndex(s => v < s)] || cols[cols.length - 1];

// ---------- routes ----------
const ROUTES = [
  ['', [['overview', 'layout-dashboard', 'Overview', 0], ['approvals', 'stamp', 'Approvals', 1]]],
  ['People', [['farms', 'users', 'Farmers and farms', 1], ['officers', 'shield-check', 'Officers and roles', 2]]],
  ['Fields and region', [['crops', 'wheat', 'Crop register', 0], ['region', 'map', 'Region data', 0]]],
  ['Act', [['alerts', 'bell-ring', 'Alerts', 1], ['inbox', 'inbox', 'Inbox', 1], ['doctor', 'stethoscope', 'The Doctor (AI)', 1], ['alwa', 'store', 'Alwa market', 1]]],
  ['Control the app', [['rules', 'sliders-horizontal', 'Rules', 1], ['app', 'smartphone', 'App control', 2], ['notify', 'send', 'Notifications and SMS', 2], ['texts', 'languages', 'Texts and languages', 1]]],
  ['System', [['jobs', 'activity', 'Data jobs', 1], ['db', 'database', 'Database', 2], ['security', 'lock', 'Security and privacy', 2], ['audit', 'history', 'History', 1], ['system', 'settings', 'System settings', 2]]],
];
const FLAT = Object.fromEntries(ROUTES.flatMap(([, items]) => items.map(i => [i[0], i])));
function badge(k) {
  const n = { approvals: S.approvals.length, inbox: S.inbox.filter(i => i.state === 'new').length, jobs: 1, security: S.flags.fixedCode ? 1 : 0, doctor: 1 }[k];
  if (!n) return '';
  return pill(k === 'jobs' || k === 'security' || k === 'doctor' ? '!' : n, k === 'approvals' ? 'warn' : 'danger');
}
function renderSide(cur) {
  $('#side').innerHTML = ROUTES.map(([g, items]) => (g ? `<h6>${g}</h6>` : '') + items.map(([k, ic, t, r]) =>
    `<a href="#${k}" class="${k === cur ? 'on' : ''} ${rank() < r ? 'locked' : ''}">${I(ic)}<span>${t}</span>${rank() < r ? `<span class="badge">${I('lock')}</span>` : `<span class="badge">${badge(k)}</span>`}</a>`).join('')).join('') +
    `<div class="protect"><b>${I('lock')}Protected mode on</b>Phones and exact fields stay hidden until a farmer asks an officer for help. Every look is written to the history.</div>`;
  const n = S.approvals.length; $('#approvalsCount').textContent = n; $('#approvalsCount').classList.toggle('zero', !n);
}
function route() {
  const k = (location.hash || '#overview').slice(1).split('/')[0];
  const r = FLAT[k] || FLAT.overview;
  Object.keys(MAPS).forEach(x => { if (x !== 'drawer') { MAPS[x].remove(); delete MAPS[x]; } });
  renderSide(r[0]);
  const main = $('#main');
  if (rank() < r[3]) {
    main.innerHTML = `<div class="lock-page"><div class="ico warn">${I('lock')}</div><h2>${r[2]} is closed for your role</h2><p class="sub">You are signed in as <b>${ME.name}</b>, ${ROLE[ME.role][1]}. This section needs ${['a viewer', 'a district officer', 'an admin'][r[3]]}. Switch officer in the top right corner to see it.</p></div>`;
  } else {
    main.innerHTML = PAGES[r[0]]();
    (AFTER[r[0]] || (() => {}))();
  }
  refreshIcons(); window.scrollTo(0, 0);
}
function rerender() { const y = window.scrollY; route(); window.scrollTo(0, y); }

// ---------- pages ----------
const PAGES = {}, AFTER = {};

PAGES.overview = () => {
  const visible = FARMS.filter(inArea), dun = visible.reduce((a, f) => a + f.area, 0);
  const needs = [
    ['stamp', 'warn', `${S.approvals.length} changes wait for a second officer`, 'alerts, rules, prices, water plan, an account delete', '#approvals'],
    ['inbox', 'danger', `${S.inbox.filter(i => i.state === 'new').length} new in the inbox`, '2 Doctor cases ask for an officer', '#inbox'],
    ['alert-triangle', 'danger', 'One sign-in code works for every phone', 'AUTH__FIXED_SIGN_IN_CODE is set: turn it off before real farmers', '#security'],
    ['waves', 'danger', 'Dams job is late', 'no reading today: the source page did not answer', '#jobs'],
    ['key-round', 'danger', 'The Doctor has no API key', 'GEMINI_API_KEY is missing, so Ask the Doctor cannot answer', '#doctor'],
    ['smartphone', 'warn', '5% of phones run app 1.0.0 (test mode build)', 'force an update below 1.0.2', '#app'],
    ['shield-alert', 'warn', 'Rebaz Salih has no 2-step sign-in', 'blocked from farms until it is on', '#officers'],
  ];
  return head(NOW.toLocaleDateString('en-GB', { weekday: 'long', day: 'numeric', month: 'long' }) + ' · 10:46', `Good morning, ${ME.name.split(' ')[0]}`,
    `What needs you today across ${ME.areas.join(', ')}. Every number comes from the app, the data jobs and the officers' work.`,
    `<a class="btn" href="#audit">${I('history')}History</a><a class="btn primary" href="#alerts">${I('bell-ring')}New alert</a>`) +
  `<div class="grid g4" style="margin-bottom:14px">
    ${kpi('Farmers', fmt(FARMERS.length), '+37 this week · 61% use Sorani', '', 'users')}
    ${kpi('Farms', fmt(visible.length), fmt(dun) + ' dunam registered', '', 'map')}
    ${kpi('Waiting for approval', S.approvals.length, 'two officers for alerts, rules, deletes', 'warn', 'stamp')}
    ${kpi('Data jobs on time', '6 of 7', 'dams late since yesterday', 'danger', 'activity')}
  </div>
  <div class="grid g-main-l">
    <div class="stack">
      <div class="card"><div class="card-head"><span class="eyebrow">Needs you</span></div>
        ${needs.map(([ic, t, a, b, h]) => `<a class="list-item" href="${h}" style="color:inherit"><span class="ico ${t}">${I(ic)}</span><div style="flex:1"><b>${a}</b><div class="muted small">${b}</div></div>${I('chevron-right')}</a>`).join('')}
      </div>
      <div class="card"><div class="card-head"><span class="eyebrow">Last actions</span><a href="#audit" class="small">All history</a></div>
        ${auditRows(S.audit.slice(0, 6))}
      </div>
    </div>
    <div class="stack">
      <div class="card"><div class="card-head"><span class="eyebrow">Registered farms per district</span><span class="muted small">click a district</span></div>
        <div id="ovMap" class="map sm"></div>
        <div class="row small muted" style="margin-top:8px">${['#EAF2EC', '#BFDCC8', '#7FBC93', '#3E9461', '#0F5B4B'].map((c, i) => `<span><span class="dotc" style="background:${c}"></span>${['under 20', '20 to 50', '50 to 100', '100 to 200', '200+'][i]}</span>`).join('')}</div>
      </div>
      <div class="card"><div class="card-head"><span class="eyebrow">App today</span></div>
        <dl class="facts"><dt>Farmers who opened the app</dt><dd>812</dd><dt>Farms saved today</dt><dd>23 (4 made offline)</dd><dt>Questions to the Doctor</dt><dd>0 · no API key</dd><dt>Reports sent</dt><dd>7</dd><dt>Pushes sent</dt><dd>142 · 71% opened</dd><dt>Alwa listings opened</dt><dd>12</dd></dl>
      </div>
    </div>
  </div>`;
};
AFTER.overview = () => {
  const cnt = {}; FARMS.forEach(f => cnt[f.dist] = (cnt[f.dist] || 0) + 1);
  districtMap('ovMap', p => ({ fillColor: ramp(cnt[p.en] || 0, [20, 50, 100, 200], ['#EAF2EC', '#BFDCC8', '#7FBC93', '#3E9461', '#0F5B4B']) }),
    p => { S.farmFilter = Object.assign(S.farmFilter, { gov: p.gov, dist: p.en, page: 0 }); location.hash = '#farms'; });
};

function auditRows(rows) {
  return rows.map(([t, who, what, more, ic, tone]) => `<div class="list-item" style="cursor:default"><span class="muted small nowrap" style="width:64px">${t}</span><span class="ico ${tone}">${I(ic)}</span><div style="flex:1;min-width:0"><b>${esc(who)}</b> ${esc(what)}<div class="muted small">${esc(more)}</div></div></div>`).join('');
}

// approvals
PAGES.approvals = () => head('Two-officer rule', 'Approvals', 'Alerts, rule changes, price publishing, water plans, account deletes and backup restores need a second officer. You cannot approve what you asked for.') +
  `<div class="card">${S.approvals.length ? S.approvals.map(a => `<div class="list-item" style="cursor:default">
    <span class="ico ${a.kind === 'delete' ? 'danger' : 'warn'}">${I({ alert: 'bell-ring', rule: 'sliders-horizontal', price: 'store', plan: 'droplets', delete: 'trash-2', restore: 'database' }[a.kind])}</span>
    <div style="flex:1"><b>${esc(a.title)}</b><div class="muted small">${esc(a.detail)}</div><div class="small">asked by <b>${a.by}</b> · ${a.at} · needs ${a.need === 2 ? 'an admin' : 'any officer'}</div></div>
    <button class="btn sm danger" onclick="A.reject('${a.id}')">${I('x')}Turn down</button><button class="btn sm primary" onclick="A.approve('${a.id}')">${I('check')}Approve</button></div>`).join('') : '<div class="empty">Nothing waits for you.</div>'}</div>`;

// farmers and farms
function farmList() {
  const f = S.farmFilter, q = f.q.trim().toLowerCase();
  return FARMS.filter(x => inArea(x) && (!f.gov || x.gov === f.gov) && (!f.dist || x.dist === f.dist) && (!f.crop || x.main === f.crop) && (!f.level || x.level === f.level) && (!f.access || x.access === f.access) &&
    (!q || x.id.includes(q) || x.name.includes(q) || x.farmer.phone.endsWith(q) || x.sub.toLowerCase().includes(q))).sort((a, b) => b.sync - a.sync);
}
PAGES.farms = () => {
  const tab = S.tab.farms || 'farms', f = S.farmFilter;
  const list = farmList(), per = 25, pages = Math.max(1, Math.ceil(list.length / per)); f.page = Math.min(f.page, pages - 1);
  const rows = list.slice(f.page * per, f.page * per + per);
  const opt = (arr, v) => arr.map(([k, t]) => `<option value="${k}" ${k === v ? 'selected' : ''}>${t}</option>`).join('');
  const dists = DISTS.filter(d => !f.gov || d.gov === f.gov).map(d => [d.en, d.en]);
  const filters = `<div class="filters">
    <input class="grow" type="text" placeholder="Farm name, farm id, last 4 digits of the phone, sub-district" value="${esc(f.q)}" oninput="A.ff('q', this.value)">
    <select onchange="A.ff('gov', this.value); S.farmFilter.dist=''">${opt([['', 'All governorates'], ...GOVS.filter(g => ME.areas.includes('All Kurdistan') || ME.areas.includes(g)).map(g => [g, g])], f.gov)}</select>
    <select onchange="A.ff('dist', this.value)">${opt([['', 'All districts'], ...dists], f.dist)}</select>
    <select onchange="A.ff('crop', this.value)">${opt([['', 'All crops'], ...CROP_KEYS.map(c => [c, c])], f.crop)}</select>
    <select onchange="A.ff('level', this.value)">${opt([['', 'Any status'], ...Object.entries(LEVELS).map(([k, v]) => [k, v[1]])], f.level)}</select>
    <select onchange="A.ff('access', this.value)">${opt([['', 'Any access'], ['1km', '1 km only'], ['open', 'Open (farmer asked)'], ['blocked', 'Blocked']], f.access)}</select>
    <button class="btn" onclick="S.farmFilter={q:'',gov:'',dist:'',crop:'',level:'',access:'',page:0};rerender()">${I('rotate-ccw')}Clear</button></div>`;
  const dun = list.reduce((a, x) => a + x.area, 0);
  const farmsTable = `<div class="card"><div class="spread" style="margin-bottom:8px"><b>${fmt(list.length)} farms · ${fmt(dun)} dunam</b><span class="muted small">newest sync first</span></div><div class="table-wrap"><table class="t">
    <tr><th>Farm and farmer</th><th>Sub-district</th><th class="num">Area</th><th>Main crop</th><th>Status</th><th>Last sync</th><th>Access</th></tr>
    ${rows.map(x => `<tr class="click" onclick="A.farm('${x.id}')"><td><div class="ku" style="text-align:left;font-weight:600">${x.name}</div><div class="muted small">#${x.id} · ${S.revealed.has(x.id) ? full(x.farmer.phone) : mask(x.farmer.phone)}${x.offline ? ' · made offline' : ''}</div></td>
      <td>${x.sub}<div class="muted small">${x.dist}, ${x.gov}</div></td><td class="num">${fmt(x.area, 1)} du</td><td>${cropTag(x.main)}</td><td>${levelPill(x.level)}</td><td class="nowrap">${when(x.sync)}</td>
      <td>${x.access === 'open' ? pill('Open', 'brand', 'unlock') : x.access === 'blocked' ? pill('Blocked', 'danger', 'ban') : pill('1 km', '', 'lock')}</td></tr>`).join('')}
    </table></div>${list.length ? '' : '<div class="empty">No farm matches.</div>'}
    <div class="spread" style="margin-top:10px"><span class="muted small">Page ${f.page + 1} of ${pages} · Open = the farmer sent a report or a Doctor case to an officer</span>
    <div class="row"><button class="btn sm" ${f.page ? '' : 'disabled'} onclick="A.ff('page', ${f.page - 1})">${I('chevron-left')}Previous</button><button class="btn sm" ${f.page < pages - 1 ? '' : 'disabled'} onclick="A.ff('page', ${f.page + 1})">Next${I('chevron-right')}</button></div></div></div>`;
  const farmers = FARMERS.filter(p => p.farms.some(inArea)).slice(0, 40);
  const farmersTable = `<div class="card"><div class="table-wrap"><table class="t"><tr><th>Farmer (phone)</th><th class="num">Farms</th><th class="num">Dunam</th><th>Language</th><th>App</th><th>Push</th><th>Joined</th><th>State</th><th></th></tr>
    ${farmers.map(p => `<tr><td class="mono">${mask(p.phone)}</td><td class="num">${p.farms.length}</td><td class="num">${fmt(p.farms.reduce((a, x) => a + x.area, 0), 1)}</td><td>${{ ku: 'Sorani', en: 'English', kmr: 'Kurmanji', ar: 'Arabic' }[p.lang]}</td>
      <td>${pill(p.app, p.app === '1.0.0' ? 'danger' : p.app === '1.0.2' ? 'warn' : 'good')}</td><td>${p.push ? I('bell') : '<span class="muted small">no token</span>'}</td><td>${when(p.joined)}</td><td>${p.blocked ? pill('Blocked', 'danger') : pill('Active', 'good')}</td>
      <td><button class="btn sm" onclick="A.farm('${p.farms[0].id}')">Open</button></td></tr>`).join('')}</table></div><div class="muted small" style="margin-top:8px">First 40 of ${fmt(FARMERS.length)}. A farmer is only a phone number: the app never asks for a name.</div></div>`;
  return head('People', 'Farmers and farms', 'Everything the app knows about each farm: walked corners, 10 m squares and crops, exact area, sync times. Phones show the last 4 digits; fields show at 1 km until the farmer asks for help.',
    `<button class="btn" onclick="csv('farms')">${I('download')}Export CSV</button>`) +
    `<div class="grid g4" style="margin-bottom:14px">${kpi('Farmers', fmt(FARMERS.length), '61% Sorani · 22% Kurmanji', '', 'users')}${kpi('Farms', fmt(FARMS.length), '1.6 per farmer', '', 'map')}${kpi('Made offline', fmt(FARMS.filter(x => x.offline).length), 'saved in the field, sent later', '', 'wifi-off')}${kpi('Waiting for a picture', fmt(FARMS.filter(x => x.level === 'none').length), 'no satellite reading yet', 'warn', 'satellite')}</div>` +
    `<div class="tabs">${[['farms', 'Farms'], ['farmers', 'Farmers']].map(([k, t]) => `<button class="${tab === k ? 'on' : ''}" onclick="S.tab.farms='${k}';rerender()">${t}</button>`).join('')}</div>` +
    (tab === 'farms' ? filters + farmsTable : farmersTable);
};
function farmSvg(f, k = 1.39, depth = 0) {
  let s = f.seed; const r = () => { s = (s * 16807) % 2147483647; return s / 2147483647; };
  const target = Math.max(8, Math.round(f.area * 25)), cells = Math.round(target * k), W = Math.max(4, Math.round(Math.sqrt(cells * 1.5))), H = Math.max(3, Math.round(cells / W)) + 1;
  const pts = []; const n = Math.min(f.corners, 9);
  for (let k = 0; k < n; k++) { const a = k / n * Math.PI * 2 + r() * 0.4; pts.push([W / 2 + Math.cos(a) * W / 2 * (0.82 + r() * 0.18), H / 2 + Math.sin(a) * H / 2 * (0.82 + r() * 0.18)]); }
  const inside = (x, y) => { let c = false; for (let i = 0, j = n - 1; i < n; j = i++) { const [xi, yi] = pts[i], [xj, yj] = pts[j]; if ((yi > y) !== (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi) c = !c; } return c; };
  const sc = Math.min(460 / W, 230 / H), cut = f.crops.length > 1 ? f.crops[0][1] / f.area : 1; let out = '', weak = 0, total = 0;
  for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) {
    if (!inside(x + 0.5, y + 0.5)) continue; total++;
    const crop = x / W < cut ? f.crops[0][0] : f.crops[1][0]; const isWeak = f.level !== 'normal' && f.level !== 'none' && x > W * 0.55 && y < H * 0.45 && r() < 0.7; weak += isWeak;
    out += `<rect x="${x * sc + 0.5}" y="${y * sc + 0.5}" width="${sc - 1}" height="${sc - 1}" rx="${sc > 6 ? 1.5 : 0}" fill="${CROPS[crop][0]}" ${isWeak ? `stroke="#9A6410" stroke-width="${Math.max(1, sc / 7)}"` : ''}/>`;
  }
  if (depth < 4 && Math.abs(total - target) / target > 0.06) return farmSvg(f, k * target / Math.max(1, total), depth + 1);
  const poly = pts.map(([x, y]) => `${x * sc},${y * sc}`).join(' ');
  return { svg: `<svg class="farm-svg" viewBox="-8 -8 ${W * sc + 16} ${H * sc + 16}">${out}<polygon points="${poly}" fill="none" stroke="#14201A" stroke-width="2" stroke-linejoin="round"/>${pts.map(([x, y]) => `<circle cx="${x * sc}" cy="${y * sc}" r="4" fill="#fff" stroke="#14201A" stroke-width="2"/>`).join('')}</svg>`, weak, total };
}
const A_farm = id => {
  const f = FARMS.find(x => x.id === id); if (!f) return;
  const open = f.access === 'open', d = farmSvg(f), ph = S.revealed.has(f.id) ? full(f.farmer.phone) : mask(f.farmer.phone);
  drawer(`<div class="spread"><span class="eyebrow">Farm #${f.id} · ${f.sub}, ${f.dist}</span><button class="icon-btn" onclick="closeDrawer()">${I('x')}</button></div>
    <div class="spread" style="margin:6px 0 12px"><h2 class="ku" style="font-family:var(--ku)">${f.name}</h2>${levelPill(f.level)}</div>
    ${open ? `<div class="note brand">${I('unlock')}<span>Open because the farmer sent ${f.reason}. Closes again when it is closed in the inbox.</span></div>` :
      f.access === 'blocked' ? `<div class="note danger">${I('ban')}<span>This farmer is blocked: the app signs them out and refuses new farms.</span></div>` :
      `<div class="note">${I('lock')}<span class="muted">Protected: only the sub-district and a point rounded to 1 km are shown. The exact field opens when the farmer asks an officer for help.</span></div>`}
    <div style="margin:12px 0">${open ? d.svg : `<div id="farmMap" class="map sm" style="height:240px"></div>`}</div>
    ${open ? `<div class="row small" style="margin-bottom:10px">${f.crops.map(c => cropTag(c[0])).join('')}<span><span class="dotc" style="border:2px solid #9A6410"></span>Weak square</span></div>` : ''}
    <dl class="facts">
      <dt>Area inside the outline</dt><dd>${fmt(f.area, 1)} dunam · ${fmt(f.area * 2500)} m²</dd>
      <dt>10 m squares</dt><dd>${open ? `${d.total} · ${d.weak} weak` : fmt(Math.round(f.area * 25))}</dd>
      <dt>Crops</dt><dd>${f.crops.map(c => `${c[0]} ${fmt(c[1], 1)}`).join(' · ')} du</dd>
      <dt>Corners walked</dt><dd>${f.corners} · GPS within ${f.gps[0]} to ${f.gps[1]} m</dd>
      <dt>Saved</dt><dd>${f.offline ? 'offline, sent later' : 'online'} · last sync ${when(f.sync)}</dd>
      <dt>Satellite</dt><dd>${f.level === 'none' ? 'waiting for the first picture' : 'picture 5 Oct · next 10 Oct'}</dd>
      <dt>Farmer</dt><dd>${ph} · ${f.farmer.farms.length} farm${f.farmer.farms.length > 1 ? 's' : ''} · app ${f.farmer.app}</dd>
      <dt>Point</dt><dd>${open ? `${f.lat.toFixed(5)} N · ${f.lon.toFixed(5)} E` : `about ${f.lat.toFixed(2)} N · ${f.lon.toFixed(2)} E (1 km)`}</dd>
    </dl>
    <div class="grid g2" style="margin-top:16px">
      <button class="btn" onclick="A.reveal('${f.id}')" ${S.revealed.has(f.id) ? 'disabled' : ''}>${I('eye')}Show phone</button>
      <button class="btn" onclick="A.exportFarm('${f.id}')">${I('file-down')}Farmer's data copy</button>
      <button class="btn danger" onclick="A.block('${f.id}')">${I('ban')}${f.access === 'blocked' ? 'Unblock' : 'Block'}</button>
      <button class="btn danger" onclick="A.del('${f.id}')">${I('trash-2')}Delete on request</button>
    </div>
    <p class="muted small" style="margin-top:12px">Officers cannot change an outline or its crops: only the farmer can. Showing the phone asks for a reason and goes to the history.</p>`);
  if (!open) setTimeout(() => {
    const m = L.map('farmMap', { scrollWheelZoom: false }).setView([f.lat, f.lon], 13); MAPS.drawer = m;
    L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', { attribution: '© OpenStreetMap' }).addTo(m);
    L.circle([+f.lat.toFixed(2), +f.lon.toFixed(2)], { radius: 1000, color: '#0F5B4B', fillOpacity: 0.12 }).addTo(m);
  }, 30);
};

// crop register
PAGES.crops = () => {
  const vis = FARMS.filter(f => rank() < 1 || inArea(f)), tot = {}; let all = 0;
  vis.forEach(f => f.crops.forEach(([c, a]) => { tot[c] = (tot[c] || 0) + a; all += a; }));
  const byD = {}; vis.forEach(f => { const d = byD[f.dist] = byD[f.dist] || { farms: 0, dun: 0, c: {} }; d.farms++; f.crops.forEach(([c, a]) => { d.dun += a; d.c[c] = (d.c[c] || 0) + a; }); });
  const top = Object.entries(byD).sort((a, b) => b[1].dun - a[1].dun);
  const v = S.cropView, max = Math.max(...Object.values(tot));
  return head('Fields and region', 'Crop register', 'Painted 10 m squares added up per district. Only farms registered in the app, not a census: use it for trends and where to look.',
    `<button class="btn" onclick="csv('crop_register')">${I('download')}Export CSV</button>`) +
  `<div class="chips" style="margin-bottom:12px">${['all', ...CROP_KEYS].map(c => `<button class="chip ${v === c ? 'on' : ''}" onclick="S.cropView='${c}';rerender()">${c === 'all' ? 'All crops' : CROPS[c][2] + ' ' + c}</button>`).join('')}</div>
  <div class="grid g-main">
    <div class="card"><div class="card-head"><span class="eyebrow">${v === 'all' ? 'Registered dunam' : v + ' dunam'} per district</span></div><div id="cropMap" class="map"></div></div>
    <div class="stack">
      <div class="card"><div class="eyebrow">All registered farms</div><div style="font:700 28px var(--display);margin:4px 0 10px">${fmt(all)} dunam</div>
        ${Object.entries(tot).sort((a, b) => b[1] - a[1]).map(([c, a]) => `<div class="row" style="margin:5px 0;flex-wrap:nowrap"><span style="width:110px">${cropTag(c)}</span><div class="bar" style="flex:1"><i style="width:${a / max * 100}%;background:${CROPS[c][0]}"></i></div><b style="width:60px" class="right">${fmt(a)}</b><span class="muted small right" style="width:44px">${fmt(a / all * 100, 1)}%</span></div>`).join('')}
      </div>
      <div class="card"><div class="eyebrow" style="margin-bottom:6px">Top districts</div><table class="t"><tr><th>District</th><th class="num">Farms</th><th class="num">Dunam</th><th>Main crop</th></tr>
        ${top.slice(0, 8).map(([d, x]) => { const mc = Object.entries(x.c).sort((a, b) => b[1] - a[1])[0]; return `<tr><td><b>${d}</b></td><td class="num">${x.farms}</td><td class="num">${fmt(x.dun)}</td><td>${cropTag(mc[0])} <span class="muted small">${fmt(mc[1] / x.dun * 100)}%</span></td></tr>`; }).join('')}</table></div>
    </div></div>`;
};
AFTER.crops = () => {
  const val = {}; FARMS.forEach(f => f.crops.forEach(([c, a]) => { if (S.cropView === 'all' || S.cropView === c) val[f.dist] = (val[f.dist] || 0) + a; }));
  const mx = Math.max(1, ...Object.values(val));
  districtMap('cropMap', p => ({ fillColor: ramp((val[p.en] || 0) / mx, [0.05, 0.15, 0.35, 0.6], ['#F3EFD9', '#E6DA9C', '#D6BE5A', '#B8932A', '#7E5F12']) }),
    p => toast(`${p.en}: ${fmt(val[p.en] || 0)} dunam ${S.cropView === 'all' ? '' : 'of ' + S.cropView}`));
};

// region data
PAGES.region = () => {
  const tab = S.tab.region || 'dryness';
  const band = v => v < 25 ? ['Much greener', 'good'] : v < 45 ? ['Greener', 'good'] : v < 60 ? ['Normal', ''] : v < 80 ? ['Dry', 'warn'] : ['Very dry', 'danger'];
  let body = '';
  if (tab === 'dryness') {
    let s2 = 7; const rr = () => { s2 = (s2 * 16807) % 2147483647; return s2 / 2147483647; };
    const rows = DISTS.map(d => ({ d, v: Math.round(30 + rr() * 60), ch: Math.round((rr() - 0.4) * 20) })).sort((a, b) => b.v - a.v);
    body = `<div class="card"><div class="spread" style="margin-bottom:8px"><span class="muted small">Source: Sentinel-2 and CHIRPS rain via the dryness job · as of 3 Oct 04:40 · September 2026</span><button class="btn sm" onclick="A.run('Dryness by district')">${I('play')}Run again</button></div><div class="table-wrap"><table class="t">
      <tr><th>#</th><th>District</th><th>Governorate</th><th class="num">Dryness</th><th>Band</th><th class="num">vs 2025</th><th>Public dashboard</th></tr>
      ${rows.map((x, i) => `<tr><td>${i + 1}</td><td><b>${x.d.en}</b> <span class="ku muted">${x.d.ku}</span></td><td>${x.d.gov}</td><td class="num">${x.v}</td><td>${pill(...band(x.v))}</td><td class="num" style="color:${x.ch > 0 ? 'var(--danger)' : 'var(--good)'}">${x.ch > 0 ? '+' : ''}${x.ch}</td><td>${sw(true, "A.note('Shown or hidden on the public dashboard (demo)')")}</td></tr>`).join('')}</table></div></div>`;
  } else if (tab === 'dams') {
    body = `<div class="grid g2">${[['Dukan', 88, '6.1 of 7.0 bn m³', '19% in Aug 2025', 'Yesterday 07:02'], ['Darbandikhan', 54, '1.6 of 3.0 bn m³', '41% in Aug 2025', 'Yesterday 07:02']].map(([n, p, a, b, t]) =>
      `<div class="card"><div class="spread"><h2>${n}</h2>${pill('Late', 'danger')}</div><div style="font:700 34px var(--display);color:var(--water);margin:8px 0">${p}%</div><div class="bar"><i style="width:${p}%;background:var(--water)"></i></div><dl class="facts" style="margin-top:12px"><dt>Volume</dt><dd>${a}</dd><dt>A year ago</dt><dd>${b}</dd><dt>Last reading</dt><dd>${t}</dd><dt>Source</dt><dd>KRG Directorate of Dams page + Sentinel-2 lake area</dd></dl>
      <div class="row" style="margin-top:12px"><button class="btn sm" onclick="A.run('Dams')">${I('play')}Run again</button><button class="btn sm" onclick="A.modalManual('${n}')">${I('pencil')}Enter today's reading by hand</button></div></div>`).join('')}</div>`;
  } else if (tab === 'fires') {
    body = `<div class="card"><table class="t"><tr><th>Seen</th><th>District</th><th>Point</th><th>Confidence</th><th>Near farms</th><th>Satellite</th><th></th></tr>
      ${[['09:00', 'Soran', '36.65, 44.52', 'high', 'no', 'VIIRS'], ['09:00', 'Amedi', '37.09, 43.48', 'nominal', 'no', 'VIIRS'], ['06:10', 'Penjwen', '35.62, 45.94', 'nominal', '2 farms within 1 km', 'MODIS'], ['06:10', 'Kalar', '34.62, 45.31', 'high', '1 farm, report r_3371', 'VIIRS']].map(r =>
        `<tr><td>${r[0]}</td><td><b>${r[1]}</b></td><td class="mono">${r[2]}</td><td>${pill(r[3], r[3] === 'high' ? 'danger' : 'warn')}</td><td>${r[4]}</td><td>${r[5]}</td><td><button class="btn sm" onclick="A.note('Marked as checked: not a wildfire (demo)')">Not a fire</button></td></tr>`).join('')}</table>
      <p class="muted small">Source: NASA FIRMS every 3 hours. A fire within 1 km of a registered farm creates an inbox item.</p></div>`;
  } else if (tab === 'outlook') {
    body = `<div class="card"><div class="note warn" style="margin-bottom:10px">${I('info')}<span>Season outlooks are weak evidence (track record 64%). The public dashboard shows them with their confidence. Hide them if you do not trust this month's run.</span></div><table class="t"><tr><th>District</th><th>Next season</th><th>Confidence</th><th>Method track record</th><th>Show publicly</th></tr>
      ${DISTS.slice(0, 12).map((d, i) => `<tr><td><b>${d.en}</b></td><td>${pill(['Good', 'Normal', 'Bad'][i % 3], ['good', '', 'danger'][i % 3])}</td><td>${['sure', 'likely', 'unsure'][(i + 1) % 3]}</td><td>${60 + (i * 7) % 15}%</td><td>${sw(i % 4 !== 3)}</td></tr>`).join('')}</table></div>`;
  } else {
    body = `<div class="card"><div class="spread" style="margin-bottom:8px"><span><b>Season 2026/27</b> · 2.4 bn m³ can be supplied · plan 1.46 bn m³ · reserve 0.94 bn m³</span>${pill('Waiting for a second officer', 'warn')}</div><table class="t"><tr><th>#</th><th>District</th><th class="num">Need</th><th>From</th><th class="num">Send (M m³)</th><th></th></tr>
      ${[['Makhmur', 93, 'Dukan', 42], ['Kalar', 88, 'Darbandikhan', 38], ['Chamchamal', 81, 'Dukan', 31], ['Erbil', 72, 'Dukan', 24], ['Koya', 66, 'Dukan', 15], ['Taqtaq', 64, 'Darbandikhan', 11], ['Darbandikhan', 63, 'Darbandikhan', 9], ['Qushtapa', 61, 'Dukan', 8]].map((r, i) =>
        `<tr><td>${i + 1}</td><td><b>${r[0]}</b></td><td class="num">${r[1]}</td><td>${r[2]}</td><td class="num"><input class="cell" type="number" value="${r[3]}" onchange="A.note('Changed: the plan needs approval again (demo)')"></td><td>${i < 2 ? pill('Urgent', 'danger') : ''}</td></tr>`).join('')}</table>
      <div class="row" style="margin-top:12px"><button class="btn primary" onclick="location.hash='#approvals'">${I('stamp')}Go to approval</button><span class="muted small">Research rated this plan about 3/10 for trust: it is advice for the Ministry, not an order.</span></div></div>`;
  }
  return head('Fields and region', 'Region data', 'What the public dashboard shows: dryness, dams, fires, the season outlook and the water plan. Numbers come from jobs; officers can re-run a job, enter a missing reading by hand, or hide a weak number.') +
    `<div class="tabs">${[['dryness', 'Dryness by district'], ['dams', 'Dams'], ['fires', 'Fires'], ['outlook', 'Season outlook'], ['water', 'Water plan']].map(([k, t]) => `<button class="${tab === k ? 'on' : ''}" onclick="S.tab.region='${k}';rerender()">${t}</button>`).join('')}</div>` + body;
};

// alerts
const ALERT_TYPES = [['frost', 'snowflake'], ['heat', 'thermometer-sun'], ['heavy_rain', 'cloud-rain'], ['dry_spell', 'sun'], ['rust_weather', 'leaf'], ['sunn_pest', 'bug'], ['dust', 'wind'], ['spray_window', 'spray-can'], ['sowing_rain', 'sprout'], ['urea_rain', 'flask-conical']];
PAGES.alerts = () => {
  const tab = S.tab.alerts || 'new';
  const sel = [...S.alertSel], fs = FARMS.filter(f => S.alertSel.has(f.dist)), farmers = new Set(fs.map(f => f.farmer.id)).size;
  const pushedToday = Math.round(fs.length * 0.16), noTok = fs.filter(f => !f.farmer.push).length;
  const t = S.alertType || 'frost';
  const composer = `<div class="grid g-main">
    <div class="card"><div class="card-head"><span class="eyebrow">1 · Where: click districts on the map</span><span class="muted small">${sel.length} selected</span><button class="btn sm" onclick="S.alertSel.clear();rerender()">Clear</button></div><div id="alertMap" class="map"></div></div>
    <div class="stack"><div class="card">
      <div class="eyebrow">2 · What</div><div class="chips" style="margin:8px 0">${ALERT_TYPES.map(([k, ic]) => `<button class="chip ${t === k ? 'on' : ''}" onclick="S.alertType='${k}';rerender()">${I(ic)}${k.replace('_', ' ')}</button>`).join('')}</div>
      <div class="grid g3"><label class="f">Day<input type="date" value="2026-10-11"></label><label class="f">Level<select><option>alarm (pushed)</option><option>watch (in the app only)</option></select></label><label class="f">How sure<select><option>likely</option><option>sure</option><option>unsure</option></select></label></div>
      <div class="eyebrow" style="margin-top:14px">3 · Message</div>
      <label class="f" style="margin-top:8px">Sorani <small>shown first</small><textarea class="ku" rows="3" oninput="this.nextElementSibling.textContent=this.value.length+' / 160'">سەرمای توند: شەوی یەکشەممە پلەی گەرما دادەبەزێت بۆ ٣- پلە. شەممە تەماتە و خەیاری پێگەیشتوو بچنەوە.</textarea><small>104 / 160</small></label>
      <label class="f" style="margin-top:8px">English<textarea rows="2" oninput="this.nextElementSibling.textContent=this.value.length+' / 160'">Hard frost: Sunday night down to -3 °C. On Saturday pick ripe tomatoes and cucumbers and cover young plants.</textarea><small>104 / 160</small></label>
      <div class="note warn" style="margin-top:8px">${I('alert-triangle')}<span>Sorani not checked by a native speaker yet. No doses and no product names in alerts (locked rule).</span></div>
      <div class="eyebrow" style="margin-top:14px">4 · Who gets it</div>
      <dl class="facts" style="margin-top:8px"><dt>Farms in the area</dt><dd>${fmt(fs.length)} farms · ${fmt(farmers)} farmers</dd><dt>Already pushed today</dt><dd style="color:var(--warn)">${pushedToday} → get it tomorrow 06:00</dd><dt>No phone token for push</dt><dd>${noTok} → see it in the app only</dd><dt>SMS fallback</dt><dd class="muted">off: no SMS provider</dd></dl>
      <div class="phone" style="margin-top:14px"><div class="clock">09:41</div><div class="date">Sat 10 Oct</div><div class="push"><div class="row small" style="flex-wrap:nowrap"><b>JUTYAR</b><span class="muted" style="margin-left:auto">now</span></div><div class="ku" style="font-weight:700">سەرمای توند · یەکشەممە</div><div class="ku small">شەممە تەماتە و خەیاری پێگەیشتوو بچنەوە</div></div></div>
      <div class="row" style="margin-top:14px"><button class="btn primary" onclick="A.sendAlert()">${I('send')}Send for approval</button><button class="btn" onclick="A.note('Draft saved (demo)')">Save draft</button><button class="btn" onclick="A.note('Test push sent to your own phone (demo)')">${I('smartphone')}Test on my phone</button></div>
    </div></div></div>`;
  const listT = `<div class="card"><table class="t"><tr><th>Alert</th><th>Area</th><th>Day</th><th class="num">Farms</th><th>State</th><th>Asked by</th><th>Approved by</th><th class="num">Opened</th><th></th></tr>
    ${S.alerts.map(a => `<tr><td><b>${a.title}</b><div class="muted small">${a.type}</div></td><td>${a.area}</td><td>${a.day}</td><td class="num">${a.farms}</td><td>${pill({ waiting: 'Waiting approval', sent: 'Sent', stopped: 'Stopped', draft: 'Draft' }[a.state], { waiting: 'warn', sent: 'good', stopped: 'danger', draft: '' }[a.state])}</td>
      <td>${a.by}</td><td>${a.ok || '-'}</td><td class="num">${a.opened ? a.opened + '%' : '-'}</td><td>${a.state === 'sent' ? `<button class="btn sm danger" onclick="A.note('A correction push needs approval like any alert (demo)')">Correct</button>` : a.state === 'waiting' ? `<button class="btn sm" onclick="location.hash='#approvals'">Review</button>` : ''}</td></tr>`).join('')}</table></div>`;
  return head('Act', 'Alerts', 'Write one alert for an area, see who gets it, and send it after a second officer agrees. The app pushes only Alarm level, at most once per farm per day.') +
    `<div class="tabs">${[['new', 'New alert'], ['list', 'All alerts']].map(([k, tt]) => `<button class="${tab === k ? 'on' : ''}" onclick="S.tab.alerts='${k}';rerender()">${tt}</button>`).join('')}</div>` + (tab === 'new' ? composer : listT);
};
AFTER.alerts = () => {
  if ((S.tab.alerts || 'new') !== 'new') return;
  const m = districtMap('alertMap', p => ({ fillColor: S.alertSel.has(p.en) ? '#C2452A' : '#E7E4DA' }), (p, l) => {
    S.alertSel.has(p.en) ? S.alertSel.delete(p.en) : S.alertSel.add(p.en); rerender();
  }, 'main');
};

// inbox
PAGES.inbox = () => {
  const tab = S.tab.inbox || 'all';
  const list = S.inbox.filter(i => (tab === 'all' ? i.state !== 'closed' : tab === 'reports' ? i.kind === 'report' : tab === 'doctor' ? i.kind === 'case' : tab === 'unassigned' ? !i.assigned : i.state === 'closed') && (ME.areas.includes('All Kurdistan') || ME.areas.includes(i.gov)));
  const cur = S.inbox.find(i => i.id === S.sel.inbox) || list[0];
  const st = { new: ['New', 'danger'], assigned: ['Assigned', 'water'], seen: ['Seen', ''], forwarded: ['Sent on', 'brand'], closed: ['Closed', ''] };
  const detail = cur ? `<div class="card">
      <div class="spread"><div class="row">${pill(cur.id, cur.kind === 'case' ? 'water' : 'danger', cur.kind === 'case' ? 'stethoscope' : 'message-square-warning')}${pill(cur.type.replace('_', ' '))}${pill(...st[cur.state])}</div>
        <select onchange="A.assign('${cur.id}', this.value)"><option>${cur.assigned ? 'Assigned: ' + cur.assigned : 'Assign to…'}</option>${OFFICERS.filter(o => o.role !== 'viewer').map(o => `<option>${o.name}</option>`).join('')}<option>Vets, Sulaymaniyah</option><option>Plant protection, Erbil</option></select></div>
      <h2 style="margin:10px 0 4px">${cur.title}</h2><div class="muted">Farm #${cur.farm} · ${cur.where} · ${cur.at} · ${cur.photos} photo${cur.photos > 1 ? 's' : ''}</div>
      <div class="grid g2" style="margin-top:14px">
        <div class="stack"><div style="height:170px;border-radius:10px;background:linear-gradient(160deg,#6E8B3D,#8FA654);display:flex;align-items:flex-end;padding:10px;color:#fff;font-weight:600">${I('image')}&nbsp;1 of ${cur.photos} photos</div>
          <div><div class="eyebrow">Farmer wrote</div><div class="ku" style="font-size:14px;margin:4px 0">${cur.note}</div><div class="muted small">“${cur.en}” (machine translation)</div></div>
          <div class="note brand">${I('unlock')}<span>Farm #${cur.farm} is open to officers of ${cur.gov} while this is open.</span></div></div>
        <div class="stack">${cur.doctor ? `<div class="note info" style="flex-direction:column"><b>${I('stethoscope')} What the Doctor thinks · ${cur.doctor.conf}</b><span style="color:var(--ink)">${cur.doctor.likely}</span>${cur.doctor.why.map(([a, b]) => `<span class="small"><b>${a}:</b> ${b}</span>`).join('')}</div>` : `<div class="note">${I('info')}<span class="muted">The farmer did not ask the Doctor.</span></div>`}
          <label class="f">Reply to the farmer (Sorani first)<textarea class="ku" rows="3">سڵاو، سبەینێ ئەندازیاری کشتوکاڵ سەردانی کێڵگەکەت دەکات.</textarea></label>
          <div class="row"><button class="btn primary sm" onclick="A.inbox('${cur.id}','seen','replied to the farmer')">${I('send')}Send reply</button><button class="btn sm" onclick="A.inbox('${cur.id}','seen','marked seen')">${I('eye')}Mark seen</button><button class="btn sm" onclick="A.inbox('${cur.id}','assigned','planned a field visit')">${I('car')}Plan a visit</button><button class="btn sm" onclick="A.inbox('${cur.id}','closed','closed')">${I('check-check')}Close</button></div>
          <span class="muted small">The farmer sees “Seen by an officer” and your reply in the app. Closing hides the exact farm again.</span></div></div></div>` : `<div class="card empty">Nothing here.</div>`;
  return head('Act', 'Inbox', 'Farmer reports and the Doctor cases that ask for an officer. Opening one opens that farm to officers of its area until it is closed.') +
    `<div class="tabs">${[['all', 'Open'], ['reports', 'Reports'], ['doctor', 'Doctor cases'], ['unassigned', 'Not assigned'], ['closed', 'Closed']].map(([k, t]) => `<button class="${tab === k ? 'on' : ''}" onclick="S.tab.inbox='${k}';rerender()">${t}</button>`).join('')}</div>
    <div class="grid" style="grid-template-columns:360px minmax(0,1fr)"><div class="card" style="padding:8px">${list.map(i => `<div class="list-item ${cur && i.id === cur.id ? 'sel' : ''}" onclick="S.sel.inbox='${i.id}';rerender()"><span class="ico ${i.kind === 'case' ? 'water' : 'danger'}">${I(i.kind === 'case' ? 'stethoscope' : 'message-square-warning')}</span><div style="flex:1;min-width:0"><b>${i.title}</b><div class="muted small">${i.where} · #${i.farm}</div></div><div class="right"><div class="muted small">${i.at}</div>${pill(...st[i.state])}</div></div>`).join('') || '<div class="empty">Empty</div>'}</div>${detail}</div>`;
};

// the doctor
PAGES.doctor = () => head('Act', 'The Doctor (AI)', 'Which model answers farmers, the hard rules it can never break, and a review of its answers. Admins change settings; district officers review answers.',
  `<button class="btn danger" onclick="A.feature('doctor')">${I('pause')}${S.flags.features.doctor ? 'Pause Ask the Doctor' : 'Turn Ask the Doctor on'}</button>`) +
  `<div class="note danger" style="margin-bottom:14px">${I('key-round')}<span><b>No API key.</b> GEMINI_API_KEY is missing in the server settings, so every question fails. Add the key from Google AI Studio in System settings.</span></div>
  <div class="grid g3" style="margin-bottom:14px">${kpi('Questions this week', '0', 'waiting for the key', '', 'message-circle')}${kpi('Sent to an officer', '2', 'Doctor unsure', 'warn', 'user-check')}${kpi('Cost estimate', '≈ $0.002', 'per answer with Gemini Flash', '', 'coins')}</div>
  <div class="grid g2"><div class="card"><div class="eyebrow">Model</div>
    ${set('Provider', 'decided 2026-10-08: Gemini for cost; Claude kept as the second choice', `<select ${rank() < 2 ? 'disabled' : ''}><option>Gemini (Google AI Studio)</option><option>Claude (Anthropic)</option></select>`)}
    ${set('Model', 'one switch on the server: FARM_DOCTOR_MODEL', `<select ${rank() < 2 ? 'disabled' : ''}><option>gemini-2.5-flash</option><option>gemini-2.5-pro</option><option>claude-sonnet-5-5</option></select>`)}
    ${set('Head-to-head test', '20 Sorani questions scored by a native speaker before real farmers', `<button class="btn sm" onclick="A.note('Test queued: 20 questions to both models (demo)')">${I('flask-conical')}Run test</button>`)}
    ${set('Answer time target', 'the app shows “reading the field” meanwhile', '<input type="number" value="25" style="width:70px"> s')}
    ${set('Photos per question', 'jpeg, at most 4 MB each', '<input type="number" value="6" style="width:70px">')}
    ${set('Voice questions', 'deferred: not in v1 (decided 2026-10-08)', sw(false, '', true))}
  </div><div class="card"><div class="eyebrow">Hard rules</div>
    ${set('No doses of pesticide or fertilizer', 'never a number of litres, kilograms or ml', locked('safety'))}
    ${set('No product or brand names', 'the Doctor names the problem, not what to buy', locked('safety'))}
    ${set('Only numbers from the AIs', 'Field Eye, Weather, Season, Neighbours, Dams', locked('honesty'))}
    ${set('Unsure when inputs conflict', 'answer “unsure” and send it to an officer', locked('honesty'))}
    ${set('At most 3 actions this week', 'short and doable', '<input type="number" value="3" style="width:70px">')}
    ${set('Send “likely” answers to officers too', 'more work for officers, safer for farmers', sw(false))}
  </div></div>
  <div class="card" style="margin-top:14px"><div class="card-head"><span class="eyebrow">Answers to review</span><span class="muted small">officers rate answers; bad ones teach the rulebook</span></div><table class="t"><tr><th>Case</th><th>Question</th><th>Answer</th><th>How sure</th><th>Sorani checked</th><th>Rating</th></tr>
    ${[['c_812', 'Tomato stems black', 'Cannot tell: stem rot or frost', 'unsure'], ['c_809', 'Wheat yellow at the tips', 'Likely nitrogen short after rain', 'likely'], ['c_805', 'Grape leaves curling', 'Unsure: leafroll or water stress', 'unsure'], ['c_801', 'When to sow wheat', 'Wait for 20 mm in 3 days, likely from Thursday', 'sure']].map(r =>
      `<tr><td class="mono">${r[0]}</td><td>${r[1]}</td><td>${r[2]}</td><td>${pill(r[3], r[3] === 'sure' ? 'good' : r[3] === 'likely' ? 'warn' : 'danger')}</td><td>${pill('not yet', 'warn')}</td><td class="nowrap"><button class="btn sm" onclick="A.note('Rated good (demo)')">${I('thumbs-up')}</button> <button class="btn sm" onclick="A.note('Rated bad: added to the rulebook review (demo)')">${I('thumbs-down')}</button></td></tr>`).join('')}</table></div>`;

// alwa
PAGES.alwa = () => {
  const tab = S.tab.alwa || 'prices';
  const P = [['wheat', [850, 850, 850, 850], 'Government price', 1], ['barley', [450, 460, 440, 430], 'Market board'], ['tomato', [750, 750, 800, 650], 'Market board'], ['cucumber', [600, 575, 650, 550], 'Market board'],
    ['potato', [500, 525, 480, 500], 'Market board'], ['onion', [400, 425, 400, 375], 'Market board'], ['watermelon', [300, 325, 275, 250], 'Market board'], ['grape', [1250, 1300, 1400, 1200], 'Market board'], ['chickpea', [1750, 1800, 1750, 1700], 'Market board']];
  let body;
  if (tab === 'prices') body = `<div class="grid g-main"><div class="card"><div class="spread" style="margin-bottom:8px"><span class="eyebrow">Today's prices · IQD per kg · Fri 9 Oct</span><button class="btn primary sm" onclick="A.publish()">${I('upload')}Publish changes</button></div><table class="t">
      <tr><th>Crop</th><th class="num">Sulaymaniyah</th><th class="num">Erbil</th><th class="num">Duhok</th><th class="num">Kalar</th><th>Source</th><th>Fixed</th></tr>
      ${P.map(([c, v, src, fx]) => `<tr><td>${cropTag(c)}</td>${v.map(x => `<td class="num"><input class="cell" type="number" value="${x}" ${fx ? 'disabled' : ''} onchange="this.style.background='var(--brand-soft)'"></td>`).join('')}<td class="small">${src}</td><td>${fx ? I('lock') : ''}</td></tr>`).join('')}</table>
      <p class="muted small">The wheat price is fixed by the government and locked. Changed cells turn green; publishing needs an admin who did not make the change.</p></div>
      <div class="card"><div class="eyebrow">Tomato · Sulaymaniyah · 14 days</div><div class="hist" style="margin-top:14px">${[700, 725, 750, 800, 775, 750, 750, 800, 850, 825, 800, 775, 750, 750].map((v, i) => `<div><span>${v}</span><b style="height:${(v - 600) * 0.4}px;${i === 13 ? 'background:var(--brand)' : ''}"></b><span>${(i + 26) % 31 || 31}</span></div>`).join('')}</div></div></div>`;
  else if (tab === 'listings') body = `<div class="card"><table class="t"><tr><th>#</th><th>Crop</th><th class="num">Tonnes</th><th class="num">Asking IQD/kg</th><th>Market price</th><th>Seller</th><th>Closes</th><th>State</th><th></th></tr>
      ${[[518, 'tomato', 40, 2400, 750, 'flag'], [512, 'wheat', 120, 850, 850, 'repeat'], [507, 'potato', 18, 520, 500, ''], [503, 'grape', 6, 1300, 1250, ''], [499, 'chickpea', 3, 1800, 1750, ''], [494, 'barley', 55, 450, 450, '']].map(r =>
        `<tr><td>#${r[0]}</td><td>${cropTag(r[1])}</td><td class="num">${r[2]}</td><td class="num">${fmt(r[3])}</td><td>${r[3] / r[4] > 2 ? pill((r[3] / r[4]).toFixed(1) + '× market', 'danger') : pill('normal', 'good')}</td><td class="mono">+964 750 ••• ${String(r[0] * 7).slice(-4)}</td><td>14 Oct</td><td>${r[5] ? pill(r[5] === 'flag' ? 'Flagged' : 'Repeat', 'warn') : pill('Open', 'good')}</td>
        <td class="nowrap"><button class="btn sm" onclick="A.note('Listing paused, the seller is told why (demo)')">${I('pause')}</button> <button class="btn sm danger" onclick="A.note('Listing removed, logged (demo)')">${I('x')}</button></td></tr>`).join('')}</table></div>`;
  else if (tab === 'deals') body = `<div class="card"><table class="t"><tr><th>Deal</th><th>Crop</th><th class="num">Tonnes</th><th class="num">IQD/kg</th><th class="num">Total IQD</th><th>Buyer</th><th>Date</th><th>State</th></tr>
      ${[[77, 'grape', 6, 1250, 'trader', 'disputed'], [76, 'wheat', 40, 850, 'government silo', 'done'], [75, 'tomato', 8, 700, 'shop', 'done'], [74, 'potato', 20, 500, 'trader', 'done']].map(r =>
        `<tr><td>#${r[0]}</td><td>${cropTag(r[1])}</td><td class="num">${r[2]}</td><td class="num">${fmt(r[3])}</td><td class="num">${fmt(r[2] * 1000 * r[3])}</td><td>${r[4]}</td><td>6 Oct</td><td>${r[5] === 'disputed' ? `${pill('Dispute', 'water')} <button class="btn sm" onclick="A.note('Both sides called; notes saved (demo)')">Settle</button>` : pill('Done', 'good')}</td></tr>`).join('')}</table></div>`;
  else body = `<div class="card">${set('Who may make offers', 'FRONTEND.md 7.2 asks the frontend to decide', '<select><option>Any signed-in phone</option><option>Registered traders only</option></select>')}
      ${set('Accepting part of the quantity', 'today accepting any offer closes the whole listing', '<select><option>Closes the listing</option><option>Keeps the rest open</option></select>')}
      ${set('Open listings per phone', '', '<input type="number" value="5" style="width:70px">')}
      ${set('A listing stays open at most', '', '<input type="number" value="14" style="width:70px"> days')}
      ${set('Flag prices above', 'times the market price', '<input type="number" value="2" style="width:70px">×')}
      ${set('Phones hidden until a deal', 'seller sees the buyer and the buyer sees the seller only after a deal', locked('privacy'))}
      ${set('Alwa market in the app', 'switch off the whole market', sw(S.flags.features.alwa, "A.feature('alwa')"))}</div>`;
  return head('Market', 'Alwa market', 'Set the official daily prices per market, stop bad listings, settle disputes and set the market rules.') +
    `<div class="grid g4" style="margin-bottom:14px">${kpi('Open listings', '146', '2,980 t on sale', '', 'package')}${kpi('Offers today', '58', '9 accepted', '', 'hand-coins')}${kpi('Deals this week', '37', '1.2 bn IQD', 'good', 'handshake')}${kpi('Flags', '3', '1 dispute', 'warn', 'flag')}</div>
    <div class="tabs">${[['prices', 'Prices'], ['listings', 'Listings'], ['deals', 'Deals'], ['rules', 'Market rules']].map(([k, t]) => `<button class="${tab === k ? 'on' : ''}" onclick="S.tab.alwa='${k}';rerender()">${t}</button>`).join('')}</div>` + body;
};

// rules
const RULES = [
  ['Weather planner (farm_doctor/weather_planner.py)', [['frost', 'Frost night (watch)', 'night low ≤', '0', '°C'], ['hard_frost', 'Hard frost (alarm)', 'night low ≤', '-2', '°C'], ['heat', 'Heat day', 'day high ≥', '31', '°C'],
    ['rain', 'Big rain day', 'rain in a day ≥', '12', 'mm'], ['sowing', 'Sowing rain', 'rain in 3 days ≥', '20', 'mm'], ['rust', 'Rust weather', 'hours at 6 to 16 °C and air ≥ 90% wet: high from', '24', 'h'],
    ['spray', 'Spray window', 'dry hours at 15 to 24 °C, wind under 15 km/h', '6', 'h'], ['sunn', 'Sunn pest nymphs', 'degree-days over 13.3 °C', '84', 'DD'], ['dust', 'Dust', 'PM10 ≥', '150', 'µg/m³']]],
  ['Dryness bands (zones)', [['b_normal', 'Normal band starts at', 'dryness index', '45', ''], ['b_dry', 'Dry band starts at', 'dryness index', '60', ''], ['b_vdry', 'Very dry band starts at', 'dryness index', '80', '']]],
  ['Field Eye', [['weak', 'Weak square', 'greenness below', '85', '% of normal'], ['alarm_sq', 'Alarm square', 'greenness below', '70', '% of normal'], ['cloud', 'Skip picture when cloud above', '', '30', '%']]],
];
PAGES.rules = () => head('Control the app', 'Rules', 'Every number the planner, the dryness map, Field Eye and the alerts use. A change needs two officers, keeps the old value, and reaches the app at the next plan (every 6 hours).') +
  `<div class="note warn" style="margin-bottom:14px">${I('alert-triangle')}<span>Today the demo build of the app also has these rules inside it (planFromWeather). Changes here reach real phones only after the app reads the plan from the server.</span></div>` +
  RULES.map(([g, items]) => `<div class="card" style="margin-bottom:14px"><div class="eyebrow" style="margin-bottom:6px">${g}</div><table class="t"><tr><th>Rule</th><th>Meaning</th><th class="num">Value</th><th>Version</th><th></th></tr>
    ${items.map(([k, n, m, v, u]) => `<tr><td><b>${n}</b></td><td class="muted">${m}</td><td class="num"><b>${v}</b> ${u}</td><td>${k === 'heat' ? pill('v1 · change waiting', 'warn') : pill('v1')}</td><td class="right"><button class="btn sm" onclick="A.editRule('${k}','${esc(n)}','${v}','${u}')">${I('pencil')}Change</button></td></tr>`).join('')}</table></div>`).join('') +
  `<div class="card"><div class="eyebrow" style="margin-bottom:6px">Pushes and limits</div><table class="t"><tr><td><b>Pushes per farm per day</b></td><td class="num">1</td><td><a href="#notify">Notifications</a></td></tr><tr><td><b>Farms per phone</b></td><td class="num">20</td><td><a href="#app">App control</a></td></tr><tr><td><b>Sign-in code tries</b></td><td class="num">5</td><td><a href="#security">Security</a></td></tr></table></div>`;

// app control
PAGES.app = () => {
  const F = S.flags.features;
  const feats = [['add_farm', 'Add a farm', 'walk corners and paint crops'], ['walk_mode', 'Walk mode', 'record the edge while walking'], ['paint', 'Paint crops on the 10 m grid', ''], ['satellite', 'Satellite map', 'Esri World Imagery tiles'],
    ['doctor', 'Ask the Doctor', 'needs GEMINI_API_KEY'], ['reports', 'Reports (Neighbour Watch)', ''], ['alwa', 'Alwa market', ''], ['plan', 'This week plan', 'weather planner'], ['push', 'Push alerts', '']];
  return head('Control the app', 'App control', 'Versions in use, forced updates, switches for every feature, the maintenance message, limits, the crop list and the map sources.') +
  `<div class="note warn" style="margin-bottom:14px">${I('info')}<span>Needs two app changes first: the app sends <code>X-App-Version</code>, and reads these settings from the server at start. Until then they only exist in each build.</span></div>
  <div class="grid g2">
  <div class="card"><div class="eyebrow" style="margin-bottom:8px">Versions in use</div><table class="t"><tr><th>Build</th><th>Phones</th><th>Released</th><th>Test mode</th><th>State</th></tr>
    <tr><td><b>1.0.3</b></td><td><div class="bar" style="width:120px"><i style="width:80%;background:var(--good)"></i></div>80%</td><td>8 Oct</td><td>${pill('off', 'good')}</td><td>${pill('Current', 'good')}</td></tr>
    <tr><td><b>1.0.2</b></td><td><div class="bar" style="width:120px"><i style="width:15%;background:var(--warn)"></i></div>15%</td><td>6 Oct</td><td>${pill('off', 'good')}</td><td>${pill('Supported', '')}</td></tr>
    <tr><td><b>1.0.0</b></td><td><div class="bar" style="width:120px"><i style="width:5%;background:var(--danger)"></i></div>5%</td><td>4 Oct</td><td>${pill('ON', 'danger')}</td><td>${pill('Must update', 'danger')}</td></tr></table>
    ${set('Oldest version allowed', 'older phones see “Please update” and cannot send', '<select><option>1.0.2</option><option>1.0.0</option><option>1.0.3</option></select>')}
    ${set('Update message', 'shown on the blocked phones', '<button class="btn sm" onclick="A.note(\'Message saved in Sorani and English (demo)\')">Edit</button>')}
  </div>
  <div class="card"><div class="eyebrow" style="margin-bottom:4px">Feature switches</div><div class="muted small" style="margin-bottom:6px">Off = the button disappears in the app with a short note. Logged.</div>
    ${feats.map(([k, t, s]) => set(t, s || '&nbsp;', sw(F[k], `A.feature('${k}')`))).join('')}</div>
  <div class="card"><div class="eyebrow">Maintenance</div>
    ${set('Maintenance mode', 'the app shows the message and works offline only', sw(S.flags.maintenance, 'S.flags.maintenance=this.checked;A.note(this.checked?\'Maintenance on: farmers see the message (demo)\':\'Maintenance off\')'))}
    <label class="f" style="margin-top:8px">Message (Sorani)<textarea class="ku" rows="2">سێرڤەرەکە بۆ ماوەیەکی کورت ڕاگیراوە. زانیارییەکانت پارێزراون.</textarea></label>
    <label class="f" style="margin-top:8px">Message (English)<textarea rows="2">The server is paused for a short time. Your farms are safe on your phone.</textarea></label>
    <div class="grid g2" style="margin-top:8px"><label class="f">From<input type="time" value="02:00"></label><label class="f">Until<input type="time" value="03:00"></label></div></div>
  <div class="card"><div class="eyebrow">Limits</div>
    ${set('Farms per phone', 'backend refuses more (too_many_farms)', '<input type="number" value="20" style="width:80px">')}
    ${set('Largest farm in the app', 'the app refuses bigger outlines before sending', '<input type="number" value="1000" style="width:80px"> du')}
    ${set('Squares per farm', 'farm_too_large', '<input type="number" value="50000" style="width:80px">')}
    ${set('Corners per farm', 'tapped or walked', '<input type="number" value="3" style="width:60px"> to <input type="number" value="50" style="width:60px">')}
    ${set('GPS accuracy needed for a corner', 'worse points ask the farmer to wait', '<input type="number" value="15" style="width:60px"> m')}
    ${set('Corners placed by hand on the map', 'test mode only', sw(false))}</div>
  <div class="card"><div class="eyebrow" style="margin-bottom:6px">Crop list</div><table class="t"><tr><th>Code</th><th></th><th>Sorani</th><th>Colour</th><th>In the picker</th></tr>
    ${Object.entries(CROPS).map(([c, [col, ku, em]]) => `<tr><td class="mono">${c}</td><td>${em}</td><td class="ku" style="text-align:left">${ku}</td><td><span class="dotc" style="background:${col}"></span><span class="mono small">${col}</span></td><td>${c === 'empty' ? locked('needed') : sw(true)}</td></tr>`).join('')}</table>
    <div class="row" style="margin-top:10px"><button class="btn sm" disabled>${I('plus')}Add a crop</button><span class="muted small">A new crop code needs an app and backend update (BACKEND.md section 1).</span></div></div>
  <div class="card"><div class="eyebrow">Map sources</div>
    ${set('Satellite', 'used for walking corners', '<select><option>Esri World Imagery</option><option>Mapbox Satellite (key needed)</option></select>')}
    ${set('Streets', '', '<select><option>OpenStreetMap</option><option>Esri World Street</option></select>')}
    ${set('Terrain', '', '<select><option>Esri World Topo</option><option>OpenTopoMap</option></select>')}
    ${set('Kurdish place names on the satellite', 'from OpenStreetMap (Overpass)', sw(true))}
    ${set('Offline map space on each phone', 'tile cache', '<input type="number" value="150" style="width:70px"> MB')}
    <div class="note info" style="margin-top:8px">${I('info')}<span>Free tile servers have usage limits. If one blocks the app, switch here without a new app release.</span></div></div>
  </div>`;
};

// notifications
PAGES.notify = () => head('Control the app', 'Notifications and SMS', 'When and how the app reaches farmers: push rules, quiet hours, the weekly plan, message templates and the SMS provider for sign-in codes.') +
  `<div class="grid g2"><div class="card"><div class="eyebrow">Push rules</div>
    ${set('Pushes per farm per day', 'decided 2026-10-08', '<input type="number" value="1" style="width:60px">')}
    ${set('Levels that are pushed', 'watch is shown in the app only', '<select><option>Alarm only</option><option>Alarm and watch</option></select>')}
    ${set('Quiet hours', 'pushes wait until the morning', '<input type="time" value="22:00"> to <input type="time" value="06:00">')}
    ${set('Weekly plan', 'one message', '<select><option>Sunday</option><option>Saturday</option></select> <input type="time" value="06:00">')}
    ${set('Every push says what to do and how sure', '', locked('BACKEND.md 2.7'))}
    <div class="grid g3" style="margin-top:10px">${kpi('Pushes 7 days', '1,204', '', '', 'send')}${kpi('Delivered', '96%', '', 'good', 'check')}${kpi('Opened', '64%', '', '', 'mail-open')}</div></div>
  <div class="card"><div class="eyebrow">Sign-in codes by SMS</div>
    <div class="note danger" style="margin:8px 0">${I('alert-triangle')}<span>No SMS provider yet: codes go to the server log, and the demo uses one fixed code for every phone.</span></div>
    ${set('Provider', 'BACKEND.md open point 2', '<select><option>Not set</option><option>Twilio</option><option>Local gateway (Korek / Asiacell / Zain)</option></select>')}
    ${set('Sender name', 'shown on the farmer\'s phone', '<input type="text" value="JUTYAR" style="width:110px">')}
    ${set('Monthly budget', 'stop sending above it and alert admins', '<input type="number" value="300" style="width:80px"> USD')}
    ${set('Send a test code', 'to your own phone', `<button class="btn sm" onclick="A.note('No provider set: nothing sent (demo)')">${I('send')}Send</button>`)}</div>
  <div class="card" style="grid-column:1/-1"><div class="eyebrow" style="margin-bottom:6px">Message templates</div><table class="t"><tr><th>Type</th><th>Sorani</th><th>English</th><th>Checked</th><th></th></tr>
    ${[['frost', 'سەرمای توند: {day} پلەی گەرما دادەبەزێت بۆ {value}.', 'Hard frost: {day} down to {value}.'], ['heat', 'گەرمای زۆر: {day} دەگاتە {value}.', 'Heat: {day} up to {value}.'], ['heavy_rain', 'بارانی زۆر: {day} {value} ملم.', 'Heavy rain: {day}, {value} mm.'], ['dust', 'تەپوتۆز: {day}.', 'Dust: {day}. Delay spraying.'], ['weekly_plan', 'پلانی ئەم هەفتەیە ئامادەیە.', 'This week\'s plan is ready.']].map(r =>
      `<tr><td class="mono">${r[0]}</td><td class="ku">${r[1]}</td><td>${r[2]}</td><td>${pill('not yet', 'warn')}</td><td><button class="btn sm" onclick="A.note('Template editor (demo)')">${I('pencil')}</button></td></tr>`).join('')}</table></div></div>`;

// texts
PAGES.texts = () => head('Control the app', 'Texts and languages', 'Every word the app and the alerts show, in Sorani and English, with who checked it. The app shows Sorani first.') +
  `<div class="grid g-main"><div class="card"><div class="filters"><input class="grow" type="text" placeholder="Search a text"><select><option>All</option><option>Needs a native speaker check</option><option>Checked</option></select><button class="btn" onclick="A.note('Sent 48 texts to the Sorani reviewer (demo)')">${I('send')}Send to reviewer</button></div><table class="t"><tr><th>Key</th><th>English</th><th>Sorani</th><th>State</th></tr>
    ${[['home.waiting_picture', 'Waiting for the first satellite picture', 'چاوەڕێی یەکەم وێنەی مانگی دەستکرد', 'warn'], ['home.weak_line', 'Weak in the {where} corner', 'لاواز لە گۆشەی {where}', 'warn'], ['signin.wrong_code', 'Wrong code, try again', 'کۆدەکە هەڵەیە، دووبارە هەوڵ بدەرەوە', 'good'],
      ['farm.saved_offline', 'Saved on your phone. It will be sent when you have internet.', 'لە مۆبایلەکەت پاشەکەوت کرا. کاتێک ئینتەرنێت هەبوو دەنێردرێت.', 'good'], ['report.seen', 'Seen by an officer', 'ئەفسەرێک بینیویەتی', 'warn'], ['plan.nothing', 'Nothing to act on in the next 10 days', 'لە ١٠ ڕۆژی داهاتوودا هیچ کارێک نییە', 'warn'], ['alwa.price_today', 'Today\'s price', 'نرخی ئەمڕۆ', 'good']].map(r =>
      `<tr><td class="mono small">${r[0]}</td><td>${r[1]}</td><td class="ku">${r[2]}</td><td>${pill(r[3] === 'good' ? 'Checked' : 'Needs check', r[3])}</td></tr>`).join('')}</table></div>
  <div class="card"><div class="eyebrow">Languages</div>
    ${set('Sorani (سۆرانی)', 'default, right to left', sw(true, '', true))}${set('Kurmanji (Kurmancî)', 'Duhok', sw(true))}${set('Arabic (العربية)', '', sw(true))}${set('English', 'officers and tests', sw(true))}
    <div class="note warn" style="margin-top:10px">${I('alert-triangle')}<span>Home labels are English until a native Sorani check (PROGRESS.md).</span></div></div></div>`;

// officers
PAGES.officers = () => head('People', 'Officers and roles', 'Who can do what. Officers sign in with their phone and a second step. A district officer only sees their own area.',
  `<button class="btn primary" onclick="A.invite()">${I('user-plus')}Invite officer</button>`) +
  `<div class="grid g-main-l"><div class="card"><table class="t"><tr><th>Officer</th><th>Role</th><th>Area</th><th>2-step</th><th>Last seen</th><th></th></tr>
    ${OFFICERS.map(o => `<tr><td><div class="row" style="flex-wrap:nowrap"><span class="avatar" style="background:${o.color}">${o.id}</span><b>${o.name}</b></div></td><td>${pill(ROLE[o.role][1], o.role === 'admin' ? 'dark' : o.role === 'district' ? 'brand' : '')}</td><td>${o.areas.join(', ')}</td>
      <td>${o.twofa ? `<span style="color:var(--good)">${I('shield-check')}</span>` : `<span style="color:var(--danger)">${I('shield-alert')}</span>`}</td><td>${o.last}</td><td class="nowrap"><button class="btn sm" onclick="A.note('Role and area editor (demo)')">${I('pencil')}</button> <button class="btn sm danger" onclick="A.note('${o.name} signed out everywhere (demo)')">${I('log-out')}</button></td></tr>`).join('')}</table>
    <div class="note danger" style="margin-top:10px">${I('shield-alert')}<span>Rebaz Salih has no 2-step sign-in: blocked from farms until it is on.</span></div></div>
  <div class="card"><div class="eyebrow" style="margin-bottom:8px">What each role can do</div><table class="t matrix"><tr><th></th><th>Viewer</th><th>District</th><th>Admin</th></tr>
    ${[['Maps, totals, region data', 1, 1, 1], ['Farms at 1 km, inbox, alerts drafts', 0, 1, 1], ['Show a phone (with a reason)', 0, 1, 1], ['Approve alerts', 0, 1, 1], ['Rules, prices, water plan approval', 0, 0, 1], ['App control, notifications, SMS', 0, 0, 1], ['Officers, database, security', 0, 0, 1], ['Edit a farmer\'s outline or crops', 0, 0, 0]].map(r =>
      `<tr><td>${r[0]}</td>${r.slice(1).map(x => `<td>${x ? `<span style="color:var(--good)">${I('check')}</span>` : `<span class="muted">${I('minus')}</span>`}</td>`).join('')}</tr>`).join('')}</table>
    <p class="muted small">Nobody edits what a farmer drew: the data stays the farmer's word. Admins still need a second officer for alerts, rules, deletes and restores.</p></div></div>`;

// data jobs
const JOBS = [['Satellite (Sentinel-2)', 'satellite', 'daily 05:00', 'today 05:12', '1,845 of 2,031 farms read · 186 cloudy', 'oooooooooooooo', 'OK'], ['Weather planner', 'cloud-sun', 'every 6 h', 'today 06:00', '2,031 plans · 3 alarms', 'oooooooooooooo', 'OK'],
  ['Fires (NASA FIRMS)', 'flame', 'every 3 h', 'today 09:00', '4 new fires, 1 near farms', 'oooooooooooooo', 'OK'], ['Dams', 'waves', 'daily 07:00', 'yesterday 07:02', 'no reading today: the source page did not answer', 'oooooooooooopx', 'Late'],
  ['Dryness by district', 'sprout', 'monthly, day 3', '3 Oct 04:40', '33 districts, 72 sub-districts', '-----o-------o', 'OK'], ['Season outlook', 'cloud-rain-wind', 'monthly, day 5', '5 Oct 05:15', 'track record 64%', '-------o------', 'OK'],
  ['Alwa prices', 'store', 'daily 08:00', 'today 08:03', '4 markets · 36 prices', 'ooooooopoooooo', 'OK'], ['Region runner (now.json)', 'map', 'every 12 h', 'today 09:39', '16 zones in 61 s', '-o-o-o-o-o-o-o', 'OK'], ['Weekly brief', 'file-text', 'Sunday 05:00', 'Sun 4 Oct 05:20', 'Ministry brief + village messages', '------o------o', 'OK']];
PAGES.jobs = () => head('System', 'Data jobs', 'Every number in the app and on the dashboard comes from these jobs. They push data in with a service key; the server only stores and serves.') +
  `<div class="card" style="margin-bottom:14px"><table class="t"><tr><th>Job</th><th>Runs</th><th>Last run</th><th>Result</th><th>Last 14 days</th><th>State</th><th></th></tr>
    ${JOBS.map(j => `<tr><td><div class="row" style="flex-wrap:nowrap"><span class="ico ${j[6] === 'OK' ? 'good' : 'danger'}">${I(j[1])}</span><b>${j[0]}</b></div></td><td>${j[2]}</td><td>${j[3]}</td><td class="small ${j[6] === 'OK' ? 'muted' : ''}" style="${j[6] !== 'OK' ? 'color:var(--danger)' : ''}">${j[4]}</td><td>${strip(j[5])}</td><td>${pill(j[6], j[6] === 'OK' ? 'good' : 'danger')}</td>
      <td class="nowrap"><button class="btn sm" onclick="A.run('${j[0]}')">${I('play')}Run</button> <button class="btn sm" onclick="A.jobLog('${j[0]}')">${I('scroll-text')}Log</button></td></tr>`).join('')}</table></div>
  <div class="grid g2"><div class="card"><div class="card-head"><span class="eyebrow">Service keys</span><button class="btn sm" onclick="A.note('New key shown once, copy it now (demo)')">${I('key-round')}New key</button></div><table class="t"><tr><th>Key</th><th>Used by</th><th>Last used</th><th></th></tr>
    <tr><td class="mono">jobs-server-1</td><td>the jobs server</td><td>2 min ago</td><td><button class="btn sm danger" onclick="A.note('Revoked (demo)')">Revoke</button></td></tr>
    <tr><td class="mono">laptop-arya</td><td>a teammate's laptop</td><td style="color:var(--warn)">3 days ago</td><td><button class="btn sm danger" onclick="A.note('Revoked (demo)')">Revoke</button></td></tr></table></div>
  <div class="card"><div class="eyebrow">Farms the jobs still need to read</div><dl class="facts" style="margin-top:8px"><dt>Without a satellite reading</dt><dd>186</dd><dt>Without insights (water, soil, rain)</dt><dd>2,031 · no job yet</dd><dt>Groundwater at farm scale</dt><dd class="muted">no known source</dd><dt>Cells with inside_pct</dt><dd style="color:var(--warn)">not built (BACKEND.md 0.2)</dd></dl></div></div>`;

// database
const TABLES = [['farmers', 1284, '0.4 MB', 'phone', 'today 10:31'], ['sign_in_challenges', 37, '8 kB', 'phone', 'today 10:40'], ['farms', 2031, '3.1 MB', 'location', 'today 10:29'], ['farm_cells', Math.round(FARMS.reduce((a, f) => a + f.area, 0) * 25), '96 MB', '', 'today 10:29'],
  ['farm_insights', 0, '0 kB', '', '-'], ['zones', 33, '24 kB', '', '3 Oct'], ['sub_zones', 72, '40 kB', '', '3 Oct'], ['zone_readings', 1188, '0.3 MB', '', '3 Oct'], ['dams', 2, '8 kB', '', 'yesterday'], ['dam_readings', 731, '0.1 MB', '', 'yesterday'],
  ['outlook_runs', 12, '16 kB', '', '5 Oct'], ['season_outlooks', 396, '0.1 MB', '', '5 Oct'], ['water_plan_entries', 33, '16 kB', '', 'Tue'], ['fires', 2214, '0.6 MB', '', 'today 09:00'],
  ['alwa_markets', 4, '8 kB', '', '-'], ['alwa_prices', 4380, '0.5 MB', '', 'today 08:03'], ['alwa_listings', 611, '0.2 MB', 'phone', 'today 10:12'], ['alwa_offers', 1460, '0.3 MB', 'phone', 'today 10:20'],
  ['officers (new)', 6, '8 kB', 'phone', 'Mon'], ['audit_log (new, insert only)', 18332, '9 MB', '', 'just now'], ['alerts (new)', 41, '40 kB', '', 'today'], ['rule_values (new)', 15, '8 kB', '', 'Wed']];
PAGES.db = () => head('System', 'Database', 'See every table, read rows with phones masked, ask read-only questions, and manage backups. Writes only go through the screens above, so every change is checked and logged.') +
  `<div class="grid g-main-l"><div class="card"><table class="t"><tr><th>Table</th><th class="num">Rows</th><th class="num">Size</th><th>Personal data</th><th>Last write</th><th></th></tr>
    ${TABLES.map(t => `<tr class="click" onclick="A.browse('${t[0]}')"><td class="mono">${t[0]}</td><td class="num">${fmt(t[1])}</td><td class="num">${t[2]}</td><td>${t[3] ? pill(t[3], 'warn', 'lock') : ''}</td><td>${t[4]}</td><td>${I('chevron-right')}</td></tr>`).join('')}</table></div>
  <div class="stack"><div class="card"><div class="eyebrow">Read-only question (SELECT only)</div><textarea id="sql" class="mono" rows="5" style="margin-top:8px">SELECT z.name_en, count(f.id) AS farms
FROM farms f JOIN zones z ON z.id = f.zone_id
GROUP BY 1 ORDER BY 2 DESC LIMIT 5;</textarea>
    <div class="row" style="margin-top:8px"><button class="btn primary sm" onclick="A.sql()">${I('play')}Run</button><span class="muted small">Runs on a read-only copy, phones masked, 10 s limit, logged.</span></div><div id="sqlOut" style="margin-top:10px"></div></div>
    <div class="card"><div class="eyebrow">Backups</div><dl class="facts" style="margin-top:8px"><dt>Last nightly backup</dt><dd>today 03:00 · 231 MB</dd><dt>Kept</dt><dd>30 nightly, 12 monthly</dd><dt>Stored</dt><dd>encrypted, another server</dd><dt>Last restore test</dt><dd style="color:var(--warn)">never</dd></dl>
      <div class="row" style="margin-top:10px"><button class="btn sm" onclick="A.note('Backup started (demo)')">${I('database-backup')}Back up now</button><button class="btn sm danger" onclick="A.restore()">${I('history')}Restore…</button></div></div>
    <div class="card"><div class="eyebrow">Emergency edit</div><p class="sub small">For the rare case no screen covers. Opens a direct write for 30 minutes after two admins agree and type a reason. Every statement is logged with before and after.</p><button class="btn danger" onclick="A.breakGlass()">${I('siren')}Ask for emergency edit</button></div>
    <div class="card"><div class="eyebrow">Migrations</div><div class="small mono" style="margin-top:6px">m20261009_010000_add_alwa_listing_idempotency_key ✓<br>m20261008_226000_create_alwa ✓<br>m20261008_225000_create_farm_insights ✓<br><span class="muted">… 9 more, all applied</span></div></div></div></div>`;

// security
PAGES.security = () => head('System', 'Security and privacy', 'How farmers and officers sign in, what officers may see, how long data is kept, and farmers\' requests for their data.') +
  (S.flags.fixedCode ? `<div class="note danger" style="margin-bottom:14px">${I('alert-triangle')}<span><b>One sign-in code works for every phone.</b> AUTH__FIXED_SIGN_IN_CODE is set: anyone can open any farm. Fine for the demo, never with real farmers. Also turn off test mode in the app (kTestMode).</span></div>` : '') +
  `<div class="grid g2"><div class="card"><div class="eyebrow">Farmer sign-in</div>
    ${set('One fixed code for every phone', 'demo only', sw(S.flags.fixedCode, 'A.fixedCode(this)'))}
    ${set('Code length', '', '<input type="number" value="6" style="width:60px"> digits')}
    ${set('Code lives', '', '<input type="number" value="10" style="width:60px"> min')}
    ${set('Wrong tries per code', '', '<input type="number" value="5" style="width:60px">')}
    ${set('Ask again after', '', '<input type="number" value="60" style="width:60px"> s')}
    ${set('Lock a phone after many wrong codes', 'known gap: a new code resets the counter (critic pass)', '<input type="number" value="15" style="width:60px"> a day ' + sw(false))}
    ${set('Stay signed in', 'farmers work offline for weeks', '<input type="number" value="180" style="width:60px"> days')}</div>
  <div class="card"><div class="eyebrow">Officer sign-in</div>
    ${set('2-step sign-in required', 'phone code + authenticator app', locked('always'))}
    ${set('Session length', '', '<input type="number" value="8" style="width:60px"> h')}
    ${set('Only from the Ministry network', 'IP allow list', sw(false))}
    ${set('Sign every officer out now', '', `<button class="btn sm danger" onclick="A.note('All officer sessions ended (demo)')">${I('log-out')}Sign out all</button>`)}</div>
  <div class="card"><div class="eyebrow">Protected mode</div>
    ${set('Mask phone numbers', '+964 750 ••• 4567', locked('user decision'))}
    ${set('Farm shown at', 'until the farmer asks for help', '<select><option>1 km</option><option>500 m</option><option>5 km</option></select>')}
    ${set('Open the farm when the farmer sends a report or a Doctor case', 'only to officers of that area', sw(true))}
    ${set('Close again after', 'if nobody closes it', '<input type="number" value="14" style="width:60px"> days')}
    ${set('Showing a phone needs a reason', '', locked('always logged'))}
    ${set('Phones shown per officer per day', 'more needs an admin', '<input type="number" value="20" style="width:60px">')}</div>
  <div class="card"><div class="eyebrow">Keeping data</div>
    ${set('History (audit log)', 'cannot be deleted', '<input type="number" value="5" style="width:60px"> years')}
    ${set('Report photos', '', '<input type="number" value="1" style="width:60px"> year')}
    ${set('Doctor cases', '', '<input type="number" value="2" style="width:60px"> years')}
    ${set('Deleted accounts wiped from backups after', '', '<input type="number" value="30" style="width:60px"> days')}
    ${set('Farm data for research', 'only without phones, rounded to the district', sw(false))}</div>
  <div class="card" style="grid-column:1/-1"><div class="eyebrow" style="margin-bottom:6px">Farmers' requests</div><table class="t"><tr><th>Request</th><th>Farmer</th><th>Asked</th><th>How</th><th>State</th><th></th></tr>
    <tr><td>Delete my account and farms</td><td class="mono">+964 770 ••• 1182</td><td>7 Oct</td><td>phone call</td><td>${pill('Waiting for an admin', 'warn')}</td><td><a class="btn sm" href="#approvals">Review</a></td></tr>
    <tr><td>Copy of my data</td><td class="mono">+964 750 ••• 3187</td><td>5 Oct</td><td>in the app</td><td>${pill('Sent', 'good')}</td><td></td></tr>
    <tr><td>Delete my account and farms</td><td class="mono">+964 773 ••• 0412</td><td>2 Oct</td><td>in the app (Settings)</td><td>${pill('Done automatically', 'good')}</td><td></td></tr></table></div></div>`;

// history
PAGES.audit = () => {
  const k = S.tab.audit || 'all';
  const rows = S.audit.filter(a => k === 'all' || a[6] === k);
  return head('System', 'History', 'Every look at a farmer and every change, by officers and by jobs. It cannot be changed or deleted, also not by admins.', `<button class="btn" onclick="csv('history')">${I('download')}Export CSV</button>`) +
    `<div class="chips" style="margin-bottom:12px">${[['all', 'All'], ['farmers', 'Looked at farmers'], ['alerts', 'Alerts'], ['rules', 'Rules'], ['prices', 'Prices'], ['inbox', 'Inbox'], ['deletes', 'Deletes'], ['officers', 'Officers'], ['jobs', 'Jobs'], ['exports', 'Exports']].map(([x, t]) => `<button class="chip ${k === x ? 'on' : ''}" onclick="S.tab.audit='${x}';rerender()">${t}</button>`).join('')}</div>
    <div class="card">${auditRows(rows) || '<div class="empty">Nothing yet.</div>'}<div class="row muted small" style="margin-top:10px">${I('lock')}Each entry is chained to the one before it (last hash 9f3c…a71e), so a deleted or changed line would show.</div></div>`;
};

// system settings
PAGES.system = () => head('System', 'System settings', 'The organisation, units, the public dashboard, the server and its keys.') +
  `<div class="grid g2"><div class="card"><div class="eyebrow">Organisation</div>
    ${set('Name (English)', '', '<input type="text" value="Ministry of Agriculture and Water Resources, KRG" style="width:300px">')}
    ${set('Name (Sorani)', '', '<input type="text" class="ku" value="وەزارەتی کشتوکاڵ و سەرچاوەکانی ئاو" style="width:300px">')}
    ${set('Help line shown in the app', 'farmers call it from Settings', '<input type="text" value="+964 750 000 0000" style="width:160px">')}</div>
  <div class="card"><div class="eyebrow">Region and units</div>
    ${set('Time zone', '', '<select><option>Asia/Baghdad (UTC+3)</option></select>')}
    ${set('Area unit', '1 dunam = 2,500 m²', '<select><option>dunam</option><option>m²</option><option>hectare</option></select>')}
    ${set('Cell grid', 'Sentinel-2 pixels, UTM 38N', locked('10 m'))}
    ${set('Default language', '', '<select><option>Sorani</option><option>Kurmanji</option><option>Arabic</option><option>English</option></select>')}</div>
  <div class="card"><div class="eyebrow">Public dashboard</div>
    ${set('Open without login', 'the Ministry overview, water, Doctor, fires, compare years', sw(true))}
    ${set('Show the season outlook', 'weak evidence: shown with confidence', sw(true))}
    ${set('Show Alwa prices publicly', '', sw(true))}
    ${set('Sample data banner', 'until real jobs fill every number', sw(true))}</div>
  <div class="card"><div class="eyebrow">Server</div><dl class="facts" style="margin-top:8px"><dt>API</dt><dd class="mono">http://&lt;server&gt;:3000/v1</dd><dt>Health</dt><dd style="color:var(--good)">up · database answers</dd><dt>Backend version</dt><dd class="mono">30970b6</dd><dt>Requests 24 h</dt><dd>48,210 · 0.04% errors</dd><dt>Slowest 5%</dt><dd>182 ms</dd><dt>HTTPS</dt><dd style="color:var(--warn)">not yet (plain http for the demo)</dd></dl></div>
  <div class="card" style="grid-column:1/-1"><div class="eyebrow">Keys and outside services</div><table class="t" style="margin-top:6px"><tr><th>Key</th><th>For</th><th>State</th></tr>
    <tr><td class="mono">GEMINI_API_KEY</td><td>The Doctor</td><td>${pill('Missing', 'danger')}</td></tr><tr><td class="mono">AUTH__JWT_SECRET</td><td>sign-in tokens</td><td>${pill('Set', 'good')}</td></tr><tr><td class="mono">INGEST__SERVICE_KEY</td><td>data jobs</td><td>${pill('Set', 'good')}</td></tr>
    <tr><td class="mono">SMS provider</td><td>sign-in codes</td><td>${pill('Not set', 'warn')}</td></tr><tr><td class="mono">Push (Firebase)</td><td>alerts on phones</td><td>${pill('Not set', 'warn')}</td></tr></table>
    <p class="muted small">Key values are never shown here, only whether they are set. They live in the server's .env, never in git.</p></div></div>`;

// ---------- actions ----------
const A = {
  ff(k, v) { S.farmFilter[k] = v; if (k !== 'page') S.farmFilter.page = 0; rerender(); if (k === 'q') { const i = document.querySelector('.filters input'); i.focus(); i.setSelectionRange(v.length, v.length); } },
  farm: id => A_farm(id),
  note: m => toast(m),
  reveal(id) {
    if (!need(1, 'show phone numbers')) return;
    modal(`<h2>Show the phone of farm #${id}</h2><p class="sub">Write why. The reason, your name and the time go to the history, and admins see how many phones you open.</p>
      <label class="f">Reason<select id="rsn"><option>To arrange a field visit</option><option>The farmer asked us to call</option><option>Alwa dispute</option><option>Other</option></select></label>
      <label class="f" style="margin-top:8px">Note<textarea id="rsnNote" rows="2" placeholder="Required"></textarea></label>
      <div class="foot"><button class="btn" onclick="closeModal()">Cancel</button><button class="btn primary" onclick="A.doReveal('${id}')">${I('eye')}Show phone</button></div>`);
  },
  doReveal(id) {
    const n = $('#rsnNote').value.trim(); if (!n) { toast('A note is required.', 'warn'); return; }
    S.revealed.add(id); log(`showed the phone of farm #${id}`, `reason: ${$('#rsn').value}, “${n}”`, 'phone', 'warn', 'farmers'); closeModal(); A_farm(id); rerenderBehind(); toast(`${I('eye')} Phone shown and logged`);
  },
  block(id) {
    if (!need(1, 'block farmers')) return;
    const f = FARMS.find(x => x.id === id), on = f.access !== 'blocked';
    modal(`<h2>${on ? 'Block' : 'Unblock'} this farmer?</h2><p class="sub">${on ? 'The app signs them out and refuses new farms and listings. Their farms stay; nothing is deleted.' : 'They can sign in again.'}</p><label class="f">Reason<textarea id="blk" rows="2"></textarea></label>
      <div class="foot"><button class="btn" onclick="closeModal()">Cancel</button><button class="btn danger solid" onclick="A.doBlock('${id}')">${on ? 'Block' : 'Unblock'}</button></div>`);
  },
  doBlock(id) { const f = FARMS.find(x => x.id === id), on = f.access !== 'blocked'; f.farmer.farms.forEach(x => x.access = on ? 'blocked' : '1km'); f.farmer.blocked = on; log(`${on ? 'blocked' : 'unblocked'} ${mask(f.farmer.phone)}`, $('#blk').value || 'no reason given', 'ban', 'danger', 'farmers'); closeModal(); A_farm(id); rerenderBehind(); toast(on ? 'Farmer blocked' : 'Farmer unblocked'); },
  del(id) {
    if (!need(1, 'delete accounts')) return;
    const f = FARMS.find(x => x.id === id);
    modal(`<h2>Delete on the farmer's request</h2><p class="sub">Removes the phone, ${f.farmer.farms.length} farm(s), reports and Doctor cases. Only when the farmer asked. An admin must approve, then it cannot be undone (backups are wiped after 30 days).</p>
      <label class="f">How did the farmer ask?<select><option>Phone call to the help line</option><option>Visit to an office</option><option>Written letter</option></select></label>
      <label class="f" style="margin-top:8px">Type the farm id <b>${id}</b> to confirm<input type="text" id="delId"></label>
      <div class="foot"><button class="btn" onclick="closeModal()">Cancel</button><button class="btn danger solid" onclick="A.doDel('${id}')">${I('trash-2')}Ask an admin to delete</button></div>`);
  },
  doDel(id) { if ($('#delId').value.trim() !== id) { toast('The farm id does not match.', 'warn'); return; } const f = FARMS.find(x => x.id === id);
    S.approvals.push({ id: 'ap' + Date.now(), kind: 'delete', need: 2, title: `Delete account ${mask(f.farmer.phone)}`, detail: `${f.farmer.farms.length} farm(s) · asked by the farmer`, by: ME.name, at: 'just now' });
    log(`asked to delete account ${mask(f.farmer.phone)}`, 'waiting for an admin', 'trash-2', 'danger', 'deletes'); closeModal(); closeDrawer(); rerender(); toast('Sent to an admin for approval'); },
  exportFarm(id) { log(`made a data copy of farm #${id}`, 'for the farmer', 'file-down', '', 'exports'); toast(`${I('file-down')} farm_${id}.json prepared for the farmer (demo)`); },
  approve(id) {
    const a = S.approvals.find(x => x.id === id);
    if (a.by === ME.name) { toast('You asked for this. A different officer must approve it.', 'warn'); return; }
    if (rank() < a.need) { toast('This needs an admin.', 'warn'); return; }
    S.approvals = S.approvals.filter(x => x.id !== id);
    if (a.kind === 'alert') { const al = S.alerts.find(x => x.state === 'waiting'); if (al) { al.state = 'sent'; al.ok = ME.name; al.opened = 0; } }
    log(`approved: ${a.title}`, `asked by ${a.by}`, 'stamp', 'good', { alert: 'alerts', rule: 'rules', price: 'prices', delete: 'deletes' }[a.kind] || 'other'); rerender(); toast(`${I('check')} Approved and done`);
  },
  reject(id) { const a = S.approvals.find(x => x.id === id); S.approvals = S.approvals.filter(x => x.id !== id); log(`turned down: ${a.title}`, `asked by ${a.by}`, 'x', 'danger'); rerender(); toast('Turned down'); },
  sendAlert() {
    if (!S.alertSel.size) { toast('Pick at least one district on the map.', 'warn'); return; }
    const fs = FARMS.filter(f => S.alertSel.has(f.dist)).length, area = [...S.alertSel].join(', ');
    S.alerts.unshift({ id: 'a_' + Date.now(), title: (S.alertType || 'frost').replace('_', ' ') + ' alert', type: S.alertType || 'frost', area, farms: fs, state: 'waiting', by: ME.name, ok: null, opened: null, day: 'Sun 11 Oct' });
    S.approvals.push({ id: 'ap' + Date.now(), kind: 'alert', need: 1, title: `Alert: ${(S.alertType || 'frost').replace('_', ' ')}`, detail: `${area} · ${fs} farms`, by: ME.name, at: 'just now' });
    log(`asked to send an alert`, `${area} · ${fs} farms`, 'bell-ring', 'brand', 'alerts'); S.tab.alerts = 'list'; rerender(); toast('Sent for approval: another officer must agree');
  },
  assign(id, who) { const i = S.inbox.find(x => x.id === id); i.assigned = who; i.state = i.state === 'new' ? 'assigned' : i.state; if (who.startsWith('Vets') || who.startsWith('Plant')) i.state = 'forwarded'; log(`assigned ${id} to ${who}`, i.title, 'inbox', 'brand', 'inbox'); rerender(); toast('Assigned'); },
  inbox(id, state, what) { const i = S.inbox.find(x => x.id === id); i.state = state; if (!i.assigned) i.assigned = ME.name; log(`${what} on ${id}`, i.title, state === 'closed' ? 'check-check' : 'inbox', 'brand', 'inbox'); rerender(); toast(state === 'closed' ? 'Closed: the farm is protected again' : 'Done: the farmer sees it in the app'); },
  feature(k) {
    if (!need(2, 'switch app features')) { rerender(); return; }
    S.flags.features[k] = !S.flags.features[k]; log(`turned ${S.flags.features[k] ? 'on' : 'off'} the feature “${k}”`, 'all phones at next start', 'toggle-left', 'warn', 'other'); rerender(); toast(`Feature ${S.flags.features[k] ? 'on' : 'off'} for every phone`);
  },
  editRule(k, name, v, u) {
    modal(`<h2>Change: ${name}</h2><p class="sub">The new value waits for an admin who is not you. The old value stays in the history.</p>
      <div class="grid g2"><label class="f">Now<input type="text" value="${v} ${u}" disabled></label><label class="f">New value<input type="number" id="rv" value="${v}"></label></div>
      <label class="f" style="margin-top:8px">Why<textarea id="rr" rows="2" placeholder="Required"></textarea></label>
      <div class="note info" style="margin-top:8px">${I('bar-chart-3')}<span>Impact is worked out from last season's weather before anyone approves (sample).</span></div>
      <div class="foot"><button class="btn" onclick="closeModal()">Cancel</button><button class="btn primary" onclick="A.doRule('${k}','${name}','${v}','${u}')">Ask for approval</button></div>`);
  },
  doRule(k, name, v, u) { const nv = $('#rv').value, why = $('#rr').value.trim(); if (!why) { toast('Write why.', 'warn'); return; } if (nv === v) { toast('The value did not change.', 'warn'); return; }
    S.approvals.push({ id: 'ap' + Date.now(), kind: 'rule', need: 2, title: `Rule: ${name} ${v} → ${nv} ${u}`, detail: why, by: ME.name, at: 'just now' }); log(`asked to change ${name} ${v} → ${nv} ${u}`, why, 'sliders-horizontal', 'warn', 'rules'); closeModal(); rerender(); toast('Waiting for an admin'); },
  publish() { if (!need(1, 'publish prices')) return; S.approvals.push({ id: 'ap' + Date.now(), kind: 'price', need: 2, title: 'Alwa: publish price changes', detail: 'changed cells on the price board', by: ME.name, at: 'just now' }); log('asked to publish Alwa prices', 'price board', 'store', 'brand', 'prices'); rerender(); toast('Waiting for an admin who did not make the change'); },
  run(n) { log(`ran the job “${n}” by hand`, 'started now', 'play', '', 'jobs'); toast(`${I('play')} ${n} started (demo)`); },
  jobLog(n) { drawer(`<div class="spread"><h2>${n} · log</h2><button class="icon-btn" onclick="closeDrawer()">${I('x')}</button></div><pre class="sql" style="margin-top:12px">${n === 'Dams' ?
    '2026-10-09 07:00:01 start\n07:00:02 GET dams page ... timeout after 30 s\n07:00:33 retry 1 ... timeout\n07:01:05 retry 2 ... HTTP 503\n07:01:06 Sentinel-2 lake area: cloud 64%, skipped\n07:01:06 FAILED: no reading stored. Next run tomorrow 07:00' :
    '2026-10-09 start\nread 2,031 farms\npushed results with key jobs-server-1\nPUT /v1/ingest/... 200\nfinished OK'}</pre>`); },
  modalManual(n) { modal(`<h2>Enter today's ${n} reading</h2><p class="sub">Use only an official number. It is marked as entered by hand with your name.</p><div class="grid g2"><label class="f">Volume (bn m³)<input type="number" step="0.01"></label><label class="f">Source<input type="text" placeholder="e.g. KRG Directorate of Dams, phone"></label></div><div class="foot"><button class="btn" onclick="closeModal()">Cancel</button><button class="btn primary" onclick="closeModal();A.note('Reading saved, marked as entered by hand (demo)')">Save</button></div>`); },
  invite() { modal(`<h2>Invite an officer</h2><label class="f">Phone<input type="text" placeholder="+964 7.."></label><div class="grid g2" style="margin-top:8px"><label class="f">Role<select><option>Viewer</option><option>District officer</option><option>Admin</option></select></label><label class="f">Area<select>${['All Kurdistan', ...GOVS].map(g => `<option>${g}</option>`).join('')}</select></label></div><p class="muted small">They get an SMS link, set up 2-step sign-in, then can open their area.</p><div class="foot"><button class="btn" onclick="closeModal()">Cancel</button><button class="btn primary" onclick="closeModal();A.note('Invite sent (demo)')">Invite</button></div>`); },
  browse(t) {
    const rows = t.startsWith('farmers') ? FARMERS.slice(0, 8).map(p => [p.id, mask(p.phone), p.lang, p.joined.toISOString().slice(0, 10)]) : t.startsWith('farms') ? FARMS.slice(0, 8).map(f => [f.id, f.name, mask(f.farmer.phone), fmt(f.area, 2), f.dist]) : [['…', 'sample rows', 'appear here']];
    drawer(`<div class="spread"><h2 class="mono">${t}</h2><button class="icon-btn" onclick="closeDrawer()">${I('x')}</button></div><p class="muted small">Read only. Phones masked. Opening a table is logged.</p><table class="t">${rows.map(r => `<tr>${r.map(c => `<td class="${/^[؀-ۿ]/.test(c) ? 'ku' : 'mono small'}">${c}</td>`).join('')}</tr>`).join('')}</table>`);
    log(`opened table ${t}`, 'read only', 'database', '', 'other');
  },
  sql() {
    const q = $('#sql').value.trim();
    if (!/^select\b/i.test(q) || /\b(insert|update|delete|drop|alter|truncate|grant)\b/i.test(q)) { $('#sqlOut').innerHTML = `<div class="note danger">${I('x-octagon')}<span>Only SELECT questions run here. Changes go through the screens, or the emergency edit.</span></div>`; refreshIcons(); return; }
    const cnt = {}; FARMS.forEach(f => cnt[f.dist] = (cnt[f.dist] || 0) + 1);
    $('#sqlOut').innerHTML = `<table class="t"><tr><th>name_en</th><th class="num">farms</th></tr>${Object.entries(cnt).sort((a, b) => b[1] - a[1]).slice(0, 5).map(([a, b]) => `<tr><td>${a}</td><td class="num">${b}</td></tr>`).join('')}</table><div class="muted small">5 rows · 38 ms · logged</div>`;
    log('ran a read-only question', q.slice(0, 60), 'database', '', 'other');
  },
  restore() { if (!need(2, 'restore backups')) return; modal(`<h2>Restore a backup</h2><p class="sub">Everything after the backup is lost. Needs a second admin. The server goes into maintenance while it runs.</p><label class="f">Backup<select><option>today 03:00</option><option>yesterday 03:00</option><option>1 Oct (monthly)</option></select></label><label class="f" style="margin-top:8px">Why<textarea rows="2"></textarea></label><div class="foot"><button class="btn" onclick="closeModal()">Cancel</button><button class="btn danger solid" onclick="closeModal();S.approvals.push({id:'ap'+Date.now(),kind:'restore',need:2,title:'Restore backup',detail:'today 03:00',by:ME.name,at:'just now'});rerender();A.note('Waiting for a second admin')">Ask a second admin</button></div>`); },
  breakGlass() { if (!need(2, 'ask for an emergency edit')) return; modal(`<h2>Emergency edit</h2><p class="sub">A direct write to the database for 30 minutes. Two admins must agree. Every statement is logged with the row before and after, and all officers are told.</p><label class="f">What is broken and why no screen can fix it<textarea rows="3"></textarea></label><div class="foot"><button class="btn" onclick="closeModal()">Cancel</button><button class="btn danger solid" onclick="closeModal();A.note('Asked: waiting for a second admin (demo)')">${I('siren')}Ask</button></div>`); },
  fixedCode(el) { if (!need(2, 'change sign-in')) { el.checked = !el.checked; return; } S.flags.fixedCode = el.checked; log(`turned ${el.checked ? 'on' : 'off'} the fixed sign-in code`, el.checked ? 'demo mode' : 'real SMS codes needed now', 'key-round', el.checked ? 'danger' : 'good', 'other'); rerender(); toast(el.checked ? 'Fixed code on: demo only' : 'Fixed code off: farmers need real SMS codes'); },
  who(i) { ME = OFFICERS[i]; $('#whoMenu').hidden = true; setWho(); route(); toast(`Signed in as ${ME.name}, ${ROLE[ME.role][1]}`); },
};
window.A = A; window.S = S; window.rerender = rerender; window.closeModal = closeModal; window.closeDrawer = closeDrawer; window.csv = csv;
function rerenderBehind() { const y = window.scrollY; const k = (location.hash || '#overview').slice(1); if (k === 'farms') { $('#main').innerHTML = PAGES.farms(); refreshIcons(); window.scrollTo(0, y); } renderSide(k); }

// ---------- chrome: officer menu, search, approvals ----------
function setWho() { $('#whoAvatar').textContent = ME.id; $('#whoAvatar').style.background = ME.color; $('#whoName').textContent = ME.name; $('#whoRole').textContent = ROLE[ME.role][1] + ' · ' + ME.areas.join(', '); }
$('#whoBtn').onclick = e => {
  e.stopPropagation(); const m = $('#whoMenu');
  m.innerHTML = `<h5>Switch officer (demo)</h5>${OFFICERS.map((o, i) => `<button class="${o === ME ? 'on' : ''}" onclick="A.who(${i})"><span class="avatar" style="background:${o.color}">${o.id}</span><span><b>${o.name}</b><br><small class="muted">${ROLE[o.role][1]} · ${o.areas.join(', ')}</small></span></button>`).join('')}<hr><button onclick="A.note('Signed out (demo)')">${I('log-out')}Sign out</button>`;
  m.hidden = !m.hidden; refreshIcons();
};
document.addEventListener('click', e => { if (!e.target.closest('#whoMenu')) $('#whoMenu').hidden = true; });
$('#approvalsBtn').onclick = () => location.hash = '#approvals';
$('#drawerBg').onclick = closeDrawer;
$('#modalBg').onclick = e => { if (e.target.id === 'modalBg') closeModal(); };
function openSearch() {
  modal(`<input type="text" id="gs" placeholder="Farm id, last 4 digits, officer, setting…" style="width:100%;height:42px;font-size:15px" autocomplete="off"><div id="gsOut" style="margin-top:10px;max-height:380px;overflow:auto"></div>`);
  const inp = $('#gs'); inp.focus();
  const pages = Object.values(FLAT).map(r => ({ t: r[2], h: '#' + r[0], ic: r[1] }));
  const extra = [['SMS provider', '#notify'], ['Fixed sign-in code', '#security'], ['Minimum app version', '#app'], ['Maintenance mode', '#app'], ['Crop list', '#app'], ['Map sources', '#app'], ['Quiet hours', '#notify'], ['Backups', '#db'], ['Frost rule', '#rules'], ['Heat rule', '#rules'], ['API keys', '#system'], ['Protected mode', '#security'], ['Data retention', '#security'], ['Market rules', '#alwa']];
  inp.oninput = () => {
    const q = inp.value.trim().toLowerCase(); if (!q) { $('#gsOut').innerHTML = ''; return; }
    const hits = [...pages.filter(p => p.t.toLowerCase().includes(q)).map(p => `<a class="list-item" href="${p.h}" onclick="closeModal()">${I(p.ic)}<b>${p.t}</b></a>`),
      ...extra.filter(([t]) => t.toLowerCase().includes(q)).map(([t, h]) => `<a class="list-item" href="${h}" onclick="closeModal()">${I('settings')}<span>${t}</span></a>`),
      ...OFFICERS.filter(o => o.name.toLowerCase().includes(q)).map(o => `<a class="list-item" href="#officers" onclick="closeModal()">${I('user')}<span>${o.name} · ${ROLE[o.role][1]}</span></a>`),
      ...FARMS.filter(f => inArea(f) && (f.id.startsWith(q) || f.farmer.phone.endsWith(q))).slice(0, 8).map(f => `<div class="list-item" onclick="closeModal();location.hash='#farms';setTimeout(()=>A.farm('${f.id}'),50)">${I('map')}<span>Farm #${f.id} · <span class="ku">${f.name}</span> · ${f.sub} · ${mask(f.farmer.phone)}</span></div>`)];
    $('#gsOut').innerHTML = hits.join('') || '<div class="empty">Nothing found.</div>'; refreshIcons();
  };
}
$('#searchBtn').onclick = openSearch;
document.addEventListener('keydown', e => { if (e.key === '/' && !/input|textarea|select/i.test(document.activeElement.tagName)) { e.preventDefault(); openSearch(); } if (e.key === 'Escape') { closeModal(); closeDrawer(); } });
window.addEventListener('hashchange', () => { closeDrawer(); route(); });
setWho(); route();
