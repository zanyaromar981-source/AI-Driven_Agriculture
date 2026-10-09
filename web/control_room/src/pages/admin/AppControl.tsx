// App control: versions, forced update, maintenance, an in-app announcement, feature switches and limits.
// The app reads these from the backend once it is connected; until then they are stored here only.
import { useState, type ReactNode } from 'react';
import { Info, Save, Smartphone, Megaphone, Wrench, Gauge, ToggleLeft } from 'lucide-react';
import { db } from '../../data/db';
import { useDoc } from '../../data/store';
import type { AppConfig, Bi } from '../../data/types';
import { useI18n } from '../../i18n';
import { Card, Field, Note, PageHead, Pill, SetRow, Switch, useToast } from '../../components/ui';

const FEATURES = ['add_farm', 'walk_mode', 'satellite', 'doctor', 'reports', 'alwa', 'plan', 'push'];

export default function AppControl() {
  const { t, num, date } = useI18n();
  const app = useDoc(db.app);
  const toast = useToast();
  const set = (p: Partial<AppConfig>, msg = t('common.saved')) => { db.app.set(p); toast(msg, 'good'); };
  const versions = app.versions.map(v => v.version);

  return (
    <>
      <PageHead eyebrow={t('nav.g_app')} title={t('nav.app')} sub={t('appctl.sub')} />
      <div className="mb"><Note tone="info" icon={<Info />}>{t('appctl.note')}</Note></div>
      <div className="grid g2">
        <Card extra={<Head icon={<Smartphone />} text={t('appctl.versions')} />}>
          <div className="table-wrap">
            <table className="t cards">
              <thead><tr><th>{t('appctl.version')}</th><th>{t('appctl.phones')}</th><th>{t('appctl.released')}</th><th /></tr></thead>
              <tbody>
                {app.versions.map(v => {
                  const old = cmp(v.version, app.minVersion) < 0;
                  return (
                    <tr key={v.version}>
                      <td data-label={t('appctl.version')}><b className="ltr">{v.version}</b></td>
                      <td data-label={t('appctl.phones')}><div className="row" style={{ flexWrap: 'nowrap' }}><div className="bar" style={{ width: 110 }}><i style={{ width: v.share + '%', background: old ? 'var(--danger)' : 'var(--good)' }} /></div><span className="tabular">{num(v.share)}%</span></div></td>
                      <td data-label={t('appctl.released')} className="nowrap">{date(v.released, 'short')}</td>
                      <td data-label="">{v.version === app.latestVersion ? <Pill tone="good">{t('appctl.latest')}</Pill> : old ? <Pill tone="danger">{t('appctl.must_update')}</Pill> : <Pill>{t('appctl.supported')}</Pill>}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
          <SetRow title={t('appctl.latest_v')} sub={t('appctl.latest_v_sub')}>
            <select value={app.latestVersion} onChange={e => set({ latestVersion: e.target.value })} className="ltr-input">{versions.map(v => <option key={v}>{v}</option>)}</select>
          </SetRow>
          <SetRow title={t('appctl.min_v')} sub={t('appctl.min_v_sub')}>
            <select value={app.minVersion} onChange={e => set({ minVersion: e.target.value })} className="ltr-input">{versions.map(v => <option key={v}>{v}</option>)}</select>
          </SetRow>
          <BiEditor label={t('appctl.update_msg')} value={app.updateMessage} onSave={v => set({ updateMessage: v })} />
        </Card>

        <Card extra={<Head icon={<ToggleLeft />} text={t('appctl.features')} />}>
          <p className="muted small" style={{ marginTop: 0 }}>{t('appctl.features_sub')}</p>
          {FEATURES.map(k => (
            <SetRow key={k} title={t('appctl.f_' + k)} sub={t('appctl.f_' + k + '_sub')}>
              <Switch on={!!app.features[k]} label={t('appctl.f_' + k)}
                onChange={on => set({ features: { ...app.features, [k]: on } }, t(on ? 'appctl.turned_on' : 'appctl.turned_off', { f: t('appctl.f_' + k) }))} />
            </SetRow>
          ))}
        </Card>

        <Card extra={<Head icon={<Wrench />} text={t('appctl.maintenance')} />}>
          <SetRow title={t('appctl.maint_on')} sub={t('appctl.maint_on_sub')}>
            <Switch on={app.maintenance} label={t('appctl.maint_on')} onChange={on => set({ maintenance: on }, t(on ? 'appctl.maint_started' : 'appctl.maint_stopped'))} />
          </SetRow>
          {app.maintenance && <div className="mb"><Note tone="danger" icon={<Wrench />}>{t('appctl.maint_live')}</Note></div>}
          <div className="form-grid" style={{ margin: '8px 0' }}>
            <Field label={t('appctl.from')}><input type="time" value={app.maintenanceFrom} onChange={e => set({ maintenanceFrom: e.target.value })} className="ltr-input" /></Field>
            <Field label={t('appctl.until')}><input type="time" value={app.maintenanceUntil} onChange={e => set({ maintenanceUntil: e.target.value })} className="ltr-input" /></Field>
          </div>
          <BiEditor label={t('appctl.maint_msg')} value={app.maintenanceMessage} onSave={v => set({ maintenanceMessage: v })} />
        </Card>

        <Card extra={<Head icon={<Megaphone />} text={t('appctl.announce')} />}>
          <SetRow title={t('appctl.announce_on')} sub={t('appctl.announce_on_sub')}>
            <Switch on={app.announcementOn} label={t('appctl.announce_on')} onChange={on => set({ announcementOn: on })} />
          </SetRow>
          <BiEditor label={t('appctl.announce_text')} value={app.announcement} onSave={v => set({ announcement: v })} />
          <div className="eyebrow mt">{t('appctl.preview')}</div>
          <div className="phone-prev">
            <div className="phone-bar">Jutyar</div>
            {app.announcementOn && (app.announcement.ku || app.announcement.en)
              ? <div className="phone-banner"><Megaphone size={14} /><bdi dir="auto">{app.announcement.ku || app.announcement.en}</bdi></div>
              : <div className="muted small center" style={{ padding: 12 }}>{t('appctl.no_announce')}</div>}
            <div className="phone-body"><i /><i /><i /></div>
          </div>
        </Card>

        <Card extra={<Head icon={<Gauge />} text={t('appctl.limits')} />} style={{ gridColumn: '1 / -1' }}>
          <Limits app={app} onSave={(limits, helpPhone) => set({ limits, helpPhone })} />
        </Card>
      </div>
      <style>{CSS}</style>
    </>
  );
}

const CSS = `
.phone-prev { width: 240px; max-width: 100%; margin: 10px auto 0; border: 6px solid var(--ink); border-radius: 26px; overflow: hidden; background: var(--bg); }
.phone-bar { background: var(--deep); color: #fff; font-weight: 800; padding: 10px 14px; }
.phone-banner { display: flex; gap: 8px; align-items: flex-start; background: var(--gold-soft); color: var(--ray); padding: 10px 12px; font-size: 13px; font-weight: 600; }
.phone-banner bdi { flex: 1; }
.phone-body { padding: 12px; display: grid; gap: 8px; }
.phone-body i { display: block; height: 34px; border-radius: 10px; background: var(--surface); border: 1px solid var(--line-soft); }
`;

function Head({ icon, text }: { icon: ReactNode; text: string }) {
  return <span className="eyebrow row" style={{ flex: 1, gap: 6 }}>{icon}{text}</span>;
}

/** 1.0.2 < 1.0.10: compare dotted versions number by number. */
function cmp(a: string, b: string) {
  const x = a.split('.').map(Number), y = b.split('.').map(Number);
  for (let i = 0; i < Math.max(x.length, y.length); i++) { const d = (x[i] ?? 0) - (y[i] ?? 0); if (d) return d; }
  return 0;
}

/** A Kurdish and English text pair with one Save button. */
function BiEditor({ label, value, onSave }: { label: string; value: Bi; onSave: (v: Bi) => void }) {
  const { t } = useI18n();
  const [d, setD] = useState(value);
  const dirty = d.ku !== value.ku || d.en !== value.en;
  return (
    <div style={{ marginTop: 10 }}>
      <div className="form-grid">
        <Field label={label + ' · ' + t('common.kurdish')}><textarea dir="rtl" className="ku-text" value={d.ku} onChange={e => setD({ ...d, ku: e.target.value })} /></Field>
        <Field label={label + ' · ' + t('common.english')}><textarea dir="ltr" value={d.en} onChange={e => setD({ ...d, en: e.target.value })} /></Field>
      </div>
      <div className="row" style={{ justifyContent: 'flex-end', marginTop: 8 }}>
        {!d.ku.trim() && <span className="muted small">{t('common.ku_missing')}</span>}
        <button className="btn sm primary" disabled={!dirty} onClick={() => onSave({ ku: d.ku.trim(), en: d.en.trim() })}><Save />{t('common.save')}</button>
      </div>
    </div>
  );
}

function Limits({ app, onSave }: { app: AppConfig; onSave: (l: AppConfig['limits'], phone: string) => void }) {
  const { t } = useI18n();
  const [l, setL] = useState(app.limits);
  const [phone, setPhone] = useState(app.helpPhone);
  const [err, setErr] = useState('');
  const n = (k: keyof AppConfig['limits']) => (
    <input type="number" min={1} value={l[k]} onChange={e => setL({ ...l, [k]: Number(e.target.value) })} className="ltr-input" />
  );
  const save = () => {
    if (Object.values(l).some(v => !(v > 0))) return setErr('appctl.limit_positive');
    if (l.minCorners < 3 || l.minCorners >= l.maxCorners) return setErr('appctl.corners_bad');
    setErr(''); onSave(l, phone.trim());
  };
  return (
    <>
      <div className="grid g3">
        <Field label={t('appctl.l_farms')} hint={t('appctl.l_farms_sub')}>{n('farmsPerPhone')}</Field>
        <Field label={t('appctl.l_dunam')} hint={t('appctl.l_dunam_sub')}>{n('maxFarmDunam')}</Field>
        <Field label={t('appctl.l_gps')} hint={t('appctl.l_gps_sub')}>{n('gpsMeters')}</Field>
        <Field label={t('appctl.l_min_corners')}>{n('minCorners')}</Field>
        <Field label={t('appctl.l_max_corners')}>{n('maxCorners')}</Field>
        <Field label={t('appctl.help_phone')} hint={t('appctl.help_phone_sub')}><input type="tel" value={phone} onChange={e => setPhone(e.target.value)} /></Field>
      </div>
      {err && <div className="mt"><Note tone="danger">{t(err)}</Note></div>}
      <div className="row" style={{ justifyContent: 'flex-end', marginTop: 12 }}>
        <button className="btn primary" onClick={save}><Save />{t('common.save')}</button>
      </div>
    </>
  );
}
