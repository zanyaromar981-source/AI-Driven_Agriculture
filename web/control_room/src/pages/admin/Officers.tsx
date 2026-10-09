// Admins: add, edit, switch off and delete the people who sign in to the Control Room.
// One admin level. You cannot switch off or delete yourself, and one active admin must always remain.
import { useMemo, useState } from 'react';
import { Plus, Pencil, Trash2, Info } from 'lucide-react';
import { db } from '../../data/db';
import { useRows } from '../../data/store';
import { checkOfficer, newId, nowIso, saveOfficer, type Problems } from '../../data/api';
import type { Officer } from '../../data/types';
import { useI18n } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { Card, Confirm, Field, Kpi, Modal, Note, PageHead, Pill, Switch, useDebounced, useToast } from '../../components/ui';
import { DataTable, type Col } from '../../components/DataTable';
import { Phone } from '../../components/domain';

const blank = (): Officer => ({ id: '', name: '', email: '', phone: '', jobTitle: '', active: true, created: nowIso(), lastSeen: null });

export default function Officers() {
  const { t, num, ago } = useI18n();
  const { me } = useAuth();
  const toast = useToast();
  const rows = useRows(db.officers);
  const [q, setQ] = useState('');
  const dq = useDebounced(q.trim().toLowerCase());
  const [edit, setEdit] = useState<Officer | null>(null);
  const [del, setDel] = useState<Officer | null>(null);
  const activeCount = useMemo(() => rows.reduce((n, o) => n + (o.active ? 1 : 0), 0), [rows]);
  const list = useMemo(() => rows.filter(o => !dq || o.name.toLowerCase().includes(dq) || o.email.includes(dq) || o.jobTitle.toLowerCase().includes(dq) || o.phone.includes(dq)), [rows, dq]);

  /** why this admin cannot be switched off or deleted, or '' */
  const guard = (o: Officer) => (o.id === me?.id ? 'officers.not_self' : o.active && activeCount <= 1 ? 'officers.last_active' : '');
  const setActive = (o: Officer, v: boolean) => {
    const g = !v ? guard(o) : '';
    if (g) { toast(t(g), 'warn'); return; }
    db.officers.patch(o.id, { active: v });
    toast(t(v ? 'officers.on' : 'officers.off', { name: o.name }), 'good');
  };

  const cols: Col<Officer>[] = [
    { key: 'name', label: t('officers.name'), sort: o => o.name.toLowerCase(), cell: o => (
      <div className="row" style={{ flexWrap: 'nowrap' }}>
        <span className="avatar" style={{ background: o.active ? 'var(--brand)' : 'var(--ink-3)' }}>{o.name.split(' ').map(s => s[0]).join('').slice(0, 2)}</span>
        <span><b>{o.name}</b>{o.id === me?.id && <> <Pill tone="water">{t('officers.you')}</Pill></>}</span>
      </div>) },
    { key: 'email', label: t('officers.email'), sort: o => o.email, cell: o => <span className="ltr small">{o.email}</span> },
    { key: 'phone', label: t('officers.phone'), cell: o => (o.phone ? <Phone value={o.phone} /> : <span className="muted">-</span>) },
    { key: 'job', label: t('officers.job'), sort: o => o.jobTitle, cell: o => <bdi>{o.jobTitle || '-'}</bdi> },
    { key: 'active', label: t('common.status'), sort: o => (o.active ? 0 : 1), cell: o => (
      <span className="row" style={{ flexWrap: 'nowrap' }} onClick={e => e.stopPropagation()}>
        <Switch on={o.active} onChange={v => setActive(o, v)} label={t('common.active')} />
        <span className="small">{t(o.active ? 'common.active' : 'common.inactive')}</span>
      </span>) },
    { key: 'last', label: t('officers.last_seen'), sort: o => o.lastSeen ?? '', cell: o => <span className="nowrap">{ago(o.lastSeen)}</span> },
    { key: 'created', label: t('officers.added'), optional: true, sort: o => o.created, cell: o => ago(o.created) },
    { key: 'act', label: '', className: 'act', cell: o => <>
      <button className="btn sm icon" aria-label={t('common.edit')} onClick={e => { e.stopPropagation(); setEdit(o); }}><Pencil /></button>{' '}
      <button className="btn sm icon danger" aria-label={t('common.delete')} onClick={e => { e.stopPropagation(); const g = guard(o); if (g) toast(t(g), 'warn'); else setDel(o); }}><Trash2 /></button>
    </> },
  ];

  return (
    <>
      <PageHead eyebrow={t('nav.g_people')} title={t('nav.officers')} sub={t('officers.sub')}
        actions={<button className="btn primary" onClick={() => setEdit(blank())}><Plus />{t('officers.add')}</button>} />
      <div className="grid g3 mb">
        <Kpi label={t('officers.total')} value={num(rows.length)} />
        <Kpi label={t('common.active')} value={num(activeCount)} tone="good" />
        <Kpi label={t('common.inactive')} value={num(rows.length - activeCount)} />
      </div>
      <Card>
        <DataTable id="officers" rows={list} cols={cols} onRow={setEdit} defaultSort={['name', 1]}
          head={<input type="search" value={q} onChange={e => setQ(e.target.value)} placeholder={t('officers.search')} style={{ width: 260, maxWidth: '100%' }} />} />
      </Card>
      <div className="mt"><Note tone="info" icon={<Info />}>{t('officers.pw_note')}</Note></div>
      {edit && <EditOfficer o={edit} onClose={() => setEdit(null)} guard={guard} />}
      {del && <Confirm title={t('officers.del_title', { name: del.name })} text={t('officers.del_text')} okLabel={t('common.delete')}
        onOk={() => { db.officers.remove(del.id); toast(t('common.deleted')); }} onClose={() => setDel(null)} />}
    </>
  );
}

function EditOfficer({ o, onClose, guard }: { o: Officer; onClose: () => void; guard: (o: Officer) => string }) {
  const { t } = useI18n();
  const toast = useToast();
  const isNew = !o.id;
  const [d, setD] = useState(o);
  const [err, setErr] = useState<Problems>({});
  const g = !isNew ? guard(o) : '';
  const save = () => {
    const rec = isNew ? { ...d, id: newId('o') } : d;
    const p = checkOfficer(rec);
    if (!rec.active && g) p.active = g;
    setErr(p);
    if (Object.keys(p).length) return;
    saveOfficer(rec); toast(t(isNew ? 'officers.added_toast' : 'common.saved', { name: rec.name }), 'good'); onClose();
  };
  return (
    <Modal title={isNew ? t('officers.add') : d.name} onClose={onClose} foot={<>
      <button className="btn" onClick={onClose}>{t('common.cancel')}</button>
      <button className="btn primary" onClick={save}>{t('common.save')}</button></>}>
      <div className="form-grid">
        <Field label={t('officers.name')} error={err.name}><input type="text" value={d.name} onChange={e => setD({ ...d, name: e.target.value })} autoFocus /></Field>
        <Field label={t('officers.job')}><input type="text" value={d.jobTitle} onChange={e => setD({ ...d, jobTitle: e.target.value })} /></Field>
        <Field label={t('officers.email')} error={err.email}><input type="email" value={d.email} onChange={e => setD({ ...d, email: e.target.value })} /></Field>
        <Field label={t('officers.phone')} hint={t('officers.phone_hint')} error={err.phone}><input type="tel" value={d.phone} onChange={e => setD({ ...d, phone: e.target.value })} placeholder="0750 123 4567" /></Field>
        <div className="full row">
          <Switch on={d.active} onChange={v => setD({ ...d, active: v })} label={t('common.active')} disabled={!!g && d.active} />
          <span>{t('officers.active_long')}</span>
        </div>
        {err.active && <div className="full"><Note tone="warn">{t(err.active)}</Note></div>}
      </div>
      {isNew && <div className="mt"><Note tone="info">{t('officers.new_note')}</Note></div>}
    </Modal>
  );
}
