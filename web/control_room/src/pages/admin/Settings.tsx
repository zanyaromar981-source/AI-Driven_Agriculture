// Settings: the organisation, languages, the support letter, what the public View page shows,
// my own account, and resetting the sample data.
import { useState } from 'react';
import { Building2, Languages, FileSignature, Globe, UserRound, AlertTriangle, Save, RotateCcw } from 'lucide-react';
import { db } from '../../data/db';
import { clearAllData, useDoc } from '../../data/store';
import { checkOfficer, saveOfficer, type Problems } from '../../data/api';
import type { Bi, Lang, Settings as S } from '../../data/types';
import { useI18n } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { Card, Confirm, Field, Note, PageHead, Select, SetRow, Switch, useToast } from '../../components/ui';

type PV = keyof S['publicView'];
const PUBLIC: PV[] = ['map', 'water', 'fires', 'compare', 'market', 'farmTotals'];

export default function Settings() {
  const { t, num } = useI18n();
  const s = useDoc(db.settings);
  const toast = useToast();
  const [reset, setReset] = useState(false);
  const set = (p: Partial<S>) => { db.settings.set(p); toast(t('common.saved'), 'good'); };

  return (
    <>
      <PageHead eyebrow={t('nav.g_system')} title={t('nav.settings')} sub={t('settings.sub')} />
      <div className="grid g2">
        <Card extra={<H icon={<Building2 size={16} />} text={t('settings.org')} />}>
          <BiForm labels={[t('settings.org_name')]} values={[s.orgName]} onSave={([v]) => set({ orgName: v })} />
          <SinglePhone value={s.helpPhone} onSave={v => set({ helpPhone: v })} />
        </Card>

        <Card extra={<H icon={<Languages size={16} />} text={t('settings.languages')} />}>
          <SetRow title={t('settings.default_lang')} sub={t('settings.default_lang_sub')}>
            <Select value={s.defaultLang} onChange={v => set({ defaultLang: v as Lang })} options={[['ku', 'کوردی'], ['en', 'English']]} />
          </SetRow>
          <SetRow title={t('settings.digits')} sub={t('settings.digits_sub')}>
            <span className="small muted ltr">{s.kurdishDigits ? '١٢٣' : '123'}</span>
            <Switch on={s.kurdishDigits} onChange={v => set({ kurdishDigits: v })} label={t('settings.digits')} />
          </SetRow>
          <SetRow title={t('settings.sample')} sub={t('settings.sample_sub')}>
            <Switch on={s.sampleBanner} onChange={v => set({ sampleBanner: v })} label={t('settings.sample')} />
          </SetRow>
          <p className="muted small">{t('settings.example', { n: num(1234567.5, 1) })}</p>
        </Card>

        <Card extra={<H icon={<FileSignature size={16} />} text={t('settings.letter')} />}>
          <p className="muted small" style={{ marginTop: 0 }}>{t('settings.letter_sub')}</p>
          <BiForm labels={[t('settings.signer'), t('settings.signer_title')]} values={[s.letterSigner, s.letterSignerTitle]}
            onSave={([a, b]) => set({ letterSigner: a, letterSignerTitle: b })} />
          <Prefix value={s.letterPrefix} onSave={v => set({ letterPrefix: v })} />
        </Card>

        <Card extra={<H icon={<Globe size={16} />} text={t('settings.public')} />}>
          <p className="muted small" style={{ marginTop: 0 }}>{t('settings.public_sub')}</p>
          {PUBLIC.map(k => (
            <SetRow key={k} title={t('settings.pv_' + k)} sub={t('settings.pv_' + k + '_sub')}>
              <Switch on={s.publicView[k]} label={t('settings.pv_' + k)} onChange={v => set({ publicView: { ...s.publicView, [k]: v } })} />
            </SetRow>
          ))}
        </Card>

        <MyAccount />

        <Card extra={<H icon={<AlertTriangle size={16} />} text={t('settings.danger')} />} style={{ borderColor: '#E4B5AF' }}>
          <p className="ink2" style={{ marginTop: 0 }}>{t('settings.reset_sub')}</p>
          <button className="btn danger" onClick={() => setReset(true)}><RotateCcw />{t('settings.reset')}</button>
        </Card>
      </div>
      {reset && <Confirm title={t('settings.reset')} text={t('settings.reset_confirm')} okLabel={t('settings.reset')}
        onOk={() => { clearAllData(); location.reload(); }} onClose={() => setReset(false)} />}
    </>
  );
}

function H({ icon, text }: { icon: React.ReactNode; text: string }) {
  return <span className="eyebrow row" style={{ flex: 1, gap: 6 }}>{icon}{text}</span>;
}

/** One or more Kurdish + English pairs with a single Save. */
function BiForm({ labels, values, onSave }: { labels: string[]; values: Bi[]; onSave: (v: Bi[]) => void }) {
  const { t } = useI18n();
  const [d, setD] = useState(values);
  const dirty = d.some((v, i) => v.ku !== values[i].ku || v.en !== values[i].en);
  const upd = (i: number, k: keyof Bi, v: string) => setD(d.map((x, j) => (j === i ? { ...x, [k]: v } : x)));
  return (
    <div>
      <div className="form-grid">
        {labels.map((l, i) => [
          <Field key={i + 'k'} label={l + ' · ' + t('common.kurdish')}><input type="text" dir="rtl" className="ku-text" value={d[i].ku} onChange={e => upd(i, 'ku', e.target.value)} /></Field>,
          <Field key={i + 'e'} label={l + ' · ' + t('common.english')}><input type="text" dir="ltr" value={d[i].en} onChange={e => upd(i, 'en', e.target.value)} /></Field>,
        ])}
      </div>
      <div className="row" style={{ justifyContent: 'flex-end', marginTop: 8 }}>
        <button className="btn sm primary" disabled={!dirty} onClick={() => onSave(d.map(x => ({ ku: x.ku.trim(), en: x.en.trim() })))}><Save />{t('common.save')}</button>
      </div>
    </div>
  );
}

function SinglePhone({ value, onSave }: { value: string; onSave: (v: string) => void }) {
  const { t } = useI18n();
  const [v, setV] = useState(value);
  return (
    <SetRow title={t('settings.help')} sub={t('settings.help_sub')}>
      <input type="tel" value={v} onChange={e => setV(e.target.value)} style={{ width: 170 }} />
      <button className="btn sm" disabled={v.trim() === value} onClick={() => onSave(v.trim())}><Save /></button>
    </SetRow>
  );
}

function Prefix({ value, onSave }: { value: string; onSave: (v: string) => void }) {
  const { t } = useI18n();
  const [v, setV] = useState(value);
  const ok = /^[A-Z0-9]{2,6}$/.test(v);
  return (
    <SetRow title={t('settings.prefix')} sub={t('settings.prefix_sub', { ex: (ok ? v : value) + '-202610-1007' })}>
      <input type="text" className="ltr-input" value={v} maxLength={6} onChange={e => setV(e.target.value.toUpperCase())} style={{ width: 90, borderColor: ok ? undefined : 'var(--danger)' }} />
      <button className="btn sm" disabled={!ok || v === value} onClick={() => onSave(v)}><Save /></button>
    </SetRow>
  );
}

function MyAccount() {
  const { t } = useI18n();
  const { me } = useAuth();
  const toast = useToast();
  const [o, setO] = useState(me!);
  const [err, setErr] = useState<Problems>({});
  const save = () => {
    const p = checkOfficer(o); setErr(p);
    if (Object.keys(p).length) return;
    saveOfficer(o); toast(t('common.saved'), 'good');
  };
  return (
    <Card extra={<H icon={<UserRound size={16} />} text={t('settings.me')} />}>
      <div className="form-grid">
        <Field label={t('officers.name')} error={err.name}><input type="text" value={o.name} onChange={e => setO({ ...o, name: e.target.value })} /></Field>
        <Field label={t('officers.job')}><input type="text" value={o.jobTitle} onChange={e => setO({ ...o, jobTitle: e.target.value })} /></Field>
        <Field label={t('officers.phone')} error={err.phone}><input type="tel" value={o.phone} onChange={e => setO({ ...o, phone: e.target.value })} /></Field>
        <Field label={t('officers.email')} hint={t('settings.email_fixed')}><input type="email" value={o.email} disabled /></Field>
      </div>
      <div className="mt"><Note tone="info">{t('settings.password_note')}</Note></div>
      <div className="row" style={{ justifyContent: 'flex-end', marginTop: 10 }}>
        <button className="btn primary sm" onClick={save}><Save />{t('common.save')}</button>
      </div>
    </Card>
  );
}
