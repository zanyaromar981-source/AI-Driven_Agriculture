// Alwa market: what farmers put on sale in the app, seen in detail. Read only: the government does not
// set prices and nothing here approves or removes a listing. Average prices come from the listings.
// Cost: the averages are one memoized pass over the listings (api.marketPrices); filters are one pass.
import { useMemo, useState } from 'react';
import { Package, Scale, Handshake, Wheat, Info, Eye, Camera, MessageSquare } from 'lucide-react';
import { db, PLACES, ensureProducts } from '../../data/db';
import { useRows, useVersion } from '../../data/store';
import { downloadCsv, marketPrices } from '../../data/api';
import { GROUPS, useProductSync } from '../../data/backend';
import type { Listing, ListingState, ProductUnit } from '../../data/types';
import { useI18n } from '../../i18n';
import { PageHead, Card, Kpi, Pill, Note, Tabs, Select, Drawer, useDebounced, type Tone } from '../../components/ui';
import { DataTable, type Col } from '../../components/DataTable';
import { HBars } from '../../components/charts';
import { CropTag, Phone, usePlaceNames } from '../../components/domain';

const STATE_TONE: Record<ListingState, Tone> = { open: 'good', sold: 'brand', expired: '', cancelled: 'danger' };
const DAY = 864e5;
ensureProducts();

/** The unit a listing is counted and priced in: its own, else its product's, else kg (the old listings). */
const unitOf = (l: Listing): ProductUnit => l.unit ?? db.crops.get(l.crop)?.unit ?? 'kg';
const unitOfProduct = (code: string): ProductUnit => db.crops.get(code)?.unit ?? 'kg';
const groupOf = (code: string) => db.crops.get(code)?.group ?? 'crops';
/** "12 trays", "3 head", "40 litres", "500 kg" */
function Qty({ n, unit }: { n: number; unit: ProductUnit }) {
  const { t, num } = useI18n();
  return <span className="nowrap">{num(n)} {t('alwa.u_' + unit)}</span>;
}

/** Reference price for a crop: average asking price of open listings, else average sold price. */
function refPrice(crop: string) {
  const p = marketPrices().get(crop);
  return p?.avgAsk ?? p?.avgSold ?? null;
}
function VsAvg({ l }: { l: Listing }) {
  const { t, num } = useI18n();
  const ref = refPrice(l.crop);
  if (!ref) return <span className="muted">-</span>;
  const pct = l.price / ref * 100, far = pct > 125 || pct < 75;
  return far ? <Pill tone="warn">{t('alwa.vs', { p: num(pct) })}</Pill> : <span className="muted small">{t('alwa.vs', { p: num(pct) })}</span>;
}

export default function Alwa() {
  const { t, num } = useI18n();
  const [tab, setTab] = useState<'prices' | 'listings'>('prices');
  const synced = useProductSync();
  return (
    <>
      <PageHead eyebrow={t('nav.g_act')} title={t('alwa.title')} sub={t('alwa.sub')} />
      <div className="mb"><Note tone="warn" icon={<Info />}>{t('alwa.not_live')}</Note>
        {synced !== undefined && <p className="muted small" style={{ margin: '6px 0 0' }}>{synced ? t('alwa.products_live', { n: num(synced) }) : t('alwa.products_sample')}</p>}</div>
      <Kpis />
      <Tabs value={tab} onChange={setTab} items={[['prices', t('alwa.tab_prices')], ['listings', t('alwa.tab_listings')]]} />
      {tab === 'prices' ? <Prices /> : <Listings />}
    </>
  );
}

function Kpis() {
  const { t, num } = useI18n();
  const v = useVersion(db.listings);
  const k = useMemo(() => {
    let open = 0, kg = 0, sold = 0, soldKg = 0;
    const since = Date.now() - 30 * DAY;
    for (const l of db.listings.all()) {
      const isKg = unitOf(l) === 'kg'; // the tonnes count kg products only
      if (l.state === 'open') { open++; if (isKg) kg += l.kg; }
      if (l.state === 'sold' && +new Date(l.posted) >= since) { sold++; if (isKg) soldKg += l.kg; }
    }
    return { open, kg, sold, soldKg, wheat: marketPrices().get('wheat') };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [v]);
  const wheat = k.wheat?.avgSold ?? k.wheat?.avgAsk;
  return (
    <div className="grid g4 mb">
      <Kpi label={t('alwa.k_open')} value={num(k.open)} note={t('alwa.k_open_n')} icon={<Package />} />
      <Kpi label={t('alwa.k_kg')} value={num(k.kg / 1000, 1)} note={t('alwa.k_kg_n')} icon={<Scale />} />
      <Kpi label={t('alwa.k_sold')} value={num(k.sold)} note={t('alwa.k_sold_n', { t: num(k.soldKg / 1000, 1) })} icon={<Handshake />} tone="good" />
      <Kpi label={t('alwa.k_wheat')} value={wheat ? num(wheat) : '-'} note={t('alwa.k_wheat_n')} icon={<Wheat />} tone="gold" />
    </div>
  );
}

function Prices() {
  const { t, num, b } = useI18n();
  useVersion(db.listings);
  const crops = useRows(db.crops);
  const prices = marketPrices();
  const rows = useMemo(() => [...prices.values()].sort((a, z) => (z.avgSold ?? z.avgAsk ?? 0) - (a.avgSold ?? a.avgAsk ?? 0)), [prices]);
  const byId = useMemo(() => new Map(crops.map(c => [c.id, c])), [crops]);
  const exportCsv = () => downloadCsv('alwa_prices', ['product', 'unit', 'avg_asking_iqd_per_unit', 'avg_sold_iqd_per_unit', 'min', 'max', 'open_listings', 'quantity_on_sale', 'deals'],
    rows.map(r => [r.crop, unitOfProduct(r.crop), r.avgAsk == null ? '' : Math.round(r.avgAsk), r.avgSold == null ? '' : Math.round(r.avgSold), r.min ?? '', r.max ?? '', r.open, r.kgOpen, r.sold]));
  return (
    <div className="grid g-main">
      <Card title={t('alwa.avg_title')} extra={<button className="btn sm" onClick={exportCsv}>{t('common.export_csv')}</button>}>
        <div className="table-wrap">
          <table className="t cards">
            <thead><tr><th>{t('alwa.crop')}</th><th className="num">{t('alwa.avg_ask')}</th><th className="num">{t('alwa.avg_sold')}</th><th className="num">{t('alwa.range')}</th><th className="num">{t('alwa.open')}</th><th className="num">{t('alwa.qty_sale')}</th></tr></thead>
            <tbody>{rows.map(r => (
              <tr key={r.crop}>
                <td data-label={t('alwa.crop')}><CropTag id={r.crop} />{unitOfProduct(r.crop) !== 'kg' && <span className="muted small"> {t('alwa.per_' + unitOfProduct(r.crop))}</span>}</td>
                <td data-label={t('alwa.avg_ask')} className="num">{r.avgAsk == null ? '-' : num(r.avgAsk)}</td>
                <td data-label={t('alwa.avg_sold')} className="num"><b>{r.avgSold == null ? '-' : num(r.avgSold)}</b></td>
                <td data-label={t('alwa.range')} className="num">{r.min == null ? '-' : <span className="nowrap">{num(r.min)} - {num(r.max)}</span>}</td>
                <td data-label={t('alwa.open')} className="num">{num(r.open)}</td>
                <td data-label={t('alwa.qty_sale')} className="num"><Qty n={r.kgOpen} unit={unitOfProduct(r.crop)} /></td>
              </tr>))}</tbody>
          </table>
        </div>
        <p className="muted small">{t('alwa.avg_note')}</p>
      </Card>
      <Card title={t('alwa.sold_chart')}>
        <HBars unit={t('common.iqd_kg')} items={rows.filter(r => (r.avgSold ?? r.avgAsk) && unitOfProduct(r.crop) === 'kg').map(r => ({
          key: r.crop, label: byId.get(r.crop) ? b(byId.get(r.crop)!.name) : r.crop, value: Math.round(r.avgSold ?? r.avgAsk ?? 0), color: byId.get(r.crop)?.color,
        }))} />
      </Card>
    </div>
  );
}

function Listings() {
  const { t, num, b, ago } = useI18n();
  const listings = useRows(db.listings);
  const crops = useRows(db.crops);
  const place = usePlaceNames();
  const [grp, setGrp] = useState(''), [crop, setCrop] = useState(''), [state, setState] = useState(''), [gov, setGov] = useState(''), [q, setQ] = useState('');
  const dq = useDebounced(q.trim().toLowerCase());
  const [sel, setSel] = useState<Listing | null>(null);

  const rows = useMemo(() => {
    const digits = dq.replace(/\D/g, '');
    return listings.filter(l => {
      if (grp && groupOf(l.crop) !== grp) return false;
      if (crop && l.crop !== crop) return false;
      if (state && l.state !== state) return false;
      if (gov && l.gov !== gov) return false;
      if (dq) {
        if (l.id === dq) return true;
        const f = db.farmers.get(l.farmerId);
        if (!f) return false;
        return f.name.en.toLowerCase().includes(dq) || f.name.ku.includes(dq) || (digits.length >= 3 && f.phone.includes(digits));
      }
      return true;
    });
  }, [listings, crops, grp, crop, state, gov, dq]);

  const cols: Col<Listing>[] = [
    { key: 'id', label: '#', cell: l => <span className="mono">#{l.id}</span>, sort: l => +l.id },
    { key: 'crop', label: t('alwa.crop'), cell: l => <CropTag id={l.crop} />, sort: l => l.crop },
    { key: 'farmer', label: t('alwa.farmer'), cell: l => <bdi>{b(db.farmers.get(l.farmerId)?.name)}</bdi> },
    { key: 'place', label: t('common.district'), cell: l => place.dist(l.dist), sort: l => l.dist },
    { key: 'kg', label: t('alwa.qty'), num: true, cell: l => <Qty n={l.kg} unit={unitOf(l)} />, sort: l => l.kg },
    { key: 'price', label: t('alwa.price_unit'), num: true, cell: l => <span className="nowrap"><b>{num(l.price)}</b> <span className="muted small">{t('alwa.per_' + unitOf(l))}</span></span>, sort: l => l.price },
    { key: 'vs', label: t('alwa.vs_avg'), cell: l => <VsAvg l={l} /> },
    { key: 'quality', label: t('alwa.quality'), optional: true, cell: l => l.quality },
    { key: 'offers', label: t('alwa.offers'), num: true, optional: true, cell: l => num(l.offers), sort: l => l.offers },
    { key: 'state', label: t('common.status'), cell: l => <Pill tone={STATE_TONE[l.state]}>{t('alwa.st_' + l.state)}</Pill>, sort: l => l.state },
    { key: 'posted', label: t('alwa.posted'), cell: l => ago(l.posted), sort: l => l.posted },
  ];
  const exportCsv = () => downloadCsv('alwa_listings', ['id', 'product', 'group', 'farmer', 'phone', 'governorate', 'district', 'quantity', 'unit', 'asking_iqd_per_unit', 'sold_iqd_per_unit', 'quality', 'state', 'offers', 'posted'],
    rows.map(l => { const f = db.farmers.get(l.farmerId); return [l.id, l.crop, groupOf(l.crop), f?.name.en ?? '', f?.phone ?? '', l.gov, l.dist, l.kg, unitOf(l), l.price, l.soldPrice ?? '', l.quality, l.state, l.offers, l.posted]; }));

  return (
    <Card>
      <div className="filters">
        <input type="search" className="grow" value={q} onChange={e => setQ(e.target.value)} placeholder={t('alwa.search')} />
        <Select value={grp} onChange={v => { setGrp(v); setCrop(''); }} options={[['', t('alwa.all_groups')], ...GROUPS.map(g => [g, t('alwa.g_' + g)] as [string, string])]} aria-label={t('alwa.group')} />
        <Select value={crop} onChange={setCrop} options={[['', t('alwa.all_products')], ...crops.filter(c => !grp || (c.group ?? 'crops') === grp).map(c => [c.id, b(c.name)] as [string, string])]} aria-label={t('alwa.crop')} />
        <Select value={state} onChange={setState} options={[['', t('alwa.any_state')], ...(['open', 'sold', 'expired', 'cancelled'] as const).map(s => [s, t('alwa.st_' + s)] as [string, string])]} aria-label={t('common.status')} />
        <Select value={gov} onChange={setGov} options={[['', t('common.all_govs')], ...PLACES.governorates.map(g => [g.en, place.gov(g.en)] as [string, string])]} aria-label={t('common.governorate')} />
      </div>
      <DataTable id="alwa" rows={rows} cols={cols} onRow={setSel} selected={sel?.id} defaultSort={['posted', -1]}
        head={<><span className="muted small">{t('alwa.count', { n: num(rows.length) })}</span><button className="btn sm" onClick={exportCsv}>{t('common.export_csv')}</button></>} />
      {sel && <ListingDrawer l={sel} onClose={() => setSel(null)} />}
    </Card>
  );
}

function ListingDrawer({ l, onClose }: { l: Listing; onClose: () => void }) {
  const { t, num, b, date } = useI18n();
  const place = usePlaceNames();
  const f = db.farmers.get(l.farmerId), farm = db.farms.get(l.farmId);
  const ref = refPrice(l.crop);
  const c = db.crops.get(l.crop), unit = unitOf(l);
  return (
    <Drawer eyebrow={t('alwa.listing', { id: l.id })} title={<span className="row"><CropTag id={l.crop} /><Pill tone={STATE_TONE[l.state]}>{t('alwa.st_' + l.state)}</Pill></span>} onClose={onClose}>
      <div className="grid g2 mb">
        <Kpi label={t('alwa.price_unit')} value={num(l.price)} note={ref ? t('alwa.ref_note', { a: num(ref), p: num(l.price / ref * 100) }) : undefined} />
        <Kpi label={unit === 'kg' ? t('alwa.amount') : t('alwa.qty')} value={unit === 'kg' ? num(l.kg / 1000, 1) : <Qty n={l.kg} unit={unit} />} note={t('alwa.tonnes_value', { v: num(l.kg * l.price / 1e6, 1) })} />
      </div>
      <div className="mb"><VsAvg l={l} /></div>
      <Card title={t('alwa.d_sale')} className="mb">
        <dl className="facts">
          <dt>{t('alwa.crop')}</dt><dd>{c ? b(c.name) : l.crop}</dd>
          <dt>{t('alwa.group')}</dt><dd>{t('alwa.g_' + groupOf(l.crop))}</dd>
          <dt>{t('alwa.qty')}</dt><dd><Qty n={l.kg} unit={unit} /></dd>
          <dt>{t('alwa.price_unit')}</dt><dd>{num(l.price)} {t('common.iqd')} {t('alwa.per_' + unit)}</dd>
          {l.soldPrice != null && <><dt>{t('alwa.sold_at')}</dt><dd>{num(l.soldPrice)} {t('common.iqd')} {t('alwa.per_' + unit)}</dd></>}
          <dt>{t('alwa.quality')}</dt><dd>{t('alwa.q_' + l.quality)}</dd>
          <dt>{t('alwa.posted')}</dt><dd>{date(l.posted, 'datetime')}</dd>
          <dt>{t('alwa.closes')}</dt><dd>{date(l.closes, 'datetime')}</dd>
        </dl>
        <div className="row mt">
          <Pill icon={<MessageSquare />}>{t('alwa.n_offers', { n: num(l.offers) })}</Pill>
          <Pill icon={<Eye />}>{t('alwa.n_views', { n: num(l.views) })}</Pill>
          <Pill icon={<Camera />}>{t('alwa.n_photos', { n: num(l.photos) })}</Pill>
        </div>
        <div className="mt"><div className="eyebrow">{t('alwa.description')}</div><p style={{ margin: '4px 0 0' }}>{l.description ? <bdi>{l.description}</bdi> : <span className="muted">{t('alwa.no_description')}</span>}</p></div>
      </Card>
      <Card title={t('alwa.d_seller')}>
        <dl className="facts">
          <dt>{t('alwa.farmer')}</dt><dd><bdi>{f ? b(f.name) : '-'}</bdi></dd>
          <dt>{t('alwa.phone')}</dt><dd>{f ? <Phone value={f.phone} /> : '-'}</dd>
          <dt>{t('alwa.farm')}</dt><dd>{farm ? <>#{farm.id} · <bdi className="ku-text">{farm.name}</bdi></> : '-'}</dd>
          <dt>{t('common.governorate')}</dt><dd>{place.gov(l.gov)}</dd>
          <dt>{t('common.district')}</dt><dd>{place.dist(l.dist)}</dd>
          {farm && <><dt>{t('common.subdistrict')}</dt><dd>{place.sub(farm.dist, farm.sub)}</dd></>}
        </dl>
        {f && <a className="btn sm mt" href={'#/admin/farms?farmer=' + f.id}>{t('alwa.open_farmer')}</a>}
      </Card>
    </Drawer>
  );
}
