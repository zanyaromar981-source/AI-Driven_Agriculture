// Small shared pieces. Every page builds from these, so the look stays the same everywhere.
import { createContext, useCallback, useContext, useEffect, useRef, useState, type ReactNode } from 'react';
import { createPortal } from 'react-dom';
import { X, AlertTriangle } from 'lucide-react';
import { useI18n } from '../i18n';

export type Tone = '' | 'good' | 'warn' | 'danger' | 'water' | 'brand' | 'dark' | 'gold' | 'info';

export function PageHead({ eyebrow, title, sub, actions }: { eyebrow?: string; title: string; sub?: ReactNode; actions?: ReactNode }) {
  return (
    <div className="page-head">
      <div className="t">{eyebrow && <div className="eyebrow">{eyebrow}</div>}<h1>{title}</h1>{sub && <div className="sub">{sub}</div>}</div>
      {actions && <div className="actions">{actions}</div>}
    </div>
  );
}

export function Card({ children, className = '', title, extra, ...rest }: { children: ReactNode; className?: string; title?: ReactNode; extra?: ReactNode } & Omit<React.HTMLAttributes<HTMLDivElement>, 'title'>) {
  return (
    <div className={'card ' + className} {...rest}>
      {(title || extra) && <div className="card-head">{typeof title === 'string' ? <span className="eyebrow">{title}</span> : title}{extra}</div>}
      {children}
    </div>
  );
}

export function Kpi({ label, value, note, tone = '', icon }: { label: string; value: ReactNode; note?: ReactNode; tone?: Tone; icon?: ReactNode }) {
  return <div className={'card kpi ' + tone}><div className="l">{icon}{label}</div><div className="v">{value}</div>{note != null && <div className="n">{note}</div>}</div>;
}

export function Pill({ children, tone = '', icon }: { children: ReactNode; tone?: Tone; icon?: ReactNode }) {
  return <span className={'pill ' + tone}>{icon}{children}</span>;
}

export function Note({ children, tone = '', icon }: { children: ReactNode; tone?: Tone; icon?: ReactNode }) {
  return <div className={'note ' + tone}>{icon}<span>{children}</span></div>;
}

export function Switch({ on, onChange, disabled, label }: { on: boolean; onChange?: (v: boolean) => void; disabled?: boolean; label?: string }) {
  return (
    <label className="switch" aria-label={label}>
      <input type="checkbox" checked={on} disabled={disabled} onChange={e => onChange?.(e.target.checked)} />
      <span />
    </label>
  );
}

export function SetRow({ title, sub, children }: { title: ReactNode; sub?: ReactNode; children: ReactNode }) {
  return <div className="set"><div className="txt"><b>{title}</b>{sub && <small>{sub}</small>}</div><div className="ctl">{children}</div></div>;
}

export function Tabs<K extends string>({ value, onChange, items }: { value: K; onChange: (k: K) => void; items: [K, string][] }) {
  return (
    <div className="tabs" role="tablist">
      {items.map(([k, label]) => <button key={k} role="tab" aria-selected={value === k} className={value === k ? 'on' : ''} onClick={() => onChange(k)}>{label}</button>)}
    </div>
  );
}

export function Field({ label, hint, error, children, full }: { label: string; hint?: string; error?: string; children: ReactNode; full?: boolean }) {
  const { t } = useI18n();
  return (
    <label className={'field' + (error ? ' bad' : '') + (full ? ' full' : '')}>
      <span>{label}{hint && <small> {hint}</small>}</span>
      {children}
      {error && <span className="err small">{t(error)}</span>}
    </label>
  );
}

/** A select from [value, label] pairs. */
export function Select({ value, onChange, options, ...rest }: { value: string; onChange: (v: string) => void; options: [string, string][] } & Omit<React.SelectHTMLAttributes<HTMLSelectElement>, 'onChange'>) {
  return <select value={value} onChange={e => onChange(e.target.value)} {...rest}>{options.map(([v, l]) => <option key={v} value={v}>{l}</option>)}</select>;
}

// ---------- overlays ----------
function useEscape(onClose: () => void) {
  const ref = useRef(onClose); ref.current = onClose;
  useEffect(() => {
    const k = (e: KeyboardEvent) => { if (e.key === 'Escape') ref.current(); };
    addEventListener('keydown', k);
    const prev = document.body.style.overflow; document.body.style.overflow = 'hidden';
    return () => { removeEventListener('keydown', k); document.body.style.overflow = prev; };
  }, []);
}

export function Modal({ title, children, onClose, foot, wide }: { title: string; children: ReactNode; onClose: () => void; foot?: ReactNode; wide?: boolean }) {
  useEscape(onClose);
  return createPortal(
    <>
      <div className="overlay" onClick={onClose} />
      <div className="modal-wrap" onMouseDown={e => { if (e.target === e.currentTarget) onClose(); }}>
        <div className={'modal' + (wide ? ' wide' : '')} role="dialog" aria-modal="true" aria-label={title}>
          <div className="spread" style={{ marginBottom: 10 }}><h2>{title}</h2><button className="btn ghost sm icon" onClick={onClose} aria-label="close"><X /></button></div>
          {children}
          {foot && <div className="foot">{foot}</div>}
        </div>
      </div>
    </>, document.body);
}

export function Drawer({ title, eyebrow, children, onClose, head }: { title: ReactNode; eyebrow?: ReactNode; children: ReactNode; onClose: () => void; head?: ReactNode }) {
  useEscape(onClose);
  return createPortal(
    <>
      <div className="overlay" onClick={onClose} />
      <aside className="drawer" role="dialog" aria-modal="true">
        {eyebrow && <div className="eyebrow">{eyebrow}</div>}
        <div className="drawer-head"><h2>{title}</h2>{head}<button className="btn ghost sm icon" onClick={onClose} aria-label="close"><X /></button></div>
        {children}
      </aside>
    </>, document.body);
}

/** Ask before something that cannot be undone. */
export function Confirm({ title, text, okLabel, onOk, onClose, danger = true }: { title: string; text: ReactNode; okLabel: string; onOk: () => void; onClose: () => void; danger?: boolean }) {
  const { t } = useI18n();
  return (
    <Modal title={title} onClose={onClose} foot={<>
      <button className="btn" onClick={onClose}>{t('common.cancel')}</button>
      <button className={'btn ' + (danger ? 'danger solid' : 'primary')} onClick={() => { onOk(); onClose(); }}>{okLabel}</button>
    </>}>
      <Note tone={danger ? 'danger' : 'info'} icon={<AlertTriangle />}>{text}</Note>
    </Modal>
  );
}

// ---------- toasts ----------
type ToastFn = (msg: ReactNode, tone?: Tone) => void;
const ToastCtx = createContext<ToastFn>(() => {});
export function ToastProvider({ children }: { children: ReactNode }) {
  const [list, setList] = useState<{ id: number; msg: ReactNode; tone: Tone }[]>([]);
  const push = useCallback<ToastFn>((msg, tone = '') => {
    const id = Date.now() + Math.random();
    setList(l => [...l.slice(-2), { id, msg, tone }]);
    setTimeout(() => setList(l => l.filter(x => x.id !== id)), 3200);
  }, []);
  return (
    <ToastCtx.Provider value={push}>
      {children}
      {createPortal(<div className="toasts" aria-live="polite">{list.map(x => <div key={x.id} className={'toast ' + x.tone}>{x.msg}</div>)}</div>, document.body)}
    </ToastCtx.Provider>
  );
}
export const useToast = () => useContext(ToastCtx);

/** A value that settles 200 ms after typing stops, so big lists filter once, not on every key. */
export function useDebounced<T>(v: T, ms = 200): T {
  const [d, setD] = useState(v);
  useEffect(() => { const h = setTimeout(() => setD(v), ms); return () => clearTimeout(h); }, [v, ms]);
  return d;
}

export function Skeleton({ kind = 'cards' }: { kind?: 'cards' | 'table' | 'map' }) {
  const lines = (n: number) => Array.from({ length: n }, (_, i) => <i key={i} className={'sk ' + ['w80', 'w60', 'w40'][i % 3]} />);
  return (
    <div aria-busy="true">
      <div className="page-head"><div className="t"><i className="sk w20" /><i className="sk h28 w40" /><i className="sk w60" /></div></div>
      {kind === 'map' ? <div className="grid g-main"><div className="card"><i className="sk block" /></div><div className="card">{lines(10)}</div></div>
        : kind === 'table' ? <div className="card">{lines(14)}</div>
          : <div className="grid g2">{[0, 1, 2, 3].map(i => <div key={i} className="card">{lines(5)}</div>)}</div>}
    </div>
  );
}
