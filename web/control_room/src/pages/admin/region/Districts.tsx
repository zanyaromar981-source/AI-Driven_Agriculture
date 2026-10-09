// Districts tab: this month's dryness reading for the 33 districts, a map, and a hand correction
// (PUT /dashboard/zones/{slug}/readings/{month}, or POST when the district has no reading that month).
// The next data-job run replaces a hand change (FRONTEND.md 9), so the page says so and marks it.
import { useMemo, useState } from 'react';
import { Pencil } from 'lucide-react';
import { useI18n } from '../../../i18n';
import { useAuth } from '../../../auth/auth';
import { useApi, invalidate } from '../../../api/cache';
import { api, ApiError } from '../../../api/client';
import { useOverview } from '../../../api/public';
import type { ZoneOverview } from '../../../api/types';
import { DataTable, type Col } from '../../../components/DataTable';
import { DistrictMap, DRY_RAMP } from '../../../components/DistrictMap';
import { BandPill, StateBox, usePlace, useErrorText } from '../../../components/domain';
import { Field, Modal, Note, Pill, Switch, useToast } from '../../../components/ui';
import { DISTRICT_BY_SLUG } from '../../../data/places';
import { HAND, isHand, type ZoneReading } from './types';

const BAND_COLOR = { much_greener: DRY_RAMP[0], greener: DRY_RAMP[1], normal: DRY_RAMP[2], dry: DRY_RAMP[3], very_dry: DRY_RAMP[4] } as const;

export function Districts() {
  const { t, num, date, lang } = useI18n();
  const { can } = useAuth();
  const ov = useOverview();
  const place = usePlace();
  const [edit, setEdit] = useState<ZoneOverview | null>(null);
  const [focus, setFocus] = useState<string | null>(null);

  const byEn = useMemo(() => new Map((ov.data?.zones ?? []).map(z => [DISTRICT_BY_SLUG.get(z.slug)?.en ?? z.name_en, z])), [ov.data]);
  const rows = useMemo(() => [...(ov.data?.zones ?? [])].map(z => ({ ...z, id: z.slug })).sort((a, b) => (a.rank ?? 99) - (b.rank ?? 99)), [ov.data]);

  if (ov.error && !ov.data) return <StateBox kind="error" action={<button className="btn" onClick={ov.reload}>{t('common.retry')}</button>} />;

  const cols: Col<ZoneOverview & { id: string }>[] = [
    { key: 'name', label: t('region.c_district'), cell: z => <div><b>{lang === 'ku' ? z.name_ku : z.name_en}</b><div className="muted small">{place.gov(z.governorate)}</div></div>, sort: z => z.name_en },
    { key: 'dry', label: t('region.c_dryness'), num: true, cell: z => (z.dryness == null ? <span className="muted">-</span> : <b>{num(z.dryness)}</b>), sort: z => z.dryness ?? -1 },
    { key: 'band', label: t('region.c_band'), cell: z => <BandPill band={z.band} /> },
    { key: 'rank', label: t('region.c_rank'), num: true, cell: z => (z.rank == null ? '-' : num(z.rank)), sort: z => z.rank ?? 99 },
    { key: 'chg', label: t('region.c_change'), num: true, cell: z => (z.change_vs_last_year == null ? <span className="muted">{t('common.no_data')}</span> : num(z.change_vs_last_year)), optional: true },
    ...(can('zones', 'update') ? [{ key: 'act', label: '', className: 'act', cell: (z: ZoneOverview) => <button className="btn sm" onClick={e => { e.stopPropagation(); setEdit(z); }}><Pencil />{t('region.edit')}</button> }] : []),
  ];

  return (
    <div className="stack">
      <div className="rg-split">
        <section className="card">
          <div className="card-head"><span className="eyebrow">{t('region.map_title', { month: ov.data ? date(ov.data.month + '-01', 'month') : '' })}</span></div>
          <DistrictMap size="sm" styleKey={String(ov.data?.month) + lang} focus={focus} onBack={() => setFocus(null)}
            fill={en => { const z = byEn.get(en); return z?.band ? BAND_COLOR[z.band] : undefined; }}
            onDistrict={en => setFocus(en)} />
          <div className="legend">{(Object.keys(BAND_COLOR) as (keyof typeof BAND_COLOR)[]).map(b => <span key={b}><span className="dotc" style={{ background: BAND_COLOR[b] }} />{t('band.' + b)}</span>)}</div>
        </section>
        <section className="card">
          <div className="card-head"><span className="eyebrow">{t('region.summary')}</span></div>
          {ov.data ? (
            <dl className="facts">
              <dt>{t('region.avg')}</dt><dd>{ov.data.summary.average_dryness == null ? '-' : num(ov.data.summary.average_dryness, 1)}</dd>
              <dt>{t('region.with_data')}</dt><dd>{num(ov.data.summary.zones_with_data)} / {num(33)}</dd>
              <dt>{t('region.driest')}</dt><dd>{ov.data.summary.driest.map(s => place.dist(s)).join(lang === 'ku' ? '، ' : ', ')}</dd>
              <dt>{t('region.meaning_k')}</dt><dd className="small">{t('region.meaning')}</dd>
            </dl>
          ) : <i className="sk block" style={{ minHeight: 160 }} />}
        </section>
      </div>
      <section className="card">
        <DataTable id="region-districts" rows={rows} cols={cols} per={33} loading={ov.loading} defaultSort={['rank', 1]} />
      </section>
      {edit && <EditReading zone={edit} month={ov.data?.month ?? ''} onClose={() => setEdit(null)} />}
    </div>
  );
}

function EditReading({ zone, month, onClose }: { zone: ZoneOverview; month: string; onClose: () => void }) {
  const { t, lang, date } = useI18n();
  const { me } = useAuth();
  const toast = useToast();
  const errText = useErrorText();
  const cur = useApi<{ readings: ZoneReading[] }>(`/dashboard/zones/${zone.slug}/readings?from=${month}&to=${month}`, ['zones'], { auth: true });
  const r = cur.data?.readings.find(x => x.month === month);
  const [f, setF] = useState<{ dryness: string; rain: string; green: string; need: string; hold: boolean } | null>(null);
  const [reason, setReason] = useState('');
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState('');
  const blank = { dryness: '', rain: '', green: '', need: '', hold: false };
  // no reading this month (the list answered without one): start from an empty form and create it
  const v = f ?? (r ? { dryness: String(r.dryness), rain: r.rain_pct_of_normal == null ? '' : String(r.rain_pct_of_normal), green: r.greenness_pct_vs_normal == null ? '' : String(r.greenness_pct_vs_normal), need: r.water_need == null ? '' : String(r.water_need), hold: r.nitrogen_hold } : cur.data ? blank : null);
  const set = (k: string, val: string | boolean) => setF({ ...(v ?? blank), [k]: val });
  const n = (s: string) => (s.trim() === '' ? null : Number(s));

  const save = async () => {
    if (!v || busy) return;
    const d = Number(v.dryness);
    if (v.dryness.trim() === '' || !(d >= 0 && d <= 100)) { setErr(t('region.dry_range')); return; }
    if (reason.trim().length < 3) { setErr(t('region.reason_needed')); return; }
    setBusy(true); setErr('');
    try {
      const body = {
        dryness: d, rain_pct_of_normal: n(v.rain), greenness_pct_vs_normal: n(v.green), water_need: n(v.need), nitrogen_hold: v.hold,
        best_crops: r?.best_crops ?? [], source: `${HAND}${me?.name ?? ''}: ${reason.trim()}`.slice(0, 100),
      };
      const put = () => api.put(`/dashboard/zones/${zone.slug}/readings/${month}`, body);
      if (r) await put();
      else {
        // a job (or another person) may have stored this month meanwhile: then correct that one instead
        try { await api.post(`/dashboard/zones/${zone.slug}/readings`, { month, ...body }); }
        catch (e) { if (e instanceof ApiError && e.code === 'already_exists') await put(); else throw e; }
      }
      invalidate('zones');
      toast(t('common.saved'), 'good');
      onClose();
    } catch (e) { setErr(e instanceof ApiError ? errText(e) : t('err.generic')); } finally { setBusy(false); }
  };

  return (
    <Modal title={t('region.edit_title', { name: lang === 'ku' ? zone.name_ku : zone.name_en, month: date(month + '-01', 'month') })} onClose={onClose}
      foot={<><button className="btn" onClick={onClose} disabled={busy}>{t('common.cancel')}</button><button className="btn primary" disabled={busy || !v} onClick={save}>{busy ? t('common.loading') : t('common.save')}</button></>}>
      <div className="stack">
        <Note tone="warn">{t('region.hand_note')}</Note>
        {r && <div className="small muted">{t('region.now_source')}: {isHand(r.source) ? <Pill tone="warn">{t('common.edited_by_hand')}</Pill> : <Pill>{t('common.from_job')}</Pill>} <bdi>{r.source}</bdi></div>}
        {cur.loading && <i className="sk" />}
        {cur.data && !r && <Note tone="info">{t('region.no_reading_yet')}</Note>}
        {v && (
          <div className="form-grid">
            <Field label={t('region.f_dryness')} hint={t('region.f_dryness_h')}><input type="number" min={0} max={100} value={v.dryness} onChange={e => set('dryness', e.target.value)} /></Field>
            <Field label={t('region.f_rain')}><input type="number" min={0} value={v.rain} onChange={e => set('rain', e.target.value)} /></Field>
            <Field label={t('region.f_green')}><input type="number" min={0} value={v.green} onChange={e => set('green', e.target.value)} /></Field>
            <Field label={t('region.f_need')}><input type="number" min={0} max={100} value={v.need} onChange={e => set('need', e.target.value)} /></Field>
            <div className="field full"><span>{t('region.f_hold')}</span><Switch on={v.hold} onChange={x => set('hold', x)} label={t('region.f_hold')} /></div>
            <Field full label={t('region.f_reason')} hint={t('common.required')}><textarea rows={2} value={reason} onChange={e => setReason(e.target.value)} maxLength={80} /></Field>
          </div>
        )}
        {err && <Note tone="danger">{err}</Note>}
      </div>
    </Modal>
  );
}
