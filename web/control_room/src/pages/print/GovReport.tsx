// Government report (design xk3ig): farmers, farms and land by governorate, district or sub-district,
// with bars per crop, from GET /dashboard/stats/farms. Expected harvest is an estimate (typical yield
// per dunam) and says so.
import { useMemo } from 'react';
import { useSearchParams } from 'react-router-dom';
import { Printer, ArrowLeft } from 'lucide-react';
import { useI18n } from '../../i18n';
import { qs } from '../../api/client';
import { useApi } from '../../api/cache';
import type { FarmStats, StatsArea } from '../../api/types';
import { Select } from '../../components/ui';
import { HBars } from '../../components/charts';
import { StateBox, cropColor, cropName, useErrorText, useCrops } from '../../components/domain';
import { GrainSun } from '../../motion/GrainSun';
import { GOVERNORATES, DISTRICTS, DISTRICT_BY_SLUG, GOV_BY_NAME } from '../../data/places';
import { expectedTonnes, sumTonnes } from '../admin/farms/common';
import './print.css';

export default function GovReport() {
  const { t, num, date, lang } = useI18n();
  const errText = useErrorText();
  const [params, setParams] = useSearchParams();
  const gov = params.get('governorate') ?? '', zone = params.get('zone') ?? '';
  const q = useApi<FarmStats>('/dashboard/stats/farms' + qs({ governorate: gov, zone }), ['farms'], { auth: true });
  const s = q.data;
  const nm = (p?: { en: string; ku: string }) => (p ? (lang === 'ku' ? p.ku : p.en) : '');
  const setP = (k: string, v: string) => setParams(p => { v ? p.set(k, v) : p.delete(k); if (k === 'governorate') p.delete('zone'); return p; }, { replace: true });

  const areaName = zone ? nm(DISTRICT_BY_SLUG.get(zone)) : gov ? nm(GOV_BY_NAME.get(gov)) : t('common.kurdistan');
  // the level below the chosen area
  const rows: StatsArea[] = useMemo(() => {
    if (!s) return [];
    const list = zone ? s.by_sub_zone ?? [] : gov ? s.by_zone : s.by_governorate;
    return [...list].sort((a, b) => b.dunam - a.dunam);
  }, [s, gov, zone]);
  const crops = useMemo(() => (s ? s.by_crop.filter(c => c.crop !== 'empty').sort((a, b) => b.dunam - a.dunam) : []), [s]);
  const areaLabel = (r: StatsArea) => (r.slug === 'unknown' ? t('govreport.unknown') : lang === 'ku' ? r.name_ku || r.name_en : r.name_en);
  const mainCrop = (r: StatsArea) => { const c = r.crops.filter(x => x.crop !== 'empty').sort((a, b) => b.dunam - a.dunam)[0]; return c ? cropName(c.crop, lang) : '-'; };
  const harvest = sumTonnes(crops);
  const tonnes = (v: number | null) => (v == null ? '-' : num(v, 1));
  useCrops();
  const levelTitle = zone ? t('govreport.by_sub') : gov ? t('govreport.by_dist') : t('govreport.by_gov');

  return (
    <div className="print-desk">
      <div className="print-bar">
        <a className="btn" href="#/admin/farms"><ArrowLeft className="flip-rtl" />{t('common.back')}</a>
        <Select value={gov} onChange={v => setP('governorate', v)} options={[['', t('common.all_govs')], ...GOVERNORATES.map(g => [g.en, nm(g)] as [string, string])]} aria-label={t('common.governorate')} style={{ width: 'auto' }} />
        <Select value={zone} onChange={v => setP('zone', v)} options={[['', t('common.all_dists')], ...DISTRICTS.filter(d => !gov || d.gov === gov).map(d => [d.slug, nm(d)] as [string, string])]} aria-label={t('common.district')} style={{ width: 'auto' }} />
        <span className="grow" />
        <button className="btn primary" onClick={() => window.print()} disabled={!s}><Printer />{t('common.print')}</button>
      </div>
      <div className="print-page">
        <div className="print-head">
          <div className="logo"><GrainSun size={32} /></div>
          <div className="org"><b>{t('govreport.title')}</b><span>{t('letter.org')}</span></div>
          <div className="ref"><div>{t('govreport.made', { d: date(new Date().toISOString()) })}</div><div>{t('govreport.area', { a: areaName })}</div>{s && <div>{t('govreport.as_of', { d: date(s.as_of) })}</div>}</div>
        </div>
        {q.error && !s && <StateBox kind="error" text={errText(q.error)} action={<button className="btn" onClick={q.reload}>{t('common.retry')}</button>} />}
        {!s && !q.error && <div className="sk-rows">{[0, 1, 2, 3, 4, 5].map(i => <i key={i} className="sk" />)}</div>}
        {s && <>
          <div className="print-stats">
            <div><small>{t('common.farmers')}</small><b>{num(s.totals.farmers)}</b></div>
            <div><small>{t('common.farms')}</small><b>{num(s.totals.farms)}</b></div>
            <div><small>{t('govreport.dunam')}</small><b>{num(s.totals.dunam, 1)}</b></div>
            <div><small>{t('govreport.harvest')}</small><b>{harvest != null ? num(harvest, 1) + ' ' + t('common.tonnes') : '-'}</b></div>
          </div>
          {!s.totals.farms ? <StateBox kind="empty" title={t('govreport.none')} /> : <>
            <h2>{t('govreport.land_per_crop')}</h2>
            <div className="avoid-break"><HBars items={crops.map(c => ({ key: c.crop, label: cropName(c.crop, lang), value: c.dunam, color: cropColor(c.crop) }))} digits={1} unit={t('common.dunam')} showShare /></div>
            <h2>{t('govreport.farms_per_crop')}</h2>
            <div className="avoid-break"><HBars items={crops.map(c => ({ key: c.crop, label: cropName(c.crop, lang), value: c.farms, color: cropColor(c.crop) }))} /></div>
            <h2>{t('govreport.crop_table')}</h2>
            <div className="print-scroll"><table className="pt">
              <thead><tr><th>{t('govreport.crop')}</th><th className="num">{t('common.farms')}</th><th className="num">{t('common.farmers')}</th><th className="num">{t('govreport.dunam')}</th><th className="num">{t('govreport.harvest_t')}</th></tr></thead>
              <tbody>
                {crops.map(c => <tr key={c.crop}><td><span className="dotc" style={{ background: cropColor(c.crop) }} />{cropName(c.crop, lang)}</td><td className="num">{num(c.farms)}</td><td className="num">{num(c.farmers)}</td><td className="num">{num(c.dunam, 1)}</td><td className="num">{tonnes(expectedTonnes(c.crop, c.dunam))}</td></tr>)}
              </tbody>
            </table></div>
            <p className="print-note">{t('govreport.estimate_note')}</p>
            <h2>{levelTitle}</h2>
            <div className="print-scroll"><table className="pt">
              <thead><tr><th>{zone ? t('common.subdistrict') : gov ? t('common.district') : t('common.governorate')}</th><th className="num">{t('common.farmers')}</th><th className="num">{t('common.farms')}</th><th className="num">{t('govreport.dunam')}</th><th>{t('govreport.main_crop')}</th></tr></thead>
              <tbody>
                {rows.map(r => <tr key={r.slug}><td>{areaLabel(r)}</td><td className="num">{num(r.farmers)}</td><td className="num">{num(r.farms)}</td><td className="num">{num(r.dunam, 1)}</td><td>{mainCrop(r)}</td></tr>)}
                <tr className="total"><td>{t('govreport.total')}</td><td className="num">{num(s.totals.farmers)}</td><td className="num">{num(s.totals.farms)}</td><td className="num">{num(s.totals.dunam, 1)}</td><td /></tr>
              </tbody>
            </table></div>
          </>}
        </>}
        <div className="print-foot">{t('govreport.footer')}</div>
      </div>
    </div>
  );
}
