// App control (design 15): what the farmer app reads at start. GET/PUT /dashboard/app/config (a full
// replace) and GET /dashboard/app/versions (FRONTEND.md 9 App control, section 4).
//
// Each card saves on its own. Before saving it reads the config again: if someone else saved since this
// page loaded, the form reloads and asks to look again (nobody's change is overwritten). The PUT then
// sends the fresh server copy with only this card's fields replaced.
//
// Each card has its own busy flag, but saves from different cards run one after the other (a shared
// queue), so two cards saved at once never send two full copies built from the same old config. After a
// save only that card's fields take the server's answer; other cards keep their unsaved edits.
import { useEffect, useRef, useState, type ReactNode } from 'react';
import { RotateCcw, Save, Smartphone } from 'lucide-react';
import { useI18n } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { api, ApiError } from '../../api/client';
import { useApi, invalidate } from '../../api/cache';
import { PageHead, Note, Switch, Pill, useToast } from '../../components/ui';
import { StateBox, useErrorText } from '../../components/domain';
import { useAction, cmpVersion, isVersion } from './inbox/useAction';
import './appctl.css';

type Features = Record<'add_farm' | 'walk_mode' | 'satellite' | 'doctor' | 'reports' | 'alwa' | 'plan' | 'push', boolean>;
interface Limits { farms_per_phone: number; max_farm_dunam: number; min_corners: number; max_corners: number; gps_meters: number }
interface Config {
  latest_version: string; min_version: string; update_message_ku?: string | null; update_message_en?: string | null;
  maintenance: boolean; maintenance_message_ku?: string | null; maintenance_message_en?: string | null; maintenance_from?: string | null; maintenance_until?: string | null;
  announcement_on: boolean; announcement_ku?: string | null; announcement_en?: string | null;
  features: Features; limits: Limits; help_phone?: string | null; public_farm_totals: boolean;
  updated_by?: string | null; updated_at?: string;
}
const FEATURES: (keyof Features)[] = ['add_farm', 'walk_mode', 'satellite', 'doctor', 'reports', 'alwa', 'plan', 'push'];
const LIMITS: (keyof Limits)[] = ['farms_per_phone', 'max_farm_dunam', 'min_corners', 'max_corners', 'gps_meters'];

/** datetime-local <-> UTC ISO */
const toLocal = (iso?: string | null) => { if (!iso) return ''; const d = new Date(iso); return new Date(d.getTime() - d.getTimezoneOffset() * 60000).toISOString().slice(0, 16); };
const toIso = (v: string) => (v ? new Date(v).toISOString() : null);
const strip = (c: Config) => { const { updated_at: _a, updated_by: _b, ...rest } = c; return rest; };
const same = (a: unknown, b: unknown) => JSON.stringify(a) === JSON.stringify(b);
/**
 * Take a new server copy into the form. Fields listed in `take` always take the server value; every
 * other field keeps the user's edit if it differs from the old server copy (unsaved), else updates.
 */
function merge(form: Config | null, oldBase: Config | null, next: Config, take: (keyof Config)[] = []): Config {
  if (!form || !oldBase) return next;
  const out = { ...next } as Record<string, unknown>;
  for (const k of Object.keys(form) as (keyof Config)[]) {
    if (k === 'updated_at' || k === 'updated_by' || take.includes(k)) continue;
    if (!same(form[k], oldBase[k])) out[k] = form[k];
  }
  return out as unknown as Config;
}
// saves of all cards run one after the other
let saveChain: Promise<unknown> = Promise.resolve();

export default function AppControl() {
  const { t, num, date } = useI18n();
  const { can } = useAuth();
  const cfg = useApi<Config>('/dashboard/app/config', ['app_config'], { auth: true });
  const vers = useApi<{ versions: { version: string; farmers: number; share: number }[] }>('/dashboard/app/versions', [], { auth: true });
  const [form, setForm] = useState<Config | null>(null);
  // the server copy the form was built from (a ref, so a queued save sees the newest one)
  const base = useRef<Config | null>(null);
  const canEdit = can('app', 'update');
  const adopt = (next: Config, take: (keyof Config)[] = []) => {
    setForm(f => merge(f, base.current, next, take));
    base.current = next;
  };

  // take the server copy into the form when it first arrives or when someone else changed it
  useEffect(() => { if (cfg.data && cfg.data.updated_at !== base.current?.updated_at) adopt(cfg.data); }, [cfg.data]); // eslint-disable-line react-hooks/exhaustive-deps

  if (cfg.error?.status === 403 && !cfg.data) return <div><PageHead eyebrow={t('nav.g_app')} title={t('nav.app')} /><div className="card"><StateBox kind="locked" /></div></div>;
  if (cfg.error && !cfg.data) return <div><PageHead eyebrow={t('nav.g_app')} title={t('nav.app')} /><div className="card"><StateBox kind="error" action={<button className="btn sm" onClick={cfg.reload}><RotateCcw />{t('common.retry')}</button>} /></div></div>;
  if (!form) return <div><PageHead eyebrow={t('nav.g_app')} title={t('nav.app')} sub={t('appctl.sub')} /><div className="grid g2">{[0, 1, 2, 3].map(i => <div key={i} className="card"><div className="sk-rows">{Array.from({ length: 5 }, (_, j) => <i key={j} className="sk" />)}</div></div>)}</div></div>;

  const set = <K extends keyof Config>(k: K, v: Config[K]) => setForm(f => (f ? { ...f, [k]: v } : f));
  const versions = vers.data?.versions ?? [];
  const p = { form, set, canEdit, base, onSaved: adopt, onStale: adopt };

  return (
    <div className="appctl-page">
      <PageHead eyebrow={t('nav.g_app')} title={t('nav.app')} sub={t('appctl.sub')} />
      <Note tone="info" icon={<Smartphone />}>{t('appctl.reads_note')}{form.updated_at && <> · {t('appctl.last_saved', { when: date(form.updated_at, 'datetime') })}</>}</Note>
      <div className="appctl-grid">
        <div className="stack">
          <CardSave title={t('appctl.versions')} fields={['latest_version', 'min_version', 'update_message_ku', 'update_message_en']} {...p}
            check={f => !isVersion(f.latest_version) || !isVersion(f.min_version) ? t('appctl.v_format') : cmpVersion(f.min_version, f.latest_version) > 0 ? t('appctl.v_order') : null}>
            {versions.length ? (
              <div className="ver-usage">
                {versions.map(v => (
                  <div key={v.version} className="ver-row">
                    <b className="ltr tabular">{v.version}</b>
                    <div className="bar"><i style={{ width: Math.round(v.share * 100) + '%', background: cmpVersion(v.version, form.min_version) < 0 ? 'var(--danger)' : v.version === form.latest_version ? 'var(--good)' : 'var(--ink-3)' }} /></div>
                    <span className="small tabular">{num(Math.round(v.share * 100))}% · {t('appctl.farmers_n', { n: num(v.farmers) })}</span>
                    {cmpVersion(v.version, form.min_version) < 0 ? <Pill tone="danger">{t('appctl.must_update')}</Pill> : v.version === form.latest_version ? <Pill tone="good">{t('appctl.newest')}</Pill> : <Pill>{t('appctl.supported')}</Pill>}
                  </div>
                ))}
              </div>
            ) : <p className="muted small">{t('appctl.no_versions')}</p>}
            <div className="form-grid">
              <label className="field"><span>{t('appctl.latest')}</span><input type="text" dir="ltr" value={form.latest_version} disabled={!canEdit} onChange={e => set('latest_version', e.target.value)} /></label>
              <label className="field"><span>{t('appctl.min')} <small>{t('appctl.min_hint')}</small></span><input type="text" dir="ltr" value={form.min_version} disabled={!canEdit} onChange={e => set('min_version', e.target.value)} /></label>
              <label className="field full"><span>{t('appctl.update_ku')}</span><textarea dir="rtl" rows={2} value={form.update_message_ku ?? ''} disabled={!canEdit} onChange={e => set('update_message_ku', e.target.value || null)} /></label>
              <label className="field full"><span>{t('appctl.update_en')}</span><textarea dir="ltr" rows={2} value={form.update_message_en ?? ''} disabled={!canEdit} onChange={e => set('update_message_en', e.target.value || null)} /></label>
            </div>
          </CardSave>
          <CardSave title={t('appctl.maintenance')} fields={['maintenance', 'maintenance_message_ku', 'maintenance_message_en', 'maintenance_from', 'maintenance_until']} {...p}
            check={f => f.maintenance && !f.maintenance_message_ku?.trim() ? t('err.missing_text') : null}>
            <Row title={t('appctl.maint_on')} sub={t('appctl.maint_on_sub')}><Switch on={form.maintenance} disabled={!canEdit} onChange={v => set('maintenance', v)} label={t('appctl.maint_on')} /></Row>
            <div className="form-grid">
              <label className="field full"><span>{t('appctl.msg_ku')} {form.maintenance && <small>{t('common.required')}</small>}</span><textarea dir="rtl" rows={2} value={form.maintenance_message_ku ?? ''} disabled={!canEdit} onChange={e => set('maintenance_message_ku', e.target.value || null)} /></label>
              <label className="field full"><span>{t('appctl.msg_en')}</span><textarea dir="ltr" rows={2} value={form.maintenance_message_en ?? ''} disabled={!canEdit} onChange={e => set('maintenance_message_en', e.target.value || null)} /></label>
              <label className="field"><span>{t('appctl.from')}</span><input type="datetime-local" value={toLocal(form.maintenance_from)} disabled={!canEdit} onChange={e => set('maintenance_from', toIso(e.target.value))} /></label>
              <label className="field"><span>{t('appctl.until')}</span><input type="datetime-local" value={toLocal(form.maintenance_until)} disabled={!canEdit} onChange={e => set('maintenance_until', toIso(e.target.value))} /></label>
            </div>
          </CardSave>
        </div>
        <div className="stack">
          <CardSave title={t('appctl.features')} fields={['features']} {...p}>
            {FEATURES.map(k => (
              <Row key={k} title={t('appctl.f_' + k)} sub={t('appctl.f_' + k + '_sub')}>
                <Switch on={form.features[k]} disabled={!canEdit} onChange={v => set('features', { ...form.features, [k]: v })} label={t('appctl.f_' + k)} />
              </Row>
            ))}
          </CardSave>
          <CardSave title={t('appctl.announcement')} fields={['announcement_on', 'announcement_ku', 'announcement_en']} {...p}
            check={f => f.announcement_on && !f.announcement_ku?.trim() ? t('err.missing_text') : null}>
            <Row title={t('appctl.ann_on')} sub={t('appctl.ann_on_sub')}><Switch on={form.announcement_on} disabled={!canEdit} onChange={v => set('announcement_on', v)} label={t('appctl.ann_on')} /></Row>
            <div className="form-grid">
              <label className="field full"><span>{t('appctl.ann_ku')} {form.announcement_on && <small>{t('common.required')}</small>}</span><textarea dir="rtl" rows={2} value={form.announcement_ku ?? ''} disabled={!canEdit} onChange={e => set('announcement_ku', e.target.value || null)} /></label>
              <label className="field full"><span>{t('appctl.ann_en')}</span><textarea dir="ltr" rows={2} value={form.announcement_en ?? ''} disabled={!canEdit} onChange={e => set('announcement_en', e.target.value || null)} /></label>
            </div>
          </CardSave>
          <CardSave title={t('appctl.limits')} fields={['limits', 'help_phone', 'public_farm_totals']} {...p}
            check={f => LIMITS.some(k => !(f.limits[k] > 0)) ? t('appctl.l_positive') : f.limits.min_corners > f.limits.max_corners ? t('appctl.l_corners') : null}>
            {LIMITS.map(k => (
              <Row key={k} title={t('appctl.l_' + k)}>
                <input type="number" dir="ltr" className="num-input" min={1} value={form.limits[k]} disabled={!canEdit} onChange={e => set('limits', { ...form.limits, [k]: Number(e.target.value) })} />
              </Row>
            ))}
            <Row title={t('appctl.help_phone')} sub={t('appctl.help_phone_sub')}><input type="tel" className="phone-input" value={form.help_phone ?? ''} disabled={!canEdit} onChange={e => set('help_phone', e.target.value || null)} placeholder="+964 750 000 0000" /></Row>
            <Row title={t('appctl.public_totals')} sub={t('appctl.public_totals_sub')}><Switch on={form.public_farm_totals} disabled={!canEdit} onChange={v => set('public_farm_totals', v)} label={t('appctl.public_totals')} /></Row>
          </CardSave>
        </div>
      </div>
    </div>
  );
}

function Row({ title, sub, children }: { title: string; sub?: string; children: ReactNode }) {
  return <div className="set"><div className="txt"><b>{title}</b>{sub && <small>{sub}</small>}</div><div className="ctl">{children}</div></div>;
}

function CardSave({ title, fields, form, canEdit, base, onSaved, onStale, check, children }: {
  title: string; fields: (keyof Config)[]; form: Config; canEdit: boolean; base: { current: Config | null };
  onSaved: (c: Config, take: (keyof Config)[]) => void; onStale: (c: Config, take: (keyof Config)[]) => void; check?: (f: Config) => string | null; set?: unknown; children: ReactNode;
}) {
  const { t } = useI18n();
  const toast = useToast();
  const errText = useErrorText();
  const { busy, run } = useAction();
  const [err, setErr] = useState<string | null>(null);
  const save = () => run(() => {
    // the values to send are taken now; the request waits for saves of other cards to finish
    const mine = Object.fromEntries(fields.map(f => [f, form[f]])) as Partial<Config>;
    const job = saveChain.catch(() => undefined).then(() => doSave(mine));
    saveChain = job;
    return job;
  });
  const doSave = async (mine: Partial<Config>) => {
    setErr(null);
    const problem = check?.({ ...form, ...mine });
    if (problem) { setErr(problem); return; }
    const fresh = await api.get<Config>('/dashboard/app/config');
    if (fresh.updated_at !== base.current?.updated_at) { onStale(fresh, fields); invalidate('app_config'); toast(t('appctl.stale'), 'warn'); return; }
    const body = strip(fresh) as Config;
    for (const f of fields) (body as unknown as Record<string, unknown>)[f] = mine[f];
    try {
      const saved = await api.put<Config>('/dashboard/app/config', body);
      invalidate('app_config');
      onSaved(saved, fields);
      toast(t('common.saved'), 'good');
    } catch (e) {
      if (e instanceof ApiError) setErr(errText(e) + (e.field ? ' (' + e.field + ')' : ''));
      else throw e;
    }
  };
  return (
    <section className="card">
      <div className="card-head"><span className="eyebrow">{title}</span>{canEdit && <button className="btn sm primary" disabled={busy} onClick={save}><Save />{t('common.save')}</button>}</div>
      {children}
      {err && <div style={{ marginTop: 10 }}><Note tone="danger">{err}</Note></div>}
    </section>
  );
}
