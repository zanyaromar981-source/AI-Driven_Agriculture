// Data jobs (design 17): read only. The jobs that bring data in, how each stands, its last 14 days.
// GET /dashboard/jobs (FRONTEND.md 9 Data jobs). "late" happens by time alone, so the page also
// refreshes every minute.
import { Activity, CloudRain, Flame, Newspaper, Droplets, Waves, Sprout, RotateCcw } from 'lucide-react';
import { useI18n } from '../../i18n';
import { useApi } from '../../api/cache';
import { PageHead, Kpi, Pill, type Tone } from '../../components/ui';
import { StateBox } from '../../components/domain';
import './jobs.css';

type Day = 'ok' | 'late' | 'failed' | null;
interface Job {
  job: string; name_en: string; name_ku?: string | null; every_hours?: number | null; state: 'ok' | 'late' | 'failed' | 'never';
  last_run?: { started_at: string; finished_at?: string | null; ok?: boolean | null; rows?: number | null; message?: string | null } | null;
  last_ok?: string | null; next_due?: string | null; last_14_days: Day[]; message?: string | null;
}
const ICON: Record<string, typeof Activity> = { fires: Flame, dryness: CloudRain, briefs: Newspaper, groundwater: Droplets, dams: Waves, farm_analysis: Sprout };
const TONE: Record<Job['state'], Tone> = { ok: 'good', late: 'warn', failed: 'danger', never: '' };

export default function Jobs() {
  const { t, num, ago, date, pick } = useI18n();
  const q = useApi<{ jobs: Job[] }>('/dashboard/jobs', ['jobs'], { auth: true, everyMs: 60_000 });
  const jobs = q.data?.jobs ?? [];
  const count = (s: Job['state']) => jobs.filter(j => j.state === s).length;
  return (
    <div>
      <PageHead eyebrow={t('nav.g_system')} title={t('nav.jobs')} sub={t('jobs.sub')} />
      <div className="grid g4 mb">
        <Kpi label={t('jobs.k_ok')} value={num(count('ok'))} tone="good" icon={<Activity />} />
        <Kpi label={t('jobs.k_late')} value={num(count('late'))} tone={count('late') ? 'warn' : ''} />
        <Kpi label={t('jobs.k_failed')} value={num(count('failed'))} tone={count('failed') ? 'danger' : ''} />
        <Kpi label={t('jobs.k_never')} value={num(count('never'))} note={t('jobs.k_never_note')} />
      </div>
      <div className="card pad0">
        {q.error && !q.data ? <StateBox kind="error" action={<button className="btn sm" onClick={q.reload}><RotateCcw />{t('common.retry')}</button>} />
          : q.loading ? <div className="sk-rows" style={{ padding: 18 }}>{Array.from({ length: 6 }, (_, i) => <i key={i} className="sk" />)}</div>
            : !jobs.length ? <StateBox kind="empty" />
              : (
                <div className="table-wrap">
                  <table className="t cards jobs-table">
                    <thead><tr>
                      <th>{t('jobs.job')}</th><th>{t('jobs.every')}</th><th>{t('jobs.last')}</th><th>{t('jobs.next')}</th><th>{t('jobs.days')}</th><th>{t('common.status')}</th>
                    </tr></thead>
                    <tbody>
                      {jobs.map(j => {
                        const Icon = ICON[j.job] ?? Activity;
                        const msg = j.last_run?.message ?? j.message;
                        return (
                          <tr key={j.job}>
                            <td data-label={t('jobs.job')}>
                              <div className="job-name">
                                <span className={'ico ' + (j.state === 'ok' ? 'good' : j.state === 'never' ? '' : TONE[j.state])}><Icon /></span>
                                <span><b>{pick(j.name_ku, j.name_en)}</b>{msg && <span className="small muted job-msg" dir="ltr">{msg}</span>}</span>
                              </div>
                            </td>
                            <td data-label={t('jobs.every')}>{j.every_hours ? t('jobs.every_h', { n: num(j.every_hours) }) : t('jobs.on_demand')}</td>
                            <td data-label={t('jobs.last')}>{j.last_run ? <span title={date(j.last_run.started_at, 'datetime')}>{ago(j.last_run.finished_at ?? j.last_run.started_at)}</span> : <span className="muted">{t('common.never')}</span>}</td>
                            <td data-label={t('jobs.next')}>{j.next_due ? date(j.next_due, 'datetime') : '-'}</td>
                            <td data-label={t('jobs.days')}>
                              <span className="strip" role="img" aria-label={t('jobs.days')}>{j.last_14_days.map((d, i) => <i key={i} className={d ?? ''} title={d ? t('jobs.d_' + d) : t('jobs.d_none')} />)}</span>
                            </td>
                            <td data-label={t('common.status')}><Pill tone={TONE[j.state]}>{t('jobs.s_' + j.state)}</Pill></td>
                          </tr>
                        );
                      })}
                    </tbody>
                  </table>
                </div>
              )}
      </div>
      <p className="muted small" style={{ marginTop: 10 }}>{t('jobs.note')}</p>
    </div>
  );
}
