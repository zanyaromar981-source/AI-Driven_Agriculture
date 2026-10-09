// Staff (design 18). Everyone who signs in to the Admin part: list, add, change, switch off, delete.
// Routes: /dashboard/staff and /dashboard/roles (FRONTEND.md 8). Passwords are at least 10 characters;
// an email never changes; you cannot delete or switch off yourself; one active Owner must stay.
import { useMemo, useState } from 'react';
import { UserPlus, Pencil, Trash2, Info, KeyRound } from 'lucide-react';
import { useI18n } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { useApi, invalidate } from '../../api/cache';
import { api, ApiError } from '../../api/client';
import type { Staff, Permission } from '../../api/types';
import { PageHead, Pill, Note, Modal, Confirm, Field, Switch, useToast } from '../../components/ui';
import { DataTable, type Col } from '../../components/DataTable';
import { Phone, StateBox, useErrorText } from '../../components/domain';
import './staff.css';

interface RoleRow { id: string; name: string; system: boolean; staff_count: number; permissions: Permission[] }
// system roles are named in English on the server; show them in the site language
const useRoleName = () => { const { t } = useI18n(); return (n: string) => { const k = 'roles.sys_' + n.toLowerCase(); const v = t(k); return v === k ? n : v; }; };
const initials = (n: string) => n.split(/\s+/).map(s => s[0]).join('').slice(0, 2).toUpperCase();

export default function StaffPage() {
  const { t, date } = useI18n();
  const { me, can } = useAuth();
  const list = useApi<{ staff: Staff[] }>(can('staff') ? '/dashboard/staff' : null, ['staff_roles'], { auth: true });
  const [form, setForm] = useState<Staff | 'new' | null>(null);
  const [del, setDel] = useState<Staff | null>(null);
  const toast = useToast();
  const errText = useErrorText();
  const roleName = useRoleName();
  const [busy, setBusy] = useState(false);

  if (!can('staff')) return <><PageHead eyebrow={t('nav.g_people')} title={t('nav.staff')} /><StateBox kind="locked" /></>;

  const rows = list.data?.staff ?? [];
  const cols: Col<Staff>[] = [
    { key: 'name', label: t('staff.name'), sort: r => r.name.toLowerCase(), cell: r => (
      <span className="staff-name"><span className="avatar" style={{ background: r.active ? 'var(--brand)' : 'var(--ink-3)' }}>{initials(r.name)}</span><b>{r.name}</b>{r.id === me?.id && <Pill tone="water">{t('staff.you')}</Pill>}</span>) },
    { key: 'contact', label: t('staff.contact'), cell: r => <span className="stack-sm"><span className="ltr small">{r.email}</span><span className="muted small"><Phone value={r.phone} /></span></span> },
    { key: 'job', label: t('staff.job_title'), sort: r => r.job_title ?? '', cell: r => r.job_title || <span className="muted">-</span> },
    { key: 'roles', label: t('staff.roles'), cell: r => <span className="row">{r.roles.map(x => <Pill key={x.id} tone={x.name === 'Owner' ? 'dark' : 'brand'}>{roleName(x.name)}</Pill>)}</span> },
    { key: 'created', label: t('staff.created'), optional: true, sort: r => r.created_at, cell: r => <span className="small muted">{date(r.created_at, 'short')}</span> },
    { key: 'state', label: t('common.status'), sort: r => (r.active ? 0 : 1), cell: r => <Pill tone={r.active ? 'good' : ''}>{t(r.active ? 'common.active' : 'common.inactive')}</Pill> },
    { key: 'act', label: '', className: 'act', cell: r => (
      <span className="row" style={{ justifyContent: 'flex-end' }}>
        {can('staff', 'update') && <button className="btn sm icon" onClick={e => { e.stopPropagation(); setForm(r); }} aria-label={t('common.edit')}><Pencil /></button>}
        {can('staff', 'delete') && r.id !== me?.id && <button className="btn sm icon danger" disabled={busy} onClick={e => { e.stopPropagation(); setDel(r); }} aria-label={t('common.delete')}><Trash2 /></button>}
      </span>) },
  ];

  const remove = async (s: Staff) => {
    if (busy) return;
    setBusy(true);
    try { await api.del('/dashboard/staff/' + s.id); toast(t('common.deleted'), 'good'); }
    catch (e) { (e as ApiError).status === 404 ? toast(t('common.deleted'), 'good') : toast(errText(e as ApiError), 'danger'); }
    finally { setBusy(false); invalidate('staff_roles'); }
  };

  return (
    <>
      <PageHead eyebrow={t('nav.g_people')} title={t('nav.staff')} sub={t('staff.sub')}
        actions={can('staff', 'create') && <button className="btn primary" onClick={() => setForm('new')}><UserPlus />{t('staff.new')}</button>} />
      <div className="card">
        {list.error && !list.data ? <StateBox kind="error" action={<button className="btn sm" onClick={list.reload}>{t('common.retry')}</button>} /> : (
          <DataTable id="staff" rows={rows} cols={cols} loading={list.loading} onRow={can('staff', 'update') ? r => setForm(r) : undefined}
            defaultSort={['name', 1]} head={<span className="muted small">{t('staff.count', { n: rows.length })}</span>} />
        )}
      </div>
      <div className="mt"><Note tone="info" icon={<Info />}>{t('staff.rules_note')}</Note></div>
      {form && <StaffForm staff={form === 'new' ? null : form} onClose={() => setForm(null)} />}
      {del && <Confirm title={t('staff.delete_title', { name: del.name })} text={t('staff.delete_text')} okLabel={t('common.delete')} onOk={() => remove(del)} onClose={() => setDel(null)} />}
    </>
  );
}

function StaffForm({ staff, onClose }: { staff: Staff | null; onClose: () => void }) {
  const { t } = useI18n();
  const { me, perms, refreshMe } = useAuth();
  const roles = useApi<{ roles: RoleRow[] }>('/dashboard/roles', ['staff_roles'], { auth: true });
  const toast = useToast();
  const errText = useErrorText();
  const roleName = useRoleName();
  const isNew = !staff, self = staff?.id === me?.id;
  const [name, setName] = useState(staff?.name ?? '');
  const [email, setEmail] = useState(staff?.email ?? '');
  const [phone, setPhone] = useState(staff?.phone ?? '');
  const [job, setJob] = useState(staff?.job_title ?? '');
  const [active, setActive] = useState(staff?.active ?? true);
  const [roleIds, setRoleIds] = useState<string[]>(staff?.roles.map(r => r.id) ?? []);
  const [pw, setPw] = useState('');
  const [setPwOpen, setSetPwOpen] = useState(isNew);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<{ field?: string; text: string } | null>(null);

  // a role I may give = every permission it holds, I hold too (the server answers cannot_grant otherwise)
  const grantable = useMemo(() => new Map((roles.data?.roles ?? []).map(r => [r.id, r.permissions.every(p => perms.has(p.resource + ':' + p.action))])), [roles.data, perms]);

  const save = async () => {
    if (busy) return;
    setErr(null);
    if (!name.trim()) return setErr({ field: 'name', text: t('v.needed') });
    if (isNew && !/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(email.trim())) return setErr({ field: 'email', text: t('v.email_bad') });
    if ((isNew || pw) && pw.length < 10) return setErr({ field: 'password', text: t('staff.pw_short') });
    setBusy(true);
    const body = { name: name.trim(), phone: phone.trim() || null, job_title: job.trim() || null, role_ids: roleIds };
    try {
      if (isNew) await api.post('/dashboard/staff', { ...body, email: email.trim().toLowerCase(), password: pw });
      else await api.put('/dashboard/staff/' + staff!.id, { ...body, active, password: pw || null });
      toast(t('common.saved'), 'good');
      if (self) await refreshMe();
      onClose();
    } catch (e) {
      const x = e as ApiError;
      setErr({ field: x.field ?? (x.code === 'email_taken' ? 'email' : undefined), text: errText(x) });
    } finally { setBusy(false); invalidate('staff_roles'); }
  };

  const fe = (f: string) => (err?.field === f ? err.text : undefined);
  return (
    <Modal title={isNew ? t('staff.new') : t('staff.edit_title', { name: staff!.name })} onClose={onClose} wide
      foot={<><button className="btn" onClick={onClose} disabled={busy}>{t('common.cancel')}</button><button className="btn primary" disabled={busy} onClick={save}>{busy ? t('common.loading') : t('common.save')}</button></>}>
      <div className="form-grid">
        <Field label={t('staff.name')} error={fe('name')}><input type="text" value={name} onChange={e => setName(e.target.value)} autoFocus /></Field>
        <Field label={t('login.email')} hint={isNew ? undefined : t('staff.email_fixed')} error={fe('email')}>
          <input type="email" value={email} onChange={e => setEmail(e.target.value)} disabled={!isNew} dir="ltr" autoComplete="off" />
        </Field>
        <Field label={t('staff.phone')} error={fe('phone')}><input type="tel" value={phone} onChange={e => setPhone(e.target.value)} placeholder="+9647501234567" /></Field>
        <Field label={t('staff.job_title')} error={fe('job_title')}><input type="text" value={job} onChange={e => setJob(e.target.value)} /></Field>
        <div className="field full">
          <span>{t('staff.roles')}</span>
          {roles.loading ? <i className="sk" /> : (
            <div className="role-picks">
              {(roles.data?.roles ?? []).map(r => {
                const ok = grantable.get(r.id) !== false, on = roleIds.includes(r.id);
                return (
                  <label key={r.id} className={'role-pick' + (on ? ' on' : '') + (!ok ? ' off' : '')} title={!ok ? t('staff.cannot_grant_role') : undefined}>
                    <input type="checkbox" checked={on} disabled={!ok && !on} onChange={e => setRoleIds(ids => (e.target.checked ? [...ids, r.id] : ids.filter(x => x !== r.id)))} />
                    <b>{roleName(r.name)}</b>{r.system && <Pill tone="dark">{t('roles.system')}</Pill>}
                  </label>
                );
              })}
            </div>
          )}
        </div>
        {!isNew && (
          <div className="full set">
            <div className="txt"><b>{t('staff.active')}</b><small>{self ? t('staff.self_active') : t('staff.active_sub')}</small></div>
            <div className="ctl"><Switch on={active} onChange={setActive} disabled={self} label={t('staff.active')} /></div>
          </div>
        )}
        <div className="full">
          {setPwOpen ? (
            <Field label={isNew ? t('login.password') : t('staff.new_password')} hint={t('staff.pw_hint')} error={fe('password')}>
              <input type="password" value={pw} onChange={e => setPw(e.target.value)} autoComplete="new-password" dir="ltr" />
            </Field>
          ) : <button type="button" className="btn sm" onClick={() => setSetPwOpen(true)}><KeyRound />{t('staff.reset_password')}</button>}
        </div>
      </div>
      {err && !err.field && <div className="mt"><Note tone="danger">{err.text}</Note></div>}
    </Modal>
  );
}
