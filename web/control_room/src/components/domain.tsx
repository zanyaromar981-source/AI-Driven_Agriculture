// Small pieces that know Jutyar data: crops, places, dryness bands, phones, empty states.
import type { ReactNode } from 'react';
import { CloudOff, Inbox, Lock } from 'lucide-react';
import { CROP, setServerCrops } from '../data/crops';
import { useApi } from '../api/cache';
import { DISTRICT_BY_SLUG, GOV_BY_NAME, SUB_BY_SLUG, GOVERNORATES, DISTRICTS, slugify } from '../data/places';
import type { Band } from '../api/types';
import type { ApiError } from '../api/client';
import { useI18n } from '../i18n';
import { Pill, type Tone } from './ui';

/** Load the server's crop table once per page view (cached, topic crops) and use it everywhere. */
export function useCrops() {
  const q = useApi<{ crops: { code: string; name_en: string; name_ku?: string | null; color: string; yield_kg_per_dunam?: number | null; active?: boolean }[] }>('/crops', ['crops']);
  if (q.data?.crops?.length) setServerCrops(q.data.crops);
  return q;
}

export function cropName(code: string, lang: 'ku' | 'en') { const c = CROP.get(code); return c ? (lang === 'ku' ? c.ku : c.en) : code; }
export const cropColor = (code: string) => CROP.get(code)?.color ?? '#B9C2B5';

export function CropTag({ code }: { code: string }) {
  const { lang } = useI18n();
  useCrops();
  return <span className="nowrap"><span className="dotc" style={{ background: cropColor(code) }} />{cropName(code, lang)}</span>;
}

/** Place names in the current language from a governorate name/slug, a district slug, a sub-district slug. */
export function usePlace() {
  const { lang } = useI18n();
  const n = (p?: { en: string; ku: string }) => (p ? (lang === 'ku' ? p.ku || p.en : p.en) : '');
  return {
    gov: (g?: string | null) => (g ? n(GOV_BY_NAME.get(g) ?? GOV_BY_NAME.get(slugify(g))) || g : ''),
    dist: (slug?: string | null) => (slug ? n(DISTRICT_BY_SLUG.get(slug)) || slug : ''),
    sub: (slug?: string | null) => (slug ? n(SUB_BY_SLUG.get(slug)) || slug : ''),
  };
}
export function usePlaceOptions(gov: string) {
  const { lang } = useI18n();
  const n = (p: { en: string; ku: string }) => (lang === 'ku' ? p.ku : p.en);
  return {
    govs: GOVERNORATES.map(g => [g.en, n(g)] as [string, string]),
    dists: DISTRICTS.filter(d => !gov || d.gov === gov).map(d => [d.slug, n(d)] as [string, string]),
  };
}

const BAND_TONE: Record<Band, Tone> = { much_greener: 'good', greener: 'good', normal: '', dry: 'warn', very_dry: 'danger' };
export function BandPill({ band }: { band?: Band | null }) {
  const { t } = useI18n();
  if (!band) return <Pill>{t('common.no_data')}</Pill>;
  return <Pill tone={BAND_TONE[band]}>{t('band.' + band)}</Pill>;
}

/** +9647501234567 -> +964 750 123 4567, always left to right. */
export function prettyPhone(value: string) {
  const p = value.replace(/\s/g, '');
  return p.length === 14 ? `${p.slice(0, 4)} ${p.slice(4, 7)} ${p.slice(7, 10)} ${p.slice(10)}` : value;
}
export function Phone({ value }: { value?: string | null }) {
  if (!value) return <span className="muted">-</span>;
  return <span className="ltr tabular">{prettyPhone(value)}</span>;
}

/** Calm states for a block of data: empty, error (with the cache note), no permission, coming soon. */
export function StateBox({ kind, title, text, action }: { kind: 'empty' | 'error' | 'locked' | 'soon'; title?: string; text?: ReactNode; action?: ReactNode }) {
  const { t } = useI18n();
  const Icon = kind === 'error' ? CloudOff : kind === 'locked' ? Lock : Inbox;
  const tone = kind === 'error' ? 'danger' : kind === 'locked' ? 'warn' : '';
  return (
    <div className="state-box">
      <span className={'ico big ' + tone}><Icon /></span>
      <b>{title ?? t('state.' + kind + '_title')}</b>
      <span className="muted">{text ?? t('state.' + kind + '_text')}</span>
      {action}
    </div>
  );
}

/** Message for a failed call, in the current language, from the server's error code. */
export function useErrorText() {
  const { t } = useI18n();
  return (e: ApiError | null | undefined) => {
    if (!e) return '';
    const k = 'err.' + e.code;
    const s = t(k);
    return s === k ? t('err.generic') : s;
  };
}
