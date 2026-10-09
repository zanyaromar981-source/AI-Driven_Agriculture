// Helpers for the public View page: the dryness colour scale and short source labels.
// Dryness is the server's rain-against-normal score (FRONTEND.md 11): 0 to 100, 50 = normal, lower = wetter.

/** Six steps over the whole 0 to 100 scale, wettest first. */
export const DRY_STEPS: { max: number; color: string; label: string }[] = [
  { max: 10, color: '#155E43', label: '0 - 9' },
  { max: 20, color: '#3E9461', label: '10 - 19' },
  { max: 35, color: '#8EC39A', label: '20 - 34' },
  { max: 50, color: '#DCE5B0', label: '35 - 49' },
  { max: 65, color: '#E8B567', label: '50 - 64' },
  { max: 101, color: '#C2452A', label: '65+' },
];
export const NO_DATA = '#E3E6DE';
export const dryColor = (v?: number | null) => (v == null ? NO_DATA : (DRY_STEPS.find(s => v < s.max) ?? DRY_STEPS[DRY_STEPS.length - 1]).color);

/** "Open-Meteo ERA5, district centre: 365-day ..." -> "Open-Meteo ERA5, district centre" */
export const shortSource = (s?: string | null) => (s ? s.split(':')[0].trim() : '');

export type ViewTab = 'map' | 'water' | 'fires' | 'compare' | 'market';
export const TABS: { key: ViewTab; icon: string }[] = [
  { key: 'map', icon: 'map' }, { key: 'water', icon: 'waves' }, { key: 'fires', icon: 'flame' }, { key: 'compare', icon: 'chart' }, { key: 'market', icon: 'store' },
];

/** The current month as YYYY-MM (the server reads districts by month). */
export const thisMonth = () => new Date().toISOString().slice(0, 7);

/** Dates in Sorani with Latin digits (the browser has no Kurdish month names). */
const KU_MONTHS = ['کانوونی دووەم', 'شوبات', 'ئازار', 'نیسان', 'ئایار', 'حوزەیران', 'تەممووز', 'ئاب', 'ئەیلوول', 'تشرینی یەکەم', 'تشرینی دووەم', 'کانوونی یەکەم'];
const EN_MONTHS = ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December'];
export function viewDate(iso: string | null | undefined, lang: 'ku' | 'en', withTime = false, withYear = true): string {
  if (!iso) return '-';
  const d = new Date(iso.length === 10 ? iso + 'T12:00:00' : iso);
  if (Number.isNaN(+d)) return '-';
  const m = (lang === 'ku' ? KU_MONTHS : EN_MONTHS)[d.getMonth()];
  const time = withTime ? ' ' + String(d.getHours()).padStart(2, '0') + ':' + String(d.getMinutes()).padStart(2, '0') : '';
  const y = withYear ? ' ' + d.getFullYear() : '';
  return lang === 'ku' ? '‏' + d.getDate() + 'ی ' + m + y + time : d.getDate() + ' ' + m + y + time;
}
export const monthName = (i: number, lang: 'ku' | 'en') => (lang === 'ku' ? KU_MONTHS : EN_MONTHS)[i];
