// Printable crop report for one crop (or all crops) in one area (or the whole region).
// Opened from Crop register > Crop report as #/print/crops?gov=&dist=&crop=
// Cost: one pass over the farms.
import { useMemo } from 'react';
import { Link, useSearchParams } from 'react-router-dom';
import { Printer, ArrowLeft } from 'lucide-react';
import { db, PLACES } from '../../data/db';
import { useDoc, useRows, useVersion } from '../../data/store';
import { useI18n } from '../../i18n';
import { HBars } from '../../components/charts';
import { CropTag, usePlaceNames } from '../../components/domain';
import { GrainSun } from '../../motion/GrainSun';
import { LangSwitch } from '../../layouts/LangSwitch';
import './cropreport.css';

interface Row { key: string; farms: number; farmers: Set<string>; dunam: number; irrigated: number }

export default function CropReport() {
  const { t, num, b, date } = useI18n();
  const [q] = useSearchParams();
  const gov = q.get('gov') ?? '', dist = q.get('dist') ?? '', crop = q.get('crop') ?? '';
  const settings = useDoc(db.settings);
  const crops = useRows(db.crops);
  const v = useVersion(db.farms);
  const place = usePlaceNames();
  const byId = useMemo(() => new Map(crops.map(c => [c.id, c])), [crops]);
  const c = crop ? byId.get(crop) : undefined;

  const data = useMemo(() => {
    // group by district when one crop is asked for, by crop otherwise
    const groups = new Map<string, Row>();
    const tot: Row = { key: 'all', farms: 0, farmers: new Set(), dunam: 0, irrigated: 0 };
    for (const f of db.farms.all()) {
      if (gov && f.gov !== gov) continue;
      if (dist && f.dist !== dist) continue;
      for (const fc of f.crops) {
        if (crop && fc.crop !== crop) continue;
        const k = crop ? f.dist : fc.crop;
        let g = groups.get(k); if (!g) { g = { key: k, farms: 0, farmers: new Set(), dunam: 0, irrigated: 0 }; groups.set(k, g); }
        for (const r of [g, tot]) { r.farms++; r.farmers.add(f.farmerId); r.dunam += fc.dunam; if (f.irrigation !== 'rainfed') r.irrigated += fc.dunam; }
      }
    }
    return { rows: [...groups.values()].sort((a, z) => z.dunam - a.dunam), tot };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [v, gov, dist, crop]);

  const yieldOf = (code: string) => byId.get(code)?.yieldKgPerDunam ?? 0;
  const harvestOf = (r: Row) => (crop ? r.dunam * yieldOf(crop) : r.dunam * yieldOf(r.key)) / 1000;
  const totalHarvest = data.rows.reduce((a, r) => a + harvestOf(r), 0);
  const area = dist ? place.dist(dist) : gov ? place.gov(gov) : t('common.kurdistan');
  const govOfDist = useMemo(() => new Map(PLACES.districts.map(d => [d.en, d.gov])), []);
  const now = new Date().toISOString();

  return (
    <div className="cr-wrap">
      <div className="cr-bar">
        <Link to="/admin/crops" className="btn sm"><ArrowLeft className="flip-rtl" />{t('cropreport.back')}</Link>
        <div className="row"><LangSwitch /><button className="btn primary sm" onClick={() => window.print()}><Printer />{t('common.print')}</button></div>
      </div>
      <article className="cr-page">
        <header className="cr-head">
          <GrainSun size={44} />
          <div className="org"><b><bdi>{b(settings.orgName)}</bdi></b><span className="muted small">{t('cropreport.from')}</span></div>
          <div className="meta">{t('cropreport.generated')}<br /><b>{date(now, 'date')}</b></div>
        </header>
        <h1>{c ? t('cropreport.title_one', { crop: b(c.name) }) : t('cropreport.title_all')}</h1>
        <div className="muted">{t('cropreport.area', { area })}</div>

        <div className="cr-kpis">
          <div><small>{t('cropreport.k_farms')}</small><b>{num(data.tot.farms)}</b></div>
          <div><small>{t('cropreport.k_farmers')}</small><b>{num(data.tot.farmers.size)}</b></div>
          <div><small>{t('cropreport.k_dunam')}</small><b>{num(data.tot.dunam)}</b></div>
          <div><small>{t('cropreport.k_harvest')}</small><b>{num(totalHarvest)}</b></div>
        </div>

        {data.rows.length === 0 ? <p className="empty">{t('cropreport.empty')}</p> : (
          <>
            <h2>{crop ? t('cropreport.chart_dist') : t('cropreport.chart_crop')}</h2>
            <HBars showShare unit={t('common.du')} items={data.rows.slice(0, 15).map(r => ({
              key: r.key, label: crop ? place.dist(r.key) : (byId.get(r.key) ? b(byId.get(r.key)!.name) : r.key),
              value: r.dunam, color: crop ? c?.color : byId.get(r.key)?.color,
            }))} />

            <h2>{t('cropreport.table')}</h2>
            <div className="print-scroll">
            <table className="t">
              <thead><tr>
                <th>{crop ? t('common.district') : t('crops.crop')}</th>
                {crop && <th>{t('common.governorate')}</th>}
                <th className="num">{t('common.farms')}</th><th className="num">{t('common.farmers')}</th>
                <th className="num">{t('common.dunam')}</th><th className="num">{t('cropreport.irrigated')}</th><th className="num">{t('cropreport.harvest_t')}</th>
              </tr></thead>
              <tbody>{data.rows.map(r => (
                <tr key={r.key}>
                  <td>{crop ? <b>{place.dist(r.key)}</b> : <CropTag id={r.key} />}</td>
                  {crop && <td>{place.gov(govOfDist.get(r.key) ?? '')}</td>}
                  <td className="num">{num(r.farms)}</td><td className="num">{num(r.farmers.size)}</td>
                  <td className="num">{num(r.dunam)}</td><td className="num">{num(r.dunam ? r.irrigated / r.dunam * 100 : 0)}%</td>
                  <td className="num"><b>{num(harvestOf(r))}</b></td>
                </tr>))}
                <tr><td colSpan={crop ? 2 : 1}><b>{t('crops.total')}</b></td><td className="num"><b>{num(data.tot.farms)}</b></td><td className="num"><b>{num(data.tot.farmers.size)}</b></td>
                  <td className="num"><b>{num(data.tot.dunam)}</b></td><td className="num"><b>{num(data.tot.dunam ? data.tot.irrigated / data.tot.dunam * 100 : 0)}%</b></td><td className="num"><b>{num(totalHarvest)}</b></td></tr>
              </tbody>
            </table>
            </div>
          </>
        )}
        <footer className="cr-foot">{t('cropreport.note')}</footer>
      </article>
    </div>
  );
}
