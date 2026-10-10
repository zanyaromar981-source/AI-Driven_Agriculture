// The few real calls to the backend (FRONTEND.md). The rest of the site still runs on sample data.
// - Products (GET /v1/products) need no login.
// - Workers (GET and DELETE /v1/dashboard/workers) need a staff token. The site has no real staff
//   sign-in yet, so the token comes from VITE_STAFF_TOKEN (.env.local, development only: a value set at
//   build time ends up readable inside the built site). Without it the Workers page shows sample data.
import { useEffect, useState } from 'react';
import { db } from './db';
import type { ProductGroup, ProductUnit, Worker } from './types';

// The backend address. In dev the Vite server passes /v1 on to it (vite.config.ts); set VITE_API_BASE to point somewhere else.
export const API_BASE: string = import.meta.env.VITE_API_BASE ?? (import.meta.env.DEV ? '' : 'http://95.217.14.92:8790');
export const STAFF_TOKEN: string = String(import.meta.env.VITE_STAFF_TOKEN ?? '').trim();

export const GROUPS: ProductGroup[] = ['crops', 'fish_meat_eggs', 'honey_dairy', 'animals', 'nuts_dried'];
export const UNITS: ProductUnit[] = ['kg', 'tray_30', 'litre', 'head'];

// ---------- products ----------
interface ApiProduct { code: string; group: ProductGroup; unit: ProductUnit; name_en: string; name_ku: string | null }

let products: Promise<number | null> | undefined;
/** Reads the server's product list once and adds what the local list lacks (new products, and the group
 *  and unit of the ones it has). Never overwrites a local edit. Answers how many products the server has, or null. */
export const syncProducts = () => (products ??= fetch(API_BASE + '/v1/products')
  .then(r => (r.ok ? (r.json() as Promise<{ products: ApiProduct[] }>) : null))
  .then(d => {
    if (!d || !Array.isArray(d.products) || !d.products.length) return null;
    for (const p of d.products) {
      if (!p.code || !GROUPS.includes(p.group) || !UNITS.includes(p.unit)) continue;
      const cur = db.crops.get(p.code);
      if (!cur) db.crops.put({ id: p.code, name: { en: p.name_en ?? p.code, ku: p.name_ku ?? '' }, color: '#9AA5A0', category: 'other', season: 'perennial', yieldKgPerDunam: 0, active: true, notes: '', group: p.group, unit: p.unit });
      else if (!cur.group || !cur.unit) db.crops.patch(p.code, { group: cur.group ?? p.group, unit: cur.unit ?? p.unit });
    }
    return d.products.length;
  })
  .catch(() => null));

/** undefined while loading, the server's product count once read, null when the server was not reached. */
export function useProductSync(): number | null | undefined {
  const [n, setN] = useState<number | null | undefined>(undefined);
  useEffect(() => { let on = true; syncProducts().then(x => { if (on) setN(x); }); return () => { on = false; }; }, []);
  return n;
}

// ---------- workers (staff) ----------
interface ApiWorker {
  id: string; name: string; phone: string | null; cost_iqd: number; cost_per: 'day' | 'hour'; note: string | null;
  zone_slug: string | null; available?: boolean; created_at: string; updated_at: string | null;
}
const auth = () => ({ Authorization: 'Bearer ' + STAFF_TOKEN });

/** Every card that matches, read page by page (100 a page, at most 2,000 cards). Throws the HTTP status or the network error. */
export async function fetchWorkers(q: string, available: string): Promise<Worker[]> {
  const out: Worker[] = [];
  for (let page = 1; page <= 20; page++) {
    const p = new URLSearchParams({ page: String(page), rows_per_page: '100' });
    if (q) p.set('q', q);
    if (available) p.set('available', available);
    const r = await fetch(API_BASE + '/v1/dashboard/workers?' + p, { headers: auth() });
    if (!r.ok) throw new Error(String(r.status));
    const d = await r.json() as { workers: ApiWorker[]; count: number };
    for (const w of d.workers ?? []) out.push({
      id: String(w.id), name: w.name, phone: w.phone ?? '', cost: w.cost_iqd, per: w.cost_per === 'hour' ? 'hour' : 'day', note: w.note ?? '',
      zone: w.zone_slug ?? '', available: w.available !== false, updated: w.updated_at ?? w.created_at,
    });
    if (!d.workers?.length || out.length >= (d.count ?? 0)) break;
  }
  return out;
}

export async function deleteWorker(id: string) {
  const r = await fetch(API_BASE + '/v1/dashboard/workers/' + encodeURIComponent(id), { method: 'DELETE', headers: auth() });
  if (!r.ok) throw new Error(String(r.status));
}
