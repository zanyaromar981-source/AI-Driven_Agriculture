// Alwa market (design 14): what farmers put on sale in the app, in detail. The government sets no price:
// averages are worked out from the listings. Staff may close or remove a listing (alwa:update/delete).
// Routes: /dashboard/alwa/listings (moderation), public /alwa/markets and their price boards (FRONTEND.md 5, 9).
import { useEffect, useMemo, useState } from 'react';
import { Store, Package, Handshake, Wheat, RotateCcw, Ban, Trash2, Info } from 'lucide-react';
import { useI18n } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { api, qs } from '../../api/client';
import { useApi, invalidate } from '../../api/cache';
import type { Paged } from '../../api/types';
import { PageHead, Kpi, Note, Pill, Tabs, Select, Drawer, Modal, Field, Confirm, useDebounced, type Tone } from '../../components/ui';
import { DataTable, type Col } from '../../components/DataTable';
import { HBars } from '../../components/charts';
import { CropTag, Phone, StateBox, cropColor, cropName, usePlace, useCrops } from '../../components/domain';
import { CROPS } from '../../data/crops';
import { useAction } from './inbox/useAction';
import { Markets } from './alwa/Markets';
import './alwa.css';

type Status = 'open' | 'sold' | 'closed' | 'cancelled';
interface Listing {
  id: string; crop: string; quantity_kg: number; asking_price_iqd_per_kg: number; fair_price: 'fair' | 'high' | 'low' | 'unknown';
  grade?: 'a' | 'b' | 'c' | null; market: string; pickup: 'farm' | 'alwa'; seller_name?: string | null; seller_phone: string; status: Status;
  open_offers: number; zone_slug?: string | null; note?: string | null; moderation_note?: string | null; closes_at: string; created_at: string; updated_at: string;
}
interface Offer { id: string; buyer_kind: string; buyer_name: string; buyer_phone: string; price_iqd_per_kg: number; quantity_kg: number; status: 'open' | 'accepted' | 'declined' | 'withdrawn'; created_at: string }
interface Market { slug: string; name_en: string; name_ku: string }
/** wide screens (design: 1200px and up) show the listings and the average prices side by side */
function useWide() {
  const mq = '(min-width: 1200px)';
  const [wide, setWide] = useState(() => typeof window !== 'undefined' && window.matchMedia(mq).matches);
  useEffect(() => { const m = window.matchMedia(mq); const f = () => setWide(m.matches); m.addEventListener('change', f); return () => m.removeEventListener('change', f); }, []);
  return wide;
}
const STATUS_TONE: Record<Status, Tone> = { open: 'good', sold: 'water', closed: '', cancelled: 'warn' };
const FAIR_TONE: Record<Listing['fair_price'], Tone> = { fair: 'good', high: 'warn', low: 'warn', unknown: '' };
const PER = 25;

export default function Alwa() {
  const { t, num } = useI18n();
  const { can } = useAuth();
  const wide = useWide();
  useCrops(); // the crop filter and price form use the server's crop list
  const [tab, setTab] = useState<'listings' | 'prices' | 'markets'>('listings');
  const markets = useApi<{ markets: Market[] }>('/alwa/markets', ['alwa_prices']);
  const openQ = useApi<Paged>('/dashboard/alwa/listings' + qs({ status: 'open', rows_per_page: 1 }), ['alwa_listings'], { auth: true });
  const soldQ = useApi<Paged>('/dashboard/alwa/listings' + qs({ status: 'sold', rows_per_page: 1 }), ['alwa_listings'], { auth: true });
  // the latest 100 listings of every state: the averages below are worked out from these
  const sample = useApi<Paged & { listings: Listing[] }>('/dashboard/alwa/listings' + qs({ rows_per_page: 100 }), ['alwa_listings'], { auth: true });
  const avg = useMemo(() => averages(sample.data?.listings ?? []), [sample.data]);
  const wheat = avg.find(a => a.crop === 'wheat');
  // tonnes on sale: every open listing, read 100 at a time (the server's largest page)
  const [openPage, setOpenPage] = useState(1);
  const openRows = useApi<Paged & { listings: Listing[] }>('/dashboard/alwa/listings' + qs({ status: 'open', page: openPage, rows_per_page: 100 }), ['alwa_listings'], { auth: true });
  const [openKgByPage, setOpenKgByPage] = useState<Record<number, number>>({});
  useEffect(() => {
    const d = openRows.data; if (!d) return;
    setOpenKgByPage(x => ({ ...(openPage === 1 ? {} : x), [openPage]: d.listings.reduce((a, l) => a + l.quantity_kg, 0) }));
    if (openPage * 100 < d.count && openPage < 20) setOpenPage(openPage + 1);
  }, [openRows.data]); // eslint-disable-line react-hooks/exhaustive-deps
  useEffect(() => { setOpenPage(1); }, [openQ.data?.count]);
  const openKg = Object.values(openKgByPage).reduce((a, b) => a + b, 0);
  const tonnes = (kg: number) => (kg ? num(kg / 1000, kg < 1000 ? 1 : 0) : num(0));
  const fullCount = openRows.data ? openPage * 100 >= openRows.data.count : false;

  if (!can('alwa') || (sample.error?.status === 403 && !sample.data)) return <div className="alwa-page"><PageHead eyebrow={t('nav.g_act')} title={t('nav.alwa')} /><div className="card"><StateBox kind="locked" /></div></div>;
  const mk = markets.data?.markets ?? [];
  const prices = <Prices avg={avg} sampleSize={sample.data?.listings.length ?? 0} loading={sample.loading} />;

  return (
    <div className="alwa-page">
      <PageHead eyebrow={t('nav.g_act')} title={t('nav.alwa')} sub={t('alwa.sub')} />
      <Note tone="info" icon={<Info />}>{t('alwa.not_live')}</Note>
      <div className="grid g4 mb">
        <Kpi label={t('alwa.k_open')} value={openQ.data ? num(openQ.data.count) : '…'} note={t('alwa.k_open_note', { t: tonnes(openKg) })} icon={<Store />} />
        <Kpi label={t('alwa.k_sold')} value={soldQ.data ? num(soldQ.data.count) : '…'} note={t('alwa.k_sold_note')} icon={<Handshake />} />
        <Kpi label={t('alwa.k_tonnes')} value={openRows.data ? tonnes(openKg) : '…'} note={fullCount ? t('alwa.k_tonnes_note') : t('alwa.k_tonnes_counting')} icon={<Package />} />
        <Kpi label={t('alwa.k_wheat')} value={wheat?.ask ? num(Math.round(wheat.ask)) : '-'} note={wheat?.ask ? t('alwa.iqd_kg') : t('alwa.no_wheat')} tone="brand" icon={<Wheat />} />
      </div>
      {wide ? (
        <>
          <div className="alwa-split"><Listings markets={mk} />{prices}</div>
          <div style={{ marginTop: 14 }}><Markets /></div>
        </>
      ) : (
        <>
          <Tabs value={tab} onChange={setTab} items={[['listings', t('alwa.tab_listings')], ['prices', t('alwa.tab_prices')], ['markets', t('alwa.tab_markets')]]} />
          {tab === 'listings' ? <Listings markets={mk} /> : tab === 'prices' ? prices : <Markets />}
        </>
      )}
    </div>
  );
}

interface Avg { crop: string; ask: number | null; askN: number; sold: number | null; soldN: number; kg: number }
function averages(ls: Listing[]): Avg[] {
  const m = new Map<string, { a: number; an: number; s: number; sn: number; kg: number }>();
  for (const l of ls) {
    const x = m.get(l.crop) ?? { a: 0, an: 0, s: 0, sn: 0, kg: 0 };
    if (l.status === 'open') { x.a += l.asking_price_iqd_per_kg; x.an++; x.kg += l.quantity_kg; }
    if (l.status === 'sold') { x.s += l.asking_price_iqd_per_kg; x.sn++; }
    m.set(l.crop, x);
  }
  return [...m].map(([crop, x]) => ({ crop, ask: x.an ? x.a / x.an : null, askN: x.an, sold: x.sn ? x.s / x.sn : null, soldN: x.sn, kg: x.kg }))
    .sort((a, b) => (b.askN + b.soldN) - (a.askN + a.soldN));
}

function Listings({ markets }: { markets: Market[] }) {
  const { t, num, date, pick } = useI18n();
  const place = usePlace();
  const [crop, setCrop] = useState('');
  const [status, setStatus] = useState('');
  const [market, setMarket] = useState('');
  const [phone, setPhone] = useState('');
  const [page, setPage] = useState(1);
  const [sel, setSel] = useState<string | null>(null);
  const dPhone = useDebounced(phone.replace(/\s/g, ''), 400);
  const q = useApi<Paged & { listings: Listing[] }>('/dashboard/alwa/listings' + qs({ crop, status, market, seller_phone: dPhone, page, rows_per_page: PER }), ['alwa_listings'], { auth: true });
  const mName = (s: string) => { const m = markets.find(x => x.slug === s); return m ? pick(m.name_ku, m.name_en) : s; };
  const reset = (f: (v: string) => void) => (v: string) => { f(v); setPage(1); };
  const cols: Col<Listing>[] = [
    { key: 'crop', label: t('alwa.crop'), cell: l => <span><CropTag code={l.crop} />{l.grade && <span className="muted small"> · {t('alwa.grade')} {l.grade.toUpperCase()}</span>}</span> },
    { key: 'seller', label: t('alwa.seller'), cell: l => <div><b dir="auto">{l.seller_name || t('alwa.no_name')}</b><div className="small muted"><Phone value={l.seller_phone} /></div></div> },
    { key: 'place', label: t('alwa.place'), cell: l => <div>{mName(l.market)}<div className="small muted">{place.dist(l.zone_slug)}</div></div> },
    { key: 'qty', label: t('alwa.qty'), num: true, cell: l => <span className="tabular">{l.quantity_kg >= 1000 ? num(l.quantity_kg / 1000, 1) + ' ' + t('common.tonnes') : num(l.quantity_kg) + ' ' + t('common.kg')}</span> },
    { key: 'price', label: t('alwa.price'), num: true, cell: l => <div><b className="tabular">{num(l.asking_price_iqd_per_kg)}</b>{l.fair_price !== 'unknown' && <div><Pill tone={FAIR_TONE[l.fair_price]}>{t('alwa.fair_' + l.fair_price)}</Pill></div>}</div> },
    { key: 'offers', label: t('alwa.offers'), num: true, cell: l => num(l.open_offers) },
    { key: 'closes', label: t('alwa.closes'), cell: l => <span className="small">{date(l.closes_at, 'short')}</span>, optional: true },
    { key: 'status', label: t('common.status'), cell: l => <Pill tone={STATUS_TONE[l.status]}>{t('alwa.s_' + l.status)}</Pill> },
  ];
  return (
    <div className="card">
      <div className="filters">
        <input className="grow" type="tel" value={phone} onChange={e => reset(setPhone)(e.target.value)} placeholder={t('alwa.f_phone')} />
        <Select value={crop} onChange={reset(setCrop)} options={[['', t('common.all_crops')], ...CROPS.filter(c => c.code !== 'empty').map(c => [c.code, pick(c.ku, c.en)] as [string, string])]} />
        <Select value={status} onChange={reset(setStatus)} options={[['', t('alwa.all_states')], ...(['open', 'sold', 'closed', 'cancelled'] as Status[]).map(s => [s, t('alwa.s_' + s)] as [string, string])]} />
        <Select value={market} onChange={reset(setMarket)} options={[['', t('alwa.all_markets')], ...markets.map(m => [m.slug, pick(m.name_ku, m.name_en)] as [string, string])]} />
      </div>
      {q.error?.status === 403 && !q.data ? <StateBox kind="locked" />
        : q.error && !q.data ? <StateBox kind="error" action={<button className="btn sm" onClick={q.reload}><RotateCcw />{t('common.retry')}</button>} />
        : !q.loading && !q.data?.listings.length && !crop && !status && !market && !dPhone ? <StateBox kind="empty" title={t('alwa.empty_title')} text={t('alwa.empty_text')} />
          : <DataTable id="alwa-listings" rows={q.data?.listings ?? []} cols={cols} per={PER} loading={q.loading} onRow={l => setSel(l.id)} selected={sel}
            server={{ total: q.data?.count ?? 0, page, onPage: setPage }} />}
      {sel && <ListingDrawer id={sel} mName={mName} onClose={() => setSel(null)} />}
    </div>
  );
}

function ListingDrawer({ id, mName, onClose }: { id: string; mName: (s: string) => string; onClose: () => void }) {
  const { t, num, date } = useI18n();
  const { can } = useAuth();
  const place = usePlace();
  const q = useApi<{ listing: Listing & { offers?: Offer[] } }>('/dashboard/alwa/listings/' + id, ['alwa_listings'], { auth: true });
  const l = q.data?.listing;
  const { busy, run } = useAction();
  const [closing, setClosing] = useState(false);
  const [note, setNote] = useState('');
  const [del, setDel] = useState(false);
  const closeIt = () => run(async () => { await api.put('/dashboard/alwa/listings/' + id, { status: 'closed', note: note.trim() || null }); invalidate('alwa_listings'); setClosing(false); }, t('alwa.closed_ok'));
  const remove = () => run(async () => { await api.del('/dashboard/alwa/listings/' + id); invalidate('alwa_listings'); onClose(); }, t('common.deleted'));
  return (
    <Drawer eyebrow={t('alwa.listing') + ' #' + id} title={l ? <CropTag code={l.crop} /> : '…'} onClose={onClose}>
      {!l ? (q.error ? <StateBox kind="error" /> : <div className="sk-rows">{Array.from({ length: 8 }, (_, i) => <i key={i} className="sk" />)}</div>) : (
        <div className="stack">
          <div className="row"><Pill tone={STATUS_TONE[l.status]}>{t('alwa.s_' + l.status)}</Pill>{l.fair_price !== 'unknown' && <Pill tone={FAIR_TONE[l.fair_price]}>{t('alwa.fair_' + l.fair_price)}</Pill>}</div>
          <dl className="facts">
            <dt>{t('alwa.seller')}</dt><dd dir="auto">{l.seller_name || t('alwa.no_name')}</dd>
            <dt>{t('alwa.phone')}</dt><dd><Phone value={l.seller_phone} /></dd>
            <dt>{t('alwa.qty')}</dt><dd>{num(l.quantity_kg)} {t('common.kg')}</dd>
            <dt>{t('alwa.price')}</dt><dd>{num(l.asking_price_iqd_per_kg)} {t('common.iqd_kg')}</dd>
            <dt>{t('alwa.grade')}</dt><dd>{l.grade ? l.grade.toUpperCase() : '-'}</dd>
            <dt>{t('alwa.market')}</dt><dd>{mName(l.market)}</dd>
            <dt>{t('common.district')}</dt><dd>{place.dist(l.zone_slug) || '-'}</dd>
            <dt>{t('alwa.pickup')}</dt><dd>{t('alwa.pickup_' + l.pickup)}</dd>
            <dt>{t('alwa.posted')}</dt><dd>{date(l.created_at, 'datetime')}</dd>
            <dt>{t('alwa.closes')}</dt><dd>{date(l.closes_at, 'datetime')}</dd>
          </dl>
          {l.note && <div className="note"><span dir="auto">{l.note}</span></div>}
          {l.moderation_note && <div className="note warn"><span><b>{t('alwa.mod_note')}</b> <span dir="auto">{l.moderation_note}</span></span></div>}
          <h3>{t('alwa.offers')} ({num(l.offers?.length ?? 0)})</h3>
          {!l.offers?.length ? <p className="muted small">{t('alwa.no_offers')}</p> : (
            <div className="offers">
              {l.offers.map(o => (
                <div key={o.id} className="offer">
                  <div className="spread"><b dir="auto">{o.buyer_name}</b><Pill tone={o.status === 'accepted' ? 'good' : o.status === 'open' ? 'brand' : ''}>{t('alwa.o_' + o.status)}</Pill></div>
                  <div className="small muted">{t('alwa.b_' + o.buyer_kind)} · <Phone value={o.buyer_phone} /></div>
                  <div className="small">{num(o.quantity_kg)} {t('common.kg')} · <b>{num(o.price_iqd_per_kg)}</b> {t('common.iqd_kg')}</div>
                </div>
              ))}
            </div>
          )}
          <div className="row">
            {l.status === 'open' && can('alwa', 'update') && <button className="btn" disabled={busy} onClick={() => setClosing(true)}><Ban />{t('alwa.close')}</button>}
            {l.status !== 'sold' && can('alwa', 'delete') && <button className="btn danger" disabled={busy} onClick={() => setDel(true)}><Trash2 />{t('common.delete')}</button>}
          </div>
        </div>
      )}
      {closing && (
        <Modal title={t('alwa.close_title')} onClose={() => setClosing(false)} foot={<>
          <button className="btn" onClick={() => setClosing(false)}>{t('common.cancel')}</button>
          <button className="btn primary" disabled={busy} onClick={closeIt}>{t('alwa.close')}</button>
        </>}>
          <p className="muted" style={{ marginTop: 0 }}>{t('alwa.close_text')}</p>
          <Field label={t('alwa.close_note')} full><textarea rows={3} value={note} onChange={e => setNote(e.target.value)} maxLength={500} /></Field>
        </Modal>
      )}
      {del && <Confirm title={t('alwa.del_title')} text={t('alwa.del_text')} okLabel={t('common.delete')} onOk={remove} onClose={() => setDel(false)} />}
    </Drawer>
  );
}

function Prices({ avg, sampleSize, loading }: { avg: Avg[]; sampleSize: number; loading: boolean }) {
  const { t, num, lang } = useI18n();
  const withAsk = avg.filter(a => a.ask != null);
  return (
    <div className="alwa-prices">
      <div className="card">
        <div className="card-head"><span className="eyebrow">{t('alwa.avg_title')}</span></div>
        {loading ? <div className="sk-rows">{Array.from({ length: 6 }, (_, i) => <i key={i} className="sk" />)}</div>
          : !avg.length ? <StateBox kind="empty" title={t('alwa.avg_empty')} text={t('alwa.empty_text')} /> : (
            <>
              <div className="table-wrap">
                <table className="t cards">
                  <thead><tr><th>{t('alwa.crop')}</th><th className="num">{t('alwa.avg_ask')}</th><th className="num">{t('alwa.avg_sold')}</th><th className="num">{t('alwa.k_open')}</th><th className="num">{t('alwa.qty_open')}</th></tr></thead>
                  <tbody>{avg.map(a => (
                    <tr key={a.crop}>
                      <td data-label={t('alwa.crop')}><CropTag code={a.crop} /></td>
                      <td data-label={t('alwa.avg_ask')} className="num">{a.ask != null ? <b>{num(Math.round(a.ask))}</b> : '-'}</td>
                      <td data-label={t('alwa.avg_sold')} className="num">{a.sold != null ? num(Math.round(a.sold)) : '-'}</td>
                      <td data-label={t('alwa.k_open')} className="num">{num(a.askN)}</td>
                      <td data-label={t('alwa.qty_open')} className="num">{a.kg ? num(a.kg / 1000, 1) : num(0)} {t('common.tonnes')}</td>
                    </tr>))}</tbody>
                </table>
              </div>
              {withAsk.length > 0 && <div style={{ marginTop: 14 }}><HBars unit={t('common.iqd_kg')} items={withAsk.map(a => ({ key: a.crop, label: cropName(a.crop, lang), value: Math.round(a.ask!), color: cropColor(a.crop) }))} /></div>}
            </>
          )}
        <p className="muted small">{t('alwa.avg_note', { n: num(sampleSize) })}</p>
      </div>
    </div>
  );
}
