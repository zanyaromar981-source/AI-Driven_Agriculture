// Staff sign-in (design 05) against POST /v1/dashboard/auth/login. There is no self-service reset yet:
// an Owner resets a password from the Staff page (FRONTEND.md 8).
import { useState, type FormEvent } from 'react';
import { Link, Navigate, useLocation, useNavigate } from 'react-router-dom';
import { Eye, EyeOff, LogIn, ArrowLeft, Info } from 'lucide-react';
import { useI18n } from '../i18n';
import { useAuth } from '../auth/auth';
import { ApiError } from '../api/client';
import { GrainSun } from '../motion/GrainSun';
import { LangSwitch } from '../layouts/LangSwitch';
import { Field, Modal, Note } from '../components/ui';
import './login.css';

export default function Login() {
  const { t } = useI18n();
  const { signIn, me } = useAuth();
  const nav = useNavigate();
  const loc = useLocation() as { state?: { from?: string } };
  const [email, setEmail] = useState('');
  const [pw, setPw] = useState('');
  const [show, setShow] = useState(false);
  const [err, setErr] = useState('');
  const [busy, setBusy] = useState(false);
  const [forgot, setForgot] = useState(false);

  const submit = async (e: FormEvent) => {
    e.preventDefault(); setErr('');
    if (!email.trim() || !pw) { setErr('login.fill_both'); return; }
    setBusy(true);
    try {
      await signIn(email, pw);
      nav(loc.state?.from ?? '/admin', { replace: true });
    } catch (x) {
      const code = x instanceof ApiError ? x.code : '';
      setErr(code === 'bad_credentials' ? 'login.bad' : code === 'offline' ? 'err.offline' : code === 'rate_limited' ? 'err.rate_limited' : 'err.generic');
    } finally { setBusy(false); }
  };

  if (me) return <Navigate to={loc.state?.from ?? '/admin'} replace />;

  return (
    <div className="login">
      <div className="login-art" aria-hidden="true">
        <div className="login-art-in">
          <GrainSun size={96} />
          <div className="login-word"><span>Jutyar</span><span className="ku-text">جوتیار</span></div>
          <p>{t('login.art')}</p>
        </div>
      </div>
      <div className="login-side">
        <div className="spread">
          <Link to="/" className="btn ghost sm"><ArrowLeft className="flip-rtl" />{t('login.back')}</Link>
          <LangSwitch />
        </div>
        <form className="login-form" onSubmit={submit} noValidate>
          <div className="logo" style={{ marginBottom: 6 }}><span className="mark"><GrainSun size={26} /></span><span className="word-ku">جوتیار</span><span className="word-en-full">Jutyar</span></div>
          <h1>{t('login.title')}</h1>
          <p className="muted" style={{ marginTop: 0 }}>{t('login.sub')}</p>
          <Field label={t('login.email')}>
            <input type="email" autoComplete="username" value={email} onChange={e => setEmail(e.target.value)} autoFocus dir="ltr" />
          </Field>
          <Field label={t('login.password')}>
            <div style={{ position: 'relative' }}>
              <input type={show ? 'text' : 'password'} autoComplete="current-password" value={pw} onChange={e => setPw(e.target.value)} dir="ltr" style={{ paddingInlineEnd: 44 }} />
              <button type="button" className="btn ghost sm icon" onClick={() => setShow(s => !s)} aria-label={t(show ? 'login.hide' : 'login.show')}
                style={{ position: 'absolute', insetInlineEnd: 3, top: 3 }}>{show ? <EyeOff /> : <Eye />}</button>
            </div>
          </Field>
          {err && <Note tone="danger">{t(err)}</Note>}
          <button className="btn primary" type="submit" disabled={busy} style={{ width: '100%' }}><LogIn className="flip-rtl" />{t('login.submit')}</button>
          <button type="button" className="btn ghost sm" onClick={() => setForgot(true)} style={{ alignSelf: 'center' }}>{t('login.forgot')}</button>
        </form>
      </div>
      {forgot && (
        <Modal title={t('login.forgot')} onClose={() => setForgot(false)} foot={<button className="btn primary" onClick={() => setForgot(false)}>{t('common.done')}</button>}>
          <Note tone="info" icon={<Info />}>{t('login.forgot_owner')}</Note>
        </Modal>
      )}
    </div>
  );
}
