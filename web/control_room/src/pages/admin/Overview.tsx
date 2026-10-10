// Overview: only numbers the data really holds, and a list of things that need an admin today.
// Cost: totals come from the cached one-pass api.totals(); the other counts are single passes, memoized
// per collection version.
import { useMemo } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { Users, Map as MapIcon, Inbox, Store, BellRing, AlertTriangle, Activity, Ban, ChevronRight, MessageSquare, UserPlus, Send } from 'lucide-react';
import { useI18n } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { db } from '../../data/db';
import { useRows, useVersion } from '../../data/store';
import { totals } from '../../data/api';
import { Card, Kpi, Pill } from '../../components/ui';
import { HBars } from '../../components/charts';
import { SeasonOutlook } from '../../components/SeasonOutlook';
import { CropTag, usePlaceNames } from '../../components/domain';
import { DistrictMap, GREEN_RAMP, ramp } from '../../components/DistrictMap';
import './overview.css';

// Esri World Imagery tiles around Dukan lake, zoom 12 (6 x 3 tiles)
const Z = 12, LAT = 35.93, LON = 44.96;
const tileXY = (lat: number, lon: number, z: number) => {
  const n = 2 ** z, r = lat * Math.PI / 180;
  return [Math.floor((lon + 180) / 360 * n), Math.floor((1 - Math.log(Math.tan(r) + 1 / Math.cos(r)) / Math.PI) / 2 * n)];
};
const [CX, CY] = tileXY(LAT, LON, Z);
const TILES: string[] = [];
for (let y = CY - 1; y <= CY + 1; y++) for (let x = CX - 3; x <= CX + 2; x++) TILES.push(`https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/${Z}/${y}/${x}`);

function Sat({ cls }: { cls: string }) {
  return <div className={'sat ' + cls}><div className="sat-grid">{TILES.map(u => <img key={u} src={u} alt="" loading="lazy" draggable={false} />)}</div></div>;
}

export default function Overview() {
  const { t, num, b, date, ago, lang } = useI18n();
  const { me } = useAuth();
  const nav = useNavigate();
  const pn = usePlaceNames();
  const vFarms = useVersion(db.farms), vFarmers = useVersion(db.farmers);
  const messages = useRows(db.messages);
  const listings = useRows(db.listings);
  const alerts = useRows(db.alerts);
  const jobs = useRows(db.jobs);
  const farmers = useRows(db.farmers);
  const farms = useRows(db.farms);
  useRows(db.crops);

  const T = useMemo(() => totals(), [vFarms, vFarmers]);
  const counts = useMemo(() => {
    let alarm = 0; for (const f of farms) if (f.level === 'alarm') alarm++;
    let blocked = 0; for (const f of farmers) if (f.status === 'blocked') blocked++;
    return { alarm, blocked };
  }, [farms, farmers]);
  const newMsgs = useMemo(() => messages.filter(m => m.state === 'new').length, [messages]);
  const openListings = useMemo(() => { let n = 0, kg = 0; for (const l of listings) if (l.state === 'open') { n++; if ((l.unit ?? 'kg') === 'kg') kg += l.kg; } return { n, kg }; }, [listings]);
  const drafts = useMemo(() => alerts.filter(a => a.status === 'draft').length, [alerts]);
  const badJobs = useMemo(() => jobs.filter(j => j.state === 'late' || j.state === 'failed'), [jobs]);
  const lastSent = useMemo(() => alerts.filter(a => a.status === 'sent').sort((a, z) => (z.sent ?? '').localeCompare(a.sent ?? ''))[0], [alerts]);
  const latestMsgs = useMemo(() => [...messages].sort((a, z) => z.at.localeCompare(a.at)).slice(0, 5), [messages]);
  const newestFarmers = useMemo(() => [...farmers].sort((a, z) => z.joined.localeCompare(a.joined)).slice(0, 5), [farmers]);
  const cropBars = useMemo(() => [...T.all.crops].sort((a, z) => z[1] - a[1]).slice(0, 8)
    .map(([c, v]) => ({ key: c, label: <CropTag id={c} />, value: v, color: db.crops.get(c)?.color })), [T]);

  const needs = [
    newMsgs > 0 && { icon: <Inbox />, tone: 'danger', text: t('overview.need_msgs', { n: num(newMsgs) }), sub: t('overview.need_msgs_sub'), to: '/admin/inbox' },
    drafts > 0 && { icon: <BellRing />, tone: 'warn', text: t('overview.need_drafts', { n: num(drafts) }), sub: t('overview.need_drafts_sub'), to: '/admin/alerts' },
    ...badJobs.map(j => ({ icon: <Activity />, tone: 'danger', text: t('overview.need_job', { name: b(j.name) }), sub: j.result, to: '/admin/jobs' })),
    counts.alarm > 0 && { icon: <AlertTriangle />, tone: 'warn', text: t('overview.need_alarm', { n: num(counts.alarm) }), sub: t('overview.need_alarm_sub'), to: '/admin/farms' },
    counts.blocked > 0 && { icon: <Ban />, tone: '', text: t('overview.need_blocked', { n: num(counts.blocked) }), sub: t('overview.need_blocked_sub'), to: '/admin/farms' },
  ].filter(Boolean) as { icon: JSX.Element; tone: string; text: string; sub: string; to: string }[];

  const byDist = T.byDist;
  const first = me?.name.split(' ')[0] ?? '';
  const hour = new Date().getHours();
  const greet = t(hour < 12 ? 'overview.morning' : hour < 18 ? 'overview.afternoon' : 'overview.evening', { name: first });

  return (
    <>
      <div className="hero ov-hero">
        <Sat cls="soft" /><Sat cls="sharp" />
        <div className="shade" />
        <div className="txt">
          <div className="eyebrow">{date(new Date().toISOString(), 'date')}</div>
          <h1>{greet}</h1>
          <div className="sub">{t('overview.hero_sub')}</div>
        </div>
        <div className="actions">
          <Link className="btn primary" to="/admin/alerts"><BellRing />{t('overview.new_alert')}</Link>
          <Link className="btn" to="/admin/farms"><Users />{t('overview.farmers_btn')}</Link>
        </div>
      </div>

      <div className="grid g4 mb">
        <Kpi label={t('overview.k_farmers')} value={num(T.all.farmers)} note={t('overview.k_farmers_n', { n: num(T.all.female) })} icon={<Users />} />
        <Kpi label={t('overview.k_farms')} value={num(T.all.farms)} note={t('overview.k_farms_n', { n: num(T.all.area) })} icon={<MapIcon />} />
        <Kpi label={t('overview.k_msgs')} value={num(newMsgs)} note={t('overview.k_msgs_n', { n: num(messages.length) })} tone={newMsgs ? 'danger' : ''} icon={<Inbox />} />
        <Kpi label={t('overview.k_listings')} value={num(openListings.n)} note={t('overview.k_listings_n', { n: num(openListings.kg / 1000, 1) })} icon={<Store />} />
      </div>

      <SeasonOutlook className="mb" />

      <div className="grid g-main-l">
        <div className="stack">
          <Card title={t('overview.needs')}>
            {needs.length ? needs.map((n, i) => (
              <Link key={i} to={n.to} className="list-item">
                <span className={'ico ' + n.tone}>{n.icon}</span>
                <div style={{ flex: 1, minWidth: 0 }}><b>{n.text}</b><div className="muted small"><bdi>{n.sub}</bdi></div></div>
                <ChevronRight className="flip-rtl muted" />
              </Link>
            )) : <div className="empty">{t('overview.all_good')}</div>}
          </Card>
          <Card title={t('overview.land_per_crop')} extra={<Link className="small" to="/admin/crops">{t('overview.crop_register')}</Link>}>
            <HBars items={cropBars} unit={t('common.du')} showShare />
          </Card>
          <div className="grid g2">
            <Card title={t('overview.latest_msgs')} extra={<Link className="small" to="/admin/inbox">{t('overview.all')}</Link>}>
              {latestMsgs.map(m => {
                const f = db.farmers.get(m.farmerId);
                return (
                  <Link key={m.id} to={'/admin/inbox?m=' + m.id} className="list-item">
                    <span className={'ico ' + (m.state === 'new' ? 'danger' : '')}><MessageSquare /></span>
                    <div style={{ flex: 1, minWidth: 0 }}>
                      <b className="ov-ellipsis"><bdi>{m.subject}</bdi></b>
                      <div className="muted small ov-ellipsis">{f ? b(f.name) : '-'} · {ago(m.at)}</div>
                    </div>
                    {m.state === 'new' && <Pill tone="danger">{t('overview.new')}</Pill>}
                  </Link>
                );
              })}
            </Card>
            <Card title={t('overview.newest_farmers')} extra={<Link className="small" to="/admin/farms">{t('overview.all')}</Link>}>
              {newestFarmers.map(f => (
                <Link key={f.id} to={'/admin/farms?farmer=' + f.id} className="list-item">
                  <span className="ico brand"><UserPlus /></span>
                  <div style={{ flex: 1, minWidth: 0 }}>
                    <b className="ov-ellipsis">{b(f.name)}</b>
                    <div className="muted small ov-ellipsis">{pn.dist(f.dist)} · {ago(f.joined)}</div>
                  </div>
                </Link>
              ))}
            </Card>
          </div>
        </div>
        <div className="stack">
          <Card title={t('overview.farms_map')} extra={<span className="muted small">{t('overview.click_district')}</span>}>
            <DistrictMap size="sm" styleKey={vFarms + lang}
              fill={d => ramp(byDist.get(d)?.farms ?? 0, [20, 50, 100, 200], GREEN_RAMP)}
              onDistrict={d => nav('/admin/farms?dist=' + encodeURIComponent(d))} />
            <div className="legend">{GREEN_RAMP.map((c, i) => <span key={c}><i className="dotc" style={{ background: c }} />{t('overview.band_' + i)}</span>)}</div>
          </Card>
          <Card title={t('overview.last_alert')} extra={<Link className="small" to="/admin/alerts">{t('overview.all')}</Link>}>
            {lastSent ? (
              <div className="row" style={{ alignItems: 'flex-start', flexWrap: 'nowrap' }}>
                <span className="ico gold"><Send /></span>
                <div style={{ minWidth: 0 }}>
                  <b><bdi>{b(lastSent.title)}</bdi></b>
                  <div className="small ink2" dir="auto">{b(lastSent.body)}</div>
                  <div className="muted small">{t('overview.sent_by', { by: lastSent.by, when: ago(lastSent.sent) })}</div>
                </div>
              </div>
            ) : <div className="empty">{t('overview.no_alert')}</div>}
          </Card>
        </div>
      </div>
    </>
  );
}
