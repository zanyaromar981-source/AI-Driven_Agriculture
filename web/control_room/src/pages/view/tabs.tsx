// The five tabs of the public View page.
import { useMemo, useState, type ReactNode } from 'react';
import { X } from 'lucide-react';
import { useI18n } from '../../i18n';
import { db } from '../../data/db';
import { useRows, useVersion } from '../../data/store';
import { distByName, marketPrices, totals } from '../../data/api';
import type { DistrictReading, Fire } from '../../data/types';
import { DistrictMap, type MapPoint } from '../../components/DistrictMap';
import { HBars, PairBars, VBars } from '../../components/charts';
import { CropTag, usePlaceNames } from '../../components/domain';
import { Card, Note, Pill, Select, type Tone } from '../../components/ui';

// ---------- dryness bands (limits from Rules: b_dry, b_vdry) ----------
type Band = 'normal' | 'dry' | 'vdry';
const BAND_COL: Record<Band | 'hidden', string> = { normal: '#8DC28F', dry: '#E3A35A', vdry: '#B23A2E', hidden: '#E3E6DE' };
const BAND_TONE: Record<Band, Tone> = { normal: 'good', dry: 'warn', vdry: 'danger' };
function useBands() {
  const rules = useRows(db.rules);
  return useMemo(() => {
    const dry = rules.find(r => r.id === 'b_dry')?.value ?? 60, vdry = rules.find(r => r.id === 'b_vdry')?.value ?? 80;
    return { dry, vdry, band: (v: number): Band => (v >= vdry ? 'vdry' : v >= dry ? 'dry' : 'normal') };
  }, [rules]);
}

function BandLegend() {
  const { t } = useI18n();
  return (
    <div className="legend">
      {(['normal', 'dry', 'vdry', 'hidden'] as const).map(k => <span key={k}><i className="dotc" style={{ background: BAND_COL[k] }} />{t('view.band_' + k)}</span>)}
    </div>
  );
}

// ---------- Map ----------
export function MapTab({ farmTotals }: { farmTotals: boolean }) {
  const { t, num, nm, date } = useI18n();
  const pn = usePlaceNames();
  const readings = useRows(db.readings);
  const { band, dry, vdry } = useBands();
  const fv = useVersion(db.farms), pfv = useVersion(db.farmers);
  const tot = useMemo(() => totals(), [fv, pfv]); // eslint-disable-line react-hooks/exhaustive-deps
  const [sel, setSel] = useState<string | null>(null);

  const byId = useMemo(() => new Map(readings.map(r => [r.id, r])), [readings]);
  const pub = useMemo(() => readings.filter(r => r.public), [readings]);
  const driest = useMemo(() => [...pub].sort((a, z) => z.dryness - a.dryness).slice(0, 5), [pub]);
  const avg = pub.length ? pub.reduce((a, r) => a + r.dryness, 0) / pub.length : null;
  const fill = (d: string) => { const r = byId.get(d); return r?.public ? BAND_COL[band(r.dryness)] : BAND_COL.hidden; };

  const r = sel ? byId.get(sel) : undefined;
  const d = sel ? distByName.get(sel) : undefined;
  const dt = sel ? tot.byDist.get(sel) : undefined;
  const topCrops = dt ? [...dt.crops].sort((a, z) => z[1] - a[1]).slice(0, 3) : [];

  return (
    <div className="grid g-main">
      <Card title={t('view.map_title')} extra={<span className="muted small">{t('view.map_hint')}</span>}>
        <DistrictMap fill={fill} styleKey={`${readings.length}-${dry}-${vdry}-${db.readings.version}`} onDistrict={setSel} focus={sel} onBack={() => setSel(null)} />
        <BandLegend />
      </Card>

      <Card className="pub-panel" aria-live="polite">
        {!sel || !d ? (
          <>
            <h2>{t('view.region')}</h2>
            <div className="muted small">{t('view.region_sub')}</div>
            <div className="pub-stat"><span className="muted small">{t('view.region_avg')}</span><b className="tabular">{num(avg, 0)}</b>{avg != null && <Pill tone={BAND_TONE[band(avg)]}>{t('view.band_' + band(avg))}</Pill>}</div>
            {farmTotals && (
              <dl className="facts mt">
                <dt>{t('view.farms')}</dt><dd>{num(tot.all.farms)}</dd>
                <dt>{t('view.dunam')}</dt><dd>{num(tot.all.area)}</dd>
                <dt>{t('view.farmers')}</dt><dd>{num(tot.all.farmers)}</dd>
              </dl>
            )}
            <div className="eyebrow mt" style={{ marginBottom: 6 }}>{t('view.most_dry')}</div>
            {driest.map(x => (
              <button key={x.id} className="list-item click pub-row" onClick={() => setSel(x.id)}>
                <span className="dotc" style={{ background: BAND_COL[band(x.dryness)] }} />
                <span style={{ flex: 1 }}>{pn.dist(x.id)}<span className="muted small"> · {pn.gov(distByName.get(x.id)?.gov ?? '')}</span></span>
                <b className="tabular">{num(x.dryness)}</b>
              </button>
            ))}
          </>
        ) : (
          <DistrictPanel name={nm(d)} other={d.en !== nm(d) ? d.en : d.ku} gov={pn.gov(d.gov)} r={r} onClose={() => setSel(null)}>
            {farmTotals && dt && (
              <>
                <div className="eyebrow mt" style={{ marginBottom: 6 }}>{t('view.registered')}</div>
                <dl className="facts">
                  <dt>{t('view.farms')}</dt><dd>{num(dt.farms)}</dd>
                  <dt>{t('view.dunam')}</dt><dd>{num(dt.area)}</dd>
                </dl>
                {topCrops.length > 0 && <>
                  <div className="eyebrow mt" style={{ marginBottom: 4 }}>{t('view.top_crops')}</div>
                  <HBars items={topCrops.map(([c, v]) => ({ key: c, label: <CropTag id={c} />, value: v, color: db.crops.get(c)?.color }))} unit={t('common.du')} />
                </>}
              </>
            )}
            {r && <div className="muted small mt">{t('view.updated', { date: date(r.updated) })} · {t('view.source', { s: r.source })}{r.manual && <> · <Pill tone="gold">{t('view.manual')}</Pill></>}</div>}
          </DistrictPanel>
        )}
      </Card>
    </div>
  );
}

function DistrictPanel({ name, other, gov, r, onClose, children }: { name: string; other: string; gov: string; r?: DistrictReading; onClose: () => void; children: ReactNode }) {
  const { t, num } = useI18n();
  const { band } = useBands();
  const ch = r ? r.dryness - r.lastYear : 0;
  return (
    <>
      <div className="spread" style={{ alignItems: 'flex-start', flexWrap: 'nowrap' }}>
        <div style={{ minWidth: 0 }}><h2>{name}</h2><div className="muted small">{other} · {t('view.governorate')}: {gov}</div></div>
        <button className="btn ghost sm icon" onClick={onClose} aria-label={t('view.close')} title={t('view.close')}><X /></button>
      </div>
      {!r || !r.public ? <div className="mt"><Note>{t('view.not_public')}</Note></div> : (
        <div className="pub-cards">
          <div className="pub-k"><div className="eyebrow">{t('view.dryness')}</div><div className="v tabular">{num(r.dryness)}</div><Pill tone={BAND_TONE[band(r.dryness)]}>{t('view.band_' + band(r.dryness))}</Pill><div className="muted tiny">{t('view.dryness_note')}</div></div>
          <div className="pub-k"><div className="eyebrow">{t('view.greenness')}</div><div className="v tabular">{num(r.greenness)}%</div><div className="muted tiny">{t('view.greenness_note')}</div></div>
          <div className="pub-k"><div className="eyebrow">{t('view.rain')}</div><div className="v tabular">{num(r.rain)} <small>mm</small></div></div>
          <div className="pub-k"><div className="eyebrow">{t('view.last_year')}</div><div className="v tabular">{num(r.lastYear)}</div>
            <div className="small" style={{ color: ch > 0 ? 'var(--danger)' : 'var(--good)' }}>{ch > 0 ? '+' : ''}{num(ch)} {t(ch > 0 ? 'view.drier' : 'view.wetter')}</div></div>
        </div>
      )}
      {children}
    </>
  );
}

// ---------- Water ----------
export function WaterTab() {
  const { t, num, b, date, lang } = useI18n();
  const dams = useRows(db.dams);
  const monthFmt = useMemo(() => new Intl.DateTimeFormat(lang === 'ku' ? 'ckb-IQ' : 'en-GB', { month: 'short' }), [lang]);
  return (
    <>
      <div className="page-head" style={{ marginBottom: 12 }}><div className="t"><h2>{t('view.water_title')}</h2><div className="sub">{t('view.water_sub')}</div></div></div>
      <div className="grid g2">
        {dams.map(d => (
          <Card key={d.id}>
            <div className="spread"><h2>{b(d.name)}</h2>{d.manual && <Pill tone="gold">{t('view.manual')}</Pill>}</div>
            <div className="pub-big" style={{ color: 'var(--water)' }}>{num(d.pct)}%</div>
            <div className="bar" style={{ height: 12 }}><i style={{ width: Math.min(100, d.pct) + '%', background: 'var(--water)' }} /></div>
            <dl className="facts mt">
              <dt>{t('view.volume')}</dt><dd>{t('view.volume_of', { v: num(d.volume, 1), c: num(d.capacity, 1) })}</dd>
              <dt>{t('view.year_ago')}</dt><dd>{num(d.yearAgo)}%</dd>
            </dl>
            <div className="eyebrow mt">{t('view.months')}</div>
            <VBars height={110} max={100} items={d.history.map(h => ({ key: h.month, label: monthFmt.format(new Date(h.month + '-15')), value: h.pct, color: 'var(--water)' }))} />
            <div className="muted small">{t('view.updated', { date: date(d.updated) })} · {t('view.source', { s: d.source })}</div>
          </Card>
        ))}
      </div>
    </>
  );
}

// ---------- Fires ----------
const CONF_COL: Record<Fire['confidence'], string> = { low: '#E9C46A', nominal: '#E3793A', high: '#B23A2E' };
const CONF_TONE: Record<Fire['confidence'], Tone> = { low: '', nominal: 'warn', high: 'danger' };
export function FiresTab() {
  const { t, num, ago } = useI18n();
  const pn = usePlaceNames();
  const fires = useRows(db.fires);
  const sorted = useMemo(() => [...fires].sort((a, z) => (a.at < z.at ? 1 : -1)), [fires]);
  const points = useMemo<MapPoint[]>(() => sorted.map(f => ({ id: f.id, lat: f.lat, lon: f.lon, color: CONF_COL[f.confidence], radius: 8, label: `${pn.dist(f.dist)} · ${t('view.conf_' + f.confidence)}` })), [sorted, pn, t]);
  return (
    <div className="grid g-main">
      <Card title={t('view.fires_title')} extra={<span className="muted small">{t('view.fires_sub')}</span>}>
        <DistrictMap fill={() => '#EEF0EA'} styleKey="fires" points={points} showSubs={false} />
        <div className="legend">{(['high', 'nominal', 'low'] as const).map(c => <span key={c}><i className="dotc" style={{ background: CONF_COL[c], borderRadius: '50%' }} />{t('view.conf_' + c)}</span>)}</div>
      </Card>
      <Card>
        {!sorted.length && <div className="empty">{t('view.fires_none')}</div>}
        {sorted.map(f => (
          <div key={f.id} className="list-item" style={{ alignItems: 'flex-start' }}>
            <span className="dotc" style={{ background: CONF_COL[f.confidence], borderRadius: '50%', marginTop: 6 }} />
            <div style={{ flex: 1, minWidth: 0 }}>
              <div className="spread"><b>{pn.dist(f.dist)}</b><span className="muted small">{ago(f.at)}</span></div>
              <div className="row small" style={{ marginTop: 2 }}><Pill tone={CONF_TONE[f.confidence]}>{t('view.conf_' + f.confidence)}</Pill><span className="muted">{f.satellite}</span></div>
              <div className="small ink2" style={{ marginTop: 2 }}>{f.nearFarms ? t('view.near_farms', { n: num(f.nearFarms) }) : t('view.no_farms_near')}</div>
            </div>
          </div>
        ))}
      </Card>
    </div>
  );
}

// ---------- Compare years ----------
export function CompareTab() {
  const { t, num } = useI18n();
  const pn = usePlaceNames();
  const readings = useRows(db.readings);
  const pub = useMemo(() => readings.filter(r => r.public), [readings]);
  const years = useMemo(() => {
    const s = new Set<string>(); for (const r of pub) for (const y of Object.keys(r.byYear)) s.add(y);
    return [...s].sort();
  }, [pub]);
  const [ya, setYa] = useState(''), [yb, setYb] = useState('');
  const a = years.includes(ya) ? ya : years[years.length - 2] ?? years[0] ?? '';
  const b2 = years.includes(yb) ? yb : years[years.length - 1] ?? '';
  const { rows, avgA, avgB } = useMemo(() => {
    let sa = 0, sb = 0, n = 0;
    const rows = [];
    for (const r of pub) {
      const va = r.byYear[a], vb = r.byYear[b2];
      if (va == null || vb == null) continue;
      sa += va; sb += vb; n++;
      rows.push({ key: r.id, label: pn.dist(r.id), a: va, b: vb });
    }
    rows.sort((x, z) => (z.b - z.a) - (x.b - x.a));
    return { rows, avgA: n ? sa / n : null, avgB: n ? sb / n : null };
  }, [pub, a, b2, pn]);
  const opts = years.map(y => [y, y] as [string, string]);
  return (
    <>
      <div className="page-head" style={{ marginBottom: 12 }}>
        <div className="t"><h2>{t('view.compare_title')}</h2><div className="sub">{t('view.compare_sub')}</div></div>
        <div className="actions">
          <label className="field">{t('view.year_a')}<Select value={a} onChange={setYa} options={opts} /></label>
          <label className="field">{t('view.year_b')}<Select value={b2} onChange={setYb} options={opts} /></label>
        </div>
      </div>
      <div className="grid g-main">
        <Card><PairBars rows={rows} a={a} b={b2} max={100} /></Card>
        <div className="stack">
          <div className="card kpi"><div className="l">{t('view.avg_year', { y: a })}</div><div className="v">{num(avgA)}</div></div>
          <div className="card kpi"><div className="l">{t('view.avg_year', { y: b2 })}</div><div className="v">{num(avgB)}</div>
            {avgA != null && avgB != null && <div className="n" style={{ color: avgB > avgA ? 'var(--danger)' : 'var(--good)' }}>{avgB > avgA ? '+' : ''}{num(avgB - avgA, 1)} {t(avgB > avgA ? 'view.drier' : 'view.wetter')}</div>}</div>
          <Note>{t('view.dryness_note')}</Note>
        </div>
      </div>
    </>
  );
}

// ---------- Market ----------
export function MarketTab() {
  const { t, num } = useI18n();
  const lv = useVersion(db.listings);
  const crops = useRows(db.crops);
  const prices = useMemo(() => marketPrices(), [lv]); // eslint-disable-line react-hooks/exhaustive-deps
  const rows = useMemo(() => crops.filter(c => c.active && (c.unit ?? 'kg') === 'kg').map(c => ({ c, p: prices.get(c.id) })).filter(x => x.p && (x.p.avgAsk || x.p.avgSold))
    .sort((x, z) => (z.p!.avgAsk ?? z.p!.avgSold ?? 0) - (x.p!.avgAsk ?? x.p!.avgSold ?? 0)), [crops, prices]);
  return (
    <>
      <div className="page-head" style={{ marginBottom: 12 }}><div className="t"><h2>{t('view.market_title')}</h2><div className="sub">{t('view.market_sub')}</div></div></div>
      <div className="mb"><Note tone="info">{t('view.market_note')}</Note></div>
      <div className="grid g2">
        <Card title={t('view.avg_ask')}>
          <HBars items={rows.filter(x => x.p!.avgAsk).map(({ c, p }) => ({ key: c.id, label: <CropTag id={c.id} />, value: Math.round(p!.avgAsk!), color: c.color,
            note: <span className="muted small"> · {t('view.open_listings', { n: num(p!.open) })}</span> }))} unit={t('common.iqd_kg')} />
        </Card>
        <Card title={t('view.avg_sold')}>
          <HBars items={rows.filter(x => x.p!.avgSold).map(({ c, p }) => ({ key: c.id, label: <CropTag id={c.id} />, value: Math.round(p!.avgSold!), color: c.color }))} unit={t('common.iqd_kg')} />
        </Card>
      </div>
    </>
  );
}

