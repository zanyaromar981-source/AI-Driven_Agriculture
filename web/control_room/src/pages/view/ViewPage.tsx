// The public View page: the region map by district, water, fires, two years side by side and Alwa
// prices. No sign-in. What is shown is switched in Settings > Public site (settings.publicView).
import { useMemo, useState } from 'react';
import { Link } from 'react-router-dom';
import { LogIn, LayoutDashboard, Droplets, Flame, Map as MapIcon, BarChart3, Store, BellRing, Tractor } from 'lucide-react';
import { useI18n } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { db } from '../../data/db';
import { useDoc, useRows, useVersion } from '../../data/store';
import { totals } from '../../data/api';
import { GrainSun } from '../../motion/GrainSun';
import { Ticker } from '../../motion/Ticker';
import { LangSwitch } from '../../layouts/LangSwitch';
import { Tabs } from '../../components/ui';
import { MapTab, WaterTab, FiresTab, CompareTab, MarketTab } from './tabs';
import './view.css';

type TabKey = 'map' | 'water' | 'fires' | 'compare' | 'market';

export default function ViewPage() {
  const { t, num, b, date } = useI18n();
  const { me } = useAuth();
  const settings = useDoc(db.settings);
  const pv = settings.publicView;
  const dams = useRows(db.dams);
  const fires = useRows(db.fires);
  const alerts = useRows(db.alerts);
  const fv = useVersion(db.farms), pfv = useVersion(db.farmers);
  const all = useMemo(() => totals().all, [fv, pfv]); // eslint-disable-line react-hooks/exhaustive-deps

  const tabs = useMemo(() => {
    const list: [TabKey, string][] = [];
    if (pv.map) list.push(['map', t('view.tab_map')]);
    if (pv.water) list.push(['water', t('view.tab_water')]);
    if (pv.fires) list.push(['fires', t('view.tab_fires')]);
    if (pv.compare) list.push(['compare', t('view.tab_compare')]);
    if (pv.market) list.push(['market', t('view.tab_market')]);
    return list;
  }, [pv, t]);
  const [tab, setTab] = useState<TabKey>('map');
  const cur = tabs.some(x => x[0] === tab) ? tab : tabs[0]?.[0];

  const fires24 = useMemo(() => { const lim = Date.now() - 864e5; return fires.reduce((n, f) => n + (+new Date(f.at) >= lim ? 1 : 0), 0); }, [fires]);
  const lastAlert = useMemo(() => {
    let best: (typeof alerts)[number] | undefined;
    for (const a of alerts) if (a.status === 'sent' && a.sent && (!best || a.sent > best.sent!)) best = a;
    return best;
  }, [alerts]);

  return (
    <div className="pub">
      <header className="pub-top">
        <Link to="/" className="logo"><span className="mark"><GrainSun size={26} /></span><span>Jutyar</span><span className="word-ku">جوتیار</span></Link>
        <div className="top-end">
          <LangSwitch />
          {me
            ? <Link to="/admin" className="btn primary sm"><LayoutDashboard />{t('view.control_room')}</Link>
            : <Link to="/login" className="btn primary sm"><LogIn className="flip-rtl" />{t('view.sign_in')}</Link>}
        </div>
      </header>
      <Ticker where="public" />

      <main className="pub-main">
        <div className="pub-chips">
          {pv.water && dams.map(d => (
            <span className="pub-chip" key={d.id}><i style={{ background: 'var(--water)' }} /><span className="lab">{b(d.name)}</span><b><bdi>{t('view.full', { n: num(d.pct) })}</bdi></b></span>
          ))}
          {pv.farmTotals && <span className="pub-chip"><Tractor size={14} /><b><bdi>{t('view.farms_chip', { farms: num(all.farms), dunam: num(all.area) })}</bdi></b></span>}
          {pv.fires && <span className="pub-chip"><Flame size={14} style={{ color: fires24 ? 'var(--danger)' : undefined }} /><b><bdi>{t('view.fires_chip', { n: num(fires24) })}</bdi></b></span>}
        </div>

        {lastAlert && (
          <div className="pub-alert">
            <span className="ico warn"><BellRing /></span>
            <div style={{ minWidth: 0 }}>
              <div className="eyebrow">{t('view.latest_alert')} · {date(lastAlert.sent, 'short')}</div>
              <b><bdi>{b(lastAlert.title)}</bdi></b>
              <div className="small ink2"><bdi>{b(lastAlert.body)}</bdi></div>
            </div>
          </div>
        )}

        {cur ? (
          <>
            <div className="pub-tabs">
              <Tabs value={cur} onChange={setTab} items={tabs} />
              <span className="pub-tab-ico" aria-hidden="true">{cur === 'map' ? <MapIcon /> : cur === 'water' ? <Droplets /> : cur === 'fires' ? <Flame /> : cur === 'compare' ? <BarChart3 /> : <Store />}</span>
            </div>
            {cur === 'map' && <MapTab farmTotals={pv.farmTotals} />}
            {cur === 'water' && <WaterTab />}
            {cur === 'fires' && <FiresTab />}
            {cur === 'compare' && <CompareTab />}
            {cur === 'market' && <MarketTab />}
          </>
        ) : <div className="card empty">{t('view.nothing_public')}</div>}
      </main>

      <footer className="pub-foot">
        <b><bdi>{b(settings.orgName)}</bdi></b>
        <span>{t('view.footer_sources')}</span>
        {settings.sampleBanner && <span className="pill warn">{t('view.footer_sample')}</span>}
      </footer>
    </div>
  );
}
