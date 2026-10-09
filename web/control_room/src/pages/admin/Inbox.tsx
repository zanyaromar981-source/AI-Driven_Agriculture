// Inbox: messages from farmers (read, answer, close) and the news bar that moves across the top of the site.
// Cost: one filter pass per change of filters or messages; the list shows 30 at a time.
import { useEffect, useMemo, useState } from 'react';
import { Link, useSearchParams } from 'react-router-dom';
import { MessageSquare, Send, Eye, CheckCheck, RotateCcw, ArrowLeft, Image, MapPin, Plus, ArrowUp, ArrowDown, Pencil, Trash2, User } from 'lucide-react';
import { useI18n } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { db } from '../../data/db';
import { useRows } from '../../data/store';
import { newId, nowIso } from '../../data/api';
import type { Message, MessageKind, MessageState, News } from '../../data/types';
import { Card, Confirm, Field, Modal, Note, PageHead, Pill, Select, Switch, Tabs, useDebounced, useToast, type Tone } from '../../components/ui';
import { Phone, usePlaceNames } from '../../components/domain';
import './inbox.css';

const STATES: MessageState[] = ['new', 'read', 'replied', 'closed'];
const KINDS: MessageKind[] = ['question', 'report', 'complaint', 'request', 'other'];
const TONE: Record<MessageState, Tone> = { new: 'danger', read: '', replied: 'good', closed: 'dark' };
const PER = 30;

export default function Inbox() {
  const { t } = useI18n();
  const [tab, setTab] = useState<'messages' | 'news'>('messages');
  return (
    <>
      <PageHead eyebrow={t('nav.g_act')} title={t('nav.inbox')} sub={t('inbox.sub')} />
      <Tabs value={tab} onChange={setTab} items={[['messages', t('inbox.tab_messages')], ['news', t('inbox.tab_news')]]} />
      {tab === 'messages' ? <Messages /> : <NewsBar />}
    </>
  );
}

function Messages() {
  const { t, b, ago, num } = useI18n();
  const messages = useRows(db.messages);
  const [params, setParams] = useSearchParams();
  const sel = params.get('m');
  const [state, setState] = useState<string>('open');
  const [kind, setKind] = useState('');
  const [q, setQ] = useState('');
  const dq = useDebounced(q.trim().toLowerCase());
  const [shown, setShown] = useState(PER);

  const list = useMemo(() => {
    const out = messages.filter(m => {
      if (state === 'open' ? m.state === 'closed' : state && m.state !== state) return false;
      if (kind && m.kind !== kind) return false;
      if (!dq) return true;
      const f = db.farmers.get(m.farmerId);
      return m.subject.toLowerCase().includes(dq) || m.text.includes(dq) || (!!f && (f.name.en.toLowerCase().includes(dq) || f.name.ku.includes(dq) || f.phone.includes(dq)));
    });
    return out.sort((a, z) => z.at.localeCompare(a.at));
  }, [messages, state, kind, dq]);
  const counts = useMemo(() => { const c: Record<string, number> = {}; for (const m of messages) c[m.state] = (c[m.state] ?? 0) + 1; return c; }, [messages]);

  const cur = sel ? db.messages.get(sel) : undefined;
  // opening a new message marks it read
  useEffect(() => { if (cur?.state === 'new') db.messages.patch(cur.id, { state: 'read' }); }, [cur?.id]); // eslint-disable-line react-hooks/exhaustive-deps
  const open = (id: string | null) => { const p = new URLSearchParams(params); if (id) p.set('m', id); else p.delete('m'); setParams(p, { replace: true }); };

  return (
    <div className={'ib-grid' + (cur ? ' has-sel' : '')}>
      <Card className="ib-list pad0">
        <div className="ib-filters">
          <input type="search" value={q} onChange={e => { setQ(e.target.value); setShown(PER); }} placeholder={t('inbox.search')} />
          <div className="row" style={{ flexWrap: 'nowrap' }}>
            <Select value={state} onChange={v => { setState(v); setShown(PER); }} options={[['open', t('inbox.f_open')], ['', t('inbox.f_all')], ...STATES.map(s => [s, t('inbox.s_' + s) + ' (' + num(counts[s] ?? 0) + ')'] as [string, string])]} />
            <Select value={kind} onChange={v => { setKind(v); setShown(PER); }} options={[['', t('inbox.f_kinds')], ...KINDS.map(k => [k, t('inbox.k_' + k)] as [string, string])]} />
          </div>
        </div>
        <div className="ib-items">
          {list.slice(0, shown).map(m => {
            const f = db.farmers.get(m.farmerId);
            return (
              <div key={m.id} className={'list-item click' + (cur?.id === m.id ? ' sel' : '')} onClick={() => open(m.id)} role="button" tabIndex={0} onKeyDown={e => e.key === 'Enter' && open(m.id)}>
                <span className={'ico ' + (m.state === 'new' ? 'danger' : m.state === 'replied' ? 'good' : '')}><MessageSquare /></span>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <b className={'ib-ellipsis' + (m.state === 'new' ? '' : ' ib-read')}><bdi>{m.subject}</bdi></b>
                  <div className="muted small ib-ellipsis">{f ? b(f.name) : '-'} · {t('inbox.k_' + m.kind)}</div>
                </div>
                <div className="end" style={{ flex: 'none' }}><div className="muted tiny">{ago(m.at)}</div>{m.state === 'new' && <Pill tone="danger">{t('inbox.s_new')}</Pill>}</div>
              </div>
            );
          })}
          {!list.length && <div className="empty">{t('inbox.empty')}</div>}
          {list.length > shown && <div className="center" style={{ padding: 10 }}><button className="btn sm" onClick={() => setShown(s => s + PER)}>{t('inbox.more', { n: num(list.length - shown) })}</button></div>}
        </div>
      </Card>
      <div className="ib-detail">
        {cur ? <Detail key={cur.id} m={cur} onBack={() => open(null)} /> : <Card><div className="empty">{t('inbox.pick')}</div></Card>}
      </div>
    </div>
  );
}

function Detail({ m, onBack }: { m: Message; onBack: () => void }) {
  const { t, b, date } = useI18n();
  const { me } = useAuth();
  const toast = useToast();
  const pn = usePlaceNames();
  const [reply, setReply] = useState(m.reply);
  const f = db.farmers.get(m.farmerId);
  const farm = m.farmId ? db.farms.get(m.farmId) : undefined;
  const set = (p: Partial<Message>, msg: string) => { db.messages.patch(m.id, p); toast(t(msg), 'good'); };
  const send = () => {
    if (!reply.trim()) { toast(t('inbox.reply_empty'), 'warn'); return; }
    set({ reply: reply.trim(), state: 'replied', repliedBy: me?.name ?? '', repliedAt: nowIso() }, 'inbox.replied_toast');
  };
  return (
    <Card>
      <button className="btn ghost sm ib-back" onClick={onBack}><ArrowLeft className="flip-rtl" />{t('inbox.back')}</button>
      <div className="row mb"><Pill tone={TONE[m.state]}>{t('inbox.s_' + m.state)}</Pill><Pill>{t('inbox.k_' + m.kind)}</Pill>
        {m.photos > 0 && <Pill tone="water" icon={<Image />}>{t('inbox.photos', { n: m.photos })}</Pill>}<span className="muted small">{date(m.at, 'datetime')}</span></div>
      <h2 style={{ marginBottom: 10 }}><bdi>{m.subject}</bdi></h2>
      <div className="ib-text" dir="auto">{m.text}</div>

      <div className="ib-who">
        <span className="ico brand"><User /></span>
        <div style={{ flex: 1, minWidth: 0 }}>
          {f ? <>
            <b>{b(f.name)}</b>
            <div className="small muted"><Phone value={f.phone} /> · {pn.dist(f.dist)}, {pn.gov(f.gov)}</div>
            {farm && <div className="small"><MapPin size={12} /> {t('inbox.farm')} #{farm.id} · <span className="ku-text">{farm.name}</span> · {pn.sub(farm.dist, farm.sub)}</div>}
          </> : <span className="muted">{t('inbox.no_farmer')}</span>}
        </div>
        {f && <Link className="btn sm" to={'/admin/farms?farmer=' + f.id}>{t('inbox.open_farmer')}</Link>}
      </div>

      {m.state === 'replied' || m.state === 'closed' ? (m.reply && (
        <div className="mt"><div className="eyebrow">{t('inbox.your_reply')}</div>
          <div className="ib-reply" dir="auto">{m.reply}</div>
          <div className="muted small">{t('inbox.replied_by', { by: m.repliedBy, when: date(m.repliedAt, 'datetime') })}</div></div>
      )) : (
        <div className="mt">
          <Field label={t('inbox.reply')} hint={t('inbox.reply_hint')}>
            <textarea rows={4} dir="auto" value={reply} onChange={e => setReply(e.target.value)} />
          </Field>
        </div>
      )}
      <div className="row mt">
        {(m.state === 'new' || m.state === 'read') && <button className="btn primary" onClick={send}><Send className="flip-rtl" />{t('inbox.send_reply')}</button>}
        {m.state === 'new' && <button className="btn" onClick={() => set({ state: 'read' }, 'inbox.read_toast')}><Eye />{t('inbox.mark_read')}</button>}
        {m.state !== 'closed' && <button className="btn" onClick={() => set({ state: 'closed' }, 'inbox.closed_toast')}><CheckCheck />{t('inbox.close')}</button>}
        {m.state === 'closed' && <button className="btn" onClick={() => set({ state: m.reply ? 'replied' : 'read' }, 'inbox.reopened_toast')}><RotateCcw />{t('inbox.reopen')}</button>}
      </div>
      <p className="muted small">{t('inbox.app_note')}</p>
    </Card>
  );
}

// ---------- the news bar ----------
const blankNews = (order: number): News => ({ id: '', text: { en: '', ku: '' }, where: 'both', active: true, order });

function NewsBar() {
  const { t, b } = useI18n();
  const toast = useToast();
  const news = useRows(db.news);
  const sorted = useMemo(() => [...news].sort((a, z) => a.order - z.order), [news]);
  const [edit, setEdit] = useState<News | null>(null);
  const [del, setDel] = useState<News | null>(null);
  const [err, setErr] = useState('');
  const live = sorted.filter(n => n.active);

  const move = (i: number, d: -1 | 1) => {
    const j = i + d; if (j < 0 || j >= sorted.length) return;
    const arr = [...sorted]; [arr[i], arr[j]] = [arr[j], arr[i]];
    db.news.putMany(arr.map((n, k) => ({ ...n, order: k })));
  };
  const save = () => {
    if (!edit) return;
    if (!edit.text.ku.trim() && !edit.text.en.trim()) { setErr('inbox.news_need'); return; }
    db.news.put({ ...edit, id: edit.id || newId('n') });
    setEdit(null); setErr(''); toast(t('common.saved'), 'good');
  };
  const whereOpts: [string, string][] = [['both', t('inbox.w_both')], ['public', t('inbox.w_public')], ['admin', t('inbox.w_admin')]];

  return (
    <div className="stack">
      <Card title={t('inbox.preview')}>
        <div className="ticker ib-ticker">
          <span className="ticker-tag"><i className="live" />{t('common.news')}</span>
          <div className="ticker-win">
            {live.length ? <div className="mq-track mq-run" style={{ ['--mq-dur' as string]: Math.max(24, live.length * 9) + 's' }}>
              {[0, 1].map(k => <span key={k} aria-hidden={k === 1}>{live.map(n => <span className="mq-item" key={n.id}><span className="sep">●</span><bdi>{b(n.text)}</bdi></span>)}</span>)}
            </div> : <span className="mq-item">{t('inbox.news_none')}</span>}
          </div>
        </div>
        <p className="muted small">{t('inbox.preview_note')}</p>
      </Card>
      <Card title={t('inbox.news_lines', { n: sorted.length })} extra={<button className="btn primary sm" onClick={() => setEdit(blankNews(sorted.length))}><Plus />{t('inbox.news_add')}</button>}>
        {sorted.map((n, i) => (
          <div className="list-item ib-news" key={n.id}>
            <div className="ib-order">
              <button className="btn ghost sm icon" disabled={i === 0} onClick={() => move(i, -1)} aria-label={t('inbox.up')}><ArrowUp /></button>
              <button className="btn ghost sm icon" disabled={i === sorted.length - 1} onClick={() => move(i, 1)} aria-label={t('inbox.down')}><ArrowDown /></button>
            </div>
            <div style={{ flex: 1, minWidth: 0 }}>
              {n.text.ku ? <div dir="rtl" lang="ckb" className="ku-text">{n.text.ku}</div> : <div className="muted small">{t('common.ku_missing')}</div>}
              <div dir="ltr" className="small ink2">{n.text.en}</div>
              <div className="row" style={{ marginTop: 4 }}><Pill tone="brand">{t('inbox.w_' + n.where)}</Pill></div>
            </div>
            <div className="row" style={{ flexWrap: 'nowrap', flex: 'none' }}>
              <Switch on={n.active} onChange={v => db.news.patch(n.id, { active: v })} label={t('common.active')} />
              <button className="btn sm icon" onClick={() => setEdit({ ...n })} aria-label={t('common.edit')}><Pencil /></button>
              <button className="btn sm icon danger" onClick={() => setDel(n)} aria-label={t('common.delete')}><Trash2 /></button>
            </div>
          </div>
        ))}
        {!sorted.length && <div className="empty">{t('inbox.news_none')}</div>}
      </Card>
      {edit && (
        <Modal title={t(edit.id ? 'inbox.news_edit' : 'inbox.news_add')} onClose={() => { setEdit(null); setErr(''); }}
          foot={<><button className="btn" onClick={() => { setEdit(null); setErr(''); }}>{t('common.cancel')}</button><button className="btn primary" onClick={save}>{t('common.save')}</button></>}>
          <div className="form-grid">
            <Field label={t('inbox.news_ku')} full><input type="text" dir="rtl" lang="ckb" className="ku-text" value={edit.text.ku} onChange={e => setEdit({ ...edit, text: { ...edit.text, ku: e.target.value } })} /></Field>
            <Field label={t('inbox.news_en')} full><input type="text" dir="ltr" value={edit.text.en} onChange={e => setEdit({ ...edit, text: { ...edit.text, en: e.target.value } })} /></Field>
            <Field label={t('inbox.news_where')}><Select value={edit.where} onChange={v => setEdit({ ...edit, where: v as News['where'] })} options={whereOpts} /></Field>
            <Field label={t('common.active')}><Switch on={edit.active} onChange={v => setEdit({ ...edit, active: v })} /></Field>
          </div>
          {err && <div className="mt"><Note tone="danger">{t(err)}</Note></div>}
        </Modal>
      )}
      {del && <Confirm title={t('inbox.news_delete')} text={<bdi>{b(del.text)}</bdi>} okLabel={t('common.delete')} onClose={() => setDel(null)}
        onOk={() => { db.news.remove(del.id); toast(t('common.deleted')); }} />}
    </div>
  );
}
