// Region data (design 11 snyUI): numbers arrive automatically from the data jobs; staff may correct one
// by hand, and it stays marked until the next job run replaces it. Tabs follow the frame.
import { useSearchParams } from 'react-router-dom';
import { useState } from 'react';
import { Info, Download } from 'lucide-react';
import { useI18n } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { Note, PageHead, Tabs, useToast } from '../../components/ui';
import { api, qs, ApiError } from '../../api/client';
import { useOverview } from '../../api/public';
import type { RegionOverview } from '../../api/types';
import { downloadCsv } from './farms/common';
import type { DamReading, DamRef, DashFire } from './region/types';
import { StateBox, useErrorText } from '../../components/domain';
import { Districts } from './region/Districts';
import { Dams } from './region/Dams';
import { Fires } from './region/Fires';
import { OutlooksTab, WaterTab } from './region/Plans';
import './region.css';

type Tab = 'districts' | 'dams' | 'fires' | 'outlooks' | 'water';
const NEEDS: Record<Tab, 'zones' | 'dams' | 'fires' | 'outlooks' | 'water'> = { districts: 'zones', dams: 'dams', fires: 'fires', outlooks: 'outlooks', water: 'water' };

export default function Region() {
  const { t } = useI18n();
  const { can } = useAuth();
  const [params, setParams] = useSearchParams();
  const tab = (['districts', 'dams', 'fires', 'outlooks', 'water'].includes(params.get('tab') ?? '') ? params.get('tab') : 'districts') as Tab;
  const setTab = (k: Tab) => setParams(k === 'districts' ? {} : { tab: k }, { replace: true });
  const ov = useOverview();
  const toast = useToast();
  const errText = useErrorText();
  const [exporting, setExporting] = useState(false);
  const canExport = can(NEEDS[tab]) && (tab === 'districts' || tab === 'dams' || tab === 'fires');

  // the whole tab as a spreadsheet: districts for the month on screen, dams and fires read page by page
  const exportCsv = async () => {
    if (exporting) return;
    setExporting(true);
    try {
      if (tab === 'districts') {
        const o = ov.data ?? await api.get<RegionOverview>('/region/overview');
        downloadCsv('jutyar_districts_' + o.month, ['month', 'district', 'name_en', 'name_ku', 'governorate', 'dryness', 'band', 'rank', 'change_vs_last_year', 'water_need', 'nitrogen_hold'],
          o.zones.map(z => [o.month, z.slug, z.name_en, z.name_ku, z.governorate, z.dryness, z.band, z.rank, z.change_vs_last_year, z.water_need, z.nitrogen_hold == null ? '' : String(z.nitrogen_hold)]));
      } else if (tab === 'dams') {
        const rows: (string | number | null | undefined)[][] = [];
        const dams = (await api.get<{ dams: DamRef[] }>('/dashboard/dams')).dams;
        for (const d of dams) for (let p = 1; p <= 50; p++) {
          const r = await api.get<{ readings: DamReading[]; count: number }>(`/dashboard/dams/${d.slug}/readings` + qs({ page: p, rows_per_page: 100 }));
          for (const x of r.readings) rows.push([d.slug, x.day, x.pct_full, x.lake_area_km2, x.volume_bn_m3, x.source, x.updated_at]);
          if (p * 100 >= r.count) break;
        }
        downloadCsv('jutyar_dams', ['dam', 'day', 'lake_area_pct_of_full', 'lake_area_km2', 'volume_bn_m3', 'source', 'updated_at'], rows);
      } else if (tab === 'fires') {
        const rows: (string | number | null | undefined)[][] = [];
        for (let p = 1; p <= 50; p++) {
          const r = await api.get<{ fires: DashFire[]; count: number }>('/dashboard/fires' + qs({ page: p, rows_per_page: 100 }));
          for (const f of r.fires) rows.push([f.id, f.detected_at, f.lat, f.lon, f.zone_slug, f.place_en, f.status, f.area_ha, f.farms_within_5km, f.source, f.updated_at]);
          if (p * 100 >= r.count) break;
        }
        downloadCsv('jutyar_fire_detections', ['id', 'detected_at', 'lat', 'lon', 'district', 'place', 'status', 'area_ha', 'farms_within_5km', 'source', 'updated_at'], rows);
      }
    } catch (e) { toast(errText(e as ApiError), 'danger'); } finally { setExporting(false); }
  };

  return (
    <div>
      <PageHead eyebrow={t('nav.g_fields')} title={t('nav.region')} sub={t('region.sub')}
        actions={canExport && <button className="btn" onClick={exportCsv} disabled={exporting}><Download />{exporting ? t('common.loading') : t('common.export_csv')}</button>} />
      <div className="mb"><Note tone="info" icon={<Info />}>{t('region.how')}</Note></div>
      <Tabs value={tab} onChange={setTab} items={[['districts', t('region.t_districts')], ['dams', t('region.t_dams')], ['fires', t('region.t_fires')], ['outlooks', t('region.t_outlooks')], ['water', t('region.t_water')]]} />
      {!can(NEEDS[tab]) ? <StateBox kind="locked" />
        : tab === 'districts' ? <Districts /> : tab === 'dams' ? <Dams /> : tab === 'fires' ? <Fires /> : tab === 'outlooks' ? <OutlooksTab /> : <WaterTab />}
    </div>
  );
}
