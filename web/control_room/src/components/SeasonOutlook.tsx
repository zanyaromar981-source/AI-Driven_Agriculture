// Season outlook: the winter rain call from the backend (GET /v1/outlooks, no login). One region-wide
// signal, the same for every district. Shows nothing when no outlook is issued (404) or the call fails.
import { useEffect, useState } from 'react';
import { CloudRain, Sun } from 'lucide-react';
import { useI18n } from '../i18n';
import { Card, Pill, type Tone } from './ui';

type Outlook = 'good' | 'normal' | 'bad';
interface OutlookZone { zone_slug: string; outlook: Outlook; confidence_pct: number; reason_en: string | null; reason_ku: string | null }
interface Outlooks {
  season: string;
  issued: string;
  counts: Record<Outlook, number>;
  zones: OutlookZone[];
  track_record?: { seasons_tested: number; seasons_right: number; method: string };
}

// The backend address. In dev the Vite server passes /v1 on to it (vite.config.ts), so the browser's
// same-origin rule is not in the way; set VITE_API_BASE to point somewhere else.
const API_BASE: string = import.meta.env.VITE_API_BASE ?? (import.meta.env.DEV ? '' : 'http://95.217.14.92:8790');

let cache: Promise<Outlooks | null> | undefined;
const loadOutlooks = () => (cache ??= fetch(API_BASE + '/v1/outlooks')
  .then(r => (r.ok ? (r.json() as Promise<Outlooks>) : null))
  .then(d => (d && d.counts && Array.isArray(d.zones) && d.zones.length ? d : null))
  .catch(() => null));

const ORDER: Outlook[] = ['good', 'normal', 'bad'];
const TONE: Record<Outlook, Tone> = { good: 'good', normal: '', bad: 'warn' };

export function SeasonOutlook({ className = '' }: { className?: string }) {
  const { t, num, lang } = useI18n();
  const [d, setD] = useState<Outlooks | null>(null);
  useEffect(() => { let on = true; loadOutlooks().then(x => { if (on) setD(x); }); return () => { on = false; }; }, []);
  if (!d) return null;

  // the headline follows the outlook most districts have
  const top = ORDER.reduce((a, k) => ((d.counts[k] ?? 0) > (d.counts[a] ?? 0) ? k : a), ORDER[0]);
  const z = d.zones.find(x => x.outlook === top) ?? d.zones[0];
  const reason = lang === 'ku' ? z.reason_ku || z.reason_en : z.reason_en || z.reason_ku;
  const tr = d.track_record;

  return (
    <Card className={className} title={t('outlook.title')} extra={<span className="muted small"><bdi>{t('outlook.season', { season: d.season })}</bdi></span>}>
      <div className="row" style={{ alignItems: 'flex-start', flexWrap: 'nowrap' }}>
        <span className={'ico ' + TONE[top]}>{top === 'bad' ? <Sun /> : <CloudRain />}</span>
        <div style={{ minWidth: 0 }}>
          <div className="row">
            <b>{t('outlook.head_' + top)}</b>
            <Pill tone={TONE[top]}><bdi>{t('outlook.confidence', { n: num(z.confidence_pct) })}</bdi></Pill>
          </div>
          {reason && <div className="small ink2" dir="auto">{reason}</div>}
          <div className="row small" style={{ marginTop: 8 }}>
            <span className="muted">{t('outlook.districts')}</span>
            {ORDER.map(k => <Pill key={k} tone={TONE[k]}>{t('outlook.' + k)} <bdi>{num(d.counts[k] ?? 0)}</bdi></Pill>)}
          </div>
          {tr && <div className="muted small" style={{ marginTop: 6 }}>{t('outlook.track', { right: num(tr.seasons_right), tested: num(tr.seasons_tested) })}</div>}
        </div>
      </div>
    </Card>
  );
}
