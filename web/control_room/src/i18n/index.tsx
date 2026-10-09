// Kurdish (Sorani, right to left) and English (left to right).
// Texts live in i18n/en/<page>.json and i18n/ku/<page>.json; a key is "<page>.<name>", e.g. "farms.title".
// Kurdish falls back to English while a text is not written yet.
import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from 'react';
import { prefs } from '../data/store';

export type Lang = 'ku' | 'en';
export interface Bi { en: string; ku: string }

type Dict = Record<string, string>;
function load(mods: Record<string, unknown>): Dict {
  const out: Dict = {};
  for (const [path, mod] of Object.entries(mods)) {
    const ns = path.split('/').pop()!.replace('.json', '');
    const obj = ((mod as { default?: Dict }).default ?? mod) as Dict;
    for (const [k, v] of Object.entries(obj)) out[ns + '.' + k] = v;
  }
  return out;
}
export const EN: Dict = load(import.meta.glob('./en/*.json', { eager: true }));
export const KU: Dict = load(import.meta.glob('./ku/*.json', { eager: true }));

const KU_MONTHS = ['کانوونی دووەم', 'شوبات', 'ئازار', 'نیسان', 'ئایار', 'حوزەیران', 'تەممووز', 'ئاب', 'ئەیلوول', 'تشرینی یەکەم', 'تشرینی دووەم', 'کانوونی یەکەم'];

export type Vars = Record<string, string | number>;
const fill = (s: string, v?: Vars) => (v ? s.replace(/\{(\w+)\}/g, (m, k) => (k in v ? String(v[k]) : m)) : s);

export interface I18n {
  lang: Lang;
  dir: 'rtl' | 'ltr';
  setLang: (l: Lang) => void;
  kuDigits: boolean;
  setKuDigits: (v: boolean) => void;
  /** pick the server's _ku or _en text; Kurdish falls back to English when null */
  pick: (ku: string | null | undefined, en: string | null | undefined) => string;
  t: (key: string, vars?: Vars) => string;
  /** a record's own bilingual text (names, alerts, news) */
  b: (x: Bi | undefined | null) => string;
  /** a place or crop with en and ku names */
  nm: (x: { en: string; ku?: string } | undefined | null) => string;
  num: (n: number | null | undefined, digits?: number) => string;
  date: (iso: string | null | undefined, style?: 'date' | 'short' | 'time' | 'datetime' | 'month') => string;
  ago: (iso: string | null | undefined) => string;
}

const Ctx = createContext<I18n | null>(null);
const fmtCache = new Map<string, Intl.NumberFormat | Intl.DateTimeFormat>();
function nf(locale: string, d: number) {
  const k = 'n' + locale + d; let f = fmtCache.get(k) as Intl.NumberFormat | undefined;
  if (!f) { f = new Intl.NumberFormat(locale, { minimumFractionDigits: d, maximumFractionDigits: d }); fmtCache.set(k, f); }
  return f;
}
function df(locale: string, style: string) {
  const k = 'd' + locale + style; let f = fmtCache.get(k) as Intl.DateTimeFormat | undefined;
  if (!f) {
    const o: Intl.DateTimeFormatOptions =
      style === 'date' ? { day: 'numeric', month: 'long', year: 'numeric' }
      : style === 'short' ? { day: 'numeric', month: 'short' }
      : style === 'time' ? { hour: '2-digit', minute: '2-digit', hour12: false }
      : style === 'month' ? { month: 'long', year: 'numeric' }
      : { day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit', hour12: false };
    f = new Intl.DateTimeFormat(locale, o); fmtCache.set(k, f);
  }
  return f;
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [lang, setLangState] = useState<Lang>(() => prefs.get<Lang>('lang', 'ku'));
  const [kuDigits, setKuDigitsState] = useState<boolean>(() => prefs.get('kuDigits', false));
  const setKuDigits = useCallback((v: boolean) => { prefs.set('kuDigits', v); setKuDigitsState(v); }, []);
  const dir = lang === 'ku' ? 'rtl' : 'ltr';

  useEffect(() => {
    const h = document.documentElement;
    h.lang = lang === 'ku' ? 'ckb' : 'en'; h.dir = dir;
  }, [lang, dir]);

  const setLang = useCallback((l: Lang) => { prefs.set('lang', l); setLangState(l); }, []);

  const value = useMemo<I18n>(() => {
    const locale = lang === 'ku' ? (kuDigits ? 'ckb-IQ-u-nu-arab' : 'ckb-IQ-u-nu-latn') : 'en-GB';
    const t = (key: string, vars?: Vars) => {
      const s = (lang === 'ku' ? KU[key] : undefined) || EN[key];
      return s == null ? key : fill(s, vars);
    };
    const num = (n: number | null | undefined, d = 0) => (n == null || Number.isNaN(n) ? '-' : nf(locale, d).format(n));
    const date = (iso: string | null | undefined, style: 'date' | 'short' | 'time' | 'datetime' | 'month' = 'date') => {
      if (!iso) return '-';
      const d = new Date(iso); if (Number.isNaN(+d)) return '-';
      if (lang !== 'ku') return df(locale, style).format(d);
      // Browsers have no Sorani month names (ckb falls back to English): build the date by hand.
      // A leading right-to-left mark keeps "9ی تشرینی یەکەم" in order inside Kurdish text.
      const day = num(d.getDate()), mon = KU_MONTHS[d.getMonth()], yr = d.getFullYear();
      const hm = num(d.getHours()).padStart(2, '0') + ':' + String(d.getMinutes()).padStart(2, '0');
      const out = style === 'time' ? hm
        : style === 'month' ? `${mon}ی ${yr}`
        : style === 'short' ? `${day}ی ${mon}`
        : style === 'datetime' ? `${day}ی ${mon} ${hm}`
        : `${day}ی ${mon}ی ${yr}`;
      return '‏' + out;
    };
    const ago = (iso: string | null | undefined) => {
      if (!iso) return t('common.never');
      const m = Math.round((Date.now() - +new Date(iso)) / 6e4);
      if (m < 1) return t('common.just_now');
      if (m < 60) return t('common.min_ago', { n: num(m) });
      if (m < 1440) return t('common.h_ago', { n: num(Math.floor(m / 60)) });
      if (m < 2880) return t('common.yesterday');
      if (m < 43200) return t('common.d_ago', { n: num(Math.floor(m / 1440)) });
      return date(iso, 'date');
    };
    return {
      lang, dir, setLang, t, num, date, ago, kuDigits, setKuDigits,
      pick: (ku, en) => (lang === 'ku' ? ku || en || '' : en || ku || ''),
      b: x => (x ? (lang === 'ku' ? x.ku || x.en : x.en || x.ku) : ''),
      nm: x => (x ? (lang === 'ku' ? x.ku || x.en : x.en) : ''),
    };
  }, [lang, dir, setLang, kuDigits, setKuDigits]);

  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export function useI18n(): I18n {
  const c = useContext(Ctx);
  if (!c) throw new Error('useI18n outside I18nProvider');
  return c;
}
