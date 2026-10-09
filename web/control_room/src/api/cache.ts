// The cache of BACKEND.md 2.13 / FRONTEND.md 10.
//
// - Every answer is kept in memory and in IndexedDB with its ETag and the version of each topic it
//   belongs to. A page draws at once from the cache.
// - The server's topic versions (/v1/versions, plus /v1/dashboard/versions when signed in) are asked
//   every 60 s and when the tab comes back. Only answers whose topic version went up are asked again,
//   and then with If-None-Match, so an unchanged answer costs a 304 with no body.
// - After the site writes something it calls invalidate(topics) and those answers refresh at once.
// - Everything is wiped when the API version or CACHE_SCHEMA changes; a staff member's private answers
//   are kept under their own id and wiped on sign-out.
import { useEffect, useReducer, useRef, useCallback } from 'react';
import { raw, ApiError, setApiVersionHandler, getToken } from './client';

export const CACHE_SCHEMA = 2;
export type Topic =
  | 'zones' | 'sub_zones' | 'dams' | 'fires' | 'outlooks' | 'water' | 'alwa_prices' | 'alwa_listings' | 'crops' | 'rules' | 'briefs' | 'app_config'
  | 'farmers' | 'farms' | 'messages' | 'jobs' | 'staff_roles';

interface Entry { key: string; etag: string | null; data: unknown; seen: Record<string, number>; at: number; scope: string }

// ---------- IndexedDB (tiny wrapper, no library) ----------
let dbP: Promise<IDBDatabase | null> | null = null;
function idb(): Promise<IDBDatabase | null> {
  return dbP ??= new Promise(res => {
    try {
      const r = indexedDB.open('jutyar-cache', 1);
      r.onupgradeneeded = () => r.result.createObjectStore('q', { keyPath: 'key' });
      r.onsuccess = () => res(r.result);
      r.onerror = () => res(null);
    } catch { res(null); }
  });
}
async function idbGet(key: string): Promise<Entry | undefined> {
  const db = await idb(); if (!db) return undefined;
  return new Promise(res => { const t = db.transaction('q').objectStore('q').get(key); t.onsuccess = () => res(t.result as Entry | undefined); t.onerror = () => res(undefined); });
}
async function idbPut(e: Entry) { const db = await idb(); if (!db) return; try { db.transaction('q', 'readwrite').objectStore('q').put(e); } catch { /* full: memory still works */ } }
async function idbWipe(pred?: (e: Entry) => boolean) {
  const db = await idb(); if (!db) return;
  const store = db.transaction('q', 'readwrite').objectStore('q');
  if (!pred) { store.clear(); return; }
  const c = store.openCursor();
  c.onsuccess = () => { const cur = c.result; if (!cur) return; if (pred(cur.value as Entry)) cur.delete(); cur.continue(); };
}

// ---------- state ----------
const mem = new Map<string, Entry>();
interface Flight { p: Promise<Entry>; seen: Record<string, number> }
const inflight = new Map<string, Flight>();
// Bumped on sign-out, scope change and every wipe: an answer that started in an earlier generation is
// handed to whoever waits for it but never saved, so private data cannot outlive its session.
let gen = 0;
let versions: Record<string, number> = {};
let versionsKnown = false;
const listeners = new Set<() => void>();
const notify = () => listeners.forEach(l => l());
let scope = 'pub';

(function checkSchema() {
  try {
    if (localStorage.getItem('jutyar.cache.schema') !== String(CACHE_SCHEMA)) {
      localStorage.setItem('jutyar.cache.schema', String(CACHE_SCHEMA));
      localStorage.removeItem('jutyar.cache.warm');
      idbWipe();
    }
  } catch { /* storage blocked */ }
})();

setApiVersionHandler(v => {
  let prev: string | null = null;
  try { prev = localStorage.getItem('jutyar.api.version'); localStorage.setItem('jutyar.api.version', v); } catch { /* ignore */ }
  if (prev && prev !== v) { gen++; mem.clear(); idbWipe(); versions = {}; versionsKnown = false; notify(); refreshVersions(); }
});

/** Who the private answers belong to: 'pub' when signed out, else the staff id. */
export function setScope(staffId: string | null) {
  const next = staffId ? 'staff:' + staffId : 'pub';
  if (next === scope) return;
  scope = next; gen++;
  notify();
}
/** Sign-out: forget everything private. */
export function wipePrivate() {
  gen++;
  for (const [k, e] of mem) if (e.scope !== 'pub') mem.delete(k);
  idbWipe(e => e.scope !== 'pub');
  for (const t of PRIVATE) delete versions[t];
}
const PRIVATE = ['farmers', 'farms', 'messages', 'jobs', 'staff_roles'];

export const isWarm = () => { try { return localStorage.getItem('jutyar.cache.warm') === '1'; } catch { return false; } };
export const markWarm = () => { try { localStorage.setItem('jutyar.cache.warm', '1'); } catch { /* ignore */ } };

// ---------- versions ----------
let vTimer: number | undefined;
let vRunning: Promise<void> | null = null;
/** Ask the server's topic versions. Calls made while one is running share it. */
export function refreshVersions(): Promise<void> {
  return vRunning ??= doRefreshVersions().finally(() => { vRunning = null; });
}
async function doRefreshVersions() {
  try {
    const pub = await raw<{ versions: Record<string, number> }>('GET', '/versions', { auth: false });
    let next = { ...versions, ...(pub.data?.versions ?? {}) };
    if (getToken()) {
      try { const pr = await raw<{ versions: Record<string, number> }>('GET', '/dashboard/versions'); next = { ...next, ...(pr.data?.versions ?? {}) }; } catch { /* signed out meanwhile */ }
    }
    const changed = !versionsKnown || Object.keys(next).some(k => next[k] !== versions[k]);
    versions = next; versionsKnown = true;
    adoptUnknown();
    if (changed) notify();
  } catch { /* offline: keep showing the cache */ }
}
export function startVersionWatch() {
  refreshVersions();
  clearInterval(vTimer);
  vTimer = window.setInterval(refreshVersions, 60_000);
  const onShow = () => { if (document.visibilityState === 'visible') refreshVersions(); };
  document.addEventListener('visibilitychange', onShow);
  window.addEventListener('focus', onShow);
  return () => { clearInterval(vTimer); document.removeEventListener('visibilitychange', onShow); window.removeEventListener('focus', onShow); };
}

// ---------- fetching ----------
// A topic whose version is not known yet (private topics before the first signed-in check) is stored
// as -1 and adopted when the number first arrives: the answer was fetched moments ago, so it counts as
// current instead of being asked again.
const isStale = (e: Entry, topics: Topic[]) => topics.some(t => versions[t] !== undefined && ((e.seen[t] ?? -1) === -1 || versions[t] > e.seen[t]));
const snapshot = (topics: Topic[]) => Object.fromEntries(topics.map(t => [t, versions[t] ?? -1]));
function adoptUnknown() {
  const recent = Date.now() - 60_000; // only answers fetched in this page view; older -1 entries stay stale
  for (const e of mem.values()) {
    if (e.at < recent) continue;
    let changed = false;
    for (const t of Object.keys(e.seen)) if (e.seen[t] === -1 && versions[t] !== undefined) { e.seen[t] = versions[t]; changed = true; }
    if (changed) idbPut(e);
  }
}

async function load(key: string, path: string, topics: Topic[], auth: boolean, prev?: Entry): Promise<Entry> {
  // The versions this answer will be stamped with are taken BEFORE asking, so a change that lands while
  // the request runs still makes the answer stale. A running request is joined only if it is at least as
  // new as what we need now; otherwise a new one is chained after it.
  const seen = snapshot(topics);
  const running = inflight.get(key);
  if (running && topics.every(t => (running.seen[t] ?? -1) >= (seen[t] ?? -1))) return running.p;
  const myGen = gen, sc = auth ? scope : 'pub';
  const p = (async () => {
    if (running) await running.p.catch(() => undefined);
    const base = mem.get(key) ?? prev;
    const r = await raw<unknown>('GET', path, { etag: base?.etag, auth });
    const e: Entry = r.status === 304 && base
      ? { ...base, seen, at: Date.now() }
      : { key, etag: r.etag, data: r.data, seen, at: Date.now(), scope: sc };
    if (myGen === gen) { mem.set(key, e); idbPut(e); }
    return e;
  })();
  const f: Flight = { p, seen };
  inflight.set(key, f);
  try { return await p; } finally { if (inflight.get(key) === f) inflight.delete(key); }
}

/** Preload for the first-visit intro; resolves when the answer is cached. */
export function prefetch(path: string, topics: Topic[], auth = false) {
  const key = (auth ? scope : 'pub') + ' ' + path;
  return load(key, path, topics, auth, mem.get(key)).then(() => undefined, () => undefined);
}

/** The site wrote something: refresh every answer of these topics now. */
export function invalidate(...topics: Topic[]) {
  for (const t of topics) versions[t] = (versions[t] ?? 0) + 0.5; // local bump; the real number arrives with the next check
  notify();
  setTimeout(refreshVersions, 300);
}

export interface Query<T> { data: T | undefined; error: ApiError | null; loading: boolean; refreshing: boolean; reload: () => void }

/**
 * Read one route through the cache.
 * path null = do nothing yet. topics [] = always revalidate (with ETag) when the page opens.
 * auth = staff route (private, kept per staff id).
 */
export function useApi<T>(path: string | null, topics: Topic[], opts: { auth?: boolean; everyMs?: number } = {}): Query<T> {
  const auth = !!opts.auth;
  const key = path == null ? null : (auth ? scope : 'pub') + ' ' + path;
  const [, force] = useReducer((x: number) => x + 1, 0);
  const st = useRef<{ key: string | null; entry?: Entry; error: ApiError | null; refreshing: boolean }>({ key: null, error: null, refreshing: false });
  const topicsKey = topics.join(',');

  const run = useCallback(async (k: string, p: string, always: boolean) => {
    const cur = st.current.entry ?? mem.get(k) ?? await idbGet(k);
    if (cur && st.current.key === k && !st.current.entry) { st.current.entry = cur; mem.set(k, cur); force(); }
    const tps = topicsKey ? topicsKey.split(',') as Topic[] : [];
    if (cur && !always && tps.length && (!versionsKnown || !isStale(cur, tps))) return;
    st.current.refreshing = true; force();
    try {
      const e = await load(k, p, tps, auth, cur);
      if (st.current.key === k) { st.current.entry = e; st.current.error = null; }
    } catch (err) {
      // a route that now answers 404 (public farm totals switched off, a plan deleted) must not keep
      // showing its old cached answer
      if (err instanceof ApiError && err.status === 404) { mem.delete(k); idbWipe(x => x.key === k); if (st.current.key === k) st.current.entry = undefined; }
      if (st.current.key === k && err instanceof ApiError) st.current.error = err;
    } finally {
      if (st.current.key === k) { st.current.refreshing = false; force(); }
    }
  }, [auth, topicsKey]);

  useEffect(() => {
    st.current = { key, entry: key ? mem.get(key) : undefined, error: null, refreshing: false };
    force();
    if (!key || !path) return;
    run(key, path, !topicsKey);
    const l = () => { const e = st.current.entry; const tps = topicsKey ? topicsKey.split(',') as Topic[] : []; if (e && tps.length && isStale(e, tps)) run(key, path, true); };
    listeners.add(l);
    const iv = opts.everyMs ? window.setInterval(() => run(key, path, true), opts.everyMs) : undefined;
    return () => { listeners.delete(l); clearInterval(iv); };
  }, [key, path, run, topicsKey, opts.everyMs]);

  const e = st.current.key === key ? st.current.entry : undefined;
  return {
    data: e?.data as T | undefined,
    error: st.current.key === key ? st.current.error : null,
    loading: !!key && !e && !st.current.error,
    refreshing: st.current.refreshing,
    reload: () => { if (key && path) run(key, path, true); },
  };
}

/** Settings, "clear saved data": forget every saved answer in this browser (public and private). */
export async function wipeAll() {
  gen++;
  mem.clear();
  await idbWipe();
  try { localStorage.removeItem('jutyar.cache.warm'); } catch { /* ignore */ }
  notify();
}
