// Per-browser preferences only (language, sidebar, chosen columns). Never data: all data comes from the
// server through src/api.
export const prefs = {
  get<T>(k: string, d: T): T { try { const s = localStorage.getItem('jutyar.pref.' + k); return s == null ? d : JSON.parse(s) as T; } catch { return d; } },
  set(k: string, v: unknown) { try { localStorage.setItem('jutyar.pref.' + k, JSON.stringify(v)); } catch { /* blocked: keep working */ } },
};
