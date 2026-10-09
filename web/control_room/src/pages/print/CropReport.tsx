// Crop report (from the Crop register): one crop by district (or sub-district), or every crop, for an
// area. From GET /dashboard/stats/farms; a crop's land per area is read from each area's crop list.
import { useMemo } from 'react';
import { useSearchParams } from 'react-router-dom';
import { Printer, ArrowRight } from 'lucide-react';
import { useI18n } from '../../i18n';
import { qs } from '../../api/client';
import { useApi } from '../../api/cache';
import type { FarmStats } from '../../api/types';
import { Select } from '../../components/ui';
import { HBars } from '../../components/charts';
import { StateBox, cropColor, cropName, useErrorText } from '../../components/domain';
import { GrainSun } from '../../motion/GrainSun';
import { GOVERNORATES, DISTRICTS, DISTRICT_BY_SLUG, GOV_BY_NAME } from '../../data/places';
import { CROPS } from '../../data/crops';
import { expectedTonnes } from '../admin/farms/common';
import './print.css';

export default function CropReport() {
  const { t, num, date, lang } = useI18n();
  const errText = useErrorText();
  const [params, setParams] = useSearchParams();
  const crop = params.get('crop') ?? '', gov = params.get('governorate') ?? '', zone = params.get('zone') ?? '';
  const q = useApi<FarmStats>('/dashboard/stats/farms' + qs({ governorate: gov, zone }), ['farms'], { auth: true });
  const s = q.data;
  const nm = (p?: { en: string; ku: string }) => (p ? (lang === 'ku' ? p.ku : p.en) : '');
  const setP = (k: string, v: string) => setParams(p => { v ? p.set(k, v) : p.delete(k); if (k === 'governorate') p.delete('zone'); return p; }, { replace: true });
  const areaName = zone ? nm(DISTRICT_BY_SLUG.get(zone)) : gov ? nm(GOV_BY_NAME.get(gov)) : t('common.kurdistan');

  const rows = useMemo(() => {
    if (!s) return [];
    if (crop) {
      const list = zone ? s.by_sub_zone ?? [] : s.by_zone;
      return list.map(a => { const c = a.crops.find(x => x.crop === crop); return { key: a.slug, label: a.slug === 'unknown' ? t('govreport.unknown') : lang === 'ku' ? a.name_ku || a.name_en : a.name_en, farms: c?.farms ?? 0, farmers: null as number | null, dunam: c?.dunam ?? 0, crop } as const; })
        .filter(r => r.dunam > 0).sort((a, b) => b.dunam - a.dunam);
    }
    return s.by_crop.filter(c => c.crop !== 'empty').map(c => ({ key: c.crop, label: cropName(c.crop, lang), farms: c.farms, farmers: c.farmers as number | null, dunam: c.dunam, crop: c.crop })).sort((a, b) => b.dunam - a.dunam);
  }, [s, crop, zone, lang, t]);
  const total = rows.reduce((a, r) => ({ farms: a.farms + r.farms, dunam: a.dunam + r.dunam }), { farms: 0, dunam: 0 });
  const harvest = rows.reduce((a, r) => a + expectedTonnes(r.crop, r.dunam), 0);

  return (
    <div className="print-desk">
      <div className="print-bar">
        <a className="btn" href="#/admin/crops"><ArrowRight className="flip-rtl" />{t('common.back')}</a>
        <Select value={crop} onChange={v => setP('crop', v)} options={[['', t('common.all_crops')], ...CROPS.filter(c => c.code !== 'empty').map(c => [c.code, lang === 'ku' ? c.ku : c.en] as [string, string])]} aria-label={t('cropreport.crop')} style={{ width: 'auto' }} />
        <Select value={gov} onChange={v => setP('governorate', v)} options={[['', t('common.all_govs')], ...GOVERNORATES.map(g => [g.en, nm(g)] as [string, string])]} aria-label={t('common.governorate')} style={{ width: 'auto' }} />
        <Select value={zone} onChange={v => setP('zone', v)} options={[['', t('common.all_dists')], ...DISTRICTS.filter(d => !gov || d.gov === gov).map(d => [d.slug, nm(d)] as [string, string])]} aria-label={t('common.district')} style={{ width: 'auto' }} />
        <span className="grow" />
        <button className="btn primary" onClick={() => window.print()} disabled={!s}><Printer />{t('common.print')}</button>
      </div>
      <div className="print-page">
        <div className="print-head">
          <div className="logo"><GrainSun size={32} /></div>
          <div className="org"><b>{crop ? t('cropreport.title_one', { crop: cropName(crop, lang) }) : t('cropreport.title_all')}</b><span>{t('letter.org')}</span></div>
          <div className="ref"><div>{t('govreport.made', { d: date(new Date().toISOString()) })}</div><div>{t('govreport.area', { a: areaName })}</div></div>
        </div>
        {q.error && !s && <StateBox kind="error" text={errText(q.error)} action={<button className="btn" onClick={q.reload}>{t('common.retry')}</button>} />}
        {!s && !q.error && <div className="sk-rows">{[0, 1, 2, 3].map(i => <i key={i} className="sk" />)}</div>}
        {s && <>
          <div className="print-stats">
            <div><small>{crop ? t('cropreport.plots') : t('common.farms')}</small><b>{num(crop ? total.farms : s.totals.farms)}</b></div>
            <div><small>{t('common.farmers')}</small><b>{num(crop ? s.by_crop.find(c => c.crop === crop)?.farmers ?? 0 : s.totals.farmers)}</b></div>
            <div><small>{t('govreport.dunam')}</small><b>{num(total.dunam, 1)}</b></div>
            <div><small>{t('govreport.harvest')}</small><b>{num(harvest, 1)} {t('common.tonnes')}</b></div>
          </div>
          {!rows.length ? <StateBox kind="empty" title={t('cropreport.none')} /> : <>
            <h2>{crop ? t('cropreport.by_area') : t('govreport.land_per_crop')}</h2>
            <div className="avoid-break"><HBars items={rows.map(r => ({ key: r.key, label: r.label, value: r.dunam, color: cropColor(r.crop) }))} digits={1} unit={t('common.dunam')} showShare /></div>
            <h2>{t('cropreport.table')}</h2>
            <div className="print-scroll"><table className="pt">
              <thead><tr><th>{crop ? (zone ? t('common.subdistrict') : t('common.district')) : t('govreport.crop')}</th><th className="num">{crop ? t('cropreport.plots') : t('common.farms')}</th>{!crop && <th className="num">{t('common.farmers')}</th>}<th className="num">{t('govreport.dunam')}</th><th className="num">{t('govreport.harvest_t')}</th></tr></thead>
              <tbody>
                {rows.map(r => <tr key={r.key}><td>{r.label}</td><td className="num">{num(r.farms)}</td>{!crop && <td className="num">{num(r.farmers ?? 0)}</td>}<td className="num">{num(r.dunam, 1)}</td><td className="num">{num(expectedTonnes(r.crop, r.dunam), 1)}</td></tr>)}
                <tr className="total"><td>{t('govreport.total')}</td><td className="num">{num(total.farms)}</td>{!crop && <td />}<td className="num">{num(total.dunam, 1)}</td><td className="num">{num(harvest, 1)}</td></tr>
              </tbody>
            </table></div>
            <p className="print-note">{t('govreport.estimate_note')}</p>
          </>}
        </>}
        <div className="print-foot">{t('govreport.footer')}</div>
      </div>
    </div>
  );
}
