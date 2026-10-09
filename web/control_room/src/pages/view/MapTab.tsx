// Map tab (design tIFJP / rjX3Q): the big map with a district search, and the district panel.
// The panel shows the region summary until a district is picked, then that district from
// GET /v1/zones/{slug}?month= (topics zones, sub_zones).
import { useEffect, useMemo, useRef, useState } from 'react';
import { Search, X, ExternalLink } from 'lucide-react';
import { useI18n } from '../../i18n';
import { useApi } from '../../api/cache';
import { useBrief } from '../../api/public';
import type { FarmStats, RegionOverview, ZoneDetail } from '../../api/types';
import { DISTRICTS, DISTRICT_BY_SLUG, DISTRICT_BY_EN, GOV_BY_NAME } from '../../data/places';
import { DistrictMap } from '../../components/DistrictMap';
import { BandPill, StateBox, cropName, cropColor } from '../../components/domain';
import { DRY_STEPS, NO_DATA, dryColor, shortSource, thisMonth, viewDate } from './viewUtil';

const BAND_BG: Record<string, string> = { much_greener: 'var(--good-soft)', greener: 'var(--good-soft)', normal: 'var(--bg)', dry: 'var(--warn-soft)', very_dry: 'var(--danger-soft)' };
const BAND_FG: Record<string, string> = { much_greener: 'var(--good)', greener: 'var(--good)', normal: 'var(--ink)', dry: 'var(--warn)', very_dry: 'var(--danger)' };

export function MapTab({ ov, stats, slug, setSlug }: { ov?: RegionOverview; stats?: FarmStats; slug: string | null; setSlug: (s: string | null) => void }) {
  const { t, lang } = useI18n();
  const [q, setQ] = useState('');
  const bySlug = useMemo(() => new Map((ov?.zones ?? []).map(z => [z.slug, z])), [ov]);
  const fill = (en: string) => { const d = DISTRICT_BY_EN.get(en); return dryColor(d ? bySlug.get(d.slug)?.dryness : null); };
  const focusEn = slug ? DISTRICT_BY_SLUG.get(slug)?.en ?? null : null;
  const hits = useMemo(() => {
    const s = q.trim().toLowerCase(); if (!s) return [];
    return DISTRICTS.filter(d => d.ku.includes(q.trim()) || d.en.toLowerCase().includes(s)).slice(0, 6);
  }, [q]);
  const pickDistrict = (s: string) => { setSlug(s); setQ(''); };
  // one column (tablet, phone): the details are under the map, so bring them into view when picked
  const panel = useRef<HTMLElement>(null);
  useEffect(() => {
    if (slug && panel.current && matchMedia('(max-width: 1100px)').matches) panel.current.scrollIntoView({ behavior: 'smooth', block: 'start' });
  }, [slug]);

  return (
    <div className="view-grid">
      <section className="card view-map-card" aria-label={t('view.map_title')}>
        <div className="view-map-head">
          <div className="view-map-title">
            <h2>{t('view.map_title')}</h2>
            <span className="muted">{t('view.map_sub')}</span>
          </div>
          <div className="view-search">
            <Search aria-hidden="true" />
            <input type="search" value={q} onChange={e => setQ(e.target.value)} placeholder={t('view.search_ph')} aria-label={t('view.search_ph')}
              onKeyDown={e => { if (e.key === 'Enter' && hits[0]) pickDistrict(hits[0].slug); }} />
            {hits.length > 0 && (
              <ul className="view-hits" role="listbox">
                {hits.map(d => <li key={d.slug}><button onClick={() => pickDistrict(d.slug)}><b>{lang === 'ku' ? d.ku : d.en}</b><span className="muted">{lang === 'ku' ? GOV_BY_NAME.get(d.gov)?.ku : d.gov}</span></button></li>)}
              </ul>
            )}
          </div>
        </div>
        <DistrictMap fill={fill} styleKey={(ov?.month ?? '') + (slug ?? '')} focus={focusEn} onBack={() => setSlug(null)}
          onDistrict={en => { const d = DISTRICT_BY_EN.get(en); if (d) setSlug(d.slug); }} />
        <div className="view-legend" aria-label={t('view.legend')}>
          <b className="wet">{t('view.wetter')}</b>
          {DRY_STEPS.map(s => <span key={s.label} className="key"><i style={{ background: s.color }} /><small className="ltr">{s.label}</small></span>)}
          <b className="dry">{t('view.drier')}</b>
          <span className="key"><i style={{ background: NO_DATA }} /><small>{t('common.no_data')}</small></span>
          <span className="muted small">{t('view.normal_is')}</span>
        </div>
      </section>
      <aside className="card view-panel" ref={panel}>
        {slug ? <DistrictPanel slug={slug} change={bySlug.get(slug)?.change_vs_last_year} onClose={() => setSlug(null)} onPick={pickDistrict} />
          : <RegionPanel ov={ov} stats={stats} onPick={pickDistrict} />}
      </aside>
    </div>
  );
}

function DistrictPanel({ slug, change, onClose }: { slug: string; change?: number | null; onClose: () => void; onPick: (s: string) => void }) {
  const { t, lang, num, pick } = useI18n();
  const z = useApi<ZoneDetail>(`/zones/${slug}?month=${thisMonth()}`, ['zones', 'sub_zones']);
  const local = DISTRICT_BY_SLUG.get(slug);
  const name = z.data ? pick(z.data.name_ku, z.data.name_en) : local ? (lang === 'ku' ? local.ku : local.en) : slug;
  const gov = z.data?.governorate ?? local?.gov ?? '';
  const govName = lang === 'ku' ? GOV_BY_NAME.get(gov)?.ku ?? gov : gov;
  const r = z.data?.reading;
  return (
    <div className="view-panel-in">
      <div className="view-panel-head">
        <div>
          <h2 className="view-name">{name}</h2>
          <span className="muted">{t('view.district_in', { gov: govName })}</span>
        </div>
        <button className="btn icon" onClick={onClose} aria-label={t('common.close')}><X /></button>
      </div>
      {z.loading && <div aria-busy="true"><i className="sk h28" /><i className="sk block" style={{ minHeight: 120 }} /><i className="sk w60" /><i className="sk w80" /></div>}
      {z.error && !z.data && <StateBox kind="error" action={<button className="btn" onClick={z.reload}>{t('common.retry')}</button>} />}
      {z.data && !r && <StateBox kind="empty" title={t('view.no_reading')} text={t('view.no_reading_text')} />}
      {r && (
        <>
          <div className="view-answer" style={{ background: BAND_BG[r.band], color: BAND_FG[r.band] }}>
            <div className="txt">
              <b>{t('band.' + r.band)}</b>
              <span>{t('view.plain_' + r.band, { name })}</span>
            </div>
            <div className="num"><strong className="ltr">{num(r.dryness)}</strong><small>{t('view.of_100')}</small></div>
          </div>
          <dl className="view-facts">
            <dt>{t('view.rank')}</dt><dd>{r.rank === 1 ? t('view.rank_driest') : r.rank === r.rank_of ? t('view.rank_wettest') : t('view.rank_n', { n: num(r.rank), of: num(r.rank_of) })}</dd>
            <dt>{t('view.rain_pct')}</dt><dd>{r.rain_pct_of_normal == null ? t('common.no_data') : t('view.rain_pct_v', { p: num(r.rain_pct_of_normal) })}</dd>
            <dt>{t('view.vs_last_year')}</dt><dd>{change == null ? t('common.no_data') : (change > 0 ? '+' : '') + num(change)}</dd>
            <dt>{t('view.greenness')}</dt><dd>{r.greenness_pct_vs_normal == null ? t('common.no_data') : num(r.greenness_pct_vs_normal) + '%'}</dd>
            <dt>{t('common.source')}</dt><dd className="ltr">{shortSource(r.source)}</dd>
            <dt>{t('common.updated')}</dt><dd>{viewDate(r.updated_at, lang, true, false)}</dd>
          </dl>
        </>
      )}
      {z.data && z.data.sub_zones.length > 0 && (
        <>
          <h3 className="view-subs-title">{t('view.subs_of', { name })}</h3>
          <div className="view-subs">
            {z.data.sub_zones.map(s => (
              <div key={s.slug} className="view-sub">
                <b>{pick(s.name_ku, s.name_en)}</b>
                {s.dryness == null ? <small className="muted">{t('common.no_data')}</small> : <BandPill band={s.band} />}
              </div>
            ))}
          </div>
        </>
      )}
    </div>
  );
}

function RegionPanel({ ov, stats, onPick }: { ov?: RegionOverview; stats?: FarmStats; onPick: (s: string) => void }) {
  const { t, lang, num, pick } = useI18n();
  const brief = useBrief();
  if (!ov) return <div aria-busy="true"><i className="sk h28 w60" /><i className="sk block" style={{ minHeight: 140 }} /><i className="sk w80" /><i className="sk w60" /></div>;
  const avg = ov.summary.average_dryness;
  const sorted = [...ov.zones].filter(z => z.dryness != null).sort((a, b) => (b.dryness ?? 0) - (a.dryness ?? 0)).slice(0, 5);
  const b = brief.data?.brief;
  return (
    <div className="view-panel-in">
      <div className="view-panel-head">
        <div>
          <h2 className="view-name">{t('common.kurdistan')}</h2>
          <span className="muted">{t('view.region_sub', { n: num(ov.summary.zones_with_data) })}</span>
        </div>
      </div>
      {avg != null && (
        <div className="view-answer" style={{ background: 'var(--good-soft)', color: 'var(--good)' }}>
          <div className="txt"><b>{t('view.region_avg')}</b><span>{t('view.region_plain')}</span></div>
          <div className="num"><strong className="ltr">{num(avg, 1)}</strong><small>{t('view.of_100')}</small></div>
        </div>
      )}
      <h3 className="view-subs-title">{t('view.driest')}</h3>
      <ol className="view-driest">
        {sorted.map(z => (
          <li key={z.slug}><button onClick={() => onPick(z.slug)}><i style={{ background: dryColor(z.dryness) }} /><b>{pick(z.name_ku, z.name_en)}</b><span className="ltr">{num(z.dryness ?? 0)}</span></button></li>
        ))}
      </ol>
      {stats && (
        <>
          <h3 className="view-subs-title">{t('view.farms_title')}</h3>
          <dl className="view-facts">
            <dt>{t('common.farmers')}</dt><dd>{num(stats.totals.farmers)}</dd>
            <dt>{t('common.farms')}</dt><dd>{num(stats.totals.farms)}</dd>
            <dt>{t('common.dunam')}</dt><dd>{num(stats.totals.dunam, 1)}</dd>
          </dl>
          {stats.by_crop.length > 0 && (
            <div className="view-crops">
              {stats.by_crop.filter(c => c.crop !== 'empty').slice(0, 6).map(c => <span key={c.crop} className="pill"><i className="dotc" style={{ background: cropColor(c.crop) }} />{cropName(c.crop, lang)} <span className="ltr">{num(c.dunam, 1)}</span></span>)}
            </div>
          )}
        </>
      )}
      {b && (
        <div className="view-brief">
          <h3 className="view-subs-title">{t('view.brief_title', { day: viewDate(b.day, lang, false, false) })}</h3>
          <p><b>{pick(b.headline_ku, b.headline_en)}</b></p>
          <p className="muted">{pick(b.summary_ku, b.summary_en)}</p>
          <span className="pill warn">{t('view.brief_ai')}</span>
          {b.sources.length > 0 && <ul className="view-sources">{b.sources.slice(0, 4).map(s => <li key={s.url}><a href={s.url} target="_blank" rel="noreferrer"><ExternalLink size={13} />{s.title}</a></li>)}</ul>}
        </div>
      )}
    </div>
  );
}
