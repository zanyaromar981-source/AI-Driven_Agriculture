// Reads that combine collections (indexes and totals) and writes with their checks.
// Pages call these instead of touching db directly, so a Supabase version can keep the same names.
//
// Cost: every index or total is built in one pass over its collection (O(n)) and cached until that
// collection changes (checked by version number), so a page that asks 100 times pays once.
import { db, PLACES } from './db';
import type { Farm, Farmer, Listing, Message, Officer, Id } from './types';

// ---------- memo by collection versions ----------
function memo<T>(deps: () => number[], build: () => T): () => T {
  let last: number[] = [], val: T | undefined;
  return () => {
    const d = deps();
    if (val === undefined || d.length !== last.length || d.some((v, i) => v !== last[i])) { val = build(); last = d; }
    return val;
  };
}

export const newId = (prefix = '') => prefix + Date.now().toString(36) + Math.floor(Math.random() * 1296).toString(36).padStart(2, '0');
export const nowIso = () => new Date().toISOString();

// ---------- places ----------
export const govByName = new Map(PLACES.governorates.map(g => [g.en, g]));
export const distByName = new Map(PLACES.districts.map(d => [d.en, d]));
export const subByKey = new Map(PLACES.subdistricts.map(s => [s.dist + '|' + s.en, s]));
export const distsOfGov = (gov: string) => PLACES.districts.filter(d => !gov || d.gov === gov);
export const subsOfDist = (dist: string) => PLACES.subdistricts.filter(s => s.dist === dist);

// ---------- indexes ----------
/** farmerId -> that farmer's farms */
export const farmsByFarmer = memo(() => [db.farms.getVersion()], () => {
  const m = new Map<Id, Farm[]>();
  for (const f of db.farms.all()) { const a = m.get(f.farmerId); a ? a.push(f) : m.set(f.farmerId, [f]); }
  return m;
});
export const farmsOf = (farmerId: Id) => farmsByFarmer().get(farmerId) ?? [];

export interface AreaTotal { farmers: number; farms: number; area: number; crops: Map<string, number>; female: number; irrigated: number }
const blank = (): AreaTotal => ({ farmers: 0, farms: 0, area: 0, crops: new Map(), female: 0, irrigated: 0 });

/** Totals per governorate, per district and for the whole region, in one pass over farmers and farms. */
export const totals = memo(() => [db.farms.getVersion(), db.farmers.getVersion()], () => {
  const all = blank(), byGov = new Map<string, AreaTotal>(), byDist = new Map<string, AreaTotal>(), bySub = new Map<string, AreaTotal>();
  const get = (m: Map<string, AreaTotal>, k: string) => { let t = m.get(k); if (!t) { t = blank(); m.set(k, t); } return t; };
  for (const p of db.farmers.all()) {
    const ts = [all, get(byGov, p.gov), get(byDist, p.dist)];
    for (const t of ts) { t.farmers++; if (p.gender === 'female') t.female++; }
  }
  for (const f of db.farms.all()) {
    const ts = [all, get(byGov, f.gov), get(byDist, f.dist), get(bySub, f.dist + '|' + f.sub)];
    for (const t of ts) {
      t.farms++; t.area += f.area; if (f.irrigation !== 'rainfed') t.irrigated += f.area;
      for (const c of f.crops) t.crops.set(c.crop, (t.crops.get(c.crop) ?? 0) + c.dunam);
    }
  }
  return { all, byGov, byDist, bySub };
});

/** Average asking and sold price per crop from the Alwa listings (IQD per kg). */
export const marketPrices = memo(() => [db.listings.getVersion()], () => {
  const acc = new Map<string, { ask: number; askN: number; sold: number; soldN: number; kgOpen: number; open: number; min: number; max: number }>();
  for (const l of db.listings.all()) {
    let a = acc.get(l.crop);
    if (!a) { a = { ask: 0, askN: 0, sold: 0, soldN: 0, kgOpen: 0, open: 0, min: Infinity, max: 0 }; acc.set(l.crop, a); }
    if (l.state === 'open') { a.ask += l.price; a.askN++; a.kgOpen += l.kg; a.open++; a.min = Math.min(a.min, l.price); a.max = Math.max(a.max, l.price); }
    if (l.state === 'sold' && l.soldPrice) { a.sold += l.soldPrice; a.soldN++; }
  }
  return new Map([...acc].map(([crop, a]) => [crop, {
    crop, avgAsk: a.askN ? a.ask / a.askN : null, avgSold: a.soldN ? a.sold / a.soldN : null,
    open: a.open, kgOpen: a.kgOpen, sold: a.soldN, min: a.min === Infinity ? null : a.min, max: a.max || null,
  }]));
});

export const newMessages = memo(() => [db.messages.getVersion()], () => db.messages.all().reduce((n, m) => n + (m.state === 'new' ? 1 : 0), 0));

// ---------- writes ----------
const phoneOk = (p: string) => /^\+9647\d{9}$/.test(p.replace(/\s/g, ''));
export const normPhone = (p: string) => {
  let s = p.replace(/[\s-]/g, '');
  if (s.startsWith('07')) s = '+964' + s.slice(1);
  if (s.startsWith('9647')) s = '+' + s;
  return s;
};

export type Problems = Record<string, string>; // field -> i18n key of the problem

export function checkFarmer(f: Farmer): Problems {
  const p: Problems = {};
  if (!f.name.en.trim() && !f.name.ku.trim()) p.name = 'v.name_needed';
  if (!phoneOk(f.phone)) p.phone = 'v.phone_bad';
  else if (db.farmers.all().some(x => x.phone === f.phone && x.id !== f.id)) p.phone = 'v.phone_taken';
  if (!f.gov) p.gov = 'v.needed';
  if (!f.dist) p.dist = 'v.needed';
  if (f.birthYear != null && (f.birthYear < 1900 || f.birthYear > new Date().getFullYear() - 10)) p.birthYear = 'v.year_bad';
  return p;
}
export function saveFarmer(f: Farmer) { return db.farmers.put({ ...f, phone: normPhone(f.phone) }); }
/** Deleting a farmer deletes their farms, listings, messages and Doctor questions too. */
export function deleteFarmer(id: Id) {
  const farmIds = new Set(farmsOf(id).map(f => f.id));
  db.farms.removeWhere(f => farmIds.has(f.id));
  db.listings.removeWhere((l: Listing) => l.farmerId === id);
  db.messages.removeWhere((m: Message) => m.farmerId === id);
  db.questions.removeWhere(q => q.farmerId === id);
  db.farmers.remove(id);
}

export function checkFarm(f: Farm): Problems {
  const p: Problems = {};
  if (!db.farmers.has(f.farmerId)) p.farmerId = 'v.needed';
  if (!f.name.trim()) p.name = 'v.needed';
  if (!(f.area > 0)) p.area = 'v.positive';
  if (!f.dist) p.dist = 'v.needed';
  const sum = f.crops.reduce((a, c) => a + (c.dunam || 0), 0);
  if (f.crops.some(c => !c.crop || !(c.dunam > 0))) p.crops = 'v.crop_rows';
  else if (sum > f.area + 0.05) p.crops = 'v.crops_over_area';
  if (!(f.lat > 34 && f.lat < 38 && f.lon > 42 && f.lon < 47)) p.point = 'v.point_bad';
  return p;
}
export function saveFarm(f: Farm) { return db.farms.put({ ...f, updated: nowIso() }); }
export function deleteFarm(id: Id) {
  db.listings.removeWhere(l => l.farmId === id);
  db.farms.remove(id);
}

export function checkOfficer(o: Officer): Problems {
  const p: Problems = {};
  if (!o.name.trim()) p.name = 'v.needed';
  if (!/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(o.email)) p.email = 'v.email_bad';
  else if (db.officers.all().some(x => x.email.toLowerCase() === o.email.toLowerCase() && x.id !== o.id)) p.email = 'v.email_taken';
  if (o.phone && !phoneOk(normPhone(o.phone))) p.phone = 'v.phone_bad';
  return p;
}
export function saveOfficer(o: Officer) { return db.officers.put({ ...o, email: o.email.trim().toLowerCase(), phone: o.phone ? normPhone(o.phone) : '' }); }

/** Crops in use cannot be deleted, only switched off. */
export const cropInUse = (code: string) => db.farms.all().some(f => f.crops.some(c => c.crop === code));

// ---------- reference numbers for letters ----------
export function letterNumber(prefix: string, farmerId: Id) {
  const d = new Date();
  return `${prefix}-${d.getFullYear()}${String(d.getMonth() + 1).padStart(2, '0')}-${farmerId}`;
}

// ---------- csv ----------
/** Excel-friendly CSV (UTF-8 with BOM so Kurdish shows correctly). */
export function downloadCsv(name: string, header: string[], rows: (string | number | null | undefined)[][]) {
  const cell = (v: string | number | null | undefined) => { const s = v == null ? '' : String(v); return /[",\n]/.test(s) ? `"${s.replace(/"/g, '""')}"` : s; };
  const text = '﻿' + [header, ...rows].map(r => r.map(cell).join(',')).join('\r\n');
  const a = document.createElement('a');
  a.href = URL.createObjectURL(new Blob([text], { type: 'text/csv;charset=utf-8' }));
  a.download = name + '.csv'; a.click();
  setTimeout(() => URL.revokeObjectURL(a.href), 1000);
}
