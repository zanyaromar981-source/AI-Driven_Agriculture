// Farmers and farms (design HWGis, kMILR, nsu3f). Both lists are paged, filtered and sorted by the
// server; totals come from /dashboard/stats/farms. Drawers open from ?farmer= and ?farm= so a search
// hit or a link lands on them.
import { useEffect, useMemo, useState } from 'react';
import { useSearchParams } from 'react-router-dom';
import { Users, Map as MapIcon, Ruler, Ban, Download, FileText, UserPlus, Search, Scale, Pencil } from 'lucide-react';
import { useI18n } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { api, qs } from '../../api/client';
import { useApi } from '../../api/cache';
import type { FarmStats } from '../../api/types';
import { PageHead, Kpi, Tabs, Select, useDebounced, useToast } from '../../components/ui';
import { DataTable, type Col } from '../../components/DataTable';
import { CropTag, Phone, StateBox, usePlace, useErrorText } from '../../components/domain';
import { GOVERNORATES, DISTRICTS, GOV_BY_NAME } from '../../data/places';
import { CROPS } from '../../data/crops';
import { FarmerForm, FarmerDrawer, FarmDrawer } from './farms/FarmerParts';
import { type Farmer, type FarmSummary, downloadCsv } from './farms/common';
import type { ApiError } from '../../api/client';
import './farms.css';

type Tab = 'farmers' | 'farms';
const PER = 25;

export default function Farms() {
  const { t, num, date, lang } = useI18n();
  const { can } = useAuth();
  const toast = useToast();
  const errText = useErrorText();
  const place = usePlace();
  const [params, setParams] = useSearchParams();
  const [tab, setTab] = useState<Tab>(() => (params.get('farm') ? 'farms' : 'farmers'));
  const [q, setQ] = useState('');
  const dq = useDebounced(q.trim(), 250);
  const [gov, setGov] = useState('');
  const [zone, setZone] = useState('');
  const [crop, setCrop] = useState('');
  const [blocked, setBlocked] = useState('');
  const [page, setPage] = useState(1);
  const [sort, setSort] = useState<[string, 1 | -1] | null>(null);
  const [form, setForm] = useState<Farmer | null | undefined>(undefined); // undefined = closed, null = new
  const [exporting, setExporting] = useState(false);
  const farmerId = params.get('farmer'), farmId = params.get('farm');

  // a new filter starts again from page 1
  useEffect(() => { setPage(1); }, [dq, gov, zone, crop, blocked, tab, sort]);

  const nm = (p: { en: string; ku: string }) => (lang === 'ku' ? p.ku : p.en);
  // the farmers list filters by the governorate slug; farms and stats take the English name
  const govSlug = gov ? GOV_BY_NAME.get(gov)?.slug : undefined;
  const govOpts: [string, string][] = [['', t('common.all_govs')], ...GOVERNORATES.map(g => [g.en, nm(g)] as [string, string])];
  const zoneOpts: [string, string][] = [['', t('common.all_dists')], ...DISTRICTS.filter(d => !gov || d.gov === gov).map(d => [d.slug, nm(d)] as [string, string])];

  const stats = useApi<FarmStats>(can('farms') ? '/dashboard/stats/farms' + qs({ governorate: gov, zone }) : null, ['farms'], { auth: true });
  const blockedCount = useApi<{ count: number }>(can('farmers') ? '/dashboard/farmers' + qs({ blocked: true, rows_per_page: 1 }) : null, ['farmers'], { auth: true });

  const sortField = (k?: string) => (k === 'name' ? 'name' : k === 'area' ? 'area_dunam' : 'created_at');
  const farmersQ = useApi<{ farmers: Farmer[]; count: number }>(tab === 'farmers' && can('farmers') ? '/dashboard/farmers' + qs({
    q: dq, governorate: govSlug, zone, blocked: blocked === '' ? undefined : blocked === 'yes', page, rows_per_page: PER,
    sort: sort ? sortField(sort[0]) : undefined, order: sort ? (sort[1] === 1 ? 'asc' : 'desc') : undefined,
  }) : null, ['farmers'], { auth: true });
  const farmsQ = useApi<{ farms: FarmSummary[]; count: number }>(tab === 'farms' && can('farms') ? '/dashboard/farms' + qs({
    q: dq, governorate: gov, zone, crop, page, rows_per_page: PER,
    sort: sort ? sortField(sort[0]) : undefined, order: sort ? (sort[1] === 1 ? 'asc' : 'desc') : undefined,
  }) : null, ['farms'], { auth: true });

  const openFarmer = (id: string) => setParams(p => { p.delete('farm'); p.set('farmer', id); return p; });
  const openFarm = (id: string) => setParams(p => { p.delete('farmer'); p.set('farm', id); return p; });
  const closeDrawer = () => setParams(p => { p.delete('farmer'); p.delete('farm'); return p; });

  const T = stats.data?.totals;
  const farmerCols: Col<Farmer>[] = useMemo(() => [
    { key: 'name', label: t('farms.c_farmer'), serverSort: true, cell: f => <div><b><bdi>{f.name || t('farms.no_name')}</bdi></b><div className="muted small tabular">#{f.id}</div></div> },
    { key: 'phone', label: t('farms.c_phone'), cell: f => <Phone value={f.phone} /> },
    { key: 'place', label: t('farms.c_home'), cell: f => <div>{place.sub(f.sub_zone_slug) || place.dist(f.zone_slug) || '-'}<div className="muted small">{[place.dist(f.zone_slug), place.gov(f.governorate)].filter(Boolean).join('، ')}</div></div> },
    { key: 'farms', label: t('common.farms'), num: true, cell: f => num(f.farms_count) },
    { key: 'village', label: t('common.village'), optional: true, cell: f => f.village ? <bdi>{f.village}</bdi> : '-' },
    { key: 'lang', label: t('farms.f_lang'), optional: true, cell: f => t('farms.lang_' + f.lang) },
    { key: 'joined', label: t('farms.joined'), serverSort: true, cell: f => <span className="nowrap">{date(f.created_at, 'short')}</span> },
    { key: 'status', label: t('common.status'), cell: f => f.blocked ? <span className="pill danger">{t('common.blocked')}</span> : <span className="pill good">{t('common.active')}</span> },
    { key: 'actions', label: '', cell: f => <div className="row-acts" onClick={e => e.stopPropagation()}>
      {can('farmers', 'create') && <a className="btn ghost sm icon" href={'#/print/letter/' + f.id} title={t('farms.letter')} aria-label={t('farms.letter')}><FileText /></a>}
      {can('farmers', 'update') && <button className="btn ghost sm icon" onClick={() => setForm(f)} title={t('farms.edit_farmer')} aria-label={t('farms.edit_farmer')}><Pencil /></button>}
    </div> },
  ], [t, num, date, place, can]);
  const farmCols: Col<FarmSummary>[] = useMemo(() => [
    { key: 'name', label: t('farms.c_farm'), serverSort: true, cell: f => <div><b><bdi>{f.name}</bdi></b><div className="muted small tabular">#{f.id}</div></div> },
    { key: 'owner', label: t('farms.owner'), cell: f => <Phone value={f.owner_phone} /> },
    { key: 'place', label: t('farms.place'), cell: f => <div>{place.sub(f.sub_zone_slug) || t('farms.no_place')}<div className="muted small">{[place.dist(f.zone_slug), place.gov(f.governorate)].filter(Boolean).join('، ')}</div></div> },
    { key: 'area', label: t('common.dunam'), num: true, serverSort: true, cell: f => num(f.area_dunam, 1) },
    { key: 'crops', label: t('farms.crops'), cell: f => f.crops.length ? <div className="row" style={{ gap: 6 }}>{f.crops.slice(0, 3).map(c => <CropTag key={c.crop} code={c.crop} />)}</div> : <span className="muted small">{t('farms.no_crops')}</span> },
    { key: 'joined', label: t('farms.registered'), serverSort: true, cell: f => <span className="nowrap">{date(f.created_at, 'short')}</span> },
  ], [t, num, date, place]);

  const exportCsv = async () => {
    if (exporting) return;
    setExporting(true);
    try {
      const rows: (string | number | null | undefined)[][] = [];
      if (tab === 'farmers') {
        for (let p = 1; p <= 50; p++) {
          const r = await api.get<{ farmers: Farmer[]; count: number }>('/dashboard/farmers' + qs({ q: dq, governorate: govSlug, zone, blocked: blocked === '' ? undefined : blocked === 'yes', page: p, rows_per_page: 100 }));
          for (const f of r.farmers) rows.push([f.id, f.name, f.phone, f.gender, f.birth_year, f.village, f.governorate, f.zone_slug, f.sub_zone_slug, f.farms_count, f.blocked ? 'blocked' : 'active', f.created_at]);
          if (p * 100 >= r.count) break;
        }
        downloadCsv('jutyar_farmers', ['id', 'name', 'phone', 'gender', 'birth_year', 'village', 'governorate', 'district', 'sub_district', 'farms', 'status', 'joined'], rows);
      } else {
        for (let p = 1; p <= 50; p++) {
          const r = await api.get<{ farms: FarmSummary[]; count: number }>('/dashboard/farms' + qs({ q: dq, governorate: gov, zone, crop, page: p, rows_per_page: 100 }));
          for (const f of r.farms) rows.push([f.id, f.name, f.owner_phone, f.governorate, f.zone_slug, f.sub_zone_slug, f.area_dunam.toFixed(2), f.crops.map(c => `${c.crop} ${c.dunam.toFixed(2)}`).join('; '), f.centroid.lat, f.centroid.lon, f.created_at]);
          if (p * 100 >= r.count) break;
        }
        downloadCsv('jutyar_farms', ['id', 'name', 'owner_phone', 'governorate', 'district', 'sub_district', 'dunam', 'crops', 'lat', 'lon', 'registered'], rows);
      }
    } catch (e) { toast(errText(e as ApiError), 'danger'); } finally { setExporting(false); }
  };

  if (!can('farmers') && !can('farms')) return <><PageHead eyebrow={t('nav.g_people')} title={t('nav.farms')} /><StateBox kind="locked" /></>;
  const list = tab === 'farmers' ? farmersQ : farmsQ;
  const total = tab === 'farmers' ? farmersQ.data?.count ?? 0 : farmsQ.data?.count ?? 0;
  const reportHref = '#/print/government' + qs({ governorate: gov, zone });

  return (
    <div>
      <PageHead eyebrow={t('nav.g_people')} title={t('nav.farms')} sub={t('farms.sub')} actions={<>
        <button className="btn" onClick={exportCsv} disabled={exporting}><Download />{exporting ? t('common.loading') : t('common.export_csv')}</button>
        {can('farms') && <a className="btn" href={reportHref} target="_blank" rel="noopener"><FileText />{t('farms.gov_report')}</a>}
        {can('farmers', 'create') && <button className="btn primary" onClick={() => setForm(null)}><UserPlus />{t('farms.add_farmer')}</button>}
      </>} />
      <div className="grid farms-kpis mb">
        <Kpi label={t('common.farmers')} icon={<Users />} value={T ? num(T.farmers) : '-'} note={blockedCount.data ? t('farms.n_blocked', { n: num(blockedCount.data.count) }) : ' '} />
        <Kpi label={t('common.farms')} icon={<MapIcon />} value={T ? num(T.farms) : '-'} note={T && T.farmers ? t('farms.per_farmer', { n: num(T.farms / T.farmers, 1) }) : ' '} />
        <Kpi label={t('farms.k_dunam')} icon={<Ruler />} value={T ? num(T.dunam, 1) : '-'} note={T && T.farms ? t('farms.avg_farm', { n: num(T.dunam / T.farms, 1) }) : ' '} />
        <Kpi label={t('farms.k_crops')} icon={<Scale />} value={stats.data ? num(stats.data.by_crop.filter(c => c.crop !== 'empty').length) : '-'} note={t('farms.k_crops_note')} />
        <Kpi label={t('farms.k_blocked')} icon={<Ban />} tone={blockedCount.data?.count ? 'danger' : ''} value={blockedCount.data ? num(blockedCount.data.count) : '-'} note={t('farms.k_blocked_note')} />
      </div>
      <Tabs<Tab> value={tab} onChange={k => { setTab(k); setSort(null); }} items={[
        ...(can('farmers') ? [['farmers', t('farms.tab_farmers') + (T ? ` (${num(T.farmers)})` : '')] as [Tab, string]] : []),
        ...(can('farms') ? [['farms', t('farms.tab_farms') + (T ? ` (${num(T.farms)})` : '')] as [Tab, string]] : []),
      ]} />
      <div className="card">
        <div className="filters">
          <label className="grow search-field"><Search /><input type="search" value={q} onChange={e => setQ(e.target.value)} placeholder={tab === 'farmers' ? t('farms.search_farmers') : t('farms.search_farms')} /></label>
          <Select value={gov} onChange={v => { setGov(v); setZone(''); }} options={govOpts} aria-label={t('common.governorate')} />
          <Select value={zone} onChange={setZone} options={zoneOpts} aria-label={t('common.district')} />
          {tab === 'farms' && <Select value={crop} onChange={setCrop} options={[['', t('common.all_crops')], ...CROPS.filter(c => c.code !== 'empty').map(c => [c.code, lang === 'ku' ? c.ku : c.en] as [string, string])]} aria-label={t('farms.crops')} />}
          {tab === 'farmers' && <Select value={blocked} onChange={setBlocked} options={[['', t('farms.any_status')], ['no', t('common.active')], ['yes', t('common.blocked')]]} aria-label={t('common.status')} />}
          {(q || gov || zone || crop || blocked) && <button className="btn ghost" onClick={() => { setQ(''); setGov(''); setZone(''); setCrop(''); setBlocked(''); }}>{t('common.clear')}</button>}
        </div>
        {list.error && !list.data ? <StateBox kind="error" text={errText(list.error)} action={<button className="btn" onClick={list.reload}>{t('common.retry')}</button>} /> : tab === 'farmers' ? (
          <DataTable<Farmer> id="farmers" rows={farmersQ.data?.farmers ?? []} cols={farmerCols} per={PER} loading={farmersQ.loading} onRow={f => openFarmer(f.id)} selected={farmerId}
            head={<span className="muted small">{t('farms.n_rows', { n: num(total) })}</span>}
            empty={dq || gov || zone || blocked ? t('table.empty') : t('farms.empty_farmers')}
            server={{ total, page, onPage: setPage, sort, onSort: setSort }} />
        ) : (
          <DataTable<FarmSummary> id="farms" rows={farmsQ.data?.farms ?? []} cols={farmCols} per={PER} loading={farmsQ.loading} onRow={f => openFarm(f.id)} selected={farmId}
            head={<span className="muted small">{t('farms.n_rows', { n: num(total) })}</span>}
            empty={dq || gov || zone || crop ? t('table.empty') : t('farms.empty_farms')}
            server={{ total, page, onPage: setPage, sort, onSort: setSort }} />
        )}
      </div>
      {farmerId && <FarmerDrawer id={farmerId} onClose={closeDrawer} onOpenFarm={openFarm} onEdit={f => setForm(f)} />}
      {farmId && <FarmDrawer id={farmId} onClose={closeDrawer} onOpenFarmer={openFarmer} />}
      {form !== undefined && <FarmerForm farmer={form} onClose={() => setForm(undefined)} onSaved={f => { setForm(undefined); openFarmer(f.id); }} />}
    </div>
  );
}
