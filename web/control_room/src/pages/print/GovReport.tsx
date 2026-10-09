// The government report: farmers, farms and land per governorate and district, and what is grown, for the
// whole region or one governorate or district (?gov=&dist=). Printable on A4.
// Cost: area totals come from api.totals() (one memoized pass); crop farm counts take one more pass over
// the farms in scope.
import { useMemo } from 'react';
import { Link, useSearchParams } from 'react-router-dom';
import { Printer, ArrowLeft } from 'lucide-react';
import { useI18n } from '../../i18n';
import { db, PLACES } from '../../data/db';
import { useDoc, useRows } from '../../data/store';
import { totals, type AreaTotal } from '../../data/api';
import { HBars } from '../../components/charts';
import { usePlaceNames, usePlaceOptions } from '../../components/domain';
import { Select } from '../../components/ui';
import { GrainSun } from '../../motion/GrainSun';
import './print.css';

const mainCrop = (t: AreaTotal | undefined) => {
  let best = '', v = 0;
  t?.crops.forEach((a, c) => { if (a > v) { v = a; best = c; } });
  return best;
};

export default function GovReport() {
  const { t, num, b, date, lang } = useI18n();
  const [params, setParams] = useSearchParams();
  const gov = params.get('gov') ?? '', dist = params.get('dist') ?? '';
  const settings = useDoc(db.settings);
  const farms = useRows(db.farms);
  const farmers = useRows(db.farmers);
  const cropsList = useRows(db.crops);
  const pn = usePlaceNames();
  const opts = usePlaceOptions(gov, dist);
  const T = useMemo(() => totals(), [farms, farmers]);

  const scope = dist ? T.byDist.get(dist) : gov ? T.byGov.get(gov) : T.all;
  const scopeName = dist ? `${pn.dist(dist)}, ${pn.gov(distGov(dist))}` : gov ? pn.gov(gov) : t('common.kurdistan');

  // per crop: dunam and number of farms that grow it, one pass over the farms in scope
  const perCrop = useMemo(() => {
    const m = new Map<string, { dunam: number; farms: number }>();
    for (const f of farms) {
      if (dist ? f.dist !== dist : gov ? f.gov !== gov : false) continue;
      for (const c of f.crops) { const x = m.get(c.crop) ?? { dunam: 0, farms: 0 }; x.dunam += c.dunam; x.farms++; m.set(c.crop, x); }
    }
    return [...m].sort((a, z) => z[1].dunam - a[1].dunam);
  }, [farms, gov, dist]);

  const govs = PLACES.governorates.filter(g => !gov && !dist ? true : g.en === (gov || distGov(dist)));
  const crop = (id: string) => cropsList.find(c => c.id === id);
  const pct = (a: number, z: number) => (z ? num(a / z * 100, 1) + '%' : '-');
  const setQ = (g: string, d: string) => { const s = new URLSearchParams(); if (g) s.set('gov', g); if (d) s.set('dist', d); setParams(s, { replace: true }); };
  const s = scope ?? { farmers: 0, farms: 0, area: 0, crops: new Map(), female: 0, irrigated: 0 };

  const row = (label: React.ReactNode, x: AreaTotal | undefined, cls = '') => (
    <tr className={cls}>
      <td>{label}</td>
      <td className="num">{num(x?.farmers ?? 0)}</td>
      <td className="num">{num(x?.female ?? 0)} <span className="print-note">({pct(x?.female ?? 0, x?.farmers ?? 0)})</span></td>
      <td className="num">{num(x?.farms ?? 0)}</td>
      <td className="num">{num(x?.area ?? 0)}</td>
      <td className="num">{pct(x?.irrigated ?? 0, x?.area ?? 0)}</td>
      <td>{mainCrop(x) ? b(crop(mainCrop(x))?.name) : '-'}</td>
    </tr>
  );

  return (
    <div className="print-desk">
      <div className="print-bar">
        <Link to="/admin/farms" className="btn"><ArrowLeft className="flip-rtl" />{t('govreport.back')}</Link>
        <Select value={gov} onChange={v => setQ(v, '')} options={[['', t('common.all_govs')], ...opts.govs]} aria-label={t('common.governorate')} style={{ width: 'auto' }} />
        <Select value={dist} onChange={v => setQ(gov || distGov(v), v)} options={[['', t('common.all_dists')], ...opts.dists]} aria-label={t('common.district')} style={{ width: 'auto' }} />
        <span className="grow" />
        <button className="btn primary" onClick={() => window.print()}><Printer />{t('common.print')}</button>
      </div>

      <article className="print-page" lang={lang === 'ku' ? 'ckb' : 'en'}>
        <header className="print-head">
          <div className="logo"><GrainSun size={38} /></div>
          <div className="org"><b>{b(settings.orgName)}</b><span>{t('govreport.issuer')}</span></div>
          <div className="ref"><div>{t('govreport.generated')}</div><div><b>{date(new Date().toISOString(), 'datetime')}</b></div></div>
        </header>

        <div className="print-title">
          <div className="eyebrow">{t('govreport.kicker')}</div>
          <h1>{t('govreport.title', { area: scopeName })}</h1>
        </div>

        <div className="print-stats">
          <div><small>{t('common.farmers')}</small><b>{num(s.farmers)}</b></div>
          <div><small>{t('govreport.women')}</small><b>{num(s.female)}</b> <span className="print-note">{pct(s.female, s.farmers)}</span></div>
          <div><small>{t('common.farms')}</small><b>{num(s.farms)}</b></div>
          <div><small>{t('govreport.dunam')}</small><b>{num(s.area)}</b> <span className="print-note">{t('govreport.irrigated_short', { p: pct(s.irrigated, s.area) })}</span></div>
        </div>

        <h2>{t('govreport.by_area')}</h2>
        <div className="print-scroll">
          <table className="pt">
            <thead><tr>
              <th>{dist ? t('common.district') : t('govreport.area_col')}</th><th className="num">{t('common.farmers')}</th><th className="num">{t('govreport.women')}</th>
              <th className="num">{t('common.farms')}</th><th className="num">{t('govreport.dunam')}</th><th className="num">{t('govreport.irrigated')}</th><th>{t('govreport.main_crop')}</th>
            </tr></thead>
            <tbody>
              {dist ? row(pn.dist(dist), T.byDist.get(dist)) : govs.map(g => (
                <GovRows key={g.en} label={pn.gov(g.en)} gTotal={T.byGov.get(g.en)} row={row}
                  dists={PLACES.districts.filter(d => d.gov === g.en).map(d => ({ key: d.en, label: pn.dist(d.en), x: T.byDist.get(d.en) }))} />
              ))}
              {!dist && !gov && row(t('govreport.total'), T.all, 'total')}
            </tbody>
          </table>
        </div>
        <p className="print-note">{t('govreport.residence_note')}</p>

        <div className="avoid-break">
          <h2>{t('govreport.crop_area')}</h2>
          {perCrop.length ? <HBars showShare unit={t('common.du')} items={perCrop.map(([c, x]) => ({ key: c, label: b(crop(c)?.name) || c, value: Math.round(x.dunam), color: crop(c)?.color }))} />
            : <p className="print-note">{t('govreport.none')}</p>}
        </div>
        <div className="avoid-break">
          <h2>{t('govreport.crop_farms')}</h2>
          {perCrop.length > 0 && <HBars items={[...perCrop].sort((a, z) => z[1].farms - a[1].farms).map(([c, x]) => ({ key: c, label: b(crop(c)?.name) || c, value: x.farms, color: crop(c)?.color }))} />}
        </div>

        <div className="avoid-break">
          <h2>{t('govreport.harvest')}</h2>
          <div className="print-scroll">
            <table className="pt">
              <thead><tr><th>{t('govreport.crop')}</th><th className="num">{t('common.farms')}</th><th className="num">{t('govreport.dunam')}</th><th className="num">{t('govreport.yield')}</th><th className="num">{t('govreport.tonnes')}</th></tr></thead>
              <tbody>
                {perCrop.map(([c, x]) => {
                  const y = crop(c)?.yieldKgPerDunam ?? 0;
                  return <tr key={c}><td>{b(crop(c)?.name) || c}</td><td className="num">{num(x.farms)}</td><td className="num">{num(x.dunam)}</td><td className="num">{num(y)}</td><td className="num">{num(x.dunam * y / 1000)}</td></tr>;
                })}
              </tbody>
            </table>
          </div>
          <p className="print-note">{t('govreport.harvest_note')}</p>
        </div>

        {!dist && govs.length > 1 && (
          <div className="avoid-break">
            <h2>{t('govreport.matrix')}</h2>
            <div className="print-scroll">
              <table className="pt">
                <thead><tr><th>{t('govreport.crop')}</th>{govs.map(g => <th key={g.en} className="num">{pn.gov(g.en)}</th>)}<th className="num">{t('govreport.total')}</th></tr></thead>
                <tbody>
                  {perCrop.map(([c, x]) => (
                    <tr key={c}><td>{b(crop(c)?.name) || c}</td>{govs.map(g => <td key={g.en} className="num">{num(T.byGov.get(g.en)?.crops.get(c) ?? 0)}</td>)}<td className="num"><b>{num(x.dunam)}</b></td></tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}
        {gov && !dist && (
          <div className="avoid-break">
            <h2>{t('govreport.matrix_dist')}</h2>
            <div className="print-scroll">
              <table className="pt">
                <thead><tr><th>{t('common.district')}</th>{perCrop.slice(0, 6).map(([c]) => <th key={c} className="num">{b(crop(c)?.name) || c}</th>)}</tr></thead>
                <tbody>
                  {PLACES.districts.filter(d => d.gov === gov).map(d => (
                    <tr key={d.en}><td>{pn.dist(d.en)}</td>{perCrop.slice(0, 6).map(([c]) => <td key={c} className="num">{num(T.byDist.get(d.en)?.crops.get(c) ?? 0)}</td>)}</tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}

        <footer className="print-foot">{t('govreport.foot')}</footer>
      </article>
    </div>
  );
}

function GovRows({ label, gTotal, dists, row }: {
  label: string; gTotal: AreaTotal | undefined; dists: { key: string; label: string; x: AreaTotal | undefined }[];
  row: (label: React.ReactNode, x: AreaTotal | undefined, cls?: string) => React.ReactNode;
}) {
  return <>{row(label, gTotal, 'group')}{dists.map(d => <DistRow key={d.key}>{row(<span style={{ paddingInlineStart: 14, display: 'inline-block' }}>{d.label}</span>, d.x)}</DistRow>)}</>;
}
const DistRow = ({ children }: { children: React.ReactNode }) => <>{children}</>;

function distGov(dist: string) { return PLACES.districts.find(d => d.en === dist)?.gov ?? ''; }
