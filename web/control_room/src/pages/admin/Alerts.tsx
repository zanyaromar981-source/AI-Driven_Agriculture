// Alerts: one message to every farmer. Write it in Kurdish and English, see the phone preview, save a
// draft or send it. The list keeps every alert; drafts can be edited or deleted, sent alerts only copied.
import { useMemo, useState } from 'react';
import { BellRing, CloudSun, Bug, Droplets, Store, Megaphone, MoreHorizontal, Send, Save, Copy, Trash2, Pencil, Smartphone } from 'lucide-react';
import { useI18n } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { db } from '../../data/db';
import { useRows } from '../../data/store';
import { newId, nowIso } from '../../data/api';
import type { Alert, AlertType } from '../../data/types';
import { Card, Confirm, Drawer, Field, Note, PageHead, Pill, Tabs, useToast } from '../../components/ui';
import { DataTable, type Col } from '../../components/DataTable';
import './alerts.css';

const TYPES: [AlertType, JSX.Element][] = [
  ['general', <Megaphone key="g" />], ['weather', <CloudSun key="w" />], ['pest', <Bug key="p" />],
  ['water', <Droplets key="d" />], ['market', <Store key="m" />], ['other', <MoreHorizontal key="o" />],
];
const LIMIT = 160;
const blank = (): Alert => ({ id: '', type: 'general', title: { en: '', ku: '' }, body: { en: '', ku: '' }, status: 'draft', by: '', created: '', sent: null });

export default function Alerts() {
  const { t, b, num, date } = useI18n();
  const { me } = useAuth();
  const toast = useToast();
  const alerts = useRows(db.alerts);
  const farmers = useRows(db.farmers);
  const [tab, setTab] = useState<'new' | 'list'>('new');
  const [draft, setDraft] = useState<Alert>(blank);
  const [confirm, setConfirm] = useState(false);
  const [view, setView] = useState<Alert | null>(null);
  const [del, setDel] = useState<Alert | null>(null);
  const [err, setErr] = useState('');

  const reach = useMemo(() => farmers.reduce((n, f) => n + (f.status === 'active' ? 1 : 0), 0), [farmers]);
  const rows = useMemo(() => [...alerts].sort((a, z) => (z.sent ?? z.created).localeCompare(a.sent ?? a.created)), [alerts]);

  const set = (p: Partial<Alert>) => setDraft(d => ({ ...d, ...p }));
  const valid = () => {
    if (!draft.title.ku.trim() && !draft.title.en.trim()) { setErr('alerts.need_title'); return false; }
    if (!draft.body.ku.trim() && !draft.body.en.trim()) { setErr('alerts.need_body'); return false; }
    setErr(''); return true;
  };
  const store = (status: 'draft' | 'sent') => {
    const now = nowIso();
    const a: Alert = { ...draft, id: draft.id || newId('a'), status, by: me?.name ?? '', created: draft.created || now, sent: status === 'sent' ? now : null };
    db.alerts.put(a);
    setDraft(blank());
    toast(t(status === 'sent' ? 'alerts.sent_toast' : 'alerts.draft_toast', { n: num(reach) }), 'good');
    if (status === 'sent') setTab('list');
  };
  const edit = (a: Alert) => { setDraft({ ...a }); setView(null); setTab('new'); };
  const copy = (a: Alert) => { setDraft({ ...a, id: '', status: 'draft', created: '', sent: null }); setView(null); setTab('new'); toast(t('alerts.copied')); };

  const typeLabel = (k: AlertType) => t('alerts.type_' + k);
  const cols: Col<Alert>[] = [
    { key: 'title', label: t('alerts.c_title'), cell: a => <b><bdi>{b(a.title)}</bdi></b>, sort: a => b(a.title) },
    { key: 'type', label: t('alerts.c_type'), cell: a => typeLabel(a.type), sort: a => a.type },
    { key: 'status', label: t('alerts.c_status'), cell: a => <Pill tone={a.status === 'sent' ? 'good' : 'warn'}>{t('alerts.s_' + a.status)}</Pill>, sort: a => a.status },
    { key: 'when', label: t('alerts.c_when'), cell: a => date(a.sent ?? a.created, 'datetime'), sort: a => a.sent ?? a.created },
    { key: 'by', label: t('alerts.c_by'), cell: a => a.by, sort: a => a.by },
    { key: 'ku', label: t('alerts.c_ku'), cell: a => a.title.ku && a.body.ku ? <Pill tone="good">{t('common.yes')}</Pill> : <Pill tone="warn">{t('common.ku_missing')}</Pill>, optional: true },
  ];

  const count = (s: string) => <small className={s.length > LIMIT ? 'err' : ''}>{t('alerts.count', { n: num(s.length), max: num(LIMIT) })}</small>;
  const previewTitle = draft.title.ku || draft.title.en || t('alerts.preview_title');
  const previewBody = draft.body.ku || draft.body.en || t('alerts.preview_body');

  return (
    <>
      <PageHead eyebrow={t('nav.g_act')} title={t('nav.alerts')} sub={t('alerts.sub', { n: num(reach) })} />
      <Tabs value={tab} onChange={setTab} items={[['new', draft.id ? t('alerts.tab_edit') : t('alerts.tab_new')], ['list', t('alerts.tab_list', { n: num(alerts.length) })]]} />

      {tab === 'new' ? (
        <div className="grid g-main">
          <Card>
            <div className="eyebrow" style={{ marginBottom: 8 }}>{t('alerts.kind')}</div>
            <div className="chips mb">
              {TYPES.map(([k, ic]) => <button key={k} className={'chip' + (draft.type === k ? ' on' : '')} onClick={() => set({ type: k })}>{ic}{typeLabel(k)}</button>)}
            </div>
            <div className="form-grid">
              <Field label={t('alerts.title_ku')} full>
                <input type="text" dir="rtl" lang="ckb" className="ku-text" value={draft.title.ku} onChange={e => set({ title: { ...draft.title, ku: e.target.value } })} />
              </Field>
              <Field label={t('alerts.body_ku')} full>
                <textarea dir="rtl" lang="ckb" className="ku-text" rows={3} value={draft.body.ku} onChange={e => set({ body: { ...draft.body, ku: e.target.value } })} />
                {count(draft.body.ku)}
              </Field>
              <Field label={t('alerts.title_en')} full>
                <input type="text" dir="ltr" value={draft.title.en} onChange={e => set({ title: { ...draft.title, en: e.target.value } })} />
              </Field>
              <Field label={t('alerts.body_en')} full>
                <textarea dir="ltr" rows={3} value={draft.body.en} onChange={e => set({ body: { ...draft.body, en: e.target.value } })} />
                {count(draft.body.en)}
              </Field>
            </div>
            {err && <div className="mt"><Note tone="danger">{t(err)}</Note></div>}
            <div className="mt"><Note tone="info">{t('alerts.rule_note')}</Note></div>
            <div className="row mt">
              <button className="btn primary" onClick={() => valid() && setConfirm(true)}><Send className="flip-rtl" />{t('alerts.send_all')}</button>
              <button className="btn" onClick={() => valid() && store('draft')}><Save />{t('alerts.save_draft')}</button>
              {draft.id && <button className="btn ghost" onClick={() => { setDraft(blank()); setErr(''); }}>{t('alerts.new_instead')}</button>}
            </div>
          </Card>
          <div className="stack">
            <Card title={t('alerts.preview')} extra={<Smartphone className="muted" />}>
              <div className="al-phone">
                <div className="al-clock">09:41</div>
                <div className="al-push" dir="rtl" lang="ckb">
                  <div className="row small" style={{ flexWrap: 'nowrap' }}><b>JUTYAR</b><span className="muted" style={{ marginInlineStart: 'auto' }}>{t('alerts.now')}</span></div>
                  <div className="al-push-title"><bdi>{previewTitle}</bdi></div>
                  <div className="small" dir="auto">{previewBody.length > LIMIT ? previewBody.slice(0, LIMIT) + '…' : previewBody}</div>
                </div>
              </div>
              <p className="muted small">{t('alerts.preview_note')}</p>
            </Card>
            <Card title={t('alerts.who')}>
              <dl className="facts">
                <dt>{t('alerts.who_all')}</dt><dd>{num(reach)}</dd>
                <dt>{t('alerts.who_blocked')}</dt><dd>{num(farmers.length - reach)}</dd>
              </dl>
            </Card>
          </div>
        </div>
      ) : (
        <Card>
          <DataTable id="alerts" rows={rows} cols={cols} onRow={setView} defaultSort={['when', -1]} />
        </Card>
      )}

      {confirm && (
        <Confirm danger={false} title={t('alerts.confirm_title')} okLabel={t('alerts.send_all')} onClose={() => setConfirm(false)} onOk={() => store('sent')}
          text={t('alerts.confirm_text', { n: num(reach) })} />
      )}
      {del && <Confirm title={t('alerts.delete_title')} okLabel={t('common.delete')} onClose={() => setDel(null)}
        onOk={() => { db.alerts.remove(del.id); setView(null); toast(t('common.deleted')); }} text={t('alerts.delete_text')} />}
      {view && (
        <Drawer title={<bdi>{b(view.title)}</bdi>} eyebrow={typeLabel(view.type)} onClose={() => setView(null)}>
          <div className="row mb"><Pill tone={view.status === 'sent' ? 'good' : 'warn'} icon={<BellRing />}>{t('alerts.s_' + view.status)}</Pill>
            <span className="muted small">{view.by} · {date(view.sent ?? view.created, 'datetime')}</span></div>
          <div className="stack">
            <div><div className="eyebrow">{t('common.kurdish')}</div>
              {view.title.ku ? <><b dir="rtl" lang="ckb" className="ku-text" style={{ display: 'block' }}>{view.title.ku}</b><p dir="rtl" lang="ckb" className="ku-text" style={{ margin: '4px 0' }}>{view.body.ku}</p></> : <p className="muted">{t('common.ku_missing')}</p>}</div>
            <div><div className="eyebrow">{t('common.english')}</div>
              <b dir="ltr" style={{ display: 'block' }}>{view.title.en}</b><p dir="ltr" style={{ margin: '4px 0' }}>{view.body.en}</p></div>
          </div>
          <div className="foot">
            {view.status === 'draft' && <button className="btn danger" onClick={() => setDel(view)}><Trash2 />{t('common.delete')}</button>}
            <button className="btn" onClick={() => copy(view)}><Copy />{t('alerts.duplicate')}</button>
            {view.status === 'draft' && <button className="btn primary" onClick={() => edit(view)}><Pencil />{t('common.edit')}</button>}
          </div>
          {view.status === 'sent' && <p className="muted small">{t('alerts.sent_locked')}</p>}
        </Drawer>
      )}
    </>
  );
}
