// Season outlook and water plan tabs. Both answer 404 until one is issued (FRONTEND.md 11): then a calm
// empty state. When they exist they are shown read-only here (typing them in is not built in the site).
import { useI18n } from '../../../i18n';
import { useApi } from '../../../api/cache';
import { StateBox, usePlace } from '../../../components/domain';
import { Pill, type Tone } from '../../../components/ui';
import type { Outlooks, WaterPlan } from './types';

const OT: Record<string, Tone> = { good: 'good', normal: '', bad: 'danger' };

export function OutlooksTab() {
  const { t, num, pick } = useI18n();
  const place = usePlace();
  const q = useApi<Outlooks>('/outlooks', ['outlooks']);
  if (q.error?.status === 404) return <section className="card"><StateBox kind="empty" title={t('region.no_outlook')} text={t('region.no_outlook_t')} /></section>;
  if (q.error && !q.data) return <StateBox kind="error" action={<button className="btn" onClick={q.reload}>{t('common.retry')}</button>} />;
  if (!q.data) return <section className="card"><i className="sk block" /></section>;
  const d = q.data;
  return (
    <section className="card">
      <div className="card-head"><span className="eyebrow">{t('region.outlook_for', { season: d.season, issued: d.issued })}</span>
        {d.track_record && <span className="muted small">{t('region.track', { r: num(d.track_record.seasons_right), n: num(d.track_record.seasons_tested) })}</span>}</div>
      <table className="t cards">
        <thead><tr><th>{t('region.c_district')}</th><th>{t('region.c_outlook')}</th><th className="num">{t('region.c_conf')}</th><th>{t('region.c_reason')}</th></tr></thead>
        <tbody>{d.zones.map(z => (
          <tr key={z.zone_slug}>
            <td data-label={t('region.c_district')}><b>{place.dist(z.zone_slug)}</b></td>
            <td data-label={t('region.c_outlook')}><Pill tone={OT[z.outlook]}>{t('region.o_' + z.outlook)}</Pill></td>
            <td data-label={t('region.c_conf')} className="num">{num(z.confidence_pct)}%</td>
            <td data-label={t('region.c_reason')} className="small">{pick(z.reason_ku, z.reason_en)}</td>
          </tr>))}</tbody>
      </table>
    </section>
  );
}

export function WaterTab() {
  const { t, num, pick } = useI18n();
  const place = usePlace();
  const q = useApi<WaterPlan>('/water/plan', ['water']);
  if (q.error?.status === 404) return <section className="card"><StateBox kind="empty" title={t('region.no_water')} text={t('region.no_water_t')} /></section>;
  if (q.error && !q.data) return <StateBox kind="error" action={<button className="btn" onClick={q.reload}>{t('common.retry')}</button>} />;
  if (!q.data) return <section className="card"><i className="sk block" /></section>;
  const d = q.data;
  return (
    <section className="card">
      <div className="card-head"><span className="eyebrow">{t('region.water_for', { season: d.season })}</span>
        <span className="muted small">{t('region.water_total', { m: num(d.totals.planned_million_m3, 1), u: num(d.totals.urgent_zones) })}</span></div>
      <table className="t cards">
        <thead><tr><th className="num">#</th><th>{t('region.c_district')}</th><th className="num">{t('region.c_need')}</th><th className="num">{t('region.c_send')}</th><th>{t('region.c_note')}</th></tr></thead>
        <tbody>{d.entries.map(e => (
          <tr key={e.zone_slug}>
            <td data-label="#" className="num">{num(e.rank)}</td>
            <td data-label={t('region.c_district')}><b>{place.dist(e.zone_slug)}</b> {e.urgent && <Pill tone="danger">{t('region.urgent')}</Pill>}</td>
            <td data-label={t('region.c_need')} className="num">{num(e.need)}</td>
            <td data-label={t('region.c_send')} className="num">{e.send_million_m3 == null ? '-' : num(e.send_million_m3, 1)}</td>
            <td data-label={t('region.c_note')} className="small">{pick(e.note_ku, e.note_en)}</td>
          </tr>))}</tbody>
      </table>
    </section>
  );
}
