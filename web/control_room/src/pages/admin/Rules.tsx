// Rules (design 16): every number that decides a warning or a colour, where it is used, its allowed
// range and its history. GET/PUT /dashboard/rules, /rules/{code}/history, /rules/{code}/reset
// (FRONTEND.md 9 Rules). Honest limit: the jobs do not read these yet, and the page says so.
import { useMemo, useState, type ReactNode } from 'react';
import { CloudSun, Sprout, Satellite, Lightbulb, Pencil, History, RotateCcw, TriangleAlert } from 'lucide-react';
import { useI18n } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { api, qs } from '../../api/client';
import { useApi, invalidate } from '../../api/cache';
import { PageHead, Note, Modal, Drawer, Field, Pill } from '../../components/ui';
import { StateBox } from '../../components/domain';
import { useAction } from './inbox/useAction';
import './rules.css';

type UsedBy = 'weather_planner' | 'dryness' | 'field_eye';
interface Rule {
  code: string; grp: string; name_en: string; name_ku?: string | null; meaning_en: string; meaning_ku?: string | null;
  value: number; unit: string; min_value: number; max_value: number; default_value: number; used_by: UsedBy;
  updated_by?: string | null; updated_at: string;
}
interface Change { id: string; code: string; old_value: number; new_value: number; reason: string; staff_id: string; at: string }
/** readable unit: the server sends codes like c, mm, ug_m3, pct */
function useUnit() { const { t } = useI18n(); return (u: string) => { const k = 'rules.u_' + u; const s = t(k); return s === k ? u : s; }; }
/**
 * Rule name and meaning in the current language. The server has no Sorani for rules yet, so Kurdish
 * comes from rules.json (n_<code>, m_<code>); English text shown inside Kurdish stays left to right.
 */
function useRuleText() {
  const { t, lang } = useI18n();
  const one = (ku: string | null | undefined, key: string, en: string): ReactNode => {
    if (lang !== 'ku') return en;
    if (ku) return ku;
    const s = t(key);
    return s !== key ? s : <bdi dir="ltr">{en}</bdi>;
  };
  return {
    name: (r: Rule) => one(r.name_ku, 'rules.n_' + r.code, r.name_en),
    nameText: (r: Rule) => { const v = one(r.name_ku, 'rules.n_' + r.code, r.name_en); return typeof v === 'string' ? v : r.name_en; },
    meaning: (r: Rule) => one(r.meaning_ku, 'rules.m_' + r.code, r.meaning_en),
  };
}
/** a signed number kept left to right inside text, so "-2" never shows as "2-" in Kurdish */
const ltr = (s: string) => '\u2066' + s + '\u2069';
const ORDER: UsedBy[] = ['weather_planner', 'dryness', 'field_eye'];
const WHERE_ICON: Record<UsedBy, typeof CloudSun> = { weather_planner: CloudSun, dryness: Sprout, field_eye: Satellite };

export default function Rules() {
  const { t, num } = useI18n();
  const { can } = useAuth();
  const rt = useRuleText();
  const q = useApi<{ rules: Rule[] }>('/dashboard/rules', ['rules'], { auth: true });
  const [edit, setEdit] = useState<Rule | null>(null);
  const [reset, setReset] = useState<Rule | null>(null);
  const [hist, setHist] = useState<Rule | null>(null);
  const groups = useMemo(() => {
    const by = new Map<UsedBy, Rule[]>();
    for (const r of q.data?.rules ?? []) { const a = by.get(r.used_by) ?? []; a.push(r); by.set(r.used_by, a); }
    return ORDER.filter(u => by.has(u)).map(u => [u, by.get(u)!] as const);
  }, [q.data]);
  const canEdit = can('rules', 'update');
  const unit = useUnit();

  if (!can('rules') || (q.error?.status === 403 && !q.data)) return <div className="rules-page"><PageHead eyebrow={t('nav.g_app')} title={t('nav.rules')} /><div className="card"><StateBox kind="locked" /></div></div>;
  return (
    <div className="rules-page">
      <PageHead eyebrow={t('nav.g_app')} title={t('nav.rules')} sub={t('rules.sub')} />
      <Note tone="warn" icon={<TriangleAlert />}>{t('rules.honest')}</Note>
      <div className="rules-explain card">
        {ORDER.map(u => { const I = WHERE_ICON[u]; return (
          <div key={u} className="where"><span className="ico brand"><I /></span><b>{t('rules.w_' + u)}</b><span className="small ink2">{t('rules.w_' + u + '_text')}</span></div>
        ); })}
        <div className="where tip"><span className="ico gold"><Lightbulb /></span><b>{t('rules.tip')}</b><span className="small ink2">{t('rules.tip_text')}</span></div>
      </div>
      {q.error && !q.data ? <div className="card"><StateBox kind="error" action={<button className="btn sm" onClick={q.reload}><RotateCcw />{t('common.retry')}</button>} /></div>
        : q.loading ? <div className="card"><div className="sk-rows">{Array.from({ length: 8 }, (_, i) => <i key={i} className="sk" />)}</div></div>
          : groups.map(([u, rules]) => (
            <section key={u} className="card pad0 rules-group">
              <h3 className="group-title">{t('rules.w_' + u)}</h3>
              <div className="table-wrap">
                <table className="t cards">
                  <thead><tr><th>{t('rules.rule')}</th><th>{t('rules.meaning')}</th><th className="num">{t('rules.value')}</th><th>{t('rules.range')}</th><th>{t('rules.last')}</th><th /></tr></thead>
                  <tbody>
                    {rules.map(r => {
                      const changed = r.value !== r.default_value;
                      return (
                        <tr key={r.code}>
                          <td data-label={t('rules.rule')}><b>{rt.name(r)}</b><div className="mono small muted">{r.code}</div></td>
                          <td data-label={t('rules.meaning')} className="ink2 small meaning">{rt.meaning(r)}</td>
                          <td data-label={t('rules.value')} className="num"><bdi dir="ltr"><b className="val">{num(r.value, r.value % 1 ? 1 : 0)}</b> <span className="muted small">{unit(r.unit)}</span></bdi></td>
                          <td data-label={t('rules.range')} className="small ink2 nowrap">{t('rules.range_v', { a: ltr(num(r.min_value)), b: ltr(num(r.max_value)), d: ltr(num(r.default_value)) })}</td>
                          <td data-label={t('rules.last')} className="small">{changed ? <Pill tone="warn">{t('rules.changed')}</Pill> : <span className="muted">{t('rules.start_value')}</span>}</td>
                          <td className="act">
                            <button className="btn sm" onClick={() => setHist(r)} aria-label={t('rules.history')}><History /></button>{' '}
                            {canEdit && <button className="btn sm" onClick={() => setEdit(r)}><Pencil />{t('rules.change')}</button>}
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
            </section>
          ))}
      {edit && <EditRule rule={edit} onClose={() => setEdit(null)} onReset={() => { setReset(edit); setEdit(null); }} />}
      {reset && <ResetRule rule={reset} onClose={() => setReset(null)} />}
      {hist && <HistoryDrawer rule={hist} onClose={() => setHist(null)} />}
    </div>
  );
}

function EditRule({ rule, onClose, onReset }: { rule: Rule; onClose: () => void; onReset: () => void }) {
  const { t, num } = useI18n();
  const rt = useRuleText();
  const unit = useUnit();
  const [value, setValue] = useState(String(rule.value));
  const [reason, setReason] = useState('');
  const { busy, run } = useAction();
  const v = Number(value);
  const bad = value.trim() === '' || Number.isNaN(v) || v < rule.min_value || v > rule.max_value;
  const badReason = reason.trim().length < 3 || reason.trim().length > 500;
  const save = () => run(async () => {
    const r = await api.put<{ changed: boolean }>('/dashboard/rules/' + rule.code, { value: v, reason: reason.trim() });
    invalidate('rules');
    onClose();
    return r;
  }, t('common.saved'));
  return (
    <Modal title={rt.nameText(rule)} onClose={onClose} foot={<>
      <button className="btn" onClick={onReset} disabled={busy || rule.value === rule.default_value}><RotateCcw />{t('rules.reset')}</button>
      <button className="btn" onClick={onClose}>{t('common.cancel')}</button>
      <button className="btn primary" disabled={busy || bad || badReason} onClick={save}>{t('common.save')}</button>
    </>}>
      <p className="muted" style={{ marginTop: 0 }}>{rt.meaning(rule)}</p>
      <div className="form-grid">
        <Field label={t('rules.new_value') + (rule.unit ? ' (' + unit(rule.unit) + ')' : '')} hint={t('rules.range_v', { a: ltr(num(rule.min_value)), b: ltr(num(rule.max_value)), d: ltr(num(rule.default_value)) })} error={value && bad ? 'err.bad_range' : undefined}>
          <input type="number" dir="ltr" value={value} min={rule.min_value} max={rule.max_value} step="any" onChange={e => setValue(e.target.value)} />
        </Field>
        <Field label={t('rules.now')}><input type="text" dir="ltr" readOnly value={num(rule.value) + ' ' + unit(rule.unit)} /></Field>
        <Field label={t('rules.reason')} hint={t('rules.reason_hint')} full error={reason && badReason ? 'err.bad_reason' : undefined}>
          <textarea rows={3} value={reason} onChange={e => setReason(e.target.value)} placeholder={t('rules.reason_ph')} maxLength={500} />
        </Field>
      </div>
      <div style={{ marginTop: 10 }}><Note tone="warn">{t('rules.honest_short')}</Note></div>
    </Modal>
  );
}

function ResetRule({ rule, onClose }: { rule: Rule; onClose: () => void }) {
  const { t, num } = useI18n();
  const rt = useRuleText();
  const unit = useUnit();
  const [reason, setReason] = useState('');
  const { busy, run } = useAction();
  const ok = reason.trim().length >= 3 && reason.trim().length <= 500;
  const go = () => run(async () => { await api.post('/dashboard/rules/' + rule.code + '/reset', { reason: reason.trim() }); invalidate('rules'); onClose(); }, t('common.saved'));
  return (
    <Modal title={t('rules.reset_title', { name: rt.nameText(rule) })} onClose={onClose} foot={<>
      <button className="btn" onClick={onClose}>{t('common.cancel')}</button>
      <button className="btn primary" disabled={busy || !ok} onClick={go}><RotateCcw />{t('rules.reset')}</button>
    </>}>
      <p className="muted" style={{ marginTop: 0 }}>{t('rules.reset_text', { a: ltr(num(rule.value) + ' ' + unit(rule.unit)), d: ltr(num(rule.default_value) + ' ' + unit(rule.unit)) })}</p>
      <Field label={t('rules.reason')} full error={reason && !ok ? 'err.bad_reason' : undefined}>
        <textarea rows={3} value={reason} onChange={e => setReason(e.target.value)} maxLength={500} />
      </Field>
    </Modal>
  );
}

function HistoryDrawer({ rule, onClose }: { rule: Rule; onClose: () => void }) {
  const { t, num, date } = useI18n();
  const rt = useRuleText();
  const unit = useUnit();
  const [page, setPage] = useState(1);
  const q = useApi<{ changes: Change[]; count: number }>('/dashboard/rules/' + rule.code + '/history' + qs({ page, rows_per_page: 20 }), ['rules'], { auth: true });
  const rows = q.data?.changes ?? [];
  return (
    <Drawer eyebrow={t('rules.history')} title={rt.nameText(rule)} onClose={onClose}>
      {q.error?.status === 403 && !q.data ? <StateBox kind="locked" /> : q.loading ? <div className="sk-rows">{Array.from({ length: 5 }, (_, i) => <i key={i} className="sk" />)}</div>
        : !rows.length ? <StateBox kind="empty" title={t('rules.no_history')} text={t('rules.no_history_text')} />
          : <ul className="rule-history">{rows.map(c => (
            <li key={c.id}>
              <div className="spread"><b className="tabular" dir="ltr">{num(c.old_value)} → {num(c.new_value)} {unit(rule.unit)}</b><span className="small muted">{date(c.at, 'datetime')}</span></div>
              <div className="small" dir="auto">{c.reason}</div>
              <div className="small muted">{t('rules.by', { id: c.staff_id })}</div>
            </li>))}</ul>}
      {(q.data?.count ?? 0) > 20 && (
        <div className="pager">
          <button className="btn sm" disabled={page <= 1} onClick={() => setPage(p => p - 1)}>{t('table.prev')}</button>
          <button className="btn sm" disabled={page * 20 >= (q.data?.count ?? 0)} onClick={() => setPage(p => p + 1)}>{t('table.next')}</button>
        </div>
      )}
    </Drawer>
  );
}
