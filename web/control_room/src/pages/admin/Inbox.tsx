// Inbox (design 13): farmers' messages from the app, newest first, with a reply that reaches the farmer
// in the app. Routes: /dashboard/messages (FRONTEND.md 9 Inbox). Topic: messages.
import { useEffect, useRef, useState } from 'react';
import { Link, useSearchParams } from 'react-router-dom';
import { Send, CheckCheck, Eye, User, Trash2, ArrowRight, RotateCcw, ImageOff } from 'lucide-react';
import { useI18n } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { api, qs, authedBlobUrl } from '../../api/client';
import { useApi, invalidate } from '../../api/cache';
import type { Paged } from '../../api/types';
import { PageHead, Pill, Select, Confirm, useDebounced, type Tone } from '../../components/ui';
import { Phone, StateBox, usePlace, usePlaceOptions, useErrorText } from '../../components/domain';
import { useAction } from './inbox/useAction';
import './inbox.css';

type State = 'new' | 'read' | 'replied' | 'closed';
type Kind = 'question' | 'report' | 'complaint' | 'request' | 'other';
interface Photo { id: string; content_type: string; size: number; url: string }
interface Msg {
  id: string; created_at: string; updated_at: string; kind: Kind; state: State; text: string; photos: Photo[];
  farmer_id: string; farmer_name?: string | null; farmer_phone?: string | null;
  farm_id?: string | null; farm_name?: string | null; governorate?: string | null; zone_slug?: string | null;
  reply?: { text_ku: string; text_en?: string | null; replied_at: string; replied_by: string } | null;
}
const STATE_TONE: Record<State, Tone> = { new: 'warn', read: '', replied: 'good', closed: 'water' };
const KIND_TONE: Record<Kind, Tone> = { question: 'brand', report: 'danger', complaint: 'warn', request: 'water', other: '' };
const PER = 30;

export default function Inbox() {
  const { t, num } = useI18n();
  const { can } = useAuth();
  const [params, setParams] = useSearchParams();
  const [state, setState] = useState<State | ''>('new');
  const [kind, setKind] = useState('');
  const [gov, setGov] = useState('');
  const [q, setQ] = useState('');
  const [page, setPage] = useState(1);
  const dq = useDebounced(q.trim(), 300);
  const sel = params.get('m');
  const opts = usePlaceOptions(gov);

  useEffect(() => { setPage(1); }, [state, kind, gov, dq]);
  const canRead = can('messages');
  const counts = useApi<Record<State, number>>('/dashboard/messages/counts', ['messages'], { auth: true });
  const list = useApi<Paged & { messages: Msg[] }>('/dashboard/messages' + qs({ state, kind, governorate: gov, q: dq, page, rows_per_page: PER }), ['messages'], { auth: true });
  const rows = list.data?.messages ?? [];
  const pages = Math.max(1, Math.ceil((list.data?.count ?? 0) / PER));
  const open = (id: string | null) => setParams(id ? { m: id } : {}, { replace: false });
  const c = counts.data;

  if (!canRead || (list.error?.status === 403 && !list.data)) return <div className="inbox-page"><PageHead eyebrow={t('nav.g_act')} title={t('nav.inbox')} /><div className="card"><StateBox kind="locked" /></div></div>;

  const chips: [State | '', string][] = [['new', t('inbox.s_new')], ['read', t('inbox.s_read')], ['replied', t('inbox.s_replied')], ['closed', t('inbox.s_closed')], ['', t('inbox.s_all')]];
  return (
    <div className="inbox-page">
      <PageHead eyebrow={t('nav.g_act')} title={t('nav.inbox')} sub={t('inbox.sub')} />
      <div className="inbox-chips" role="tablist">
        {chips.map(([k, label]) => (
          <button key={k || 'all'} className={'chip' + (state === k ? ' on' : '')} onClick={() => setState(k)} role="tab" aria-selected={state === k}>
            {label}{k && c ? <b className="tabular"> ({num(c[k as State])})</b> : null}
          </button>
        ))}
      </div>
      <div className={'inbox-panes' + (sel ? ' has-sel' : '')}>
        <section className="card inbox-list" aria-label={t('inbox.list')}>
          <div className="inbox-filters">
            <input type="search" value={q} onChange={e => setQ(e.target.value)} placeholder={t('inbox.search')} />
            <div className="row" style={{ flexWrap: 'nowrap' }}>
              <Select value={kind} onChange={setKind} options={[['', t('inbox.k_all')], ...(['question', 'report', 'complaint', 'request', 'other'] as Kind[]).map(k => [k, t('inbox.k_' + k)] as [string, string])]} />
              <Select value={gov} onChange={setGov} options={[['', t('common.all_govs')], ...opts.govs]} />
            </div>
          </div>
          {list.error && !list.data ? <StateBox kind="error" action={<button className="btn sm" onClick={list.reload}><RotateCcw />{t('common.retry')}</button>} />
            : list.loading ? <div className="sk-rows">{Array.from({ length: 6 }, (_, i) => <i key={i} className="sk" />)}</div>
              : !rows.length ? <StateBox kind="empty" title={t('inbox.empty_title')} text={t('inbox.empty_text')} />
                : rows.map(m => <ListItem key={m.id} m={m} on={m.id === sel} onClick={() => open(m.id)} />)}
          {pages > 1 && (
            <div className="pager">
              <span className="muted small">{t('table.page', { a: num(page), b: num(pages), n: num(list.data?.count ?? 0) })}</span>
              <div className="row">
                <button className="btn sm" disabled={page <= 1} onClick={() => setPage(p => p - 1)}>{t('table.prev')}</button>
                <button className="btn sm" disabled={page >= pages} onClick={() => setPage(p => p + 1)}>{t('table.next')}</button>
              </div>
            </div>
          )}
        </section>
        <section className="inbox-detail">
          {sel ? <Detail key={sel} id={sel} canWrite={can('messages', 'update')} canDelete={can('messages', 'delete')} onClose={() => open(null)} />
            : <div className="card"><StateBox kind="empty" title={t('inbox.pick_title')} text={t('inbox.pick_text')} /></div>}
        </section>
      </div>
    </div>
  );
}

function ListItem({ m, on, onClick }: { m: Msg; on: boolean; onClick: () => void }) {
  const { t, ago } = useI18n();
  const place = usePlace();
  const where = [m.farmer_name || t('inbox.no_name'), place.dist(m.zone_slug) || place.gov(m.governorate)].filter(Boolean).join(' · ');
  return (
    <button className={'list-item click inbox-item' + (on ? ' sel' : '')} onClick={onClick} aria-current={on}>
      <span className="inbox-meta">
        <span className="small muted nowrap">{ago(m.created_at)}</span>
        {m.state === 'new' && <i className="new-dot" aria-label={t('inbox.s_new')} />}
      </span>
      <span className="inbox-text">
        <b className={m.state === 'new' ? '' : 'read'} dir="auto">{m.text.length > 70 ? m.text.slice(0, 70) + '…' : m.text}</b>
        <span className="small muted">{where} · {t('inbox.k_' + m.kind)}</span>
      </span>
    </button>
  );
}

function Detail({ id, canWrite, canDelete, onClose }: { id: string; canWrite: boolean; canDelete: boolean; onClose: () => void }) {
  const { t, date, pick } = useI18n();
  const place = usePlace();
  const errText = useErrorText();
  const one = useApi<{ message: Msg }>('/dashboard/messages/' + id, ['messages'], { auth: true });
  const m = one.data?.message;
  const { busy, run } = useAction();
  const [ku, setKu] = useState('');
  const [en, setEn] = useState('');
  const [confirmDel, setConfirmDel] = useState(false);
  const marked = useRef(false);
  // The cached copy may be old (someone replied meanwhile): ask the server again when the message opens
  // and decide on "mark read" only from that fresh answer, so a replied message is never set back to read.
  const [fresh, setFresh] = useState(false);
  const sawRefresh = useRef(false);
  useEffect(() => { one.reload(); }, []); // eslint-disable-line react-hooks/exhaustive-deps
  useEffect(() => {
    if (one.refreshing) sawRefresh.current = true;
    else if (sawRefresh.current && !one.error) setFresh(true);
  }, [one.refreshing, one.error]);

  // a new message is marked read once, through the same queue as the buttons (one write at a time)
  useEffect(() => {
    if (!fresh || !m || m.state !== 'new' || !canWrite || marked.current) return;
    marked.current = true;
    run(() => api.put('/dashboard/messages/' + m.id, { state: 'read' }).then(() => invalidate('messages')));
  }, [fresh, m, canWrite, run]);
  // the reply draft starts from the saved reply of this message (Detail is keyed by message id, so a
  // different message always starts with its own text)
  useEffect(() => { setKu(m?.reply?.text_ku ?? ''); setEn(m?.reply?.text_en ?? ''); }, [m?.reply?.replied_at]); // eslint-disable-line react-hooks/exhaustive-deps

  if (one.error && !m) return <div className="card"><StateBox kind={one.error.status === 404 ? 'empty' : one.error.status === 403 ? 'locked' : 'error'} text={errText(one.error)} /></div>;
  if (!m) return <div className="card"><div className="sk-rows">{Array.from({ length: 8 }, (_, i) => <i key={i} className="sk" />)}</div></div>;

  const setStateTo = (s: State) => run(() => api.put('/dashboard/messages/' + m.id, { state: s }).then(() => invalidate('messages')), t('common.saved'));
  const reply = () => {
    if (!ku.trim()) return;
    run(() => api.post('/dashboard/messages/' + m.id + '/reply', { text_ku: ku.trim(), text_en: en.trim() || null }).then(() => invalidate('messages')), t('inbox.sent'));
  };
  const del = () => run(() => api.del('/dashboard/messages/' + m.id).then(() => { invalidate('messages'); onClose(); }), t('common.deleted'));
  const where = [m.farm_name, place.dist(m.zone_slug), place.gov(m.governorate)].filter(Boolean).join('، ');

  return (
    <article className="card inbox-msg">
      <div className="msg-head">
        <button className="btn sm back-btn" onClick={onClose}><ArrowRight className="flip-rtl" />{t('common.back')}</button>
        <div className="msg-title">
          <div className="row">
            <Pill tone={KIND_TONE[m.kind]}>{t('inbox.k_' + m.kind)}</Pill>
            <Pill tone={STATE_TONE[m.state]}>{t('inbox.s_' + m.state)}</Pill>
          </div>
          <h2 dir="auto">{m.farmer_name || t('inbox.no_name')}</h2>
          <div className="muted small"><Phone value={m.farmer_phone} />{where && <> · {where}</>} · {date(m.created_at, 'datetime')}</div>
        </div>
        <div className="row msg-actions">
          <Link className="btn sm" to={'/admin/farms?farmer=' + m.farmer_id}><User />{t('inbox.open_farmer')}</Link>
          {canWrite && m.state !== 'closed' && <button className="btn sm" disabled={busy} onClick={() => setStateTo('closed')}><CheckCheck />{t('inbox.close')}</button>}
          {canWrite && m.state === 'closed' && <button className="btn sm" disabled={busy} onClick={() => setStateTo(m.reply ? 'replied' : 'read')}><RotateCcw />{t('inbox.reopen')}</button>}
          {canDelete && <button className="btn sm danger" disabled={busy} onClick={() => setConfirmDel(true)} aria-label={t('common.delete')}><Trash2 /></button>}
        </div>
      </div>
      <div className="farmer-wrote">
        <span className="eyebrow">{t('inbox.farmer_wrote')}</span>
        <p dir="auto">{m.text}</p>
      </div>
      {m.photos.length > 0 && <Photos msgId={m.id} photos={m.photos} />}
      {m.reply && (
        <div className="note good">
          <span><b>{t('inbox.your_reply')}</b> · <span className="small">{date(m.reply.replied_at, 'datetime')}</span><br /><span dir="auto">{pick(m.reply.text_ku, m.reply.text_en)}</span></span>
        </div>
      )}
      {canWrite ? (
        <div className="reply-box">
          <label className="field"><span>{t('inbox.reply_ku')}</span>
            <textarea dir="rtl" rows={3} value={ku} onChange={e => setKu(e.target.value)} placeholder={t('inbox.reply_ph')} maxLength={2000} />
          </label>
          <label className="field"><span>{t('inbox.reply_en')} <small>{t('inbox.optional')}</small></span>
            <textarea dir="ltr" rows={2} value={en} onChange={e => setEn(e.target.value)} maxLength={2000} />
          </label>
          <div className="row">
            <button className="btn primary" disabled={busy || !ku.trim()} onClick={reply}><Send className="flip-rtl" />{m.reply ? t('inbox.replace_reply') : t('inbox.send')}</button>
            {m.state === 'new' && <button className="btn" disabled={busy} onClick={() => setStateTo('read')}><Eye />{t('inbox.mark_read')}</button>}
          </div>
          <span className="muted small">{t('inbox.reply_note')}</span>
        </div>
      ) : <div className="note">{t('inbox.read_only')}</div>}
      {confirmDel && <Confirm title={t('inbox.del_title')} text={t('inbox.del_text')} okLabel={t('common.delete')} onOk={del} onClose={() => setConfirmDel(false)} />}
    </article>
  );
}

/** Photos sit behind the token: fetch the bytes, show them, free them when the message closes. */
function Photos({ msgId, photos }: { msgId: string; photos: Photo[] }) {
  const { t } = useI18n();
  const [urls, setUrls] = useState<Record<string, string | null>>({});
  useEffect(() => {
    let dead = false;
    const made: string[] = [];
    for (const p of photos) {
      authedBlobUrl(`/dashboard/messages/${msgId}/photos/${p.id}`).then(u => { if (dead) { URL.revokeObjectURL(u); return; } made.push(u); setUrls(x => ({ ...x, [p.id]: u })); }, () => !dead && setUrls(x => ({ ...x, [p.id]: null })));
    }
    return () => { dead = true; made.forEach(u => URL.revokeObjectURL(u)); setUrls({}); };
  }, [msgId, photos]);
  return (
    <div className="msg-photos">
      {photos.map(p => {
        const u = urls[p.id];
        return u ? <a key={p.id} href={u} target="_blank" rel="noreferrer"><img src={u} alt={t('inbox.photo')} /></a>
          : <span key={p.id} className={u === null ? 'photo-fail' : 'sk photo-sk'}>{u === null && <ImageOff />}</span>;
      })}
    </div>
  );
}
