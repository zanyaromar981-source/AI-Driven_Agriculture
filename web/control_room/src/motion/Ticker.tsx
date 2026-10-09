// The moving news bar, built from live data (decided 2026-10-09: no news route; BACKEND.md 2.12 C):
// the daily brief, fire detections, the driest districts and the dams. It stays hidden until its data
// is in the cache, and pauses under the mouse. Pure CSS animation.
import { useMemo } from 'react';
import { useI18n } from '../i18n';
import { useBrief, useDams, useFires, useOverview } from '../api/public';
import { DISTRICT_BY_SLUG } from '../data/places';

export function useNewsLines(): string[] | null {
  const { t, pick, num, lang } = useI18n();
  const ov = useOverview(), fi = useFires(), dm = useDams(), br = useBrief();
  return useMemo(() => {
    if (!ov.data && !fi.data && !br.data) return null;
    const out: string[] = [];
    const b = br.data?.brief;
    if (b) out.push(t('news.brief', { text: pick(b.headline_ku, b.headline_en) }));
    if (fi.data) out.push(fi.data.fires.length ? t('news.fires', { n: num(fi.data.fires.length), z: num(fi.data.summary.zones.length) }) : t('news.no_fires'));
    if (ov.data?.summary.driest.length) {
      const names = ov.data.summary.driest.slice(0, 3).map(s => { const d = DISTRICT_BY_SLUG.get(s); return d ? (lang === 'ku' ? d.ku : d.en) : s; });
      out.push(t('news.driest', { list: names.join(lang === 'ku' ? '، ' : ', ') }));
    }
    for (const d of dm.data?.dams ?? []) {
      const name = pick(d.name_ku, d.name_en);
      out.push(d.latest ? t('news.dam', { name, pct: num(d.latest.pct_full) }) : t('news.dam_none', { name }));
    }
    return out;
  }, [ov.data, fi.data, dm.data, br.data, t, pick, num, lang]);
}

export function Ticker() {
  const { t } = useI18n();
  const lines = useNewsLines();
  if (!lines || !lines.length) return null;
  const line = lines.map((s, i) => <span className="mq-item" key={i}><span className="sep">●</span><bdi>{s}</bdi></span>);
  return (
    <div className="ticker" role="region" aria-label={t('common.news')}>
      <span className="ticker-tag"><i className="live" />{t('common.news')}</span>
      <div className="ticker-win">
        <div className="mq-track mq-run" style={{ ['--mq-dur' as string]: Math.max(28, lines.length * 11) + 's' }}>
          <span>{line}</span><span aria-hidden="true">{line}</span>
        </div>
      </div>
    </div>
  );
}
