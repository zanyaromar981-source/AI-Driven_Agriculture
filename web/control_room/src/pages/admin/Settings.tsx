// Settings (design 20). My account (PUT /dashboard/me), how this browser shows the site (language,
// Kurdish digits, sidebar), where the public page switch lives, and facts about the connection.
// The organisation name and the letter signer of the design are not stored by the server yet.
import { useState } from 'react';
import { Link } from 'react-router-dom';
import { KeyRound, Check, Trash2, ExternalLink, Info } from 'lucide-react';
import { useI18n, type Lang } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { useApi, wipeAll } from '../../api/cache';
import { api, ApiError, API_BASE } from '../../api/client';
import { prefs } from '../../data/store';
import { PageHead, SetRow, Switch, Field, Note, Confirm, Select, Pill, useToast } from '../../components/ui';
import { useErrorText } from '../../components/domain';
import './settings.css';

export default function SettingsPage() {
  const { t, lang, setLang, kuDigits, setKuDigits, date } = useI18n();
  const { me, perms, can } = useAuth();
  const versions = useApi<{ api: string; server_time: string }>('/versions', []);
  const [mini, setMiniState] = useState<boolean>(() => prefs.get('side.mini', false));
  const [wipe, setWipe] = useState(false);
  if (!me) return null;

  const clearSaved = async () => { await wipeAll(); location.reload(); };

  return (
    <>
      <PageHead eyebrow={t('nav.g_system')} title={t('nav.settings')} sub={t('settings.sub')} />
      <div className="settings-grid">
        {/* first column = the inline-start side (right in Kurdish), as in design 20 */}
        <div className="stack">
          <div className="card">
            <div className="eyebrow">{t('settings.view_title')}</div>
            <SetRow title={t('settings.language')} sub={t('settings.language_sub')}>
              <Select value={lang} onChange={v => setLang(v as Lang)} options={[['ku', 'کوردی'], ['en', 'English']]} aria-label={t('settings.language')} />
            </SetRow>
            <SetRow title={t('settings.ku_digits')} sub={t('settings.ku_digits_sub')}><Switch on={kuDigits} onChange={setKuDigits} label={t('settings.ku_digits')} /></SetRow>
            <SetRow title={t('settings.side_mini')} sub={t('settings.side_mini_sub')}>
              <Switch on={mini} onChange={v => { prefs.set('side.mini', v); setMiniState(v); }} label={t('settings.side_mini')} />
            </SetRow>
          </div>
          <div className="card">
            <div className="eyebrow">{t('settings.org_title')}</div>
            <Note tone="info" icon={<Info />}>{t('settings.org_soon')}</Note>
          </div>
          <div className="card">
            <div className="eyebrow">{t('settings.about_title')}</div>
            <dl className="facts">
              <dt>{t('settings.api_address')}</dt><dd className="ltr mono">{API_BASE}</dd>
              <dt>{t('settings.api_version')}</dt><dd className="ltr mono">{versions.data?.api ?? '-'}</dd>
              <dt>{t('settings.server_time')}</dt><dd>{versions.data ? date(versions.data.server_time, 'datetime') : '-'}</dd>
              <dt>{t('settings.my_perms')}</dt><dd>{t('settings.perm_count', { n: perms.size })}</dd>
            </dl>
            <SetRow title={t('settings.clear')} sub={t('settings.clear_sub')}>
              <button className="btn sm danger" onClick={() => setWipe(true)}><Trash2 />{t('settings.clear_btn')}</button>
            </SetRow>
          </div>
        </div>
        <div className="stack">
          <MyAccount />
          <div className="card">
            <div className="eyebrow">{t('settings.public_title')}</div>
            <p className="muted small">{t('settings.public_text')}</p>
            {can('app') ? <Link to="/admin/app" className="btn sm"><ExternalLink />{t('settings.public_go')}</Link> : <Pill>{t('settings.public_no_perm')}</Pill>}
          </div>
        </div>
      </div>
      {wipe && <Confirm title={t('settings.clear')} text={t('settings.clear_confirm')} okLabel={t('settings.clear_btn')} onOk={clearSaved} onClose={() => setWipe(false)} />}
    </>
  );
}

function MyAccount() {
  const { t } = useI18n();
  const { me, refreshMe } = useAuth();
  const toast = useToast();
  const errText = useErrorText();
  const [name, setName] = useState(me?.name ?? '');
  const [phone, setPhone] = useState(me?.phone ?? '');
  const [pwOpen, setPwOpen] = useState(false);
  const [cur, setCur] = useState('');
  const [next, setNext] = useState('');
  const [again, setAgain] = useState('');
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<{ field?: string; text: string } | null>(null);
  if (!me) return null;
  const changed = name.trim() !== me.name || (phone.trim() || null) !== (me.phone || null) || (pwOpen && !!next);

  const save = async () => {
    setErr(null);
    if (!name.trim()) return setErr({ field: 'name', text: t('v.needed') });
    if (pwOpen && next) {
      if (next.length < 10) return setErr({ field: 'new', text: t('staff.pw_short') });
      if (next !== again) return setErr({ field: 'again', text: t('settings.pw_mismatch') });
      if (!cur) return setErr({ field: 'cur', text: t('v.needed') });
    }
    setBusy(true);
    try {
      await api.put('/dashboard/me', { name: name.trim(), phone: phone.trim() || null, ...(pwOpen && next ? { current_password: cur, new_password: next } : {}) });
      await refreshMe();
      setCur(''); setNext(''); setAgain(''); setPwOpen(false);
      toast(t('common.saved'), 'good');
    } catch (e) {
      const x = e as ApiError;
      setErr({ field: x.code === 'wrong_password' ? 'cur' : x.field, text: errText(x) });
    } finally { setBusy(false); }
  };
  const fe = (f: string) => (err?.field === f ? err.text : undefined);

  return (
    <div className="card">
      <div className="eyebrow">{t('settings.account_title')}</div>
      <div className="account-head"><span className="avatar">{me.name.split(/\s+/).map(s => s[0]).join('').slice(0, 2).toUpperCase()}</span>
        <div><b>{me.email}</b><div className="row">{me.roles.map(r => <Pill key={r.id} tone={r.name === 'Owner' ? 'dark' : 'brand'}>{r.name}</Pill>)}</div></div></div>
      <div className="form-grid">
        <Field label={t('staff.name')} error={fe('name')}><input type="text" value={name} onChange={e => setName(e.target.value)} /></Field>
        <Field label={t('staff.phone')} error={fe('phone')}><input type="tel" value={phone} onChange={e => setPhone(e.target.value)} placeholder="+9647501234567" /></Field>
        {me.job_title && <Field label={t('staff.job_title')} hint={t('settings.job_by_admin')}><input type="text" value={me.job_title} disabled /></Field>}
      </div>
      {pwOpen ? (
        <div className="form-grid mt">
          <Field label={t('settings.pw_current')} error={fe('cur')} full><input type="password" value={cur} onChange={e => setCur(e.target.value)} autoComplete="current-password" dir="ltr" /></Field>
          <Field label={t('staff.new_password')} hint={t('staff.pw_hint')} error={fe('new')}><input type="password" value={next} onChange={e => setNext(e.target.value)} autoComplete="new-password" dir="ltr" /></Field>
          <Field label={t('settings.pw_again')} error={fe('again')}><input type="password" value={again} onChange={e => setAgain(e.target.value)} autoComplete="new-password" dir="ltr" /></Field>
        </div>
      ) : <button className="btn sm mt" onClick={() => setPwOpen(true)}><KeyRound />{t('settings.change_pw')}</button>}
      {err && !['name', 'phone', 'cur', 'new', 'again'].includes(err.field ?? '') && <div className="mt"><Note tone="danger">{err.text}</Note></div>}
      <div className="row mt" style={{ justifyContent: 'flex-end' }}>
        <button className="btn primary" disabled={busy || !changed} onClick={save}><Check />{t('common.save')}</button>
      </div>
    </div>
  );
}
