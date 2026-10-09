// The public View page (design v2: tIFJP desktop, rjX3Q phone, Dha2F English). No login.
// Order on screen: top bar, news bar, today's three numbers, big tabs, then the tab. On the map tab the
// district panel sits on the physical right in both languages (approved design).
import { useEffect, useState } from 'react';
import { Link, useSearchParams } from 'react-router-dom';
import { CloudRain, Flame, Waves, Map as MapIcon, ChartColumn, Store, LogIn, LayoutDashboard } from 'lucide-react';
import { useI18n } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { useApi } from '../../api/cache';
import { useDams, useFires, useOverview } from '../../api/public';
import type { FarmStats } from '../../api/types';
import { GrainSun } from '../../motion/GrainSun';
import { Ticker } from '../../motion/Ticker';
import { LangSwitch } from '../../layouts/LangSwitch';
import { prefs } from '../../data/store';
import { MapTab } from './MapTab';
import { WaterTab, FiresTab, CompareTab, MarketTab } from './OtherTabs';
import { TABS, type ViewTab } from './viewUtil';
import './view.css';

const TAB_ICON = { map: MapIcon, water: Waves, fires: Flame, compare: ChartColumn, market: Store };
const BAND_SHORT = (avg: number) => (avg < 25 ? 'much_greener' : avg < 45 ? 'greener' : avg < 60 ? 'normal' : avg < 80 ? 'dry' : 'very_dry');

export default function ViewPage() {
  const { t, num, lang } = useI18n();
  const { me } = useAuth();
  const [params, setParams] = useSearchParams();
  const [tab, setTabState] = useState<ViewTab>(() => (params.get('tab') as ViewTab) || prefs.get<ViewTab>('view.tab', 'map'));
  const slug = params.get('d');
  const setSlug = (s: string | null) => { const p = new URLSearchParams(params); if (s) p.set('d', s); else p.delete('d'); setParams(p, { replace: true }); };
  const setTab = (k: ViewTab) => { setTabState(k); prefs.set('view.tab', k); };
  const ov = useOverview(), fires = useFires(), dams = useDams();
  const stats = useApi<FarmStats>('/stats/farms', []);

  useEffect(() => { document.title = lang === 'ku' ? 'جوتیار · کێڵگە و ئاوی هەرێمی کوردستان' : 'Jutyar · Farms and water of the Kurdistan Region'; }, [lang]);

  const avg = ov.data?.summary.average_dryness;
  const damWith = dams.data?.dams.filter(d => d.latest) ?? [];
  return (
    <div className="pub view">
      <header className="pub-top view-top">
        <Link to="/" className="logo view-logo" aria-label="Jutyar">
          <span className="mark"><GrainSun size={30} /></span>
          <span className="view-logo-words">
            <b>{lang === 'ku' ? 'جوتیار' : 'Jutyar'}</b>
            <small>{t('view.tagline')}</small>
          </span>
        </Link>
        <div className="view-top-actions">
          <LangSwitch />
          {me
            ? <Link to="/admin" className="btn primary view-signin"><LayoutDashboard /><span>{t('view.to_admin')}</span></Link>
            : <Link to="/login" className="btn primary view-signin" aria-label={t('common.staff_sign_in')}><LogIn className="flip-rtl" /><span>{t('common.staff_sign_in')}</span></Link>}
        </div>
      </header>
      <Ticker />
      <main className="view-main">
        <div className="view-today" role="list" aria-label={t('view.today')}>
          <Stat icon={<CloudRain />} tone="good" title={t('view.rain')} loading={!ov.data}
            value={avg == null ? t('common.no_data') : t('view.band_short_' + BAND_SHORT(avg))} line={ov.data ? t('view.rain_line', { n: num(ov.data.summary.zones_with_data) }) : ''} small />
          <Stat icon={<Flame />} tone="danger" title={t('view.fires')} loading={!fires.data}
            value={fires.data ? num(fires.data.fires.length) : ''} unit={t('view.detections')} line={fires.data ? t('view.fires_line', { z: num(fires.data.summary.zones.length) }) : ''} />
          <Stat icon={<Waves />} tone="water" title={t('view.dams')} loading={!dams.data} small={!damWith.length}
            value={damWith.length ? '' : t('common.no_data')} line={damWith.length ? '' : t('view.dams_line_none')}
            extra={damWith.length ? <span className="view-dams">{damWith.map(d => <span key={d.slug}><b className="ltr">{num(d.latest!.pct_full, 0)}%</b><small>{lang === 'ku' ? d.name_ku : d.name_en}</small></span>)}</span> : undefined} />
        </div>
        <nav className="view-tabs" role="tablist" aria-label={t('view.tabs')}>
          {TABS.map(x => { const I = TAB_ICON[x.key]; return (
            <button key={x.key} role="tab" aria-selected={tab === x.key} className={'view-tab' + (tab === x.key ? ' on' : '')} onClick={() => setTab(x.key)}><I /><span>{t('view.tab_' + x.key)}</span></button>
          ); })}
        </nav>
        {tab === 'map' && <MapTab ov={ov.data} stats={stats.data} slug={slug} setSlug={setSlug} />}
        {tab === 'water' && <WaterTab dams={dams.data?.dams} />}
        {tab === 'fires' && <FiresTab fires={fires.data} />}
        {tab === 'compare' && <CompareTab />}
        {tab === 'market' && <MarketTab />}
      </main>
      <footer className="pub-foot">
        <span>{t('view.org')}</span>
        <span className="muted">{t('view.sources')}</span>
      </footer>
    </div>
  );
}

function Stat({ icon, tone, title, value, unit, line, loading, small, extra }: { icon: React.ReactNode; tone: string; title: string; value: string; unit?: string; line: string; loading: boolean; small?: boolean; extra?: React.ReactNode }) {
  return (
    <div className="card view-stat" role="listitem">
      <span className={'ico ' + tone}>{icon}</span>
      <div className="view-stat-txt">
        <span className="view-stat-title">{title}</span>
        {loading ? <i className="sk h28 w60" /> : extra ?? <span className={'view-stat-v ' + tone + (small ? ' small' : '')}><b>{value}</b>{unit && <small>{unit}</small>}</span>}
        {loading ? <i className="sk w80" /> : line && <span className="muted view-stat-line">{line}</span>}
      </div>
    </div>
  );
}
