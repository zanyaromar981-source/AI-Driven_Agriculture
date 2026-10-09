// Run one write with a busy flag: the button stays disabled until the answer comes, so a double click
// cannot send twice. Errors show as a toast in the current language. Used by the P4 pages.
import { useCallback, useRef, useState } from 'react';
import { ApiError } from '../../../api/client';
import { useToast } from '../../../components/ui';
import { useErrorText } from '../../../components/domain';

export function useAction() {
  const [busy, setBusy] = useState(false);
  const running = useRef(false);
  const toast = useToast();
  const errText = useErrorText();
  const run = useCallback(async <T,>(fn: () => Promise<T>, ok?: string): Promise<T | undefined> => {
    if (running.current) return undefined;
    running.current = true; setBusy(true);
    try {
      const r = await fn();
      if (ok) toast(ok, 'good');
      return r;
    } catch (e) {
      toast(e instanceof ApiError ? errText(e) : String(e), 'danger');
      return undefined;
    } finally { running.current = false; setBusy(false); }
  }, [toast, errText]);
  return { busy, run };
}

/** a.b.c compared as numbers; -1, 0 or 1 */
export function cmpVersion(a: string, b: string) {
  const pa = a.split('.').map(Number), pb = b.split('.').map(Number);
  for (let i = 0; i < 3; i++) { const d = (pa[i] || 0) - (pb[i] || 0); if (d) return d < 0 ? -1 : 1; }
  return 0;
}
export const isVersion = (v: string) => /^\d+\.\d+\.\d+$/.test(v.trim());
