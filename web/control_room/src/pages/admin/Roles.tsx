// Roles and permissions (design 19). A role is a name and a set of resource:action permissions; staff hold
// one or more roles (FRONTEND.md 8). The Owner role is fixed. You can only give permissions you hold.
// A PUT replaces the whole set, so before saving the role is read again: if someone else changed it
// meanwhile, nothing is saved and the newer version is shown.
import { useEffect, useMemo, useState } from 'react';
import { Plus, Check, Trash2, Users, Info, Lock } from 'lucide-react';
import { useI18n } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { useApi, invalidate } from '../../api/cache';
import { api, ApiError } from '../../api/client';
import type { Action, Permission, Resource } from '../../api/types';
import { PageHead, Pill, Note, Confirm, Field, useToast } from '../../components/ui';
import { StateBox, useErrorText } from '../../components/domain';
import './roles.css';

interface Role { id: string; name: string; description?: string | null; system: boolean; staff_count: number; permissions: Permission[]; updated_at: string; created_at: string }
const ACTIONS: Action[] = ['read', 'create', 'update', 'delete'];
const RESOURCES: Resource[] = ['farmers', 'farms', 'insights', 'zones', 'dams', 'fires', 'outlooks', 'water', 'briefs', 'alwa', 'crops', 'rules', 'messages', 'app', 'jobs', 'staff', 'roles'];
const key = (r: string, a: string) => r + ':' + a;
// system roles are named in English on the server; show them in the site language
const useRoleName = () => { const { t } = useI18n(); return (n: string) => { const k = 'roles.sys_' + n.toLowerCase(); const v = t(k); return v === k ? n : v; }; };

export default function RolesPage() {
  const { t } = useI18n();
  const { can, perms, me, refreshMe } = useAuth();
  const list = useApi<{ roles: Role[] }>(can('roles') ? '/dashboard/roles' : null, ['staff_roles'], { auth: true });
  const cat = useApi<{ resources: Resource[]; actions: Action[] }>('/dashboard/permissions', ['staff_roles'], { auth: true });
  const toast = useToast();
  const errText = useErrorText();
  const roleName = useRoleName();
  const roles = list.data?.roles ?? [];
  const [sel, setSel] = useState<string | 'new' | null>(null);
  const cur = sel === 'new' ? null : roles.find(r => r.id === sel) ?? null;
  const [name, setName] = useState('');
  const [desc, setDesc] = useState('');
  const [set, setSet] = useState<Set<string>>(new Set());
  const [base, setBase] = useState<Role | null>(null);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [ask, setAsk] = useState<{ title: string; text: string; ok: string; run: () => void; danger?: boolean } | null>(null);

  // pick the first role once the list arrives
  useEffect(() => { if (sel == null && roles.length) setSel(roles.find(r => !r.system)?.id ?? roles[0].id); }, [roles, sel]);
  // load the chosen role into the editor (only when the choice changes, not on every refresh)
  useEffect(() => {
    if (sel === 'new') { setName(''); setDesc(''); setSet(new Set()); setBase(null); setErr(null); return; }
    const r = roles.find(x => x.id === sel); if (!r) return;
    if (base?.id === r.id) return;
    setName(r.name); setDesc(r.description ?? ''); setSet(new Set(r.permissions.map(p => key(p.resource, p.action)))); setBase(r); setErr(null);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [sel, roles]);

  const resources = useMemo(() => { const s = cat.data?.resources ?? RESOURCES; return [...RESOURCES.filter(r => s.includes(r)), ...s.filter(r => !RESOURCES.includes(r))]; }, [cat.data]);
  const readOnly = !!cur?.system || (sel === 'new' ? !can('roles', 'create') : !can('roles', 'update'));
  const dirty = sel === 'new' ? !!(name || set.size) : !!base && (name !== base.name || desc !== (base.description ?? '') || set.size !== base.permissions.length || base.permissions.some(p => !set.has(key(p.resource, p.action))));
  const mayGive = (k: string) => perms.has(k);

  const choose = (id: string | 'new') => {
    if (id === sel) return;
    if (dirty && !readOnly) setAsk({ title: t('roles.discard_title'), text: t('roles.discard_text'), ok: t('roles.discard_ok'), run: () => { setBase(null); setSel(id); } });
    else { setBase(null); setSel(id); }
  };
  const toggle = (k: string, on: boolean) => setSet(s => { const n = new Set(s); on ? n.add(k) : n.delete(k); return n; });
  const toggleMany = (ks: string[], on: boolean) => setSet(s => { const n = new Set(s); for (const k of ks) if (mayGive(k)) on ? n.add(k) : n.delete(k); return n; });

  const save = async () => {
    if (busy) return;
    setErr(null);
    if (!name.trim()) return setErr(t('roles.name_needed'));
    setBusy(true);
    const permissions = [...set].map(k => { const [resource, action] = k.split(':'); return { resource, action }; });
    const body = { name: name.trim(), description: desc.trim() || null, permissions };
    try {
      if (sel === 'new') {
        const r = await api.post<{ role: Role }>('/dashboard/roles', body);
        toast(t('common.saved'), 'good'); setBase(r.role); setSel(r.role.id);
      } else if (base) {
        const fresh = await api.get<{ role: Role }>('/dashboard/roles/' + base.id);
        if (fresh.role.updated_at !== base.updated_at) {
          setErr(t('roles.changed_meanwhile'));
          setName(fresh.role.name); setDesc(fresh.role.description ?? ''); setSet(new Set(fresh.role.permissions.map(p => key(p.resource, p.action)))); setBase(fresh.role);
          return;
        }
        const r = await api.put<{ role: Role }>('/dashboard/roles/' + base.id, body);
        setBase(r.role); toast(t('common.saved'), 'good');
        if (me?.roles.some(x => x.id === base.id)) await refreshMe();
      }
    } catch (e) { setErr(errText(e as ApiError)); }
    finally { setBusy(false); invalidate('staff_roles'); }
  };
  const remove = async () => {
    if (!cur || busy) return;
    setBusy(true);
    try { await api.del('/dashboard/roles/' + cur.id); toast(t('common.deleted'), 'good'); setBase(null); setSel(null); }
    catch (e) {
      // already gone (a second click, or another admin) counts as deleted
      if ((e as ApiError).status === 404) { toast(t('common.deleted'), 'good'); setBase(null); setSel(null); } else toast(errText(e as ApiError), 'danger');
    }
    finally { setBusy(false); invalidate('staff_roles'); }
  };

  if (!can('roles')) return <><PageHead eyebrow={t('nav.g_people')} title={t('nav.roles')} /><StateBox kind="locked" /></>;

  return (
    <>
      <PageHead eyebrow={t('nav.g_people')} title={t('nav.roles')} sub={t('roles.sub')}
        actions={can('roles', 'create') && <button className="btn primary" onClick={() => choose('new')}><Plus />{t('roles.new')}</button>} />
      {list.error && !list.data ? <div className="card"><StateBox kind="error" action={<button className="btn sm" onClick={list.reload}>{t('common.retry')}</button>} /></div> : (
        <div className="roles-grid">
          <div className="card roles-list">
            <div className="eyebrow">{t('roles.list')}</div>
            {list.loading && <><i className="sk" /><i className="sk" /><i className="sk" /></>}
            {roles.map(r => (
              <button key={r.id} className={'role-item' + (sel === r.id ? ' on' : '')} onClick={() => choose(r.id)}>
                <span className="role-item-main"><b>{roleName(r.name)}</b>{r.system && <Pill tone="dark">{t('roles.system')}</Pill>}</span>
                <span className="muted small">{r.description || t('roles.perm_count', { n: r.permissions.length })}</span>
                <span className="role-holders muted small"><Users />{r.staff_count}</span>
              </button>
            ))}
            {sel === 'new' && <div className="role-item on"><b>{name || t('roles.new')}</b></div>}
          </div>

          <div className="card role-editor">
            {!cur && sel !== 'new' ? <StateBox kind="empty" title={t('roles.pick')} text={t('roles.pick_text')} /> : (
              <>
                <div className="spread">
                  <div>
                    <h2>{sel === 'new' ? t('roles.new') : roleName(cur!.name)}</h2>
                    {cur && <span className="muted small">{t('roles.holders', { n: cur.staff_count })}</span>}
                  </div>
                  {!readOnly && (
                    <div className="row">
                      {cur && can('roles', 'delete') && <button className="btn danger" disabled={busy || cur.staff_count > 0} title={cur.staff_count > 0 ? t('err.role_in_use') : undefined}
                        onClick={() => setAsk({ title: t('roles.delete_title', { name: roleName(cur.name) }), text: t('roles.delete_text'), ok: t('common.delete'), run: remove })}><Trash2 />{t('roles.delete')}</button>}
                      <button className="btn primary" disabled={busy || !dirty} onClick={save}><Check />{sel === 'new' ? t('roles.create') : t('roles.save')}</button>
                    </div>
                  )}
                </div>
                {cur?.system && <Note tone="warn" icon={<Lock />}>{t('roles.system_note')}</Note>}
                <div className="form-grid">
                  <Field label={t('roles.name')}><input type="text" value={name} onChange={e => setName(e.target.value)} disabled={readOnly} /></Field>
                  <Field label={t('roles.description')}><input type="text" value={desc} onChange={e => setDesc(e.target.value)} disabled={readOnly} /></Field>
                </div>
                <div className="eyebrow">{t('roles.permissions')}</div>
                <div className="perm-wrap">
                  <table className="t perm-matrix">
                    <thead>
                      <tr>
                        <th>{t('roles.area')}</th>
                        {ACTIONS.map(a => {
                          const ks = resources.map(r => key(r, a)).filter(mayGive), all = ks.length > 0 && ks.every(k => set.has(k));
                          return <th key={a} className="center">
                            <label className="perm-head"><span>{t('roles.a_' + a)}</span>
                              {!readOnly && <input type="checkbox" checked={all} onChange={e => toggleMany(ks, e.target.checked)} aria-label={t('roles.all_col', { a: t('roles.a_' + a) })} />}</label>
                          </th>;
                        })}
                        <th className="center">{t('roles.all')}</th>
                      </tr>
                    </thead>
                    <tbody>
                      {resources.map(r => {
                        const ks = ACTIONS.map(a => key(r, a)), give = ks.filter(mayGive), all = give.length > 0 && give.every(k => set.has(k));
                        return (
                          <tr key={r}>
                            <td><b>{t('roles.r_' + r)}</b> <span className="mono muted small">{r}</span></td>
                            {ACTIONS.map(a => {
                              const k = key(r, a), may = mayGive(k);
                              return <td key={a} className="center">
                                <input type="checkbox" className="perm-box" checked={set.has(k)} disabled={readOnly || !may} title={!may ? t('roles.cannot_give') : undefined}
                                  onChange={e => toggle(k, e.target.checked)} aria-label={t('roles.r_' + r) + ' · ' + t('roles.a_' + a)} />
                              </td>;
                            })}
                            <td className="center">{!readOnly && <input type="checkbox" className="perm-box" checked={all} disabled={!give.length} onChange={e => toggleMany(ks, e.target.checked)} aria-label={t('roles.all_row', { r: t('roles.r_' + r) })} />}</td>
                          </tr>
                        );
                      })}
                    </tbody>
                  </table>
                </div>
                {err && <Note tone="danger">{err}</Note>}
                <Note tone="info" icon={<Info />}>{t('roles.note')}</Note>
              </>
            )}
          </div>
        </div>
      )}
      {ask && <Confirm title={ask.title} text={ask.text} okLabel={ask.ok} onOk={ask.run} onClose={() => setAsk(null)} />}
    </>
  );
}
