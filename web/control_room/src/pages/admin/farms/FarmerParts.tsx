// Farmer form (create and full edit), farmer drawer and farm drawer (design frames HWGis, kMILR).
// Writes: one request at a time per form (buttons disabled while pending); PUT /dashboard/farmers/{id}
// is a full replace, so every field is always sent (FRONTEND.md 9).
import { useMemo, useRef, useState } from 'react';
import { FileText, Pencil, Trash2, Ban, CircleCheck, MapPin, UserRound, TriangleAlert } from 'lucide-react';
import { useI18n } from '../../../i18n';
import { useAuth } from '../../../auth/auth';
import { api, qs, ApiError } from '../../../api/client';
import { useApi, invalidate } from '../../../api/cache';
import { Drawer, Modal, Confirm, Field, Select, Note, Pill, useToast } from '../../../components/ui';
import { CropTag, Phone, StateBox, usePlace, useErrorText, cropColor } from '../../../components/domain';
import { DistrictMap } from '../../../components/DistrictMap';
import { GOVERNORATES, DISTRICTS, SUBDISTRICTS, DISTRICT_BY_SLUG, GOV_BY_NAME } from '../../../data/places';
import { type Farmer, type FarmSummary, type FarmDetail, type FarmerLang, normPhone, phoneOk, newKey } from './common';

type Problems = Record<string, string>;

/** The body PUT needs: every field, from a farmer plus changes. */
// farmers store their governorate as a place slug ('sulaymaniyah'); farms and stats use the English name
const govSlug = (g?: string | null) => (g ? GOV_BY_NAME.get(g)?.slug ?? g : null);
const govName = (g?: string | null) => (g ? GOV_BY_NAME.get(g)?.en ?? g : '');

export const farmerBody = (f: Farmer, p: Partial<Farmer> = {}) => {
  const m = { ...f, ...p };
  return {
    lang: m.lang, name: m.name || null, gender: m.gender ?? null, birth_year: m.birth_year ?? null, village: m.village || null,
    governorate: govSlug(m.governorate), zone_slug: m.zone_slug || null, sub_zone_slug: m.sub_zone_slug || null, notes: m.notes || null, blocked: m.blocked,
  };
};

export function FarmerForm({ farmer, onClose, onSaved }: { farmer: Farmer | null; onClose: () => void; onSaved: (f: Farmer) => void }) {
  const { t, lang: uiLang } = useI18n();
  const toast = useToast();
  const errText = useErrorText();
  const [f, setF] = useState(() => ({
    phone: farmer?.phone ?? '', name: farmer?.name ?? '', lang: (farmer?.lang ?? 'ku') as FarmerLang,
    gender: farmer?.gender ?? '', birth_year: farmer?.birth_year != null ? String(farmer.birth_year) : '', village: farmer?.village ?? '',
    governorate: govName(farmer?.governorate), zone_slug: farmer?.zone_slug ?? '', sub_zone_slug: farmer?.sub_zone_slug ?? '', notes: farmer?.notes ?? '',
  }));
  const [problems, setProblems] = useState<Problems>({});
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState('');
  // one key per form, and the farmer once created: a retry after a failed details save only redoes the PUT
  const key = useRef(newKey());
  const created = useRef<Farmer | null>(null);
  const set = (k: keyof typeof f, v: string) => setF(p => ({ ...p, [k]: v, ...(k === 'governorate' ? { zone_slug: '', sub_zone_slug: '' } : k === 'zone_slug' ? { sub_zone_slug: '' } : {}) }));
  const nm = (p: { en: string; ku: string }) => (uiLang === 'ku' ? p.ku : p.en);
  const zoneEn = DISTRICT_BY_SLUG.get(f.zone_slug)?.en;
  const govs = GOVERNORATES.map(g => [g.en, nm(g)] as [string, string]);
  const dists = DISTRICTS.filter(d => !f.governorate || d.gov === f.governorate).map(d => [d.slug, nm(d)] as [string, string]);
  const subs = SUBDISTRICTS.filter(s => s.dist === zoneEn).map(s => [s.slug, nm(s)] as [string, string]);

  const check = (): Problems => {
    const p: Problems = {};
    if (!farmer && !phoneOk(normPhone(f.phone))) p.phone = 'v.phone_bad';
    const y = f.birth_year.trim();
    if (y && (!/^\d{4}$/.test(y) || +y < 1900 || +y > new Date().getFullYear() - 10)) p.birth_year = 'v.year_bad';
    if (f.notes.length > 1000) p.notes = 'farms.notes_long';
    return p;
  };

  const save = async () => {
    if (busy) return;
    const p = check(); setProblems(p); setErr('');
    if (Object.keys(p).length) return;
    setBusy(true);
    try {
      let base: Farmer;
      if (farmer) base = farmer;
      else if (created.current) base = created.current;
      else {
        const r = await api.post<{ farmer: Farmer }>('/dashboard/farmers', { phone: normPhone(f.phone), name: f.name.trim() || null, lang: f.lang }, key.current);
        base = created.current = r.farmer;
        invalidate('farmers');
      }
      const details: Partial<Farmer> = {
        name: f.name.trim() || null, lang: f.lang, gender: (f.gender || null) as Farmer['gender'], birth_year: f.birth_year.trim() ? +f.birth_year : null,
        village: f.village.trim() || null, governorate: govSlug(f.governorate), zone_slug: f.zone_slug || null, sub_zone_slug: f.sub_zone_slug || null, notes: f.notes.trim() || null,
      };
      const needPut = !!farmer || Object.entries(details).some(([k, v]) => k !== 'name' && k !== 'lang' && v != null);
      const saved = needPut ? (await api.put<{ farmer: Farmer }>('/dashboard/farmers/' + base.id, farmerBody(base, details))).farmer : base;
      invalidate('farmers');
      toast(t('common.saved'), 'good');
      onSaved(saved);
    } catch (e) {
      const code = e instanceof ApiError ? e.code : '';
      setErr(code === 'already_exists' ? t('farms.phone_taken') : errText(e as ApiError));
      if (e instanceof ApiError && e.field) setProblems(pp => ({ ...pp, [e.field!]: 'v.needed' }));
    } finally { setBusy(false); }
  };

  return (
    <Modal title={farmer ? t('farms.edit_farmer') : t('farms.add_farmer')} onClose={onClose} wide foot={<>
      <button className="btn" onClick={onClose} disabled={busy}>{t('common.cancel')}</button>
      <button className="btn primary" onClick={save} disabled={busy}>{busy ? t('common.loading') : t('common.save')}</button>
    </>}>
      <div className="form-grid">
        <Field label={t('farms.f_name')}><input type="text" value={f.name} onChange={e => set('name', e.target.value)} maxLength={120} /></Field>
        <Field label={t('farms.f_phone')} hint={farmer ? t('farms.phone_fixed') : undefined} error={problems.phone}>
          <input type="tel" value={f.phone} onChange={e => set('phone', e.target.value)} disabled={!!farmer} placeholder="0750 123 4567" />
        </Field>
        <Field label={t('farms.f_lang')}><Select value={f.lang} onChange={v => set('lang', v)} options={[['ku', t('farms.lang_ku')], ['kmr', t('farms.lang_kmr')], ['ar', t('farms.lang_ar')], ['en', t('farms.lang_en')]]} /></Field>
        <Field label={t('farms.f_gender')}><Select value={f.gender} onChange={v => set('gender', v)} options={[['', t('farms.not_said')], ['male', t('farms.male')], ['female', t('farms.female')]]} /></Field>
        <Field label={t('farms.f_birth_year')} error={problems.birth_year}><input type="number" inputMode="numeric" value={f.birth_year} onChange={e => set('birth_year', e.target.value)} placeholder="1975" /></Field>
        <Field label={t('farms.f_village')}><input type="text" value={f.village} onChange={e => set('village', e.target.value)} maxLength={120} /></Field>
        <Field label={t('common.governorate')}><Select value={f.governorate} onChange={v => set('governorate', v)} options={[['', t('farms.not_set')], ...govs]} /></Field>
        <Field label={t('common.district')}><Select value={f.zone_slug} onChange={v => set('zone_slug', v)} options={[['', t('farms.not_set')], ...dists]} /></Field>
        <Field label={t('common.subdistrict')}><Select value={f.sub_zone_slug} onChange={v => set('sub_zone_slug', v)} options={[['', t('farms.not_set')], ...subs]} disabled={!f.zone_slug} /></Field>
        <Field label={t('farms.f_notes')} hint={t('farms.notes_hint')} error={problems.notes} full>
          <textarea rows={3} value={f.notes} onChange={e => set('notes', e.target.value)} maxLength={1000} />
        </Field>
      </div>
      {err && <div style={{ marginTop: 10 }}><Note tone="danger">{err}</Note></div>}
    </Modal>
  );
}

const FarmCropStrip = ({ crops, area }: { crops: { crop: string; dunam: number }[]; area: number }) => (
  <div className="crop-strip" aria-hidden="true">{crops.map(c => <i key={c.crop} style={{ width: Math.max(2, c.dunam / Math.max(area, 0.001) * 100) + '%', background: cropColor(c.crop) }} />)}</div>
);

export function FarmerDrawer({ id, onClose, onOpenFarm, onEdit }: { id: string; onClose: () => void; onOpenFarm: (id: string) => void; onEdit: (f: Farmer) => void }) {
  const { t, num, date } = useI18n();
  const { can } = useAuth();
  const toast = useToast();
  const errText = useErrorText();
  const place = usePlace();
  const q = useApi<{ farmer: Farmer }>('/dashboard/farmers/' + id, ['farmers'], { auth: true });
  const f = q.data?.farmer;
  const farms = useApi<{ farms: FarmSummary[]; count: number }>(f && can('farms') ? '/dashboard/farms' + qs({ owner_phone: f.phone, rows_per_page: 100 }) : null, ['farms'], { auth: true });
  const [confirm, setConfirm] = useState<'delete' | 'block' | null>(null);
  const [busy, setBusy] = useState(false);
  const list = farms.data?.farms ?? [];
  const total = list.reduce((a, x) => a + x.area_dunam, 0);

  const run = async (what: 'delete' | 'block') => {
    if (!f || busy) return;
    setBusy(true);
    try {
      if (what === 'delete') { await api.del('/dashboard/farmers/' + f.id); invalidate('farmers', 'farms'); toast(t('common.deleted'), 'good'); onClose(); }
      else { await api.put('/dashboard/farmers/' + f.id, farmerBody(f, { blocked: !f.blocked })); invalidate('farmers'); toast(f.blocked ? t('farms.unblocked') : t('farms.blocked_done'), 'good'); }
    } catch (e) { toast(errText(e as ApiError), 'danger'); } finally { setBusy(false); }
  };

  if (q.error?.status === 404) return <Drawer title={t('farms.farmer')} onClose={onClose}><StateBox kind="empty" title={t('farms.farmer_gone')} text={t('farms.farmer_gone_text')} /></Drawer>;
  if (!f) return <Drawer title={t('common.loading')} onClose={onClose}>{q.error ? <StateBox kind="error" action={<button className="btn" onClick={q.reload}>{t('common.retry')}</button>} /> : <div className="sk-rows">{[0, 1, 2, 3, 4].map(i => <i key={i} className="sk" />)}</div>}</Drawer>;

  const home = [f.village, place.sub(f.sub_zone_slug), place.dist(f.zone_slug), place.gov(f.governorate)].filter(Boolean).join('، ');
  return (
    <Drawer eyebrow={t('farms.farmer') + ' #' + f.id} title={<span className="row" style={{ gap: 10 }}>{f.name || t('farms.no_name')} {f.blocked ? <Pill tone="danger">{t('common.blocked')}</Pill> : <Pill tone="good">{t('common.active')}</Pill>}</span>} onClose={onClose}>
      <div className="farm-actions" style={{ marginBottom: 14 }}>
        {can('farmers', 'create') && <a className="btn primary" href={'#/print/letter/' + f.id} target="_blank" rel="noopener"><FileText />{t('farms.letter')}</a>}
        {can('farmers', 'update') && <button className="btn" onClick={() => onEdit(f)} disabled={busy}><Pencil />{t('common.edit')}</button>}
        {can('farmers', 'update') && <button className="btn" onClick={() => setConfirm('block')} disabled={busy}>{f.blocked ? <CircleCheck /> : <Ban />}{f.blocked ? t('farms.unblock') : t('farms.block')}</button>}
        {can('farmers', 'delete') && <button className="btn danger" onClick={() => setConfirm('delete')} disabled={busy}><Trash2 />{t('common.delete')}</button>}
      </div>
      <dl className="facts card" style={{ background: 'var(--bg)', boxShadow: 'none' }}>
        <dt>{t('farms.f_phone')}</dt><dd><Phone value={f.phone} /></dd>
        <dt>{t('farms.f_gender')}</dt><dd>{f.gender ? t('farms.' + f.gender) : '-'}</dd>
        <dt>{t('farms.f_birth_year')}</dt><dd>{f.birth_year ?? '-'}</dd>
        <dt>{t('farms.home')}</dt><dd>{home || '-'}</dd>
        <dt>{t('farms.f_lang')}</dt><dd>{t('farms.lang_' + f.lang)}</dd>
        <dt>{t('farms.joined')}</dt><dd>{date(f.created_at)}</dd>
        <dt>{t('common.farms')}</dt><dd>{t('farms.n_farms_dunam', { n: num(f.farms_count), d: num(total, 1) })}</dd>
        {f.notes && <><dt>{t('farms.f_notes')}</dt><dd style={{ fontWeight: 400 }}><bdi>{f.notes}</bdi></dd></>}
      </dl>
      <h3 style={{ margin: '18px 0 8px' }}>{t('farms.their_farms')}</h3>
      {farms.loading && <div className="sk-rows">{[0, 1].map(i => <i key={i} className="sk" />)}</div>}
      {!farms.loading && !list.length && <div className="muted small">{t('farms.no_farms')}</div>}
      <div className="stack" style={{ gap: 8 }}>
        {list.map(x => (
          <button key={x.id} className="card farm-card click" onClick={() => onOpenFarm(x.id)} style={{ textAlign: 'start', font: 'inherit', cursor: 'pointer' }}>
            <div className="spread" style={{ flexWrap: 'nowrap' }}>
              <div style={{ minWidth: 0 }}><b><bdi>{x.name}</bdi></b><div className="muted small">{[place.sub(x.sub_zone_slug), place.dist(x.zone_slug)].filter(Boolean).join('، ') || t('farms.no_place')}</div></div>
              <div className="end"><b className="tabular" style={{ fontSize: 18 }}>{num(x.area_dunam, 1)}</b><div className="muted small">{t('common.dunam')}</div></div>
            </div>
            {x.crops.length > 0 && <><div style={{ margin: '8px 0 6px' }}><FarmCropStrip crops={x.crops} area={x.area_dunam} /></div>
              <div className="row small">{x.crops.map(c => <span key={c.crop}><CropTag code={c.crop} /> {num(c.dunam, 1)}</span>)}</div></>}
          </button>
        ))}
      </div>
      {can('farmers', 'delete') && <div style={{ marginTop: 16 }}><Note tone="danger" icon={<TriangleAlert />}>{t('farms.delete_note')}</Note></div>}
      {confirm === 'delete' && <Confirm title={t('farms.delete_farmer_q', { name: f.name || f.phone })} text={t('farms.delete_farmer_text', { n: num(f.farms_count) })} okLabel={t('common.delete')} onOk={() => run('delete')} onClose={() => setConfirm(null)} />}
      {confirm === 'block' && <Confirm danger={!f.blocked} title={f.blocked ? t('farms.unblock_q') : t('farms.block_q')} text={f.blocked ? t('farms.unblock_text') : t('farms.block_text')} okLabel={f.blocked ? t('farms.unblock') : t('farms.block')} onOk={() => run('block')} onClose={() => setConfirm(null)} />}
    </Drawer>
  );
}

export function FarmDrawer({ id, onClose, onOpenFarmer }: { id: string; onClose: () => void; onOpenFarmer: (id: string) => void }) {
  const { t, num, date } = useI18n();
  const { can } = useAuth();
  const toast = useToast();
  const errText = useErrorText();
  const place = usePlace();
  const q = useApi<{ farm: FarmDetail }>('/dashboard/farms/' + id, ['farms'], { auth: true });
  const farm = q.data?.farm;
  const owner = useApi<{ farmers: Farmer[] }>(farm?.owner_phone && can('farmers') ? '/dashboard/farmers' + qs({ phone: farm.owner_phone, rows_per_page: 1 }) : null, ['farmers'], { auth: true });
  const ownerRow = owner.data?.farmers[0];
  const [rename, setRename] = useState<string | null>(null);
  const [confirm, setConfirm] = useState(false);
  const [busy, setBusy] = useState(false);
  const distEn = farm?.zone_slug ? DISTRICT_BY_SLUG.get(farm.zone_slug)?.en ?? null : null;
  const points = useMemo(() => (farm ? [{ id: farm.id, lat: farm.centroid.lat, lon: farm.centroid.lon, color: '#B98A2B', radius: 8 }] : []), [farm]);

  const doRename = async () => {
    if (!farm || busy || rename == null) return;
    const name = rename.trim();
    if (!name || name.length > 100) { toast(t('farms.name_bad'), 'warn'); return; }
    setBusy(true);
    try { await api.put('/dashboard/farms/' + farm.id, { name }); invalidate('farms'); setRename(null); toast(t('common.saved'), 'good'); }
    catch (e) { toast(errText(e as ApiError), 'danger'); } finally { setBusy(false); }
  };
  const doDelete = async () => {
    if (!farm || busy) return;
    setBusy(true);
    try { await api.del('/dashboard/farms/' + farm.id); invalidate('farms', 'farmers'); toast(t('common.deleted'), 'good'); onClose(); }
    catch (e) { toast(errText(e as ApiError), 'danger'); } finally { setBusy(false); }
  };

  if (q.error?.status === 404) return <Drawer title={t('farms.farm')} onClose={onClose}><StateBox kind="empty" title={t('farms.farm_gone')} /></Drawer>;
  if (!farm) return <Drawer title={t('common.loading')} onClose={onClose}>{q.error ? <StateBox kind="error" action={<button className="btn" onClick={q.reload}>{t('common.retry')}</button>} /> : <div className="sk-rows">{[0, 1, 2, 3].map(i => <i key={i} className="sk" />)}</div>}</Drawer>;
  const used = farm.crops.reduce((a, c) => a + c.dunam, 0);
  return (
    <Drawer eyebrow={t('farms.farm') + ' #' + farm.id} title={<bdi>{farm.name}</bdi>} onClose={onClose}>
      <div className="farm-actions" style={{ marginBottom: 14 }}>
        {ownerRow && <button className="btn" onClick={() => onOpenFarmer(ownerRow.id)}><UserRound />{t('farms.open_owner')}</button>}
        {can('farms', 'update') && <button className="btn" onClick={() => setRename(farm.name)} disabled={busy}><Pencil />{t('farms.rename')}</button>}
        {can('farms', 'delete') && <button className="btn danger" onClick={() => setConfirm(true)} disabled={busy}><Trash2 />{t('common.delete')}</button>}
      </div>
      <DistrictMap size="xs" styleKey={farm.id} focus={distEn} showSubs fill={d => (d === distEn ? '#BFDCC8' : '#E3E6DE')} points={points} />
      <dl className="facts" style={{ marginTop: 14 }}>
        <dt>{t('common.area')}</dt><dd>{num(farm.area_dunam, 2)} {t('common.dunam')} · {num(farm.area_dunam * 2500)} m²</dd>
        <dt>{t('farms.place')}</dt><dd>{[place.sub(farm.sub_zone_slug), place.dist(farm.zone_slug), place.gov(farm.governorate)].filter(Boolean).join('، ') || t('farms.no_place')}</dd>
        <dt>{t('farms.point')}</dt><dd><span className="ltr tabular">{farm.centroid.lat.toFixed(5)}, {farm.centroid.lon.toFixed(5)}</span> <a href={`https://www.google.com/maps?q=${farm.centroid.lat},${farm.centroid.lon}`} target="_blank" rel="noopener noreferrer"><MapPin size={13} /></a></dd>
        <dt>{t('farms.owner')}</dt><dd>{ownerRow?.name ? <bdi>{ownerRow.name}</bdi> : null} <Phone value={farm.owner_phone} /></dd>
        <dt>{t('farms.registered')}</dt><dd>{date(farm.created_at)}</dd>
        <dt>{t('farms.planted')}</dt><dd>{num(used, 1)} / {num(farm.area_dunam, 1)} {t('common.dunam')}</dd>
      </dl>
      <h3 style={{ margin: '18px 0 8px' }}>{t('farms.crops')}</h3>
      {farm.crops.length ? <>
        <FarmCropStrip crops={farm.crops} area={farm.area_dunam} />
        <div className="stack" style={{ gap: 6, marginTop: 10 }}>{farm.crops.map(c => <div key={c.crop} className="spread"><CropTag code={c.crop} /><b className="tabular">{num(c.dunam, 2)} {t('common.dunam')}</b></div>)}</div>
      </> : <div className="muted small">{t('farms.no_crops')}</div>}
      {rename != null && <Modal title={t('farms.rename')} onClose={() => setRename(null)} foot={<>
        <button className="btn" onClick={() => setRename(null)} disabled={busy}>{t('common.cancel')}</button>
        <button className="btn primary" onClick={doRename} disabled={busy}>{t('common.save')}</button></>}>
        <Field label={t('farms.farm_name')} hint={t('farms.rename_hint')}><input type="text" value={rename} maxLength={100} onChange={e => setRename(e.target.value)} autoFocus /></Field>
      </Modal>}
      {confirm && <Confirm title={t('farms.delete_farm_q', { name: farm.name })} text={t('farms.delete_farm_text')} okLabel={t('common.delete')} onOk={doDelete} onClose={() => setConfirm(false)} />}
    </Drawer>
  );
}
