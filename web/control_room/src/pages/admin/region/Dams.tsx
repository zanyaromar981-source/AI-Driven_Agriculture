// Dams tab: Dukan and Darbandikhan, latest level, the last readings as bars, and the readings table.
// Readings can be added (POST; a day that already exists answers 409, then the user edits it),
// corrected (PUT, safe to repeat) and deleted (DELETE, a second delete is also fine).
import { useState } from 'react';
import { Plus, Pencil, Trash2, Waves } from 'lucide-react';
import { useI18n } from '../../../i18n';
import { useAuth } from '../../../auth/auth';
import { useApi, invalidate } from '../../../api/cache';
import { api, ApiError } from '../../../api/client';
import { useDams } from '../../../api/public';
import { VBars } from '../../../components/charts';
import { StateBox, useErrorText } from '../../../components/domain';
import { Confirm, Field, Modal, Note, Pill, useToast } from '../../../components/ui';
import { HAND, isHand, type DamReading, type DamRef } from './types';

export function Dams() {
  const { t } = useI18n();
  const refs = useApi<{ dams: DamRef[] }>('/dashboard/dams', ['dams'], { auth: true });
  if (refs.error && !refs.data) return <StateBox kind="error" action={<button className="btn" onClick={refs.reload}>{t('common.retry')}</button>} />;
  if (!refs.data) return <div className="grid g2"><div className="card"><i className="sk block" /></div><div className="card"><i className="sk block" /></div></div>;
  return <div className="grid g2 rg-dams">{refs.data.dams.map(d => <DamCard key={d.slug} dam={d} />)}</div>;
}

function DamCard({ dam }: { dam: DamRef }) {
  const { t, num, date, pick } = useI18n();
  const { can } = useAuth();
  const pub = useDams();
  const live = pub.data?.dams.find(d => d.slug === dam.slug);
  const rd = useApi<{ readings: DamReading[]; count: number }>(`/dashboard/dams/${dam.slug}/readings?rows_per_page=12&from=${new Date(Date.now() - 400 * 864e5).toISOString().slice(0, 10)}`, ['dams'], { auth: true });
  const [form, setForm] = useState<{ day?: string; r?: DamReading } | null>(null);
  const [del, setDel] = useState<DamReading | null>(null);
  const toast = useToast();
  const errText = useErrorText();
  const readings = [...(rd.data?.readings ?? [])].sort((a, b) => b.day.localeCompare(a.day));
  const latest = live?.latest;

  const remove = async (r: DamReading) => {
    try { await api.del(`/dashboard/dams/${dam.slug}/readings/${r.day}`); invalidate('dams'); toast(t('common.deleted'), 'good'); }
    catch (e) { toast(e instanceof ApiError ? errText(e) : t('err.generic'), 'danger'); }
  };

  return (
    <section className="card">
      <div className="spread">
        {can('dams', 'create') && <button className="btn sm" onClick={() => setForm({})}><Plus />{t('region.add_reading')}</button>}
        <div className="rg-dam-title"><h2>{t('region.dam_name', { name: pick(dam.name_ku, dam.name_en) })}</h2><span className="muted small">{t('region.capacity', { v: num(dam.capacity_bn_m3, 2) })}</span></div>
      </div>
      {latest ? (
        <div className="rg-dam-now">
          <div><b className="rg-big">{num(latest.pct_full, 1)}%</b><span className="muted small">{t('region.full_on', { day: date(latest.day, 'date') })}</span></div>
          <div className="bar" style={{ height: 12 }}><i style={{ width: Math.min(100, latest.pct_full) + '%', background: 'var(--water)' }} /></div>
          <dl className="facts">
            {live?.year_ago && <><dt>{t('region.year_ago')}</dt><dd>{num(live.year_ago.pct_full, 1)}%</dd></>}
            {latest.lake_area_km2 != null && <><dt>{t('region.lake_area')}</dt><dd>{num(latest.lake_area_km2, 1)} km²</dd></>}
            {latest.volume_bn_m3 != null && <><dt>{t('region.volume')}</dt><dd>{num(latest.volume_bn_m3, 2)}</dd></>}
          </dl>
        </div>
      ) : <StateBox kind="empty" title={t('region.dam_empty')} text={t('region.dam_empty_t')} />}
      {readings.length > 1 && <VBars height={110} unit="%" digits={0} max={100} items={[...readings].reverse().map(r => ({ key: r.day, label: date(r.day, 'short'), value: r.pct_full, color: 'var(--water)' }))} />}
      {readings.length > 0 && (
        <table className="t rg-readings">
          <thead><tr><th>{t('region.c_day')}</th><th className="num">%</th><th>{t('common.source')}</th><th /></tr></thead>
          <tbody>
            {readings.map(r => (
              <tr key={r.day}>
                <td className="nowrap">{date(r.day, 'date')}</td>
                <td className="num"><b>{num(r.pct_full, 1)}</b></td>
                <td className="small">{isHand(r.source) ? <Pill tone="warn">{t('common.edited_by_hand')}</Pill> : <bdi className="muted rg-src" title={r.source}>{r.source}</bdi>}</td>
                <td className="act">
                  {can('dams', 'update') && <button className="btn sm icon" aria-label={t('common.edit')} onClick={() => setForm({ r })}><Pencil /></button>}{' '}
                  {can('dams', 'delete') && <button className="btn sm icon danger" aria-label={t('common.delete')} onClick={() => setDel(r)}><Trash2 /></button>}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      <p className="muted small"><Waves size={12} /> {t('region.dam_source')}</p>
      {form && <ReadingForm slug={dam.slug} reading={form.r} onClose={() => setForm(null)} />}
      {del && <Confirm title={t('region.del_title')} text={t('region.del_text', { day: date(del.day, 'date') })} okLabel={t('common.delete')} onOk={() => remove(del)} onClose={() => setDel(null)} />}
    </section>
  );
}

function ReadingForm({ slug, reading, onClose }: { slug: string; reading?: DamReading; onClose: () => void }) {
  const { t } = useI18n();
  const { me } = useAuth();
  const toast = useToast();
  const errText = useErrorText();
  const [day, setDay] = useState(reading?.day ?? new Date().toISOString().slice(0, 10));
  const [pct, setPct] = useState(reading ? String(reading.pct_full) : '');
  const [area, setArea] = useState(reading?.lake_area_km2 == null ? '' : String(reading.lake_area_km2));
  const [vol, setVol] = useState(reading?.volume_bn_m3 == null ? '' : String(reading.volume_bn_m3));
  const [note, setNote] = useState('');
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState('');
  const n = (s: string) => (s.trim() === '' ? null : Number(s));

  const save = async () => {
    const p = Number(pct);
    if (!(p >= 0 && p <= 100) || pct.trim() === '') { setErr(t('region.pct_range')); return; }
    if (!/^\d{4}-\d{2}-\d{2}$/.test(day)) { setErr(t('region.day_bad')); return; }
    setBusy(true); setErr('');
    const body = { pct_full: p, lake_area_km2: n(area), volume_bn_m3: n(vol), farm_supply_bn_m3: reading?.farm_supply_bn_m3 ?? null, source: `${HAND}${me?.name ?? ''}${note.trim() ? ': ' + note.trim() : ''}`.slice(0, 100) };
    try {
      if (reading) await api.put(`/dashboard/dams/${slug}/readings/${reading.day}`, body);
      else await api.post(`/dashboard/dams/${slug}/readings`, { day, ...body });
      invalidate('dams');
      toast(t('common.saved'), 'good');
      onClose();
    } catch (e) {
      if (e instanceof ApiError && e.code === 'already_exists') setErr(t('region.day_exists'));
      else setErr(e instanceof ApiError ? errText(e) : t('err.generic'));
    } finally { setBusy(false); }
  };

  return (
    <Modal title={reading ? t('region.edit_reading') : t('region.add_reading')} onClose={onClose}
      foot={<><button className="btn" onClick={onClose}>{t('common.cancel')}</button><button className="btn primary" disabled={busy} onClick={save}>{t('common.save')}</button></>}>
      <div className="form-grid">
        <Field label={t('region.c_day')}><input type="date" value={day} disabled={!!reading} onChange={e => setDay(e.target.value)} /></Field>
        <Field label={t('region.f_pct')} hint="0 - 100"><input type="number" min={0} max={100} step="0.1" value={pct} onChange={e => setPct(e.target.value)} /></Field>
        <Field label={t('region.lake_area') + ' (km²)'}><input type="number" min={0} value={area} onChange={e => setArea(e.target.value)} /></Field>
        <Field label={t('region.volume')}><input type="number" min={0} step="0.01" value={vol} onChange={e => setVol(e.target.value)} /></Field>
        <Field full label={t('region.f_note')}><input type="text" maxLength={60} value={note} onChange={e => setNote(e.target.value)} /></Field>
      </div>
      {err && <div className="mt"><Note tone="danger">{err}</Note></div>}
    </Modal>
  );
}
