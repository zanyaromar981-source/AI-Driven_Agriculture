// Staff sign-in against the backend (FRONTEND.md 8). The token lasts 12 hours; it is kept in this
// browser until then. Permissions come from /dashboard/me and decide which pages and buttons show
// (the server checks every call anyway).
import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from 'react';
import { Navigate, useLocation } from 'react-router-dom';
import { api, setToken, getToken, setUnauthorizedHandler, ApiError } from '../api/client';
import { setScope, wipePrivate, refreshVersions } from '../api/cache';
import type { Action, Me, Permission, Resource, SignedIn, Staff } from '../api/types';

const KEY = 'jutyar.auth';
const TTL = 12 * 3600 * 1000;
interface Saved { token: string; staff: Staff; permissions: Permission[]; until: number }

function readSaved(): Saved | null {
  try {
    const s = JSON.parse(localStorage.getItem(KEY) || 'null') as Saved | null;
    return s && s.until > Date.now() ? s : null;
  } catch { return null; }
}

interface Auth {
  me: Staff | null;
  perms: Set<string>;
  can: (resource: Resource, action?: Action) => boolean;
  signIn: (email: string, password: string) => Promise<void>;
  signOut: () => void;
  refreshMe: () => Promise<void>;
}
const Ctx = createContext<Auth | null>(null);

// set before the first render so the first requests already carry the token
const initial = readSaved();
if (initial) { setToken(initial.token); setScope(initial.staff.id); }

export function AuthProvider({ children }: { children: ReactNode }) {
  const [saved, setSaved] = useState<Saved | null>(initial);

  const store = useCallback((s: Saved | null) => {
    setSaved(s);
    try { s ? localStorage.setItem(KEY, JSON.stringify(s)) : localStorage.removeItem(KEY); } catch { /* private mode */ }
  }, []);

  const signOut = useCallback(() => {
    setToken(null); wipePrivate(); setScope(null); store(null);
  }, [store]);

  useEffect(() => { setUnauthorizedHandler(signOut); }, [signOut]);

  const refreshMe = useCallback(async () => {
    const cur = readSaved(); if (!cur) return;
    try {
      const me = await api.get<Me>('/dashboard/me');
      if (getToken() !== cur.token) return; // signed out or in again meanwhile: never bring this session back
      store({ ...cur, staff: me.staff, permissions: me.permissions });
    } catch (e) { if (e instanceof ApiError && e.status === 401) signOut(); }
  }, [store, signOut]);

  // permissions can change while signed in (a role edit): re-read them when the tab comes back
  useEffect(() => {
    if (!saved) return;
    refreshMe();
    const t = setTimeout(signOut, Math.max(0, saved.until - Date.now()));
    const onShow = () => { if (document.visibilityState === 'visible') refreshMe(); };
    document.addEventListener('visibilitychange', onShow);
    return () => { clearTimeout(t); document.removeEventListener('visibilitychange', onShow); };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [saved?.token]);

  const signIn = useCallback(async (email: string, password: string) => {
    const r = await api.post<SignedIn>('/dashboard/auth/login', { email: email.trim(), password });
    setToken(r.token); setScope(r.staff.id);
    store({ token: r.token, staff: r.staff, permissions: r.permissions, until: Date.now() + TTL });
    refreshVersions();
  }, [store]);

  const value = useMemo<Auth>(() => {
    const perms = new Set((saved?.permissions ?? []).map(p => p.resource + ':' + p.action));
    return { me: saved?.staff ?? null, perms, can: (r, a = 'read') => perms.has(r + ':' + a), signIn, signOut, refreshMe };
  }, [saved, signIn, signOut, refreshMe]);

  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export function useAuth(): Auth {
  const c = useContext(Ctx);
  if (!c) throw new Error('useAuth outside AuthProvider');
  return c;
}

export function RequireStaff({ children }: { children: ReactNode }) {
  const { me } = useAuth();
  const loc = useLocation();
  if (!me) return <Navigate to="/login" replace state={{ from: loc.pathname + loc.search }} />;
  return <>{children}</>;
}
