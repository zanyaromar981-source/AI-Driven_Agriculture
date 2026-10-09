// Water, Fires, Compare years and Market tabs of the public View page (design GQYlZ, ad1I0).
import { useMemo, useState } from 'react';
import { useI18n } from '../../i18n';
import { useApi } from '../../api/cache';
import type { Dam, DamHistory, Fires, RegionCompare } from '../../api/types';
import { DISTRICT_BY_SLUG } from '../../data/places';
import { DistrictMap, type MapPoint } from '../../components/DistrictMap';
import { VBars, PairBars, HBars } from '../../components/charts';
import { StateBox, cropName, cropColor } from '../../components/domain';
import { Select } from '../../components/ui';
import { shortSource, viewDate, monthName } from './viewUtil';

// ---------- water ----------
export function WaterTab({ dams }: { dams?: Dam[] }) {
  if (!dams) return <div className="view-two"><div className="card"><i className="sk block" /></div><div className="card"><i className="sk block" /></div></div>;
  return <div className="view-two">{dams.map(d => <DamCard key={d.slug} d={d} />)}</div>;
}

function DamCard({ d }: { d: Dam }) {
  const { t, num, lang, pick } = useI18n();
  const h = useApi<DamHistory>(`/dams/${d.slug}/history`, ['dams']);
  const last12 = useMemo(() => {
    // only the last 365 days; the last reading of each month (the history is not monthly)
    const since = new Date(Date.now() - 365 * 864e5).toISOString().slice(0, 10);
    const byMonth = new Map<string, number>();
    for (const r of h.data?.readings ?? []) if (r.day >= since) byMonth.set(r.day.slice(0, 7), r.pct_full);
    return [...byMonth].sort((a, b) => (a[0] < b[0] ? -1 : 1));
  }, [h.data]);
  const L = d.latest;
  return (
    <section className="card view-dam">
      <div className="view-panel-head">
        <div>
          <h2 className="view-name">{t('view.dam_name', { name: pick(d.name_ku, d.name_en) })}</h2>
          <span className="muted">{t('view.capacity', { v: num(d.capacity_bn_m3, 2) })}</span>
        </div>
        {!L && <span className="pill">{t('common.no_data')}</span>}
      </div>
      {L ? (
        <>
          <div className="view-dam-big">
            <strong className="ltr">{num(L.pct_full, 1)}%</strong>
            <span>{t('view.dam_full')}</span>
          </div>
          <div className="bar view-dam-bar"><i style={{ width: Math.min(100, L.pct_full) + '%', background: 'var(--water)' }} /></div>
          <dl className="view-facts">
            <dt>{t('view.year_ago')}</dt><dd>{d.year_ago ? <span className="ltr">{num(d.year_ago.pct_full, 1)}%</span> : t('common.no_data')}</dd>
            {L.lake_area_km2 != null && <><dt>{t('view.lake_area')}</dt><dd>{t('view.km2', { v: num(L.lake_area_km2, 1) })}</dd></>}
            <dt>{t('view.reading_day')}</dt><dd>{viewDate(L.day, lang)}</dd>
            <dt>{t('common.source')}</dt><dd className="ltr">{shortSource(L.source).split(',')[0]}</dd>
          </dl>
          <p className="muted small">{t('view.dam_note')}</p>
        </>
      ) : (
        <StateBox kind="empty" title={t('view.dam_empty')} text={t('view.dam_empty_text')} />
      )}
      {last12.length > 1 && (
        <>
          <h3 className="view-subs-title">{t('view.last_12')}</h3>
          <VBars height={130} unit="%" items={last12.map(([m, v]) => ({ key: m, label: monthName(+m.slice(5, 7) - 1, lang).split(' ')[0], value: v, color: 'var(--water)' }))} />
        </>
      )}
    </section>
  );
}

// ---------- fires ----------
const FIRE_COLOR: Record<string, string> = { active: '#E0533F', spreading: '#B23A2E', under_control: '#E8B567', out: '#9AA59D' };
export function FiresTab({ fires }: { fires?: Fires }) {
  const { t, num, ago, lang } = useI18n();
  const counts = useMemo(() => {
    const m = new Map<string, number>();
    for (const f of fires?.fires ?? []) { const k = f.zone_slug ?? '?'; m.set(k, (m.get(k) ?? 0) + 1); }
    return [...m].sort((a, b) => b[1] - a[1]);
  }, [fires]);
  const points = useMemo<MapPoint[]>(() => (fires?.fires ?? []).map(f => ({ id: f.id, lat: f.lat, lon: f.lon, color: FIRE_COLOR[f.status] ?? '#E0533F', radius: 5, label: ago(f.detected_at) })), [fires, ago]);
  if (!fires) return <div className="view-grid"><div className="card"><i className="sk block" /></div><div className="card"><i className="sk block" /></div></div>;
  const name = (s: string) => { const d = DISTRICT_BY_SLUG.get(s); return d ? (lang === 'ku' ? d.ku : d.en) : s; };
  const latest = fires.fires.reduce<string | null>((a, f) => (!a || f.detected_at > a ? f.detected_at : a), null);
  return (
    <div className="view-grid">
      <section className="card view-map-card">
        <div className="view-map-head"><div className="view-map-title"><h2>{t('view.fires_map')}</h2><span className="muted">{t('view.fires_dot')}</span></div></div>
        <DistrictMap fill={() => '#E3E9DF'} styleKey="fires" points={points} />
      </section>
      <aside className="card view-panel">
        <div className="view-panel-in">
          <h2 className="view-name">{t('view.fires_title')}</h2>
          <p className="muted">{fires.fires.length ? t('view.fires_lead', { n: num(fires.fires.length), z: num(counts.length) }) : t('news.no_fires')}</p>
          <p className="muted small">{t('view.fires_note')}</p>
          {counts.length > 0 && <HBars items={counts.map(([s, n]) => ({ key: s, label: name(s), value: n, color: 'var(--danger)' }))} />}
          <p className="muted small">{t('view.fires_source', { when: latest ? ago(latest) : '-' })}</p>
        </div>
      </aside>
    </div>
  );
}

// ---------- compare years ----------
export function CompareTab() {
  const { t, num, pick, lang } = useI18n();
  const now = new Date();
  const [year, setYear] = useState(now.getFullYear());
  const [withY, setWithY] = useState(now.getFullYear() - 1);
  const [month, setMonth] = useState(now.getMonth() + 1);
  const c = useApi<RegionCompare>(`/region/compare?year=${year}&with=${withY}&month=${month}`, ['zones']);
  const years = Array.from({ length: 9 }, (_, i) => String(now.getFullYear() - i));
  const months = Array.from({ length: 12 }, (_, i) => [String(i + 1), monthName(i, lang)] as [string, string]);
  const rows = useMemo(() => (c.data?.zones ?? []).filter(z => z.dryness != null || z.dryness_with != null)
    .sort((a, b) => Math.abs(b.change ?? 0) - Math.abs(a.change ?? 0))
    .map(z => ({ key: z.slug, label: pick(z.name_ku, z.name_en), a: z.dryness_with ?? 0, b: z.dryness ?? 0 })), [c.data, pick]);
  const hasWith = (c.data?.zones ?? []).some(z => z.dryness_with != null);
  return (
    <section className="card">
      <div className="view-map-head">
        <div className="view-map-title"><h2>{t('view.compare_title')}</h2><span className="muted">{t('view.compare_sub')}</span></div>
        <div className="row">
          <label className="field"><span>{t('view.month')}</span><Select value={String(month)} onChange={v => setMonth(+v)} options={months} /></label>
          <label className="field"><span>{t('view.year')}</span><Select value={String(year)} onChange={v => setYear(+v)} options={years.map(y => [y, y])} /></label>
          <label className="field"><span>{t('view.with')}</span><Select value={String(withY)} onChange={v => setWithY(+v)} options={years.map(y => [y, y])} /></label>
        </div>
      </div>
      {c.loading && <i className="sk block" style={{ minHeight: 280 }} />}
      {c.error && !c.data && <StateBox kind="error" action={<button className="btn" onClick={c.reload}>{t('common.retry')}</button>} />}
      {c.data && (
        <>
          <div className="view-avgs">
            {c.data.region.map(r => <div key={r.year} className="pill"><b className="ltr">{r.year}</b>{t('view.avg_v', { v: num(r.average_dryness, 1) })}</div>)}
          </div>
          {!hasWith && <StateBox kind="empty" title={t('view.compare_empty', { y: String(withY) })} text={t('view.compare_empty_text')} />}
          {hasWith && rows.length > 0 && <PairBars rows={rows} a={String(withY)} b={String(year)} max={100} />}
          {!hasWith && rows.length > 0 && <HBars max={100} items={[...rows].sort((a, b) => b.b - a.b).map(r => ({ key: r.key, label: r.label, value: r.b, color: 'var(--brand)' }))} />}
        </>
      )}
    </section>
  );
}

// ---------- market ----------
interface Market { slug: string; name_en: string; name_ku: string }
interface Prices { market: string; day?: string | null; prices: { crop: string; price_iqd_per_kg: number; change_pct_7d?: number | null }[] }
export function MarketTab() {
  const { t, pick } = useI18n();
  const m = useApi<{ markets: Market[] }>('/alwa/markets', ['alwa_prices']);
  const [slug, setSlug] = useState<string | null>(null);
  const cur = slug ?? m.data?.markets[0]?.slug ?? null;
  return (
    <section className="card">
      <div className="view-map-head">
        <div className="view-map-title"><h2>{t('view.market_title')}</h2><span className="muted">{t('view.market_sub')}</span></div>
        {m.data && m.data.markets.length > 0 && (
          <div className="chips" role="tablist">
            {m.data.markets.map(x => <button key={x.slug} className={'chip' + (x.slug === cur ? ' on' : '')} onClick={() => setSlug(x.slug)}>{pick(x.name_ku, x.name_en)}</button>)}
          </div>
        )}
      </div>
      {m.loading && <i className="sk block" style={{ minHeight: 200 }} />}
      {m.data && !cur && <StateBox kind="empty" title={t('view.market_none')} />}
      {cur && <MarketPrices slug={cur} />}
    </section>
  );
}
function MarketPrices({ slug }: { slug: string }) {
  const { t, num, lang } = useI18n();
  const p = useApi<Prices>(`/alwa/markets/${slug}/prices`, ['alwa_prices']);
  if (p.loading) return <i className="sk block" style={{ minHeight: 200 }} />;
  if (!p.data || !p.data.prices.length) return <StateBox kind="empty" title={t('view.prices_empty')} text={t('view.prices_empty_text')} />;
  return (
    <>
      {p.data.day && <p className="muted small">{t('view.prices_day', { day: viewDate(p.data.day, lang) })}</p>}
      <HBars unit={t('common.iqd_kg')} items={p.data.prices.map(x => ({ key: x.crop, label: cropName(x.crop, lang), value: x.price_iqd_per_kg, color: cropColor(x.crop), note: x.change_pct_7d != null ? <span className="muted small"> ({x.change_pct_7d > 0 ? '+' : ''}{num(x.change_pct_7d, 1)}%)</span> : undefined }))} />
    </>
  );
}

