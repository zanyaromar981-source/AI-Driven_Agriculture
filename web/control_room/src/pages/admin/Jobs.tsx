// Data jobs: read-only status of the jobs that fill every number. Connections are not changed from here.
import { useMemo } from 'react';
import { Info, Satellite, CloudSun, Flame, Waves, Sprout, Store, Activity, type LucideIcon } from 'lucide-react';
import { db } from '../../data/db';
import { useRows } from '../../data/store';
import type { Job } from '../../data/types';
import { useI18n } from '../../i18n';
import { Card, Kpi, Note, PageHead, Pill, type Tone } from '../../components/ui';

const ICONS: Record<string, LucideIcon> = { satellite: Satellite, 'cloud-sun': CloudSun, flame: Flame, waves: Waves, sprout: Sprout, store: Store };
const TONE: Record<Job['state'], Tone> = { ok: 'good', late: 'warn', failed: 'danger', off: '' };

export default function Jobs() {
  const { t, b, num, ago, date } = useI18n();
  const jobs = useRows(db.jobs);
  const count = useMemo(() => {
    const c = { ok: 0, late: 0, failed: 0, off: 0 };
    for (const j of jobs) c[j.state]++;
    return c;
  }, [jobs]);
  return (
    <>
      <PageHead eyebrow={t('nav.g_system')} title={t('nav.jobs')} sub={t('jobs.sub')} />
      <div className="grid g3 mb">
        <Kpi label={t('jobs.on_time')} value={num(count.ok) + ' / ' + num(jobs.length)} tone="good" icon={<Activity size={16} />} />
        <Kpi label={t('jobs.late')} value={num(count.late)} tone={count.late ? 'warn' : 'good'} />
        <Kpi label={t('jobs.failed')} value={num(count.failed)} tone={count.failed ? 'danger' : 'good'} />
      </div>
      <Card>
        <div className="table-wrap">
          <table className="t cards">
            <thead><tr><th>{t('jobs.job')}</th><th>{t('jobs.runs')}</th><th>{t('jobs.last')}</th><th>{t('jobs.next')}</th><th>{t('jobs.result')}</th><th>{t('jobs.days')}</th><th>{t('common.status')}</th></tr></thead>
            <tbody>
              {jobs.map(j => {
                const Ic = ICONS[j.icon] ?? Activity;
                return (
                  <tr key={j.id}>
                    <td data-label={t('jobs.job')}><div className="row" style={{ flexWrap: 'nowrap' }}><span className={'ico ' + (TONE[j.state] || '')}><Ic size={16} /></span><b>{b(j.name)}</b></div></td>
                    <td data-label={t('jobs.runs')} className="small">{j.every}</td>
                    <td data-label={t('jobs.last')} className="nowrap" title={date(j.last, 'datetime')}>{ago(j.last)}</td>
                    <td data-label={t('jobs.next')} className="nowrap">{date(j.next, 'datetime')}</td>
                    <td data-label={t('jobs.result')} className="small" style={{ color: j.state === 'ok' ? 'var(--ink-2)' : 'var(--' + (j.state === 'late' ? 'warn' : 'danger') + ')' }}><bdi dir="ltr">{j.result}</bdi></td>
                    <td data-label={t('jobs.days')}><span className="strip" aria-label={t('jobs.days')} style={{ direction: 'ltr' }}>{j.days.map((d, i) => <i key={i} className={d === 'none' ? '' : d} title={t('jobs.d_' + d)} />)}</span></td>
                    <td data-label={t('common.status')}><Pill tone={TONE[j.state]}>{t('jobs.s_' + j.state)}</Pill></td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
        <div className="legend">
          <span><i className="dotc" style={{ background: 'var(--good)' }} />{t('jobs.d_ok')}</span>
          <span><i className="dotc" style={{ background: 'var(--warn)' }} />{t('jobs.d_late')}</span>
          <span><i className="dotc" style={{ background: 'var(--danger)' }} />{t('jobs.d_failed')}</span>
          <span><i className="dotc" style={{ background: 'var(--line)' }} />{t('jobs.d_none')}</span>
        </div>
      </Card>
      <div className="mt"><Note tone="info" icon={<Info />}>{t('jobs.note')}</Note></div>
    </>
  );
}
