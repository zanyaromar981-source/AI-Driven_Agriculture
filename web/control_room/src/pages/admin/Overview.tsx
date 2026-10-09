// Admin overview (design 06 HEWv1, tablet CSRsU, phone S7v3l). Only numbers the server really holds:
// farm totals, new messages, fire detections, data jobs, dams, the brief. Each "needs attention" line
// links to its page. Parts the staff member cannot read are left out.
import { useMemo } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { Users, Map as MapIcon, Inbox, Flame, Waves, CloudRain, Activity, ChevronLeft, ChevronRight, FileText, UserPlus, CheckCircle2, Newspaper } from 'lucide-react';
import { useI18n } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { useApi } from '../../api/cache';
import { useBrief, useDams, useFires } from '../../api/public';
import type { FarmStats } from '../../api/types';
import { Kpi, Pill, type Tone } from '../../components/ui';
import { HBars } from '../../components/charts';
import { DistrictMap, GREEN_RAMP, ramp } from '../../components/DistrictMap';
import { cropColor, cropName } from '../../components/domain';
import { DISTRICT_BY_SLUG } from '../../data/places';
import type { JobStatus } from './region/types';
import './overview.css';

interface Need { key: string; icon: typeof Inbox; tone: Tone; title: string; sub: string; to: string }

export default function Overview() {
  const { t, num, date, lang, pick, dir } = useI18n();
  const { me, can } = useAuth();
  const nav = useNavigate();
  const stats = useApi<FarmStats>(can('farms') ? '/dashboard/stats/farms' : null, ['farms'], { auth: true });
  const counts = useApi<{ new: number; read: number; replied: number; closed: number }>(can('messages') ? '/dashboard/messages/counts' : null, ['messages'], { auth: true });
  const jobs = useApi<{ jobs: JobStatus[] }>(can('jobs') ? '/dashboard/jobs' : null, ['jobs'], { auth: true, everyMs: 120000 });
  const fires = useFires();
  const dams = useDams();
  const brief = useBrief();
  const outlook = useApi<unknown>('/outlooks', ['outlooks']);
  const water = useApi<unknown>('/water/plan', ['water']);

  const hour = new Date().getHours();
  const greet = t(hour < 12 ? 'overview.morning' : hour < 18 ? 'overview.afternoon' : 'overview.evening', { name: (me?.name ?? '').split(/\s+/)[0] });

  const firesByZone = useMemo(() => {
    const m = new Map<string, number>();
    for (const f of fires.data?.fires ?? []) if (f.zone_slug) m.set(f.zone_slug, (m.get(f.zone_slug) ?? 0) + 1);
    return m;
  }, [fires.data]);

  const needs = useMemo<Need[]>(() => {
    const out: Need[] = [];
    const n = counts.data?.new ?? 0;
    if (n > 0) out.push({ key: 'msg', icon: Inbox, tone: 'warn', title: t('overview.need_msgs', { n: num(n) }), sub: t('overview.need_msgs_sub'), to: '/admin/inbox' });
    const fc = fires.data?.fires.length ?? 0;
    if (fc > 0) out.push({ key: 'fire', icon: Flame, tone: 'danger', title: t('overview.need_fires', { n: num(fc), z: num(firesByZone.size) }), sub: t('overview.need_fires_sub'), to: '/admin/alerts' });
    const never: string[] = [];
    for (const j of jobs.data?.jobs ?? []) {
      if (j.state === 'ok') continue;
      if (j.state === 'never') { never.push(pick(j.name_ku, j.name_en)); continue; }
      out.push({ key: 'job-' + j.job, icon: Activity, tone: 'danger', title: t('overview.need_job_' + j.state, { name: pick(j.name_ku, j.name_en) }), sub: j.message ?? t('overview.need_job_sub'), to: '/admin/jobs' });
    }
    if (never.length) out.push({ key: 'job-never', icon: Activity, tone: '', title: t('overview.need_jobs_never', { n: num(never.length) }), sub: never.join(lang === 'ku' ? '، ' : ', '), to: '/admin/jobs' });
    for (const d of dams.data?.dams ?? []) if (!d.latest) out.push({ key: 'dam-' + d.slug, icon: Waves, tone: 'water', title: t('overview.need_dam', { name: pick(d.name_ku, d.name_en) }), sub: t('overview.need_dam_sub'), to: '/admin/region?tab=dams' });
    const noOutlook = outlook.error?.status === 404, noWater = water.error?.status === 404;
    if (noOutlook || noWater) out.push({ key: 'plans', icon: CloudRain, tone: '', title: t(noOutlook && noWater ? 'overview.need_plans' : noOutlook ? 'overview.need_outlook' : 'overview.need_water'), sub: t('overview.need_plans_sub'), to: '/admin/region?tab=' + (noOutlook ? 'outlooks' : 'water') });
    return out;
  }, [counts.data, fires.data, firesByZone, jobs.data, dams.data, outlook.error, water.error, t, num, pick, lang]);

  const byZone = useMemo(() => new Map((stats.data?.by_zone ?? []).map(z => [DISTRICT_BY_SLUG.get(z.slug)?.en ?? z.name_en, z.farms])), [stats.data]);
  const maxFarms = Math.max(1, ...byZone.values());
  const stops = [0.2, 0.4, 0.6, 0.8].map(x => x * maxFarms);
  const crops = (stats.data?.by_crop ?? []).filter(c => c.crop !== 'empty').sort((a, b) => b.dunam - a.dunam).slice(0, 8);
  const Chevron = dir === 'rtl' ? ChevronLeft : ChevronRight;
  const b = brief.data?.brief;
  const tot = stats.data?.totals;
  const allFarmers = useApi<{ count: number }>(can('farmers') ? '/dashboard/farmers?rows_per_page=1' : null, ['farmers'], { auth: true });
  const nFarmers = allFarmers.data?.count ?? tot?.farmers;

  return (
    <div className="ov">
      <div className="page-head">
        <div className="t">
          <div className="eyebrow">{date(new Date().toISOString(), 'date')}</div>
          <h1>{greet}</h1>
          <div className="sub">{t('overview.sub')}</div>
        </div>
        <div className="actions">
          {can('farms') && <Link className="btn" to="/print/government" target="_blank"><FileText />{t('overview.gov_report')}</Link>}
          {can('farmers', 'create') && <Link className="btn primary" to="/admin/farms?new=farmer"><UserPlus />{t('overview.new_farmer')}</Link>}
        </div>
      </div>

      <div className="grid g4 mb">
        {can('farms') && <Kpi icon={<Users />} label={t('overview.k_farmers')} value={nFarmers != null ? num(nFarmers) : '-'} note={t('overview.k_farmers_n')} />}
        {can('farms') && <Kpi icon={<MapIcon />} label={t('overview.k_farms')} value={tot ? num(tot.farms) : '-'} note={tot ? t('overview.k_farms_n', { du: num(tot.dunam, 1) }) : ''} />}
        {can('messages') && <Kpi icon={<Inbox />} tone={(counts.data?.new ?? 0) > 0 ? 'warn' : ''} label={t('overview.k_msgs')} value={counts.data ? num(counts.data.new) : '-'} note={counts.data ? t('overview.k_msgs_n', { n: num(counts.data.new + counts.data.read + counts.data.replied + counts.data.closed) }) : ''} />}
        <Kpi icon={<Flame />} tone={(fires.data?.fires.length ?? 0) > 0 ? 'danger' : ''} label={t('overview.k_fires')} value={fires.data ? num(fires.data.fires.length) : '-'} note={fires.data ? t('overview.k_fires_n', { z: num(firesByZone.size) }) : ''} />
      </div>

      <div className="ov-row mb">
        <section className="card ov-needs">
          <div className="card-head"><span className="eyebrow">{t('overview.needs')}</span></div>
          {!needs.length && <div className="ov-clear"><CheckCircle2 />{t('overview.all_clear')}</div>}
          {needs.map(n => (
            <Link key={n.key} to={n.to} className="list-item ov-need">
              <span className={'ico ' + n.tone}><n.icon /></span>
              <span className="ov-need-text"><b>{n.title}</b><small className="muted">{n.sub}</small></span>
              <Chevron className="muted" />
            </Link>
          ))}
        </section>
        {can('farms') && (
          <section className="card ov-map">
            <div className="card-head"><span className="eyebrow">{t('overview.map_title')}</span><span className="muted small">{t('overview.map_hint')}</span></div>
            {stats.data && stats.data.totals.farms === 0 ? <div className="empty">{t('overview.no_farms')}</div> : (
              <>
                <DistrictMap size="sm" styleKey={String(stats.data?.as_of) + lang} fill={en => { const v = byZone.get(en); return v ? ramp(v, stops, GREEN_RAMP) : undefined; }}
                  onDistrict={en => { const d = [...DISTRICT_BY_SLUG.values()].find(x => x.en === en); if (d) nav('/admin/farms?zone=' + d.slug); }} />
                <div className="legend">
                  {GREEN_RAMP.map((c, i) => <span key={c}><span className="dotc" style={{ background: c }} />{i === 0 ? t('overview.fewer') : i === GREEN_RAMP.length - 1 ? t('overview.more') : ''}</span>)}
                </div>
              </>
            )}
          </section>
        )}
      </div>

      <div className="ov-row">
        <div className="stack ov-side">
          {b && (
            <section className="card ov-brief">
              <div className="card-head"><span className="eyebrow"><Newspaper size={14} /> {t('overview.brief', { day: date(b.day, 'short') })}</span><Pill tone="water">{t('overview.ai_draft')}</Pill></div>
              <b className="ov-brief-h">{pick(b.headline_ku, b.headline_en)}</b>
            </section>
          )}
          {can('jobs') && (
            <section className="card">
              <div className="card-head"><span className="eyebrow">{t('overview.jobs')}</span><Link className="small" to="/admin/jobs">{t('overview.all')}</Link></div>
              {(jobs.data?.jobs ?? []).map(j => (
                <div key={j.job} className="ov-job">
                  <span>{pick(j.name_ku, j.name_en)}</span>
                  <Pill tone={j.state === 'ok' ? 'good' : j.state === 'never' ? '' : 'danger'}>{t('overview.job_' + j.state)}</Pill>
                </div>
              ))}
              {jobs.loading && <i className="sk" />}
            </section>
          )}
        </div>
        {can('farms') && (
          <section className="card ov-crops">
            <div className="card-head"><span className="eyebrow">{t('overview.crops_title')}</span><Link className="small" to="/admin/crops">{t('overview.all')}</Link></div>
            {crops.length ? <HBars digits={1} unit={t('common.dunam')} items={crops.map(c => ({ key: c.crop, label: cropName(c.crop, lang), value: c.dunam, color: cropColor(c.crop) }))} />
              : <div className="empty">{stats.loading ? t('common.loading') : t('overview.no_farms')}</div>}
          </section>
        )}
      </div>
    </div>
  );
}
