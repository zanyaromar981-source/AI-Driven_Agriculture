// Admin sign-in. For now any active officer signs in with their email and the demo password below.
// When Supabase is connected, signIn() becomes supabase.auth.signInWithPassword and forgot-password
// becomes supabase.auth.resetPasswordForEmail. Nothing else in the site changes.
import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from 'react';
import { Navigate, useLocation } from 'react-router-dom';
import { db } from '../data/db';
import type { Officer } from '../data/types';

export const DEMO_PASSWORD = 'jutyar2026';
const KEY = 'jutyar.session';

interface Auth { me: Officer | null; signIn: (email: string, password: string) => Promise<'ok' | 'bad' | 'inactive'>; signOut: () => void }
const Ctx = createContext<Auth>({ me: null, signIn: async () => 'bad', signOut: () => {} });

export function AuthProvider({ children }: { children: ReactNode }) {
  const [id, setId] = useState<string | null>(() => { try { return sessionStorage.getItem(KEY); } catch { return null; } });
  const me = id ? db.officers.get(id) ?? null : null;
  const signIn = useCallback(async (email: string, password: string) => {
    const o = db.officers.all().find(x => x.email.toLowerCase() === email.trim().toLowerCase());
    if (!o || password !== DEMO_PASSWORD) return 'bad' as const;
    if (!o.active) return 'inactive' as const;
    db.officers.patch(o.id, { lastSeen: new Date().toISOString() });
    try { sessionStorage.setItem(KEY, o.id); } catch { /* private mode: signed in for this tab only */ }
    setId(o.id);
    return 'ok' as const;
  }, []);
  const signOut = useCallback(() => { try { sessionStorage.removeItem(KEY); } catch { /* ignore */ } setId(null); }, []);
  const value = useMemo(() => ({ me: me && me.active ? me : null, signIn, signOut }), [me, signIn, signOut]);
  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export const useAuth = () => useContext(Ctx);

export function RequireAdmin({ children }: { children: ReactNode }) {
  const { me } = useAuth();
  const loc = useLocation();
  if (!me) return <Navigate to="/login" replace state={{ from: loc.pathname }} />;
  return <>{children}</>;
}
