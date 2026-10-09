// Region data (design 11 snyUI): numbers arrive automatically from the data jobs; staff may correct one
// by hand, and it stays marked until the next job run replaces it. Tabs follow the frame.
import { useSearchParams } from 'react-router-dom';
import { Info } from 'lucide-react';
import { useI18n } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { Note, PageHead, Tabs } from '../../components/ui';
import { StateBox } from '../../components/domain';
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
  return (
    <div>
      <PageHead eyebrow={t('nav.g_fields')} title={t('nav.region')} sub={t('region.sub')} />
      <div className="mb"><Note tone="info" icon={<Info />}>{t('region.how')}</Note></div>
      <Tabs value={tab} onChange={setTab} items={[['districts', t('region.t_districts')], ['dams', t('region.t_dams')], ['fires', t('region.t_fires')], ['outlooks', t('region.t_outlooks')], ['water', t('region.t_water')]]} />
      {!can(NEEDS[tab]) ? <StateBox kind="locked" />
        : tab === 'districts' ? <Districts /> : tab === 'dams' ? <Dams /> : tab === 'fires' ? <Fires /> : tab === 'outlooks' ? <OutlooksTab /> : <WaterTab />}
    </div>
  );
}
