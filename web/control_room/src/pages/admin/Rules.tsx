// Rules: the numbers that decide warnings, map colours and field status. Each change keeps a history.
import { useMemo, useState } from 'react';
import { Info, Pencil, History, RotateCcw, CloudSun, Sprout, Satellite } from 'lucide-react';
import { db } from '../../data/db';
import { useRows } from '../../data/store';
import { seedRules } from '../../data/seed';
import type { Rule } from '../../data/types';
import { useI18n } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { Card, Drawer, Field, Modal, Note, PageHead, Pill, useToast } from '../../components/ui';

const GROUPS: Rule['group'][] = ['weather', 'dryness', 'field'];
const GROUP_ICON = { weather: CloudSun, dryness: Sprout, field: Satellite };

export default function Rules() {
  const { t, b, num, ago, date } = useI18n();
  const rules = useRows(db.rules);
  const start = useMemo(() => new Map(seedRules().map(r => [r.id, r.value])), []);
  const byGroup = useMemo(() => {
    const m = new Map<string, Rule[]>();
    for (const r of rules) { const a = m.get(r.group); a ? a.push(r) : m.set(r.group, [r]); }
    return m;
  }, [rules]);
  const [edit, setEdit] = useState<Rule | null>(null);
  const [hist, setHist] = useState<Rule | null>(null);

  return (
    <>
      <PageHead eyebrow={t('nav.g_app')} title={t('nav.rules')} sub={t('rules.sub')} />
      <Card className="mb">
        <div className="row" style={{ alignItems: 'flex-start', flexWrap: 'nowrap', gap: 12 }}>
          <span className="ico brand"><Info /></span>
          <div style={{ minWidth: 0 }}>
            <h3>{t('rules.what_title')}</h3>
            <p className="ink2" style={{ margin: '4px 0 12px' }}>{t('rules.what_text')}</p>
          </div>
        </div>
        <div className="grid g3">
          {GROUPS.map(g => {
            const Ic = GROUP_ICON[g];
            return (
              <div key={g} className="note" style={{ flexDirection: 'column', gap: 4 }}>
                <b className="row" style={{ color: 'var(--ink)' }}><Ic size={16} />{t('rules.where_' + g)}</b>
                <span>{t('rules.where_' + g + '_text')}</span>
              </div>
            );
          })}
        </div>
        <div className="grid g2 mt">
          <Note tone="warn" icon={<Info />}>{t('rules.today')}</Note>
          <Note tone="good" icon={<Info />}>{t('rules.advice')}</Note>
        </div>
      </Card>

      {GROUPS.map(g => (
        <Card key={g} className="mb" title={t('rules.group_' + g)}>
          <div className="table-wrap">
            <table className="t cards">
              <thead><tr><th>{t('rules.rule')}</th><th>{t('rules.meaning')}</th><th className="num">{t('rules.value')}</th><th>{t('rules.used_by')}</th><th>{t('rules.last_change')}</th><th /></tr></thead>
              <tbody>
                {(byGroup.get(g) ?? []).map(r => {
                  const last = r.history[r.history.length - 1];
                  const changed = start.get(r.id) !== r.value;
                  return (
                    <tr key={r.id}>
                      <td data-label={t('rules.rule')}><b>{b(r.name)}</b></td>
                      <td data-label={t('rules.meaning')} className="ink2 small" style={{ maxWidth: 380 }}>{b(r.meaning)}</td>
                      <td data-label={t('rules.value')} className="num nowrap"><b>{num(r.value)}</b> <span className="ltr muted">{r.unit}</span>{changed && <> <Pill tone="warn">{t('rules.changed')}</Pill></>}</td>
                      <td data-label={t('rules.used_by')}><Pill>{t('rules.by_' + r.usedBy)}</Pill></td>
                      <td data-label={t('rules.last_change')} className="small">{last ? <>{ago(last.at)} · {last.by}</> : <span className="muted">{t('rules.never')}</span>}</td>
                      <td data-label="" className="act">
                        <button className="btn sm" onClick={() => setEdit(r)}><Pencil />{t('common.edit')}</button>{' '}
                        <button className="btn sm icon" onClick={() => setHist(r)} title={t('rules.history')} aria-label={t('rules.history')}><History /></button>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </Card>
      ))}

      {edit && <EditRule rule={edit} startValue={start.get(edit.id) ?? edit.value} onClose={() => setEdit(null)} />}
      {hist && (
        <Drawer title={b(hist.name)} eyebrow={t('rules.history')} onClose={() => setHist(null)}>
          <p className="ink2" style={{ marginTop: 0 }}>{b(hist.meaning)}</p>
          <dl className="facts mb">
            <dt>{t('rules.now')}</dt><dd>{num(hist.value)} <span className="ltr">{hist.unit}</span></dd>
            <dt>{t('rules.start')}</dt><dd>{num(start.get(hist.id))} <span className="ltr">{hist.unit}</span></dd>
            <dt>{t('rules.range')}</dt><dd><span className="ltr">{hist.min} … {hist.max}</span></dd>
          </dl>
          {hist.history.length === 0 ? <div className="empty">{t('rules.no_history')}</div> : (
            [...hist.history].reverse().map((h, i) => (
              <div key={i} className="list-item" style={{ alignItems: 'flex-start' }}>
                <span className="ico gold"><History /></span>
                <div style={{ minWidth: 0 }}>
                  <b>{t('rules.set_to', { v: num(h.value) })} <span className="ltr">{db.rules.get(hist.id)?.unit}</span></b>
                  <div className="small muted">{h.by} · {date(h.at, 'datetime')}</div>
                  <div className="small">{h.why}</div>
                </div>
              </div>
            ))
          )}
        </Drawer>
      )}
    </>
  );
}

function EditRule({ rule, startValue, onClose }: { rule: Rule; startValue: number; onClose: () => void }) {
  const { t, b, num } = useI18n();
  const { me } = useAuth();
  const toast = useToast();
  const [v, setV] = useState(String(rule.value));
  const [why, setWhy] = useState('');
  const [err, setErr] = useState<{ v?: string; why?: string }>({});
  const save = (value: number, reason: string) => {
    const e: typeof err = {};
    if (!Number.isFinite(value) || value < rule.min || value > rule.max) e.v = 'rules.out_of_range';
    if (!reason.trim()) e.why = 'rules.why_needed';
    setErr(e);
    if (e.v || e.why) return;
    db.rules.patch(rule.id, { value, history: [...rule.history, { value, by: me?.name ?? '', at: new Date().toISOString(), why: reason.trim() }] });
    toast(t('common.saved'), 'good');
    onClose();
  };
  return (
    <Modal title={b(rule.name)} onClose={onClose} foot={<>
      <button className="btn" onClick={() => save(startValue, why || t('rules.back_reason'))} disabled={rule.value === startValue}><RotateCcw />{t('rules.back_start', { v: num(startValue) })}</button>
      <button className="btn" onClick={onClose}>{t('common.cancel')}</button>
      <button className="btn primary" onClick={() => save(Number(v), why)}>{t('common.save')}</button>
    </>}>
      <p className="ink2" style={{ marginTop: 0 }}>{b(rule.meaning)}</p>
      <div className="form-grid">
        <Field label={t('rules.value') + ' (' + rule.unit + ')'} hint={t('rules.allowed', { min: rule.min, max: rule.max })} error={err.v}>
          <input type="number" value={v} min={rule.min} max={rule.max} step="any" onChange={e => setV(e.target.value)} className="ltr-input" autoFocus />
        </Field>
        <Field label={t('rules.why')} error={err.why} full>
          <textarea value={why} onChange={e => setWhy(e.target.value)} placeholder={t('rules.why_ph')} />
        </Field>
      </div>
    </Modal>
  );
}
