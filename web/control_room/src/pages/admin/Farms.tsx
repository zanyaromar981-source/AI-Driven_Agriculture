// Farmers and farms: every registered farmer and farm, with filters, chosen columns, full create, edit and
// delete, a support letter per farmer and the government report.
// Cost: filters run in one pass over the list (O(n)) after typing settles; owner lookups are O(1) by id;
// only one page of rows is drawn.
import { useMemo, useState, useEffect } from 'react';
import { useSearchParams } from 'react-router-dom';
import {
  Users, Map as MapIcon, Ruler, UserRound, Droplets, Plus, Download, FileText, Pencil, Trash2, Ban, CircleCheck,
  FileSignature, RotateCcw, X, MapPin,
} from 'lucide-react';
import { useI18n } from '../../i18n';
import { db } from '../../data/db';
import { useRows } from '../../data/store';
import {
  totals, farmsByFarmer, farmsOf, checkFarmer, saveFarmer, deleteFarmer, checkFarm, saveFarm, deleteFarm,
  normPhone, nowIso, distByName, subByKey, downloadCsv, type Problems,
} from '../../data/api';
import type { Farmer, Farm, FarmCrop, Irrigation, WaterSource, Ownership, FieldLevel, Gender } from '../../data/types';
import { PageHead, Kpi, Tabs, Drawer, Modal, Confirm, Field, Select, Pill, Note, useDebounced, useToast } from '../../components/ui';
import { DataTable, type Col } from '../../components/DataTable';
import { CropTag, LevelPill, Phone, useCrops, usePlaceNames, usePlaceOptions } from '../../components/domain';
import { DistrictMap } from '../../components/DistrictMap';
import './farms.css';

type Tab = 'farmers' | 'farms';
interface Filter { q: string; gov: string; dist: string; crop: string; level: string; irrigation: string; status: string }
const NO_FILTER: Filter = { q: '', gov: '', dist: '', crop: '', level: '', irrigation: '', status: '' };
const LEVELS: FieldLevel[] = ['normal', 'watch', 'alarm', 'none'];
const IRRIGATIONS: Irrigation[] = ['rainfed', 'irrigated', 'mixed'];
const WATERS: WaterSource[] = ['rain', 'well', 'river', 'canal', 'spring'];
const OWNERSHIPS: Ownership[] = ['owned', 'rented', 'shared'];

/** Next free number id (ids are numbers kept as strings). One pass, only when saving something new. */
const nextId = (rows: { id: string }[]) => String(rows.reduce((m, r) => Math.max(m, Number(r.id) || 0), 0) + 1);
const openPrint = (path: string) => window.open(`${location.pathname}${location.search}#${path}`, '_blank', 'noopener');

export default function Farms() {
  const { t, num, b, date } = useI18n();
  const toast = useToast();
  const farmers = useRows(db.farmers);
  const farms = useRows(db.farms);
  const crops = useCrops();
  const pn = usePlaceNames();
  const [params, setParams] = useSearchParams();
  const [tab, setTab] = useState<Tab>(() => (params.get('farm') ? 'farms' : 'farmers'));
  // ?dist=<district> from the Overview map opens the list filtered to that district
  const [f, setF] = useState<Filter>(() => {
    const d = params.get('dist'), dd = d ? distByName.get(d) : undefined;
    return dd ? { ...NO_FILTER, gov: dd.gov, dist: dd.en } : NO_FILTER;
  });
  const q = useDebounced(f.q.trim().toLowerCase(), 200);
  const opts = usePlaceOptions(f.gov, f.dist);

  // drawers and forms
  const [farmerId, setFarmerId] = useState<string | null>(null);
  const [farmId, setFarmId] = useState<string | null>(null);
  const [farmerForm, setFarmerForm] = useState<Farmer | null>(null);
  const [farmForm, setFarmForm] = useState<Farm | null>(null);
  const [confirm, setConfirm] = useState<{ title: string; text: string; ok: string; run: () => void } | null>(null);

  // ?farmer=<id> or ?farm=<id> from the global search
  useEffect(() => {
    const pf = params.get('farmer'), pm = params.get('farm');
    if (pf && db.farmers.has(pf)) { setTab('farmers'); setFarmerId(pf); setFarmId(null); }
    if (pm && db.farms.has(pm)) { setTab('farms'); setFarmId(pm); setFarmerId(null); }
    if (pf || pm || params.get('dist')) setParams({}, { replace: true });
  }, [params, setParams]);

  const T = useMemo(() => totals(), [farms, farmers]);
  const byFarmer = useMemo(() => farmsByFarmer(), [farms]);
  const set = (k: keyof Filter, v: string) => setF(p => ({ ...p, [k]: v, ...(k === 'gov' ? { dist: '' } : {}) }));

  // ---------- filtering: one pass each ----------
  const farmMatches = (x: Farm) => (!f.crop || x.crops.some(c => c.crop === f.crop)) && (!f.level || x.level === f.level) && (!f.irrigation || x.irrigation === f.irrigation);
  const farmFilterOn = !!(f.crop || f.level || f.irrigation);

  const farmerRows = useMemo(() => {
    const digits = q.replace(/\D/g, '');
    return farmers.filter(p => {
      if (f.gov && p.gov !== f.gov) return false;
      if (f.dist && p.dist !== f.dist) return false;
      if (f.status && p.status !== f.status) return false;
      if (q && !(p.name.en.toLowerCase().includes(q) || p.name.ku.includes(q) || p.village.toLowerCase().includes(q) || p.id === q || (digits.length >= 3 && p.phone.includes(digits)))) return false;
      if (farmFilterOn && !(byFarmer.get(p.id) ?? []).some(farmMatches)) return false;
      return true;
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [farmers, byFarmer, q, f.gov, f.dist, f.status, f.crop, f.level, f.irrigation]);

  const farmRows = useMemo(() => {
    const digits = q.replace(/\D/g, '');
    return farms.filter(x => {
      if (f.gov && x.gov !== f.gov) return false;
      if (f.dist && x.dist !== f.dist) return false;
      if (!farmMatches(x)) return false;
      if (f.status) { const o = db.farmers.get(x.farmerId); if (!o || o.status !== f.status) return false; }
      if (q) {
        if (x.id === q || x.name.includes(q) || x.village.toLowerCase().includes(q)) return true;
        const o = db.farmers.get(x.farmerId);
        return !!o && (o.name.en.toLowerCase().includes(q) || o.name.ku.includes(q) || (digits.length >= 3 && o.phone.includes(digits)));
      }
      return true;
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [farms, farmers, q, f.gov, f.dist, f.status, f.crop, f.level, f.irrigation]);

  const listArea = useMemo(() => farmRows.reduce((a, x) => a + x.area, 0), [farmRows]);

  // ---------- columns ----------
  const farmerArea = (id: string) => (byFarmer.get(id) ?? []).reduce((a, x) => a + x.area, 0);
  const farmerCols: Col<Farmer>[] = [
    { key: 'name', label: t('farms.c_name'), sort: r => b(r.name), cell: r => <div><b><bdi>{b(r.name)}</bdi></b><div className="muted small">#{r.id}</div></div> },
    { key: 'phone', label: t('farms.c_phone'), cell: r => <Phone value={r.phone} /> },
    { key: 'place', label: t('farms.c_place'), sort: r => r.gov + r.dist, cell: r => <div>{pn.sub(r.dist, r.sub)}<div className="muted small">{pn.dist(r.dist)}, {pn.gov(r.gov)}</div></div> },
    { key: 'village', label: t('common.village'), optional: true, sort: r => r.village, cell: r => <bdi>{r.village || '-'}</bdi> },
    { key: 'gender', label: t('farms.f_gender'), optional: true, sort: r => r.gender, cell: r => t('farms.g_' + r.gender) },
    { key: 'age', label: t('farms.c_age'), optional: true, num: true, sort: r => r.birthYear ?? 0, cell: r => (r.birthYear ? num(new Date().getFullYear() - r.birthYear) : '-') },
    { key: 'farms', label: t('common.farms'), num: true, sort: r => (byFarmer.get(r.id) ?? []).length, cell: r => num((byFarmer.get(r.id) ?? []).length) },
    { key: 'area', label: t('farms.c_area'), num: true, sort: r => farmerArea(r.id), cell: r => num(farmerArea(r.id), 1) },
    { key: 'joined', label: t('farms.c_joined'), sort: r => r.joined, cell: r => <span className="nowrap">{date(r.joined, 'short')}</span> },
    { key: 'status', label: t('common.status'), sort: r => r.status, cell: r => <Pill tone={r.status === 'active' ? 'good' : 'danger'}>{t('common.' + r.status)}</Pill> },
  ];
  const farmCols: Col<Farm>[] = [
    { key: 'name', label: t('farms.c_farm'), sort: r => Number(r.id), cell: r => <div><b className="ku-text"><bdi>{r.name}</bdi></b><div className="muted small">#{r.id}</div></div> },
    { key: 'owner', label: t('farms.c_owner'), sort: r => b(db.farmers.get(r.farmerId)?.name), cell: r => { const o = db.farmers.get(r.farmerId); return o ? <bdi>{b(o.name)}</bdi> : '-'; } },
    { key: 'ophone', label: t('farms.c_phone'), optional: true, cell: r => { const o = db.farmers.get(r.farmerId); return o ? <Phone value={o.phone} /> : '-'; } },
    { key: 'place', label: t('farms.c_place'), sort: r => r.gov + r.dist + r.sub, cell: r => <div>{pn.sub(r.dist, r.sub)}<div className="muted small">{pn.dist(r.dist)}, {pn.gov(r.gov)}</div></div> },
    { key: 'village', label: t('common.village'), optional: true, sort: r => r.village, cell: r => <bdi>{r.village || '-'}</bdi> },
    { key: 'area', label: t('farms.c_area'), num: true, sort: r => r.area, cell: r => num(r.area, 1) },
    { key: 'crops', label: t('farms.c_crops'), sort: r => r.crops[0]?.crop ?? '', cell: r => <div className="row" style={{ gap: 10 }}>{r.crops.map(c => <CropTag key={c.crop} id={c.crop} />)}</div> },
    { key: 'irrigation', label: t('farms.f_irrigation'), sort: r => r.irrigation, cell: r => t('farms.irr_' + r.irrigation) },
    { key: 'water', label: t('farms.f_water'), optional: true, sort: r => r.water, cell: r => t('farms.w_' + r.water) },
    { key: 'ownership', label: t('farms.f_ownership'), optional: true, sort: r => r.ownership, cell: r => t('farms.o_' + r.ownership) },
    { key: 'level', label: t('farms.c_level'), sort: r => LEVELS.indexOf(r.level), cell: r => <LevelPill level={r.level} /> },
    { key: 'point', label: t('farms.c_point'), optional: true, cell: r => <span className="ltr mono">{r.lat.toFixed(4)}, {r.lon.toFixed(4)}</span> },
    { key: 'created', label: t('farms.c_created'), optional: true, sort: r => r.created, cell: r => <span className="nowrap">{date(r.created, 'short')}</span> },
    { key: 'updated', label: t('common.updated'), sort: r => r.updated, cell: r => <span className="nowrap">{date(r.updated, 'short')}</span> },
  ];

  // ---------- actions ----------
  const newFarmer = (): Farmer => ({
    id: '', name: { en: '', ku: '' }, phone: '', gender: 'male', birthYear: null, gov: f.gov || 'Sulaymaniyah', dist: f.dist, sub: '',
    village: '', status: 'active', joined: nowIso(), notes: '',
  });
  const newFarm = (owner: Farmer | null): Farm => {
    const c = owner ? (subByKey.get(owner.dist + '|' + owner.sub) ?? distByName.get(owner.dist))?.c : undefined;
    return {
      id: '', farmerId: owner?.id ?? '', name: '', gov: owner?.gov ?? '', dist: owner?.dist ?? '', sub: owner?.sub ?? '', village: owner?.village ?? '',
      lat: c?.[0] ?? 0, lon: c?.[1] ?? 0, area: 0, crops: [{ crop: 'wheat', dunam: 0 }], irrigation: 'rainfed', water: 'rain', ownership: 'owned',
      level: 'none', created: nowIso(), updated: nowIso(), notes: '',
    };
  };
  const askDeleteFarmer = (p: Farmer) => {
    const n = farmsOf(p.id).length;
    setConfirm({ title: t('farms.del_farmer_t'), text: t('farms.del_farmer_x', { name: b(p.name), n: num(n) }), ok: t('common.delete'),
      run: () => { deleteFarmer(p.id); setFarmerId(null); toast(t('common.deleted'), 'good'); } });
  };
  const askDeleteFarm = (x: Farm) => setConfirm({ title: t('farms.del_farm_t'), text: t('farms.del_farm_x', { id: x.id }), ok: t('common.delete'),
    run: () => { deleteFarm(x.id); setFarmId(null); toast(t('common.deleted'), 'good'); } });
  const toggleBlock = (p: Farmer) => {
    db.farmers.patch(p.id, { status: p.status === 'active' ? 'blocked' : 'active' });
    toast(t(p.status === 'active' ? 'farms.blocked_toast' : 'farms.unblocked_toast'), 'good');
  };

  const exportCsv = () => {
    if (tab === 'farmers') {
      downloadCsv('jutyar_farmers', ['id', 'name_en', 'name_ku', 'phone', 'gender', 'birth_year', 'governorate', 'district', 'sub_district', 'village', 'farms', 'dunam', 'status', 'joined'],
        farmerRows.map(p => [p.id, p.name.en, p.name.ku, p.phone, p.gender, p.birthYear, p.gov, p.dist, p.sub, p.village, (byFarmer.get(p.id) ?? []).length, farmerArea(p.id).toFixed(1), p.status, p.joined.slice(0, 10)]));
    } else {
      downloadCsv('jutyar_farms', ['id', 'name', 'farmer_id', 'farmer', 'phone', 'governorate', 'district', 'sub_district', 'village', 'lat', 'lon', 'dunam', 'crops', 'irrigation', 'water', 'ownership', 'status', 'created', 'updated'],
        farmRows.map(x => { const o = db.farmers.get(x.farmerId); return [x.id, x.name, x.farmerId, o?.name.en, o?.phone, x.gov, x.dist, x.sub, x.village, x.lat, x.lon, x.area, x.crops.map(c => `${c.crop} ${c.dunam}`).join('; '), x.irrigation, x.water, x.ownership, x.level, x.created.slice(0, 10), x.updated.slice(0, 10)]; }));
    }
    toast(t('farms.exported'), 'good');
  };
  const govReport = () => {
    const s = new URLSearchParams(); if (f.gov) s.set('gov', f.gov); if (f.dist) s.set('dist', f.dist);
    openPrint('/print/government' + (s.toString() ? '?' + s : ''));
  };

  const A = T.all, women = A.farmers ? A.female / A.farmers * 100 : 0, irr = A.area ? A.irrigated / A.area * 100 : 0;
  const filtersOn = JSON.stringify(f) !== JSON.stringify(NO_FILTER);

  return (
    <>
      <PageHead eyebrow={t('nav.g_people')} title={t('nav.farms')} sub={t('farms.sub')} actions={<>
        <button className="btn" onClick={exportCsv}><Download />{t('common.export_csv')}</button>
        <button className="btn" onClick={govReport}><FileText />{t('farms.gov_report')}</button>
        <button className="btn" onClick={() => setFarmForm(newFarm(null))}><MapPin />{t('farms.add_farm')}</button>
        <button className="btn primary" onClick={() => setFarmerForm(newFarmer())}><Plus />{t('farms.add_farmer')}</button>
      </>} />

      <div className="grid farms-kpis mb">
        <Kpi label={t('common.farmers')} value={num(A.farmers)} note={t('farms.k_farmers_n', { n: num(farmers.filter(p => p.status === 'blocked').length) })} icon={<Users size={15} />} />
        <Kpi label={t('common.farms')} value={num(A.farms)} note={t('farms.k_farms_n', { n: num(A.farmers ? A.farms / A.farmers : 0, 1) })} icon={<MapIcon size={15} />} />
        <Kpi label={t('farms.k_area')} value={num(A.area)} note={t('farms.k_area_n', { n: num(A.farms ? A.area / A.farms : 0, 1) })} icon={<Ruler size={15} />} />
        <Kpi label={t('farms.k_women')} value={num(women, 1) + '%'} note={t('farms.k_women_n', { n: num(A.female) })} icon={<UserRound size={15} />} />
        <Kpi label={t('farms.k_irrigated')} value={num(irr, 1) + '%'} note={t('farms.k_irrigated_n', { n: num(A.irrigated) })} tone="water" icon={<Droplets size={15} />} />
      </div>

      <Tabs value={tab} onChange={setTab} items={[['farmers', t('farms.tab_farmers', { n: num(farmerRows.length) })], ['farms', t('farms.tab_farms', { n: num(farmRows.length) })]]} />

      <div className="card">
        <div className="filters">
          <input className="grow" type="search" value={f.q} onChange={e => set('q', e.target.value)} placeholder={t(tab === 'farmers' ? 'farms.search_farmers' : 'farms.search_farms')} aria-label={t('common.search')} />
          <Select value={f.gov} onChange={v => set('gov', v)} options={[['', t('common.all_govs')], ...opts.govs]} aria-label={t('common.governorate')} />
          <Select value={f.dist} onChange={v => set('dist', v)} options={[['', t('common.all_dists')], ...opts.dists]} aria-label={t('common.district')} />
          <Select value={f.crop} onChange={v => set('crop', v)} options={[['', t('common.all_crops')], ...crops.list.map(c => [c.id, b(c.name)] as [string, string])]} aria-label={t('farms.c_crops')} />
          <Select value={f.level} onChange={v => set('level', v)} options={[['', t('farms.any_level')], ...LEVELS.map(l => [l, t('level.' + l)] as [string, string])]} aria-label={t('farms.c_level')} />
          <Select value={f.irrigation} onChange={v => set('irrigation', v)} options={[['', t('farms.any_irrigation')], ...IRRIGATIONS.map(l => [l, t('farms.irr_' + l)] as [string, string])]} aria-label={t('farms.f_irrigation')} />
          <Select value={f.status} onChange={v => set('status', v)} options={[['', t('farms.any_status')], ['active', t('common.active')], ['blocked', t('common.blocked')]]} aria-label={t('common.status')} />
          {filtersOn && <button className="btn" onClick={() => setF(NO_FILTER)}><RotateCcw />{t('common.clear')}</button>}
        </div>
        {tab === 'farmers'
          ? <DataTable id="farmers" rows={farmerRows} cols={farmerCols} onRow={r => setFarmerId(r.id)} selected={farmerId}
            head={<span className="muted small">{t('farms.showing_farmers', { n: num(farmerRows.length) })}</span>} />
          : <DataTable id="farms" rows={farmRows} cols={farmCols} onRow={r => setFarmId(r.id)} selected={farmId} defaultSort={['updated', -1]}
            head={<span className="muted small">{t('farms.showing_farms', { n: num(farmRows.length), a: num(listArea) })}</span>} />}
      </div>

      {farmerId && db.farmers.get(farmerId) && (
        <FarmerDrawer p={db.farmers.get(farmerId)!} onClose={() => setFarmerId(null)}
          onEdit={p => setFarmerForm(p)} onDelete={askDeleteFarmer} onBlock={toggleBlock}
          onFarm={id => { setFarmerId(null); setFarmId(id); }} onAddFarm={p => setFarmForm(newFarm(p))} />
      )}
      {farmId && db.farms.get(farmId) && (
        <FarmDrawer x={db.farms.get(farmId)!} onClose={() => setFarmId(null)} onEdit={x => setFarmForm(x)} onDelete={askDeleteFarm}
          onOwner={id => { setFarmId(null); setFarmerId(id); }} />
      )}
      {farmerForm && <FarmerForm init={farmerForm} onClose={() => setFarmerForm(null)} onSaved={id => { setFarmerForm(null); setFarmerId(id); }} />}
      {farmForm && <FarmForm init={farmForm} onClose={() => setFarmForm(null)} onSaved={id => { setFarmForm(null); setFarmerId(null); setFarmId(id); }} />}
      {confirm && <Confirm title={confirm.title} text={confirm.text} okLabel={confirm.ok} onOk={confirm.run} onClose={() => setConfirm(null)} />}
    </>
  );
}

// ---------- farmer drawer ----------
function FarmerDrawer({ p, onClose, onEdit, onDelete, onBlock, onFarm, onAddFarm }: {
  p: Farmer; onClose: () => void; onEdit: (p: Farmer) => void; onDelete: (p: Farmer) => void; onBlock: (p: Farmer) => void;
  onFarm: (id: string) => void; onAddFarm: (p: Farmer) => void;
}) {
  const { t, b, num, date } = useI18n();
  const pn = usePlaceNames();
  useRows(db.farms);
  const list = farmsOf(p.id);
  const area = list.reduce((a, x) => a + x.area, 0);
  const byCrop = new Map<string, number>();
  for (const x of list) for (const c of x.crops) byCrop.set(c.crop, (byCrop.get(c.crop) ?? 0) + c.dunam);
  const listings = db.listings.all().filter(l => l.farmerId === p.id).length;
  const msgs = db.messages.all().filter(m => m.farmerId === p.id).length;
  return (
    <Drawer eyebrow={t('farms.farmer') + ' #' + p.id} title={<bdi>{b(p.name)}</bdi>} onClose={onClose}
      head={<Pill tone={p.status === 'active' ? 'good' : 'danger'}>{t('common.' + p.status)}</Pill>}>
      {p.status === 'blocked' && <div className="mb"><Note tone="danger" icon={<Ban />}>{t('farms.blocked_note')}</Note></div>}
      <div className="farm-actions mb">
        <button className="btn primary" onClick={() => openPrint('/print/farmer/' + p.id)}><FileSignature />{t('farms.letter')}</button>
        <button className="btn" onClick={() => onEdit(p)}><Pencil />{t('common.edit')}</button>
        <button className="btn" onClick={() => onAddFarm(p)}><Plus />{t('farms.add_farm')}</button>
        <button className="btn" onClick={() => onBlock(p)}>{p.status === 'active' ? <><Ban />{t('farms.block')}</> : <><CircleCheck />{t('farms.unblock')}</>}</button>
        <button className="btn danger" onClick={() => onDelete(p)}><Trash2 />{t('common.delete')}</button>
      </div>
      <div className="grid g3 mb farm-mini">
        <div className="card"><div className="eyebrow">{t('common.farms')}</div><b className="big">{num(list.length)}</b></div>
        <div className="card"><div className="eyebrow">{t('farms.c_area')}</div><b className="big">{num(area, 1)}</b></div>
        <div className="card"><div className="eyebrow">{t('farms.d_listings')}</div><b className="big">{num(listings)}</b></div>
      </div>
      <dl className="facts" style={{ marginBottom: 16 }}>
        <dt>{t('farms.f_name_ku')}</dt><dd className="ku-text"><bdi>{p.name.ku || '-'}</bdi></dd>
        <dt>{t('farms.f_name_en')}</dt><dd><bdi>{p.name.en || '-'}</bdi></dd>
        <dt>{t('farms.c_phone')}</dt><dd><Phone value={p.phone} /></dd>
        <dt>{t('farms.f_gender')}</dt><dd>{t('farms.g_' + p.gender)}</dd>
        <dt>{t('farms.f_birth')}</dt><dd>{p.birthYear ? `${p.birthYear} (${t('farms.years', { n: num(new Date().getFullYear() - p.birthYear) })})` : '-'}</dd>
        <dt>{t('farms.c_place')}</dt><dd>{pn.sub(p.dist, p.sub)}, {pn.dist(p.dist)}, {pn.gov(p.gov)}</dd>
        <dt>{t('common.village')}</dt><dd><bdi>{p.village || '-'}</bdi></dd>
        <dt>{t('farms.c_joined')}</dt><dd>{date(p.joined)}</dd>
        <dt>{t('farms.d_messages')}</dt><dd>{num(msgs)}</dd>
        {p.notes && <><dt>{t('common.notes')}</dt><dd><bdi>{p.notes}</bdi></dd></>}
      </dl>
      {byCrop.size > 0 && (
        <div className="mb">
          <div className="eyebrow" style={{ marginBottom: 6 }}>{t('farms.d_crops')}</div>
          <div className="row" style={{ gap: 14 }}>{[...byCrop].sort((a, z) => z[1] - a[1]).map(([c, a]) => <span key={c}><CropTag id={c} /> <b className="tabular">{num(a, 1)}</b> <span className="muted small">{t('common.du')}</span></span>)}</div>
        </div>
      )}
      <div className="eyebrow" style={{ marginBottom: 6 }}>{t('farms.d_farms', { n: num(list.length) })}</div>
      {list.length === 0 ? <div className="empty">{t('farms.no_farms')}</div> : list.map(x => (
        <div key={x.id} className="list-item click" role="button" tabIndex={0} onClick={() => onFarm(x.id)} onKeyDown={e => { if (e.key === 'Enter') onFarm(x.id); }}>
          <span className="ico brand"><MapIcon /></span>
          <div style={{ flex: 1, minWidth: 0 }}>
            <b className="ku-text"><bdi>{x.name}</bdi></b> <span className="muted small">#{x.id}</span>
            <div className="muted small">{pn.sub(x.dist, x.sub)} · {num(x.area, 1)} {t('common.du')} · {x.crops.map(c => b(db.crops.get(c.crop)?.name)).join(', ')}</div>
          </div>
          <LevelPill level={x.level} />
        </div>
      ))}
    </Drawer>
  );
}

// ---------- farm drawer ----------
function FarmDrawer({ x, onClose, onEdit, onDelete, onOwner }: { x: Farm; onClose: () => void; onEdit: (x: Farm) => void; onDelete: (x: Farm) => void; onOwner: (id: string) => void }) {
  const { t, b, num, date } = useI18n();
  const pn = usePlaceNames();
  const o = db.farmers.get(x.farmerId);
  const points = useMemo(() => [{ id: x.id, lat: x.lat, lon: x.lon, color: '#C2452A', radius: 8, label: '#' + x.id }], [x.id, x.lat, x.lon]);
  const fill = (d: string) => (d === x.dist ? '#BFDCC8' : undefined);
  const cropSum = x.crops.reduce((a, c) => a + c.dunam, 0);
  return (
    <Drawer eyebrow={t('farms.farm') + ' #' + x.id} title={<span className="ku-text"><bdi>{x.name}</bdi></span>} onClose={onClose} head={<LevelPill level={x.level} />}>
      <div className="farm-actions mb">
        <button className="btn" onClick={() => onEdit(x)}><Pencil />{t('common.edit')}</button>
        {o && <button className="btn" onClick={() => onOwner(o.id)}><Users />{t('farms.open_owner')}</button>}
        <button className="btn danger" onClick={() => onDelete(x)}><Trash2 />{t('common.delete')}</button>
      </div>
      <div className="mb"><DistrictMap fill={fill} styleKey={x.id} focus={x.dist} points={points} size="xs" /></div>
      <div className="mb">
        <div className="eyebrow" style={{ marginBottom: 6 }}>{t('farms.c_crops')}</div>
        <div className="crop-strip" aria-hidden="true">{x.crops.map(c => <i key={c.crop} style={{ flex: c.dunam || 0.01, background: db.crops.get(c.crop)?.color ?? '#ccc' }} />)}
          {x.area > cropSum + 0.05 && <i style={{ flex: x.area - cropSum, background: 'var(--line)' }} />}</div>
        <div className="row" style={{ gap: 14, marginTop: 8 }}>{x.crops.map(c => <span key={c.crop}><CropTag id={c.crop} /> <b className="tabular">{num(c.dunam, 1)}</b> <span className="muted small">{t('common.du')}</span></span>)}
          {x.area > cropSum + 0.05 && <span className="muted small">{t('farms.unplanted', { n: num(x.area - cropSum, 1) })}</span>}</div>
      </div>
      <dl className="facts">
        <dt>{t('farms.c_owner')}</dt><dd>{o ? <><bdi>{b(o.name)}</bdi> · <Phone value={o.phone} /></> : '-'}</dd>
        <dt>{t('farms.c_area')}</dt><dd><bdi>{num(x.area, 1)} {t('common.du')}</bdi> · <bdi>{num(x.area * 2500)} m²</bdi></dd>
        <dt>{t('farms.c_place')}</dt><dd>{pn.sub(x.dist, x.sub)}, {pn.dist(x.dist)}, {pn.gov(x.gov)}</dd>
        <dt>{t('common.village')}</dt><dd><bdi>{x.village || '-'}</bdi></dd>
        <dt>{t('farms.c_point')}</dt><dd><span className="ltr mono">{x.lat.toFixed(5)}, {x.lon.toFixed(5)}</span></dd>
        <dt>{t('farms.f_irrigation')}</dt><dd>{t('farms.irr_' + x.irrigation)}</dd>
        <dt>{t('farms.f_water')}</dt><dd>{t('farms.w_' + x.water)}</dd>
        <dt>{t('farms.f_ownership')}</dt><dd>{t('farms.o_' + x.ownership)}</dd>
        <dt>{t('farms.c_level')}</dt><dd><LevelPill level={x.level} /></dd>
        <dt>{t('farms.c_created')}</dt><dd>{date(x.created)}</dd>
        <dt>{t('common.updated')}</dt><dd>{date(x.updated)}</dd>
        {x.notes && <><dt>{t('common.notes')}</dt><dd><bdi>{x.notes}</bdi></dd></>}
      </dl>
      <p className="muted small" style={{ marginTop: 14 }}>{t('farms.level_note')}</p>
    </Drawer>
  );
}

// ---------- farmer form ----------
function FarmerForm({ init, onClose, onSaved }: { init: Farmer; onClose: () => void; onSaved: (id: string) => void }) {
  const { t } = useI18n();
  const toast = useToast();
  const [d, setD] = useState<Farmer>(init);
  const [err, setErr] = useState<Problems>({});
  const opts = usePlaceOptions(d.gov, d.dist);
  const up = (p: Partial<Farmer>) => setD(s => ({ ...s, ...p }));
  const save = () => {
    const draft = { ...d, phone: normPhone(d.phone), name: { en: d.name.en.trim(), ku: d.name.ku.trim() } };
    const p = checkFarmer(draft); setErr(p);
    if (Object.keys(p).length) { toast(t('v.fix_first'), 'warn'); return; }
    const id = draft.id || nextId(db.farmers.all());
    saveFarmer({ ...draft, id });
    toast(t('common.saved'), 'good'); onSaved(id);
  };
  return (
    <Modal wide title={init.id ? t('farms.edit_farmer') : t('farms.add_farmer')} onClose={onClose}
      foot={<><button className="btn" onClick={onClose}>{t('common.cancel')}</button><button className="btn primary" onClick={save}>{t('common.save')}</button></>}>
      <div className="form-grid">
        <Field label={t('farms.f_name_ku')} error={err.name}><input type="text" dir="rtl" lang="ckb" className="ku-text" value={d.name.ku} onChange={e => up({ name: { ...d.name, ku: e.target.value } })} /></Field>
        <Field label={t('farms.f_name_en')} error={err.name}><input type="text" dir="ltr" value={d.name.en} onChange={e => up({ name: { ...d.name, en: e.target.value } })} /></Field>
        <Field label={t('farms.c_phone')} hint={t('farms.phone_hint')} error={err.phone}><input type="tel" inputMode="tel" value={d.phone} onChange={e => up({ phone: e.target.value })} placeholder="0750 123 4567" /></Field>
        <Field label={t('farms.f_gender')}><Select value={d.gender} onChange={v => up({ gender: v as Gender })} options={[['male', t('farms.g_male')], ['female', t('farms.g_female')]]} /></Field>
        <Field label={t('farms.f_birth')} hint={t('farms.optional')} error={err.birthYear}><input type="number" inputMode="numeric" value={d.birthYear ?? ''} onChange={e => up({ birthYear: e.target.value ? Number(e.target.value) : null })} /></Field>
        <Field label={t('common.status')}><Select value={d.status} onChange={v => up({ status: v as Farmer['status'] })} options={[['active', t('common.active')], ['blocked', t('common.blocked')]]} /></Field>
        <Field label={t('common.governorate')} error={err.gov}><Select value={d.gov} onChange={v => up({ gov: v, dist: '', sub: '' })} options={[['', '-'], ...opts.govs]} /></Field>
        <Field label={t('common.district')} error={err.dist}><Select value={d.dist} onChange={v => up({ dist: v, sub: '' })} options={[['', '-'], ...opts.dists]} /></Field>
        <Field label={t('common.subdistrict')}><Select value={d.sub} onChange={v => up({ sub: v })} options={[['', '-'], ...opts.subs]} /></Field>
        <Field label={t('common.village')}><input type="text" dir="auto" value={d.village} onChange={e => up({ village: e.target.value })} /></Field>
        <Field label={t('common.notes')} full><textarea dir="auto" value={d.notes} onChange={e => up({ notes: e.target.value })} /></Field>
      </div>
    </Modal>
  );
}

// ---------- farm form ----------
function FarmForm({ init, onClose, onSaved }: { init: Farm; onClose: () => void; onSaved: (id: string) => void }) {
  const { t, b, num } = useI18n();
  const toast = useToast();
  const crops = useCrops();
  const [d, setD] = useState<Farm>(() => ({ ...init, crops: init.crops.map(c => ({ ...c })) }));
  const [err, setErr] = useState<Problems>({});
  const opts = usePlaceOptions(d.gov, d.dist);
  const up = (p: Partial<Farm>) => setD(s => ({ ...s, ...p }));
  const centre = (dist: string, sub: string) => (subByKey.get(dist + '|' + sub) ?? distByName.get(dist))?.c;
  const setPlace = (p: Partial<Farm>) => setD(s => {
    const n = { ...s, ...p }, c = centre(n.dist, n.sub);
    return c ? { ...n, lat: c[0], lon: c[1] } : n;
  });
  const setCrop = (i: number, p: Partial<FarmCrop>) => setD(s => ({ ...s, crops: s.crops.map((c, k) => (k === i ? { ...c, ...p } : c)) }));
  const owner = db.farmers.get(d.farmerId);
  const sum = d.crops.reduce((a, c) => a + (c.dunam || 0), 0);
  const save = () => {
    const p = checkFarm(d); setErr(p);
    if (Object.keys(p).length) { toast(t('v.fix_first'), 'warn'); return; }
    const id = d.id || nextId(db.farms.all());
    saveFarm({ ...d, id, name: d.name.trim() });
    toast(t('common.saved'), 'good'); onSaved(id);
  };
  return (
    <Modal wide title={init.id ? t('farms.edit_farm') : t('farms.add_farm')} onClose={onClose}
      foot={<><button className="btn" onClick={onClose}>{t('common.cancel')}</button><button className="btn primary" onClick={save}>{t('common.save')}</button></>}>
      <div className="form-grid">
        <Field label={t('farms.c_owner')} error={err.farmerId} full>
          {owner && !init.farmerId ? (
            <div className="row" style={{ justifyContent: 'space-between' }}><span><b><bdi>{b(owner.name)}</bdi></b> · <Phone value={owner.phone} /></span>
              <button type="button" className="btn sm" onClick={() => up({ farmerId: '' })}><X />{t('farms.change')}</button></div>
          ) : owner ? <span><b><bdi>{b(owner.name)}</bdi></b> · <Phone value={owner.phone} /></span>
            : <OwnerPicker onPick={p => setPlace({ farmerId: p.id, gov: p.gov, dist: p.dist, sub: p.sub, village: d.village || p.village })} />}
        </Field>
        <Field label={t('farms.f_farm_name')} error={err.name}><input type="text" dir="auto" value={d.name} onChange={e => up({ name: e.target.value })} /></Field>
        <Field label={t('farms.f_area')} error={err.area}><input type="number" inputMode="decimal" min={0} step={0.1} value={d.area || ''} onChange={e => up({ area: Number(e.target.value) })} /></Field>
        <Field label={t('common.governorate')}><Select value={d.gov} onChange={v => setPlace({ gov: v, dist: '', sub: '' })} options={[['', '-'], ...opts.govs]} /></Field>
        <Field label={t('common.district')} error={err.dist}><Select value={d.dist} onChange={v => setPlace({ dist: v, sub: '', gov: distByName.get(v)?.gov ?? d.gov })} options={[['', '-'], ...opts.dists]} /></Field>
        <Field label={t('common.subdistrict')}><Select value={d.sub} onChange={v => setPlace({ sub: v })} options={[['', '-'], ...opts.subs]} /></Field>
        <Field label={t('common.village')}><input type="text" dir="auto" value={d.village} onChange={e => up({ village: e.target.value })} /></Field>
        <Field label={t('farms.f_lat')} hint={t('farms.point_hint')} error={err.point}><input type="number" step={0.00001} className="ltr-input" value={d.lat} onChange={e => up({ lat: Number(e.target.value) })} /></Field>
        <Field label={t('farms.f_lon')} error={err.point}><input type="number" step={0.00001} className="ltr-input" value={d.lon} onChange={e => up({ lon: Number(e.target.value) })} /></Field>
        <Field label={t('farms.f_irrigation')}><Select value={d.irrigation} onChange={v => up({ irrigation: v as Irrigation, water: v === 'rainfed' ? 'rain' : d.water === 'rain' ? 'well' : d.water })} options={IRRIGATIONS.map(x => [x, t('farms.irr_' + x)] as [string, string])} /></Field>
        <Field label={t('farms.f_water')}><Select value={d.water} onChange={v => up({ water: v as WaterSource })} options={WATERS.map(x => [x, t('farms.w_' + x)] as [string, string])} /></Field>
        <Field label={t('farms.f_ownership')}><Select value={d.ownership} onChange={v => up({ ownership: v as Ownership })} options={OWNERSHIPS.map(x => [x, t('farms.o_' + x)] as [string, string])} /></Field>
        <Field label={t('farms.c_level')} hint={t('farms.level_hint')}><Select value={d.level} onChange={v => up({ level: v as FieldLevel })} options={LEVELS.map(x => [x, t('level.' + x)] as [string, string])} /></Field>
        <div className="full">
          <div className="spread" style={{ marginBottom: 6 }}>
            <span className="field" style={{ display: 'inline' }}>{t('farms.c_crops')}</span>
            <span className={'small ' + (sum > d.area + 0.05 ? 'err' : 'muted')} style={sum > d.area + 0.05 ? { color: 'var(--danger)', fontWeight: 600 } : undefined}>{t('farms.crop_sum', { a: num(sum, 1), b: num(d.area || 0, 1) })}</span>
          </div>
          {d.crops.map((c, i) => (
            <div className="crop-row" key={i}>
              <Select value={c.crop} onChange={v => setCrop(i, { crop: v })} options={[['', '-'], ...crops.list.filter(x => x.active || x.id === c.crop).map(x => [x.id, b(x.name)] as [string, string])]} aria-label={t('farms.c_crops')} />
              <input type="number" inputMode="decimal" min={0} step={0.1} value={c.dunam || ''} placeholder={t('common.dunam')} onChange={e => setCrop(i, { dunam: Number(e.target.value) })} aria-label={t('common.dunam')} />
              <button type="button" className="btn sm icon" disabled={d.crops.length < 2} onClick={() => up({ crops: d.crops.filter((_, k) => k !== i) })} aria-label={t('common.delete')}><Trash2 /></button>
            </div>
          ))}
          {err.crops && <div className="small" style={{ color: 'var(--danger)', fontWeight: 600 }}>{t(err.crops)}</div>}
          <button type="button" className="btn sm" style={{ marginTop: 6 }} onClick={() => up({ crops: [...d.crops, { crop: '', dunam: Math.max(0, Math.round((d.area - sum) * 10) / 10) }] })}><Plus />{t('farms.add_crop')}</button>
        </div>
        <Field label={t('common.notes')} full><textarea dir="auto" value={d.notes} onChange={e => up({ notes: e.target.value })} /></Field>
      </div>
    </Modal>
  );
}

/** Find the owner by name or phone. Stops after 20 hits, so it stays quick with thousands of farmers. */
function OwnerPicker({ onPick }: { onPick: (p: Farmer) => void }) {
  const { t, b } = useI18n();
  const [q, setQ] = useState('');
  const dq = useDebounced(q.trim().toLowerCase(), 150);
  const hits = useMemo(() => {
    if (!dq) return [];
    const digits = dq.replace(/\D/g, ''), out: Farmer[] = [];
    for (const p of db.farmers.all()) {
      if (p.name.en.toLowerCase().includes(dq) || p.name.ku.includes(dq) || p.id === dq || (digits.length >= 3 && p.phone.includes(digits))) { out.push(p); if (out.length >= 20) break; }
    }
    return out;
  }, [dq]);
  return (
    <div>
      <input type="search" value={q} onChange={e => setQ(e.target.value)} placeholder={t('farms.owner_search')} />
      {dq && (
        <div className="owner-hits">
          {hits.length === 0 ? <div className="muted small" style={{ padding: 8 }}>{t('search.none')}</div> : hits.map(p => (
            <button type="button" key={p.id} className="list-item click owner-hit" onClick={() => onPick(p)}>
              <span style={{ flex: 1, textAlign: 'start' }}><b><bdi>{b(p.name)}</bdi></b> <span className="muted small">#{p.id}</span></span><Phone value={p.phone} />
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

