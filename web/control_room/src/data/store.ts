// The local database. Every page reads and writes through here (see api.ts), so moving to Supabase
// later only replaces this file and api.ts.
//
// Cost: a Collection keeps a Map by id (get, put, remove are O(1)) and one cached array that is rebuilt
// at most once per change (O(n)), only when someone asks for it. Saving to localStorage is debounced.
import { useSyncExternalStore } from 'react';

const PREFIX = 'jutyar.db.v1.';
type Listener = () => void;

function readLS<T>(key: string): T | null {
  try { const s = localStorage.getItem(PREFIX + key); return s ? JSON.parse(s) as T : null; } catch { return null; }
}
function writeLS(key: string, v: unknown) {
  try { localStorage.setItem(PREFIX + key, JSON.stringify(v)); } catch { /* full or blocked: the session keeps working in memory */ }
}

export class Collection<T extends { id: string }> {
  private map = new Map<string, T>();
  private cache: T[] | null = null;
  private listeners = new Set<Listener>();
  private saveTimer: number | undefined;
  private loaded = false;
  version = 0;

  constructor(readonly name: string, private seed: () => T[]) {}

  private load() {
    if (this.loaded) return;
    this.loaded = true;
    const rows = readLS<T[]>(this.name) ?? this.seed();
    for (const r of rows) this.map.set(r.id, r);
  }
  private changed() {
    this.cache = null;
    this.version++;
    this.listeners.forEach(l => l());
    clearTimeout(this.saveTimer);
    this.saveTimer = window.setTimeout(() => writeLS(this.name, this.all()), 400);
  }

  all(): T[] { this.load(); return this.cache ??= [...this.map.values()]; }
  get(id: string | null | undefined): T | undefined { this.load(); return id == null ? undefined : this.map.get(id); }
  has(id: string) { this.load(); return this.map.has(id); }
  get size() { this.load(); return this.map.size; }

  put(item: T) { this.load(); this.map.set(item.id, item); this.changed(); return item; }
  putMany(items: T[]) { this.load(); for (const i of items) this.map.set(i.id, i); this.changed(); }
  patch(id: string, p: Partial<T>) {
    const cur = this.get(id); if (!cur) return undefined;
    const next = { ...cur, ...p, id } as T; this.map.set(id, next); this.changed(); return next;
  }
  remove(id: string) { this.load(); if (this.map.delete(id)) this.changed(); }
  removeWhere(pred: (t: T) => boolean) {
    this.load(); let n = 0;
    for (const [k, v] of this.map) if (pred(v)) { this.map.delete(k); n++; }
    if (n) this.changed();
    return n;
  }
  reset() { this.map.clear(); this.loaded = true; for (const r of this.seed()) this.map.set(r.id, r); this.changed(); }

  subscribe = (l: Listener) => { this.listeners.add(l); return () => { this.listeners.delete(l); }; };
  getVersion = () => { this.load(); return this.version; };
}

/** A single settings record (app config, site settings) stored like a one-row collection. */
export class Doc<T extends { id: string }> {
  private col: Collection<T>;
  constructor(name: string, seed: () => T) { this.col = new Collection<T>(name, () => [seed()]); }
  get value(): T { return this.col.all()[0]; }
  set(p: Partial<T>) { this.col.patch(this.value.id, p); }
  reset() { this.col.reset(); }
  subscribe = (l: Listener) => this.col.subscribe(l);
  getVersion = () => this.col.getVersion();
}

/** Re-render when the collection changes and get its rows. */
export function useRows<T extends { id: string }>(c: Collection<T>): T[] {
  useSyncExternalStore(c.subscribe, c.getVersion);
  return c.all();
}
export function useDoc<T extends { id: string }>(d: Doc<T>): T {
  useSyncExternalStore(d.subscribe, d.getVersion);
  return d.value;
}
/** Version number only, for useMemo keys over several collections. */
export function useVersion(c: { subscribe: (l: Listener) => () => void; getVersion: () => number }): number {
  return useSyncExternalStore(c.subscribe, c.getVersion);
}

/** Per-browser preferences (language, sidebar, last tab). Never data that must be shared. */
export const prefs = {
  get<T>(k: string, d: T): T { try { const s = localStorage.getItem('jutyar.pref.' + k); return s == null ? d : JSON.parse(s) as T; } catch { return d; } },
  set(k: string, v: unknown) { try { localStorage.setItem('jutyar.pref.' + k, JSON.stringify(v)); } catch { /* ignore */ } },
};

export function clearAllData() {
  try { Object.keys(localStorage).filter(k => k.startsWith(PREFIX)).forEach(k => localStorage.removeItem(k)); } catch { /* ignore */ }
}
