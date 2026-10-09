// Crop register (design psYq3): land, farms and expected harvest per crop from GET /dashboard/stats/farms,
// a district map for one crop, a crop-by-governorate table, and the crop list. Adding or editing crops
// waits for the server's crops table (FRONTEND.md 14), so that part says "coming soon".
import { useMemo, useState } from 'react';
import { Wheat, Ruler, Scale, Sprout, FileText } from 'lucide-react';
import { useI18n } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { qs } from '../../api/client';
import { useApi } from '../../api/cache';
import type { FarmStats } from '../../api/types';
import { PageHead, Kpi, Tabs, Card, Modal, Select, Field } from '../../components/ui';
import { HBars } from '../../components/charts';
import { DistrictMap, ramp, GOLD_RAMP } from '../../components/DistrictMap';
import { StateBox, cropColor, cropName, useErrorText } from '../../components/domain';
import { CROPS } from '../../data/crops';
import { GOVERNORATES, DISTRICTS, DISTRICT_BY_EN } from '../../data/places';
import { YIELD_KG_PER_DUNAM, expectedTonnes } from './farms/common';
import './crops.css';

type Tab = 'overview' | 'list';

export default function Crops() {
  const { t, num, lang } = useI18n();
  const { can } = useAuth();
  const errText = useErrorText();
  const [tab, setTab] = useState<Tab>('overview');
  const [pick, setPick] = useState('');
  const [report, setReport] = useState(false);
  const [rp, setRp] = useState({ crop: '', governorate: '', zone: '' });
  const q = useApi<FarmStats>(can('farms') ? '/dashboard/stats/farms' : null, ['farms'], { auth: true });
  const s = q.data;
  const crops = useMemo(() => (s ? s.by_crop.filter(c => c.crop !== 'empty').sort((a, b) => b.dunam - a.dunam) : []), [s]);
  const planted = crops.reduce((a, c) => a + c.dunam, 0);
  const harvest = crops.reduce((a, c) => a + expectedTonnes(c.crop, c.dunam), 0);
  const top = crops[0];
  // land of the picked crop (or all planted land) per district, read in one pass over by_zone
  const perZone = useMemo(() => {
    const m = new Map<string, number>();
    for (const z of s?.by_zone ?? []) m.set(z.slug, pick ? z.crops.find(c => c.crop === pick)?.dunam ?? 0 : z.crops.filter(c => c.crop !== 'empty').reduce((a, c) => a + c.dunam, 0));
    return m;
  }, [s, pick]);
  const maxZone = Math.max(1e-9, ...perZone.values());
  const topZones = useMemo(() => [...(s?.by_zone ?? [])].map(z => ({ z, v: perZone.get(z.slug) ?? 0 })).filter(x => x.v > 0).sort((a, b) => b.v - a.v).slice(0, 8), [s, perZone]);
  const nm = (p: { en: string; ku: string }) => (lang === 'ku' ? p.ku : p.en);

  if (!can('farms')) return <><PageHead eyebrow={t('nav.g_fields')} title={t('nav.crops')} /><StateBox kind="locked" /></>;

  return (
    <div>
      <PageHead eyebrow={t('nav.g_fields')} title={t('nav.crops')} sub={t('crops.sub')} actions={<>
        <button className="btn" onClick={() => setReport(true)}><FileText />{t('crops.report')}</button>
      </>} />
      <div className="grid g4 mb">
        <Kpi label={t('crops.k_active')} icon={<Sprout />} value={s ? num(crops.length) : '-'} note={t('crops.k_active_note', { n: num(CROPS.length - 1) })} />
        <Kpi label={t('crops.k_planted')} icon={<Ruler />} value={s ? num(planted, 1) : '-'} note={s ? t('crops.k_planted_note', { n: num(s.totals.farms) }) : ' '} />
        <Kpi label={t('crops.k_top')} icon={<Wheat />} value={top ? cropName(top.crop, lang) : '-'} note={top && planted ? t('crops.k_top_note', { p: num(top.dunam / planted * 100) }) : ' '} />
        <Kpi label={t('crops.k_harvest')} icon={<Scale />} value={s ? num(harvest, 1) + ' ' + t('common.tonnes') : '-'} note={t('crops.k_harvest_note')} />
      </div>
      <Tabs<Tab> value={tab} onChange={setTab} items={[['overview', t('crops.tab_overview')], ['list', t('crops.tab_list')]]} />
      {q.error && !s ? <StateBox kind="error" text={errText(q.error)} action={<button className="btn" onClick={q.reload}>{t('common.retry')}</button>} /> : !s ? <div className="card"><div className="sk-rows">{[0, 1, 2, 3, 4].map(i => <i key={i} className="sk" />)}</div></div> : tab === 'overview' ? (
        !crops.length ? <Card><StateBox kind="empty" title={t('crops.none_title')} text={t('crops.none_text')} /></Card> : <>
          <div className="chips mb">
            <button className={'chip' + (!pick ? ' on' : '')} onClick={() => setPick('')}>{t('common.all_crops')}</button>
            {crops.map(c => <button key={c.crop} className={'chip' + (pick === c.crop ? ' on' : '')} onClick={() => setPick(c.crop)}><span className="dotc" style={{ background: cropColor(c.crop) }} />{cropName(c.crop, lang)}</button>)}
          </div>
          <div className="grid g-main mb">
            <Card title={pick ? t('crops.map_one', { crop: cropName(pick, lang) }) : t('crops.map_all')}>
              <DistrictMap styleKey={pick + ':' + q.data?.as_of} size="sm" fill={en => { const d = DISTRICT_BY_EN.get(en); const v = d ? perZone.get(d.slug) ?? 0 : 0; return v > 0 ? ramp(v / maxZone, [0.15, 0.35, 0.6, 0.85], GOLD_RAMP) : undefined; }} />
              <div className="legend">{GOLD_RAMP.map((c, i) => <span key={c}><i className="dotc" style={{ background: c }} />{[t('crops.l1'), t('crops.l2'), t('crops.l3'), t('crops.l4'), t('crops.l5')][i]}</span>)}</div>
            </Card>
            <div className="stack">
              <Card title={t('crops.top_districts')}>
                {topZones.length ? <HBars items={topZones.map(({ z, v }) => ({ key: z.slug, label: lang === 'ku' ? z.name_ku || z.name_en : z.name_en, value: v, color: pick ? cropColor(pick) : 'var(--gold)' }))} digits={1} unit={t('common.dunam')} /> : <div className="muted small">{t('crops.no_land')}</div>}
              </Card>
            </div>
          </div>
          <div className="grid g2 mb">
            <Card title={t('crops.land_per_crop')}>
              <HBars items={crops.map(c => ({ key: c.crop, label: cropName(c.crop, lang), value: c.dunam, color: cropColor(c.crop) }))} digits={1} unit={t('common.dunam')} showShare />
            </Card>
            <Card title={t('crops.table_title')}>
              <div className="table-wrap"><table className="t cards">
                <thead><tr><th>{t('crops.crop')}</th><th className="num">{t('common.farms')}</th><th className="num">{t('common.farmers')}</th><th className="num">{t('common.dunam')}</th><th className="num">{t('crops.harvest_t')}</th></tr></thead>
                <tbody>{crops.map(c => <tr key={c.crop}>
                  <td data-label={t('crops.crop')}><span className="dotc" style={{ background: cropColor(c.crop) }} />{cropName(c.crop, lang)}</td>
                  <td className="num" data-label={t('common.farms')}>{num(c.farms)}</td><td className="num" data-label={t('common.farmers')}>{num(c.farmers)}</td>
                  <td className="num" data-label={t('common.dunam')}><b>{num(c.dunam, 1)}</b></td><td className="num" data-label={t('crops.harvest_t')}>{num(expectedTonnes(c.crop, c.dunam), 1)}</td>
                </tr>)}</tbody>
              </table></div>
              <p className="muted small">{t('crops.estimate_note')}</p>
            </Card>
          </div>
          <Card title={t('crops.by_gov')}>
            <div className="table-wrap"><table className="t">
              <thead><tr><th>{t('crops.crop')}</th>{s.by_governorate.map(g => <th key={g.slug} className="num">{lang === 'ku' ? g.name_ku || g.name_en : g.name_en}</th>)}</tr></thead>
              <tbody>{crops.map(c => <tr key={c.crop}><td><span className="dotc" style={{ background: cropColor(c.crop) }} />{cropName(c.crop, lang)}</td>
                {s.by_governorate.map(g => <td key={g.slug} className="num">{num(g.crops.find(x => x.crop === c.crop)?.dunam ?? 0, 1)}</td>)}</tr>)}</tbody>
            </table></div>
          </Card>
        </>
      ) : (
        <div className="stack">
          <Card>
            <StateBox kind="soon" title={t('crops.soon_title')} text={t('crops.soon_text')} />
          </Card>
          <Card title={t('crops.list_title')}>
            <div className="table-wrap"><table className="t cards">
              <thead><tr><th>{t('crops.crop')}</th><th>{t('crops.code')}</th><th>{t('crops.name_other')}</th><th className="num">{t('crops.yield')}</th><th className="num">{t('common.dunam')}</th><th>{t('common.status')}</th></tr></thead>
              <tbody>{CROPS.filter(c => c.code !== 'empty').map(c => {
                const used = s.by_crop.find(x => x.crop === c.code);
                return <tr key={c.code}>
                  <td data-label={t('crops.crop')}><span className="dotc" style={{ background: c.color }} /><b>{lang === 'ku' ? c.ku : c.en}</b></td>
                  <td data-label={t('crops.code')} className="mono">{c.code}</td>
                  <td data-label={t('crops.name_other')}>{lang === 'ku' ? c.en : <span className="ku-text">{c.ku}</span>}</td>
                  <td data-label={t('crops.yield')} className="num">{YIELD_KG_PER_DUNAM[c.code] ? num(YIELD_KG_PER_DUNAM[c.code]) + ' ' + t('common.kg') : '-'}</td>
                  <td data-label={t('common.dunam')} className="num">{used ? num(used.dunam, 1) : '0'}</td>
                  <td data-label={t('common.status')}>{used ? <span className="pill good">{t('crops.in_use')}</span> : <span className="pill">{t('crops.not_used')}</span>}</td>
                </tr>;
              })}</tbody>
            </table></div>
          </Card>
        </div>
      )}
      {report && <Modal title={t('crops.report')} onClose={() => setReport(false)} foot={<>
        <button className="btn" onClick={() => setReport(false)}>{t('common.cancel')}</button>
        <a className="btn primary" href={'#/print/crops' + qs(rp)} target="_blank" rel="noopener" onClick={() => setReport(false)}><FileText />{t('crops.open_report')}</a>
      </>}>
        <div className="form-grid">
          <Field label={t('crops.crop')} full><Select value={rp.crop} onChange={v => setRp(p => ({ ...p, crop: v }))} options={[['', t('common.all_crops')], ...CROPS.filter(c => c.code !== 'empty').map(c => [c.code, lang === 'ku' ? c.ku : c.en] as [string, string])]} /></Field>
          <Field label={t('common.governorate')}><Select value={rp.governorate} onChange={v => setRp(p => ({ ...p, governorate: v, zone: '' }))} options={[['', t('common.all_govs')], ...GOVERNORATES.map(g => [g.en, nm(g)] as [string, string])]} /></Field>
          <Field label={t('common.district')}><Select value={rp.zone} onChange={v => setRp(p => ({ ...p, zone: v }))} options={[['', t('common.all_dists')], ...DISTRICTS.filter(d => !rp.governorate || d.gov === rp.governorate).map(d => [d.slug, nm(d)] as [string, string])]} /></Field>
        </div>
      </Modal>}
    </div>
  );
}
