// Alerts (design 12 l4Sbxx): a read-only feed built from live data, no route of its own (decided
// 2026-10-09): satellite fire detections per district, the dryness picture, the daily brief's points,
// and data jobs that never ran or failed. Sending messages to farmers waits for push (BACKEND.md 2.7).
import { useMemo, useState } from 'react';
import { Link } from 'react-router-dom';
import { Flame, CloudRain, Newspaper, Activity, Waves, type LucideIcon } from 'lucide-react';
import { useI18n } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { useApi } from '../../api/cache';
import { useBrief, useDams, useFires, useOverview } from '../../api/public';
import { PageHead, Pill, type Tone } from '../../components/ui';
import { StateBox, usePlace } from '../../components/domain';
import type { JobStatus } from './region/types';
import './alerts.css';

type Kind = 'fires' | 'dryness' | 'brief' | 'jobs';
type Level = 'alarm' | 'watch' | 'info';
interface Item { key: string; kind: Kind; level: Level; icon: LucideIcon; title: string; sub: string; source: string; at: string | null; to?: string }
const LEVEL_TONE: Record<Level, Tone> = { alarm: 'danger', watch: 'warn', info: '' };
const ICON_TONE: Record<Level, string> = { alarm: 'danger', watch: 'warn', info: 'good' };

export default function Alerts() {
  const { t, num, ago, pick, lang } = useI18n();
  const { can } = useAuth();
  const place = usePlace();
  const fires = useFires(), ov = useOverview(), brief = useBrief(), dams = useDams();
  const jobs = useApi<{ jobs: JobStatus[] }>(can('jobs') ? '/dashboard/jobs' : null, ['jobs'], { auth: true, everyMs: 120000 });
  const [kind, setKind] = useState<Kind | 'all'>('all');

  const items = useMemo<Item[]>(() => {
    const out: Item[] = [];
    // fires grouped by district, newest detection first
    const g = new Map<string, { n: number; near: number; last: string }>();
    for (const f of fires.data?.fires ?? []) {
      const k = f.zone_slug ?? '?';
      const e = g.get(k) ?? { n: 0, near: 0, last: f.detected_at };
      e.n++; e.near += f.farms_within_5km ?? 0; if (f.detected_at > e.last) e.last = f.detected_at;
      g.set(k, e);
    }
    for (const [z, e] of g) out.push({
      key: 'f-' + z, kind: 'fires', level: e.near > 0 ? 'alarm' : 'watch', icon: Flame,
      title: t('alerts.fires_in', { n: num(e.n), place: z === '?' ? t('alerts.unknown_place') : place.dist(z) }),
      sub: e.near > 0 ? t('alerts.fires_near', { n: num(e.near) }) : t('alerts.fires_far'), source: 'NASA FIRMS', at: e.last, to: '/admin/region?tab=fires',
    });
    // dryness: one line per band present, plus the driest
    if (ov.data) {
      const bands = new Map<string, number>();
      for (const z of ov.data.zones) if (z.band) bands.set(z.band, (bands.get(z.band) ?? 0) + 1);
      for (const [b, n] of bands) out.push({
        key: 'd-' + b, kind: 'dryness', level: b === 'very_dry' ? 'alarm' : b === 'dry' ? 'watch' : 'info', icon: CloudRain,
        title: t('alerts.band_count', { n: num(n), band: t('band.' + b) }),
        sub: t('alerts.band_sub', { avg: ov.data.summary.average_dryness == null ? '-' : num(ov.data.summary.average_dryness, 1), driest: ov.data.summary.driest.slice(0, 3).map(s => place.dist(s)).join(lang === 'ku' ? '، ' : ', ') }),
        source: 'Open-Meteo ERA5', at: null, to: '/admin/region',
      });
    }
    // the brief's points
    const b = brief.data?.brief;
    if (b) b.points.forEach((p, i) => out.push({ key: 'b-' + i, kind: 'brief', level: p.level, icon: Newspaper, title: pick(p.text_ku, p.text_en), sub: t('alerts.brief_sub'), source: t('alerts.brief_src'), at: b.generated_at }));
    // dams with no reading, jobs that never ran or failed
    for (const d of dams.data?.dams ?? []) if (!d.latest) out.push({ key: 'dam-' + d.slug, kind: 'jobs', level: 'info', icon: Waves, title: t('alerts.dam_none', { name: pick(d.name_ku, d.name_en) }), sub: t('alerts.dam_none_sub'), source: t('alerts.jobs_src'), at: null, to: '/admin/region?tab=dams' });
    for (const j of jobs.data?.jobs ?? []) if (j.state !== 'ok') out.push({
      key: 'j-' + j.job, kind: 'jobs', level: j.state === 'failed' ? 'alarm' : j.state === 'late' ? 'watch' : 'info', icon: Activity,
      title: t('alerts.job_' + j.state, { name: pick(j.name_ku, j.name_en) }), sub: j.message ?? '', source: t('alerts.jobs_src'), at: j.last_run?.finished_at ?? null, to: '/admin/jobs',
    });
    const rank = { alarm: 0, watch: 1, info: 2 };
    return out.sort((a, b) => rank[a.level] - rank[b.level] || (b.at ?? '').localeCompare(a.at ?? ''));
  }, [fires.data, ov.data, brief.data, dams.data, jobs.data, t, num, pick, place, lang]);

  const shown = kind === 'all' ? items : items.filter(i => i.kind === kind);
  const loading = !fires.data && !ov.data && !brief.data;
  const count = (k: Kind) => items.filter(i => i.kind === k).length;

  return (
    <div>
      <PageHead eyebrow={t('nav.g_act')} title={t('nav.alerts')} sub={t('alerts.sub')} />
      <div className="chips mb al-chips" role="tablist">
        {(['all', 'fires', 'dryness', 'brief', 'jobs'] as const).map(k => (
          <button key={k} className={'chip' + (kind === k ? ' on' : '')} onClick={() => setKind(k)} aria-pressed={kind === k}>
            {t('alerts.k_' + k)}{k !== 'all' && <span className="al-n">{num(count(k))}</span>}
          </button>
        ))}
      </div>
      <section className="card al-feed">
        {loading && Array.from({ length: 5 }, (_, i) => <div key={i} className="al-row"><i className="sk" style={{ width: '60%' }} /></div>)}
        {!loading && !shown.length && <StateBox kind="empty" title={t('alerts.none')} text={t('alerts.none_t')} />}
        {shown.map(i => {
          const body = (
            <>
              <span className={'ico ' + ICON_TONE[i.level]}><i.icon /></span>
              <span className="al-text">
                <span className="al-title"><b><bdi>{i.title}</bdi></b><Pill tone={LEVEL_TONE[i.level]}>{t('alerts.l_' + i.level)}</Pill></span>
                {i.sub && <small className="muted"><bdi>{i.sub}</bdi></small>}
              </span>
              <span className="al-meta"><span>{i.at ? ago(i.at) : ''}</span><small className="muted">{i.source}</small></span>
            </>
          );
          return i.to ? <Link key={i.key} to={i.to} className="al-row al-link">{body}</Link> : <div key={i.key} className="al-row">{body}</div>;
        })}
      </section>
      <p className="muted small mt">{t('alerts.foot')}</p>
    </div>
  );
}
