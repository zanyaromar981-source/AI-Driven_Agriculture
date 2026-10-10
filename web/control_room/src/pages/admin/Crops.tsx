// Crop register: every crop the farms grow, added up per district and governorate, with the expected
// harvest (area x typical yield, an estimate), and the crop list itself (add, change, switch off, delete).
// Cost: one pass over the farms builds every total on this page; it reruns only when farms change.
import { useMemo, useState } from 'react';
import { Plus, FileText, Wheat, Map as MapIcon, Sprout, Scale, Trash2, Info } from 'lucide-react';
import { db, PLACES, ensureProducts } from '../../data/db';
import { useRows, useVersion } from '../../data/store';
import { cropInUse, downloadCsv, isCrop } from '../../data/api';
import { GROUPS, UNITS, useProductSync } from '../../data/backend';
import type { Crop, CropCategory, CropSeason, ProductGroup, ProductUnit } from '../../data/types';
import { useI18n } from '../../i18n';
import { PageHead, Card, Kpi, Pill, Note, Switch, Tabs, Field, Select, Modal, Confirm, useToast } from '../../components/ui';
import { DataTable, type Col } from '../../components/DataTable';
import { HBars } from '../../components/charts';
import { DistrictMap, GOLD_RAMP, ramp } from '../../components/DistrictMap';
import { CropTag, usePlaceNames, usePlaceOptions } from '../../components/domain';

const CATS: CropCategory[] = ['cereal', 'vegetable', 'fruit', 'legume', 'oil', 'fodder', 'other'];
const SEASONS: CropSeason[] = ['winter', 'summer', 'perennial'];
ensureProducts();

interface CropStat { dunam: number; farms: number }
/** One pass: per crop, per district per crop, per governorate per crop. */
function useCropStats() {
  const v = useVersion(db.farms);
  return useMemo(() => {
    const byCrop = new Map<string, CropStat>();
    const byDist = new Map<string, Map<string, number>>();
    const byGov = new Map<string, Map<string, number>>();
    let area = 0, farms = 0;
    const add = (m: Map<string, Map<string, number>>, k: string, crop: string, d: number) => {
      let x = m.get(k); if (!x) { x = new Map(); m.set(k, x); }
      x.set(crop, (x.get(crop) ?? 0) + d);
    };
    for (const f of db.farms.all()) {
      farms++; area += f.area;
      for (const c of f.crops) {
        let s = byCrop.get(c.crop); if (!s) { s = { dunam: 0, farms: 0 }; byCrop.set(c.crop, s); }
        s.dunam += c.dunam; s.farms++;
        add(byDist, f.dist, c.crop, c.dunam);
        add(byGov, f.gov, c.crop, c.dunam);
      }
    }
    return { byCrop, byDist, byGov, area, farms };
    // v: rebuild when farms change
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [v]);
}

const sumMap = (m?: Map<string, number>) => { let s = 0; m?.forEach(v => { s += v; }); return s; };

export default function Crops() {
  const { t } = useI18n();
  const [tab, setTab] = useState<'overview' | 'list'>('overview');
  const [report, setReport] = useState(false);
  return (
    <>
      <PageHead eyebrow={t('nav.g_fields')} title={t('crops.title')} sub={t('crops.sub')}
        actions={<button className="btn" onClick={() => setReport(true)}><FileText />{t('crops.report')}</button>} />
      <Tabs value={tab} onChange={setTab} items={[['overview', t('crops.tab_overview')], ['list', t('crops.tab_list')]]} />
      {tab === 'overview' ? <CropOverview /> : <CropList />}
      {report && <ReportModal onClose={() => setReport(false)} />}
    </>
  );
}

function CropOverview() {
  const { t, num, b } = useI18n();
  const crops = useRows(db.crops);
  const st = useCropStats();
  const place = usePlaceNames();
  const [sel, setSel] = useState('');
  const byId = useMemo(() => new Map(crops.map(c => [c.id, c])), [crops]);

  const rows = useMemo(() => [...st.byCrop].map(([id, s]) => {
    const c = byId.get(id);
    return { id, ...s, crop: c, harvest: c ? s.dunam * c.yieldKgPerDunam / 1000 : 0 };
  }).sort((a, z) => z.dunam - a.dunam), [st, byId]);
  const totalHarvest = rows.reduce((a, r) => a + r.harvest, 0);
  const cropDunam = (m?: Map<string, number>) => (sel ? m?.get(sel) ?? 0 : sumMap(m));

  const distVals = useMemo(() => {
    const m = new Map<string, number>();
    for (const d of PLACES.districts) m.set(d.en, cropDunam(st.byDist.get(d.en)));
    return m;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [st, sel]);
  const maxD = Math.max(1, ...distVals.values());
  const topDists = useMemo(() => [...distVals].filter(([, v]) => v > 0).sort((a, z) => z[1] - a[1]).slice(0, 10), [distVals]);
  const selCrop = sel ? byId.get(sel) : undefined;
  const regionTotal = useMemo(() => { let s = 0; distVals.forEach(v => { s += v; }); return s; }, [distVals]);

  return (
    <>
      <div className="grid g4 mb">
        <Kpi label={t('crops.k_crops')} value={num(crops.filter(c => c.active && isCrop(c)).length)} note={t('crops.k_crops_n', { n: num(crops.filter(isCrop).length) })} icon={<Wheat />} />
        <Kpi label={t('crops.k_area')} value={num(st.area)} note={t('common.dunam')} icon={<MapIcon />} />
        <Kpi label={t('crops.k_farms')} value={num(st.farms)} note={t('crops.k_farms_n')} icon={<Sprout />} />
        <Kpi label={t('crops.k_harvest')} value={num(totalHarvest)} note={t('crops.k_harvest_n')} tone="gold" icon={<Scale />} />
      </div>

      <div className="chips mb" role="group" aria-label={t('crops.pick')}>
        <button className={'chip' + (sel === '' ? ' on' : '')} onClick={() => setSel('')}>{t('common.all_crops')}</button>
        {rows.map(r => (
          <button key={r.id} className={'chip' + (sel === r.id ? ' on' : '')} onClick={() => setSel(r.id)}>
            <span className="dotc" style={{ background: r.crop?.color ?? '#ccc', margin: 0 }} />{r.crop ? b(r.crop.name) : r.id}
          </button>
        ))}
      </div>

      <div className="grid g-main mb">
        <Card title={selCrop ? t('crops.map_one', { crop: b(selCrop.name) }) : t('crops.map_all')}>
          <DistrictMap styleKey={sel + ':' + st.area} fill={d => { const v = distVals.get(d) ?? 0; return v ? ramp(v / maxD, [0.05, 0.15, 0.35, 0.6], GOLD_RAMP) : undefined; }} />
          <div className="legend">{GOLD_RAMP.map((c, i) => <span key={c}><i className="dotc" style={{ background: c }} />{['< 5%', '5 to 15%', '15 to 35%', '35 to 60%', '60%+'][i]}</span>)}<span className="muted">{t('crops.legend_of_max')}</span></div>
        </Card>
        <div className="stack">
          <Card title={t('crops.top_dists')}>
            {topDists.length ? (
              <table className="t">
                <thead><tr><th>{t('common.district')}</th><th className="num">{t('common.dunam')}</th><th className="num">{t('crops.share_region')}</th></tr></thead>
                <tbody>{topDists.map(([d, v]) => (
                  <tr key={d}><td><b>{place.dist(d)}</b></td><td className="num">{num(v)}</td><td className="num muted">{num(regionTotal ? v / regionTotal * 100 : 0, 1)}%</td></tr>
                ))}</tbody>
              </table>
            ) : <div className="empty">{t('crops.none_here')}</div>}
          </Card>
          {selCrop && (
            <Card title={t('crops.this_crop')}>
              <dl className="facts">
                <dt>{t('crops.f_area')}</dt><dd>{num(st.byCrop.get(sel)?.dunam ?? 0)} {t('common.du')}</dd>
                <dt>{t('crops.f_farms')}</dt><dd>{num(st.byCrop.get(sel)?.farms ?? 0)}</dd>
                <dt>{t('crops.f_yield')}</dt><dd>{num(selCrop.yieldKgPerDunam)} {t('crops.kg_du')}</dd>
                <dt>{t('crops.f_harvest')}</dt><dd>{num((st.byCrop.get(sel)?.dunam ?? 0) * selCrop.yieldKgPerDunam / 1000)} {t('common.tonnes')}</dd>
                <dt>{t('crops.f_season')}</dt><dd>{t('crops.season_' + selCrop.season)}</dd>
              </dl>
            </Card>
          )}
        </div>
      </div>

      <div className="grid g2 mb">
        <Card title={t('crops.dunam_per_crop')}>
          <HBars showShare unit={t('common.du')} items={rows.map(r => ({ key: r.id, label: r.crop ? b(r.crop.name) : r.id, value: r.dunam, color: r.crop?.color }))} />
        </Card>
        <Card title={t('crops.harvest_title')} extra={<Pill tone="gold">{t('crops.estimate')}</Pill>}>
          <div className="table-wrap">
            <table className="t cards">
              <thead><tr><th>{t('crops.crop')}</th><th className="num">{t('common.farms')}</th><th className="num">{t('common.dunam')}</th><th className="num">{t('crops.kg_du')}</th><th className="num">{t('crops.tonnes_est')}</th></tr></thead>
              <tbody>{rows.map(r => (
                <tr key={r.id}>
                  <td data-label={t('crops.crop')}><CropTag id={r.id} /></td>
                  <td data-label={t('common.farms')} className="num">{num(r.farms)}</td>
                  <td data-label={t('common.dunam')} className="num">{num(r.dunam)}</td>
                  <td data-label={t('crops.kg_du')} className="num">{num(r.crop?.yieldKgPerDunam ?? 0)}</td>
                  <td data-label={t('crops.tonnes_est')} className="num"><b>{num(r.harvest)}</b></td>
                </tr>))}</tbody>
            </table>
          </div>
          <p className="muted small">{t('crops.harvest_note')}</p>
        </Card>
      </div>

      <Card title={t('crops.by_gov')}>
        <div className="table-wrap">
          <table className="t">
            <thead><tr><th>{t('crops.crop')}</th>{PLACES.governorates.map(g => <th key={g.en} className="num">{place.gov(g.en)}</th>)}<th className="num">{t('crops.total')}</th></tr></thead>
            <tbody>{rows.map(r => (
              <tr key={r.id}>
                <td><CropTag id={r.id} /></td>
                {PLACES.governorates.map(g => <td key={g.en} className="num">{num(st.byGov.get(g.en)?.get(r.id) ?? 0)}</td>)}
                <td className="num"><b>{num(r.dunam)}</b></td>
              </tr>))}</tbody>
          </table>
        </div>
      </Card>
    </>
  );
}

function CropList() {
  const { t, num, b } = useI18n();
  const toast = useToast();
  const crops = useRows(db.crops);
  const st = useCropStats();
  const [edit, setEdit] = useState<Crop | 'new' | null>(null);
  const synced = useProductSync();

  const cols: Col<Crop>[] = [
    { key: 'name', label: t('crops.crop'), cell: c => <span className="row" style={{ flexWrap: 'nowrap' }}><span className="dotc" style={{ background: c.color }} /><b><bdi>{b(c.name)}</bdi></b></span>, sort: c => b(c.name) },
    { key: 'code', label: t('crops.code'), cell: c => <span className="mono ltr">{c.id}</span>, sort: c => c.id },
    { key: 'group', label: t('crops.group'), cell: c => t('crops.group_' + (c.group ?? 'crops')), sort: c => c.group ?? 'crops' },
    { key: 'unit', label: t('crops.unit'), cell: c => t('crops.unit_' + (c.unit ?? 'kg')), sort: c => c.unit ?? 'kg' },
    { key: 'cat', label: t('crops.category'), cell: c => t('crops.cat_' + c.category), sort: c => c.category },
    { key: 'season', label: t('crops.season'), cell: c => t('crops.season_' + c.season), sort: c => c.season },
    { key: 'yield', label: t('crops.kg_du'), num: true, cell: c => num(c.yieldKgPerDunam), sort: c => c.yieldKgPerDunam },
    { key: 'farms', label: t('common.farms'), num: true, cell: c => num(st.byCrop.get(c.id)?.farms ?? 0), sort: c => st.byCrop.get(c.id)?.farms ?? 0 },
    { key: 'dunam', label: t('common.dunam'), num: true, cell: c => num(st.byCrop.get(c.id)?.dunam ?? 0), sort: c => st.byCrop.get(c.id)?.dunam ?? 0 },
    { key: 'ku', label: t('crops.ku_name'), optional: true, cell: c => c.name.ku ? <bdi className="ku-text">{c.name.ku}</bdi> : <Pill tone="warn">{t('common.ku_missing')}</Pill> },
    { key: 'active', label: t('common.status'), cell: c => c.active ? <Pill tone="good">{t('crops.in_app')}</Pill> : <Pill>{t('crops.off')}</Pill>, sort: c => (c.active ? 0 : 1) },
  ];
  const exportCsv = () => downloadCsv('crops', ['code', 'name_en', 'name_ku', 'group', 'unit', 'category', 'season', 'yield_kg_per_dunam', 'farms', 'dunam', 'active'],
    crops.map(c => [c.id, c.name.en, c.name.ku, c.group ?? 'crops', c.unit ?? 'kg', c.category, c.season, c.yieldKgPerDunam, st.byCrop.get(c.id)?.farms ?? 0, Math.round(st.byCrop.get(c.id)?.dunam ?? 0), c.active ? 'yes' : 'no']));

  return (
    <Card>
      <DataTable id="crops" rows={crops} cols={cols} onRow={c => setEdit(c)} defaultSort={['dunam', -1]}
        head={<><button className="btn primary sm" onClick={() => setEdit('new')}><Plus />{t('crops.add')}</button>
          <button className="btn sm" onClick={exportCsv}>{t('common.export_csv')}</button></>} />
      {synced !== undefined && <p className="muted small">{synced ? t('crops.products_live', { n: num(synced) }) : t('crops.products_sample')}</p>}
      {edit && <CropForm crop={edit === 'new' ? null : edit} onClose={() => setEdit(null)} onSaved={msg => { toast(msg, 'good'); setEdit(null); }} />}
    </Card>
  );
}

function CropForm({ crop, onClose, onSaved }: { crop: Crop | null; onClose: () => void; onSaved: (msg: string) => void }) {
  const { t, b } = useI18n();
  const [f, setF] = useState<Crop>(() => crop ?? { id: '', name: { en: '', ku: '' }, color: '#7FBC93', category: 'vegetable', season: 'summer', yieldKgPerDunam: 1000, active: true, notes: '', group: 'crops', unit: 'kg' });
  const [err, setErr] = useState<Record<string, string>>({});
  const [ask, setAsk] = useState<'delete' | 'off' | null>(null);
  const set = (p: Partial<Crop>) => setF(x => ({ ...x, ...p }));

  const save = () => {
    const e: Record<string, string> = {};
    if (!crop) {
      if (!/^[a-z_]{2,24}$/.test(f.id)) e.id = 'crops.v_code';
      else if (db.crops.has(f.id)) e.id = 'crops.v_code_taken';
    }
    if (!f.name.en.trim() && !f.name.ku.trim()) e.name = 'v.name_needed';
    if (isCrop(f) && !(f.yieldKgPerDunam > 0)) e.yield = 'v.positive'; // only a crop has a yield per dunam
    setErr(e);
    if (Object.keys(e).length) return;
    db.crops.put({ ...f, name: { en: f.name.en.trim(), ku: f.name.ku.trim() } });
    onSaved(t('common.saved'));
  };
  const inUse = crop ? cropInUse(crop.id) : false;

  return (
    <Modal title={crop ? b(crop.name) : t('crops.add')} onClose={onClose} wide foot={<>
      {crop && <button className="btn danger" style={{ marginInlineEnd: 'auto' }} onClick={() => setAsk(inUse ? 'off' : 'delete')}><Trash2 />{t('common.delete')}</button>}
      <button className="btn" onClick={onClose}>{t('common.cancel')}</button>
      <button className="btn primary" onClick={save}>{t('common.save')}</button>
    </>}>
      <div className="form-grid">
        <Field label={t('crops.code')} hint={crop ? t('crops.code_fixed') : t('crops.code_hint')} error={err.id}>
          <input type="text" className="ltr-input mono" value={f.id} disabled={!!crop} onChange={e => set({ id: e.target.value.toLowerCase() })} />
        </Field>
        <Field label={t('crops.colour')}>
          <div className="row" style={{ flexWrap: 'nowrap' }}>
            <input type="color" value={f.color} onChange={e => set({ color: e.target.value })} style={{ width: 52, height: 38, padding: 2, border: '1px solid var(--line)', borderRadius: 9, background: 'var(--surface)' }} />
            <input type="text" className="ltr-input mono" value={f.color} onChange={e => set({ color: e.target.value })} />
          </div>
        </Field>
        <Field label={t('crops.name_en')} error={err.name}><input type="text" dir="ltr" value={f.name.en} onChange={e => set({ name: { ...f.name, en: e.target.value } })} /></Field>
        <Field label={t('crops.ku_name')}><input type="text" dir="rtl" className="ku-text" value={f.name.ku} onChange={e => set({ name: { ...f.name, ku: e.target.value } })} /></Field>
        <Field label={t('crops.group')} hint={t('crops.group_hint')}><Select value={f.group ?? 'crops'} onChange={v => set({ group: v as ProductGroup })} options={GROUPS.map(g => [g, t('crops.group_' + g)])} /></Field>
        <Field label={t('crops.unit')} hint={t('crops.unit_hint')}><Select value={f.unit ?? 'kg'} onChange={v => set({ unit: v as ProductUnit })} options={UNITS.map(u => [u, t('crops.unit_' + u)])} /></Field>
        <Field label={t('crops.category')}><Select value={f.category} onChange={v => set({ category: v as CropCategory })} options={CATS.map(c => [c, t('crops.cat_' + c)])} /></Field>
        <Field label={t('crops.season')}><Select value={f.season} onChange={v => set({ season: v as CropSeason })} options={SEASONS.map(s => [s, t('crops.season_' + s)])} /></Field>
        <Field label={t('crops.yield')} hint={t('crops.yield_hint')} error={err.yield}>
          <input type="number" min={0} value={f.yieldKgPerDunam} onChange={e => set({ yieldKgPerDunam: +e.target.value })} />
        </Field>
        <Field label={t('crops.in_app_q')}>
          <div className="row" style={{ minHeight: 38 }}><Switch on={f.active} onChange={v => set({ active: v })} label={t('crops.in_app_q')} /><span className="small muted">{f.active ? t('crops.in_app') : t('crops.off')}</span></div>
        </Field>
        <Field label={t('common.notes')} full><textarea value={f.notes} onChange={e => set({ notes: e.target.value })} rows={2} /></Field>
      </div>
      {ask === 'delete' && <Confirm title={t('crops.delete_q')} text={t('crops.delete_text')} okLabel={t('common.delete')} onClose={() => setAsk(null)}
        onOk={() => { db.crops.remove(crop!.id); onSaved(t('common.deleted')); }} />}
      {ask === 'off' && <Confirm danger={false} title={t('crops.in_use_q')} text={t('crops.in_use_text')} okLabel={t('crops.switch_off')} onClose={() => setAsk(null)}
        onOk={() => { db.crops.patch(crop!.id, { active: false }); onSaved(t('crops.switched_off')); }} />}
    </Modal>
  );
}

function ReportModal({ onClose }: { onClose: () => void }) {
  const { t, b } = useI18n();
  const crops = useRows(db.crops);
  const [gov, setGov] = useState(''), [dist, setDist] = useState(''), [crop, setCrop] = useState('');
  const o = usePlaceOptions(gov, dist);
  const url = `#/print/crops?gov=${encodeURIComponent(gov)}&dist=${encodeURIComponent(dist)}&crop=${encodeURIComponent(crop)}`;
  return (
    <Modal title={t('crops.report')} onClose={onClose} foot={<>
      <button className="btn" onClick={onClose}>{t('common.cancel')}</button>
      <a className="btn primary" href={url} target="_blank" rel="noopener" onClick={onClose}><FileText />{t('crops.open_report')}</a>
    </>}>
      <p className="muted" style={{ marginTop: 0 }}>{t('crops.report_sub')}</p>
      <div className="form-grid">
        <Field label={t('crops.crop')} full><Select value={crop} onChange={setCrop} options={[['', t('common.all_crops')], ...crops.filter(isCrop).map(c => [c.id, b(c.name)] as [string, string])]} /></Field>
        <Field label={t('common.governorate')}><Select value={gov} onChange={v => { setGov(v); setDist(''); }} options={[['', t('common.all_govs')], ...o.govs]} /></Field>
        <Field label={t('common.district')}><Select value={dist} onChange={setDist} options={[['', t('common.all_dists')], ...o.dists]} /></Field>
      </div>
      <div style={{ marginTop: 12 }}><Note tone="info" icon={<Info />}>{t('crops.report_note')}</Note></div>
    </Modal>
  );
}
