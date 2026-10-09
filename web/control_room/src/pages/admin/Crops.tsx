// Crop register (design psYq3): land, farms and expected harvest per crop from GET /dashboard/stats/farms,
// a district map for one crop, a crop-by-governorate table, and the crop list. The list is the server's
// crops table (FRONTEND.md, Crops): add, edit (PUT sends the whole crop), switch off, delete. A crop that
// farms or Alwa still use cannot be deleted (409 crop_in_use); the page offers to switch it off instead.
import { useMemo, useState } from 'react';
import { Wheat, Ruler, Scale, Sprout, FileText, Plus, Pencil, Trash2 } from 'lucide-react';
import { useI18n } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { api, qs, ApiError } from '../../api/client';
import { useApi, invalidate } from '../../api/cache';
import type { FarmStats } from '../../api/types';
import { PageHead, Kpi, Tabs, Card, Modal, Select, Field, Confirm, Note, Switch, useToast } from '../../components/ui';
import { HBars } from '../../components/charts';
import { DistrictMap, ramp, GOLD_RAMP } from '../../components/DistrictMap';
import { StateBox, cropColor, cropName, useErrorText, useCrops } from '../../components/domain';
import { CROPS } from '../../data/crops';
import { GOVERNORATES, DISTRICTS, DISTRICT_BY_EN } from '../../data/places';
import { expectedTonnes, sumTonnes } from './farms/common';
import './crops.css';

type Tab = 'overview' | 'list';
type Category = 'cereal' | 'vegetable' | 'fruit' | 'legume' | 'oil' | 'fodder' | 'other';
type Season = 'winter' | 'summer' | 'perennial';
interface ServerCrop {
  code: string; name_en: string; name_ku?: string | null; color: string; category: Category; season: Season;
  yield_kg_per_dunam?: number | null; active: boolean; sort_order: number; created_at?: string; updated_at?: string;
}
const CATEGORIES: Category[] = ['cereal', 'vegetable', 'fruit', 'legume', 'oil', 'fodder', 'other'];
const SEASONS: Season[] = ['winter', 'summer', 'perennial'];
/** The body PUT and POST take: every field, never the timestamps. */
const cropBody = (c: ServerCrop) => ({
  code: c.code, name_en: c.name_en, name_ku: c.name_ku || null, color: c.color, category: c.category, season: c.season,
  yield_kg_per_dunam: c.yield_kg_per_dunam ?? null, active: c.active, sort_order: c.sort_order,
});

export default function Crops() {
  const { t, num, lang } = useI18n();
  const { can } = useAuth();
  const errText = useErrorText();
  const [tab, setTab] = useState<Tab>(() => (can('farms') ? 'overview' : 'list'));
  const [pick, setPick] = useState('');
  const [report, setReport] = useState(false);
  const [rp, setRp] = useState({ crop: '', governorate: '', zone: '' });
  const q = useApi<FarmStats>(can('farms') ? '/dashboard/stats/farms' : null, ['farms'], { auth: true });
  // every crop, also the switched-off ones (the public /crops list leaves those out)
  const list = useApi<{ crops: ServerCrop[] }>(can('crops') ? '/dashboard/crops' : null, ['crops'], { auth: true });
  const s = q.data;
  const crops = useMemo(() => (s ? s.by_crop.filter(c => c.crop !== 'empty').sort((a, b) => b.dunam - a.dunam) : []), [s]);
  const planted = crops.reduce((a, c) => a + c.dunam, 0);
  const harvest = sumTonnes(crops);
  const tonnes = (v: number | null) => (v == null ? '-' : num(v, 1));
  useCrops();
  const top = crops[0];
  // land of the picked crop (or all planted land) per district, read in one pass over by_zone
  const perZone = useMemo(() => {
    const m = new Map<string, number>();
    for (const z of s?.by_zone ?? []) m.set(z.slug, pick ? z.crops.find(c => c.crop === pick)?.dunam ?? 0 : z.crops.filter(c => c.crop !== 'empty').reduce((a, c) => a + c.dunam, 0));
    return m;
  }, [s, pick]);
  const maxZone = Math.max(1e-9, ...perZone.values());
  const topZones = useMemo(() => [...(s?.by_zone ?? [])].map(z => ({ z, v: perZone.get(z.slug) ?? 0 })).filter(x => x.v > 0).sort((a, b) => b.v - a.v).slice(0, 8), [s, perZone]);
  const nm = (p: { en: string; ku: string }) => (lang === 'ku' ? p.ku : p.en);

  if (!can('farms') && !can('crops')) return <><PageHead eyebrow={t('nav.g_fields')} title={t('nav.crops')} /><StateBox kind="locked" /></>;

  return (
    <div>
      <PageHead eyebrow={t('nav.g_fields')} title={t('nav.crops')} sub={t('crops.sub')} actions={<>
        {can('farms') && <button className="btn" onClick={() => setReport(true)}><FileText />{t('crops.report')}</button>}
      </>} />
      <div className="grid g4 mb">
        <Kpi label={t('crops.k_active')} icon={<Sprout />} value={s ? num(crops.length) : '-'} note={t('crops.k_active_note', { n: num(CROPS.filter(c => c.active !== false).length) })} />
        <Kpi label={t('crops.k_planted')} icon={<Ruler />} value={s ? num(planted, 1) : '-'} note={s ? t('crops.k_planted_note', { n: num(s.totals.farms) }) : ' '} />
        <Kpi label={t('crops.k_top')} icon={<Wheat />} value={top ? cropName(top.crop, lang) : '-'} note={top && planted ? t('crops.k_top_note', { p: num(top.dunam / planted * 100) }) : ' '} />
        <Kpi label={t('crops.k_harvest')} icon={<Scale />} value={s && harvest != null ? num(harvest, 1) + ' ' + t('common.tonnes') : '-'} note={t('crops.k_harvest_note')} />
      </div>
      <Tabs<Tab> value={tab} onChange={setTab} items={[...(can('farms') ? [['overview', t('crops.tab_overview')] as [Tab, string]] : []), ['list', t('crops.tab_list')]]} />
      {tab === 'list' ? <CropList stats={s} server={list.data?.crops} loading={list.loading} error={list.error} reload={list.reload} /> : q.error && !s ? <StateBox kind="error" text={errText(q.error)} action={<button className="btn" onClick={q.reload}>{t('common.retry')}</button>} /> : !s ? <div className="card"><div className="sk-rows">{[0, 1, 2, 3, 4].map(i => <i key={i} className="sk" />)}</div></div> : (
        !crops.length ? <Card><StateBox kind="empty" title={t('crops.none_title')} text={t('crops.none_text')} /></Card> : <>
          <div className="chips mb">
            <button className={'chip' + (!pick ? ' on' : '')} onClick={() => setPick('')}>{t('common.all_crops')}</button>
            {crops.map(c => <button key={c.crop} className={'chip' + (pick === c.crop ? ' on' : '')} onClick={() => setPick(c.crop)}><span className="dotc" style={{ background: cropColor(c.crop) }} />{cropName(c.crop, lang)}</button>)}
          </div>
          <div className="grid g-main mb">
            <Card title={pick ? t('crops.map_one', { crop: cropName(pick, lang) }) : t('crops.map_all')}>
              <DistrictMap styleKey={pick + ':' + q.data?.as_of} size="sm" fill={en => { const d = DISTRICT_BY_EN.get(en); const v = d ? perZone.get(d.slug) ?? 0 : 0; return v > 0 ? ramp(v / maxZone, [0.15, 0.35, 0.6, 0.85], GOLD_RAMP) : undefined; }} />
              <div className="legend">{GOLD_RAMP.map((c, i) => <span key={c}><i className="dotc" style={{ background: c }} />{[t('crops.l1'), t('crops.l2'), t('crops.l3'), t('crops.l4'), t('crops.l5')][i]}</span>)}</div>
            </Card>
            <div className="stack">
              <Card title={t('crops.top_districts')}>
                {topZones.length ? <HBars items={topZones.map(({ z, v }) => ({ key: z.slug, label: lang === 'ku' ? z.name_ku || z.name_en : z.name_en, value: v, color: pick ? cropColor(pick) : 'var(--gold)' }))} digits={1} unit={t('common.dunam')} /> : <div className="muted small">{t('crops.no_land')}</div>}
              </Card>
            </div>
          </div>
          <div className="grid g2 mb">
            <Card title={t('crops.land_per_crop')}>
              <HBars items={crops.map(c => ({ key: c.crop, label: cropName(c.crop, lang), value: c.dunam, color: cropColor(c.crop) }))} digits={1} unit={t('common.dunam')} showShare />
            </Card>
            <Card title={t('crops.table_title')}>
              <div className="table-wrap"><table className="t cards">
                <thead><tr><th>{t('crops.crop')}</th><th className="num">{t('common.farms')}</th><th className="num">{t('common.farmers')}</th><th className="num">{t('common.dunam')}</th><th className="num">{t('crops.harvest_t')}</th></tr></thead>
                <tbody>{crops.map(c => <tr key={c.crop}>
                  <td data-label={t('crops.crop')}><span className="dotc" style={{ background: cropColor(c.crop) }} />{cropName(c.crop, lang)}</td>
                  <td className="num" data-label={t('common.farms')}>{num(c.farms)}</td><td className="num" data-label={t('common.farmers')}>{num(c.farmers)}</td>
                  <td className="num" data-label={t('common.dunam')}><b>{num(c.dunam, 1)}</b></td><td className="num" data-label={t('crops.harvest_t')}>{tonnes(expectedTonnes(c.crop, c.dunam))}</td>
                </tr>)}</tbody>
              </table></div>
              <p className="muted small">{t('crops.estimate_note')}</p>
            </Card>
          </div>
          <Card title={t('crops.by_gov')}>
            <div className="table-wrap"><table className="t">
              <thead><tr><th>{t('crops.crop')}</th>{s.by_governorate.map(g => <th key={g.slug} className="num">{lang === 'ku' ? g.name_ku || g.name_en : g.name_en}</th>)}</tr></thead>
              <tbody>{crops.map(c => <tr key={c.crop}><td><span className="dotc" style={{ background: cropColor(c.crop) }} />{cropName(c.crop, lang)}</td>
                {s.by_governorate.map(g => <td key={g.slug} className="num">{num(g.crops.find(x => x.crop === c.crop)?.dunam ?? 0, 1)}</td>)}</tr>)}</tbody>
            </table></div>
          </Card>
        </>
      )}
      {report && <Modal title={t('crops.report')} onClose={() => setReport(false)} foot={<>
        <button className="btn" onClick={() => setReport(false)}>{t('common.cancel')}</button>
        <a className="btn primary" href={'#/print/crops' + qs(rp)} target="_blank" rel="noopener" onClick={() => setReport(false)}><FileText />{t('crops.open_report')}</a>
      </>}>
        <div className="form-grid">
          <Field label={t('crops.crop')} full><Select value={rp.crop} onChange={v => setRp(p => ({ ...p, crop: v }))} options={[['', t('common.all_crops')], ...CROPS.filter(c => c.code !== 'empty').map(c => [c.code, lang === 'ku' ? c.ku : c.en] as [string, string])]} /></Field>
          <Field label={t('common.governorate')}><Select value={rp.governorate} onChange={v => setRp(p => ({ ...p, governorate: v, zone: '' }))} options={[['', t('common.all_govs')], ...GOVERNORATES.map(g => [g.en, nm(g)] as [string, string])]} /></Field>
          <Field label={t('common.district')}><Select value={rp.zone} onChange={v => setRp(p => ({ ...p, zone: v }))} options={[['', t('common.all_dists')], ...DISTRICTS.filter(d => !rp.governorate || d.gov === rp.governorate).map(d => [d.slug, nm(d)] as [string, string])]} /></Field>
        </div>
      </Modal>}
    </div>
  );
}

/** The crop list: the server's table when this person may read it, else the list the site knows. */
function CropList({ stats, server, loading, error, reload }: { stats?: FarmStats; server?: ServerCrop[]; loading: boolean; error: ApiError | null; reload: () => void }) {
  const { t, num, lang } = useI18n();
  const { can } = useAuth();
  const toast = useToast();
  const errText = useErrorText();
  const [form, setForm] = useState<ServerCrop | 'new' | null>(null);
  const [ask, setAsk] = useState<{ crop: ServerCrop; kind: 'delete' | 'in_use' } | null>(null);
  const [busy, setBusy] = useState(false);
  const rows: ServerCrop[] = useMemo(() => server ? [...server].sort((a, b) => a.sort_order - b.sort_order || a.code.localeCompare(b.code))
    : CROPS.map((c, i) => ({ code: c.code, name_en: c.en, name_ku: c.ku, color: c.color, category: 'other', season: 'winter', yield_kg_per_dunam: c.yieldKgPerDunam ?? null, active: c.active ?? true, sort_order: i })), [server]);

  // the crop as the server has it now: a PUT replaces every field, so it never starts from an old copy
  const fresh = async (code: string) => (await api.get<{ crops: ServerCrop[] }>('/dashboard/crops')).crops.find(c => c.code === code);

  const remove = async (c: ServerCrop) => {
    if (busy) return;
    setBusy(true);
    try { await api.del('/dashboard/crops/' + encodeURIComponent(c.code)); invalidate('crops'); toast(t('common.deleted'), 'good'); }
    catch (e) {
      const x = e as ApiError;
      if (x.status === 404) { invalidate('crops'); toast(t('common.deleted'), 'good'); }
      else if (x.code === 'crop_in_use') setAsk({ crop: c, kind: 'in_use' });
      else toast(errText(x), 'danger');
    } finally { setBusy(false); }
  };
  const switchOff = async (c: ServerCrop) => {
    if (busy) return;
    setBusy(true);
    try {
      const cur = await fresh(c.code);
      if (!cur) { invalidate('crops'); toast(t('crops.gone'), 'warn'); return; }
      await api.put('/dashboard/crops/' + encodeURIComponent(c.code), cropBody({ ...cur, active: false }));
      invalidate('crops'); toast(t('crops.switched_off'), 'good');
    } catch (e) { toast(errText(e as ApiError), 'danger'); } finally { setBusy(false); }
  };

  const nameOf = (c: ServerCrop) => (lang === 'ku' ? c.name_ku || c.name_en : c.name_en);
  return (
    <div className="stack">
      {!server && can('crops') && error && <StateBox kind="error" text={errText(error)} action={<button className="btn" onClick={reload}>{t('common.retry')}</button>} />}
      <Card title={t('crops.list_title')} extra={can('crops', 'create') ? <button className="btn primary sm" onClick={() => setForm('new')}><Plus />{t('crops.add')}</button> : undefined}>
        {loading && !server ? <div className="sk-rows">{[0, 1, 2, 3].map(i => <i key={i} className="sk" />)}</div> : (
          <div className="table-wrap"><table className="t cards">
            <thead><tr><th>{t('crops.crop')}</th><th>{t('crops.code')}</th><th>{t('crops.name_other')}</th><th>{t('crops.f_category')}</th><th className="num">{t('crops.yield')}</th>{stats && <th className="num">{t('common.dunam')}</th>}<th>{t('common.status')}</th><th /></tr></thead>
            <tbody>{rows.map(c => {
              const used = stats?.by_crop.find(x => x.crop === c.code);
              return <tr key={c.code} className={c.active ? '' : 'muted'}>
                <td data-label={t('crops.crop')}><span className="dotc" style={{ background: c.color }} /><b>{nameOf(c)}</b></td>
                <td data-label={t('crops.code')} className="mono">{c.code}</td>
                <td data-label={t('crops.name_other')}>{lang === 'ku' ? c.name_en : c.name_ku ? <span className="ku-text">{c.name_ku}</span> : <span className="muted">{t('crops.no_ku')}</span>}</td>
                <td data-label={t('crops.f_category')}>{server ? t('crops.cat_' + c.category) + ' · ' + t('crops.season_' + c.season) : '-'}</td>
                <td data-label={t('crops.yield')} className="num">{c.yield_kg_per_dunam ? num(c.yield_kg_per_dunam) + ' ' + t('common.kg') : '-'}</td>
                {stats && <td data-label={t('common.dunam')} className="num">{used ? num(used.dunam, 1) : '0'}</td>}
                <td data-label={t('common.status')}>{!c.active ? <span className="pill">{t('crops.off')}</span> : used ? <span className="pill good">{t('crops.in_use')}</span> : <span className="pill brand">{t('common.active')}</span>}</td>
                <td className="act">{server && <span className="row" style={{ justifyContent: 'flex-end' }}>
                  {can('crops', 'update') && <button className="btn sm icon" disabled={busy} onClick={() => setForm(c)} aria-label={t('common.edit')} title={t('common.edit')}><Pencil /></button>}
                  {can('crops', 'delete') && <button className="btn sm icon danger" disabled={busy} onClick={() => setAsk({ crop: c, kind: 'delete' })} aria-label={t('common.delete')} title={t('common.delete')}><Trash2 /></button>}
                </span>}</td>
              </tr>;
            })}</tbody>
          </table></div>
        )}
        {!server && <p className="muted small">{t('crops.local_note')}</p>}
        <p className="muted small">{t('crops.yield_note')}</p>
      </Card>
      {form && <CropForm crop={form === 'new' ? null : form} taken={rows.map(r => r.code)} nextOrder={Math.max(0, ...rows.map(r => r.sort_order)) + 10} onClose={() => setForm(null)} fresh={fresh} />}
      {ask?.kind === 'delete' && <Confirm title={t('crops.delete_q', { name: nameOf(ask.crop) })} text={t('crops.delete_text')} okLabel={t('common.delete')} onOk={() => remove(ask.crop)} onClose={() => setAsk(null)} />}
      {ask?.kind === 'in_use' && <Confirm danger={false} title={t('crops.in_use_q', { name: nameOf(ask.crop) })} text={t('crops.in_use_text')} okLabel={t('crops.switch_off')} onOk={() => switchOff(ask.crop)} onClose={() => setAsk(null)} />}
    </div>
  );
}

function CropForm({ crop, taken, nextOrder, onClose, fresh }: { crop: ServerCrop | null; taken: string[]; nextOrder: number; onClose: () => void; fresh: (code: string) => Promise<ServerCrop | undefined> }) {
  const { t } = useI18n();
  const toast = useToast();
  const errText = useErrorText();
  const [f, setF] = useState(() => ({
    code: crop?.code ?? '', name_en: crop?.name_en ?? '', name_ku: crop?.name_ku ?? '', color: crop?.color ?? '#7A9A3A',
    category: (crop?.category ?? 'vegetable') as Category, season: (crop?.season ?? 'summer') as Season,
    yield: crop?.yield_kg_per_dunam != null ? String(crop.yield_kg_per_dunam) : '', active: crop?.active ?? true, sort: String(crop?.sort_order ?? nextOrder),
  }));
  const [problems, setProblems] = useState<Record<string, string>>({});
  const [err, setErr] = useState('');
  const [busy, setBusy] = useState(false);
  const set = <K extends keyof typeof f>(k: K, v: (typeof f)[K]) => setF(p => ({ ...p, [k]: v }));

  const save = async () => {
    if (busy) return;
    const p: Record<string, string> = {};
    const code = f.code.trim();
    if (!crop && !/^[a-z_]{2,24}$/.test(code)) p.code = 'crops.code_bad';
    else if (!crop && code === 'empty') p.code = 'err.reserved_code';
    else if (!crop && taken.includes(code)) p.code = 'crops.code_taken';
    if (!f.name_en.trim()) p.name_en = 'v.needed';
    if (!/^#[0-9a-fA-F]{6}$/.test(f.color)) p.color = 'crops.color_bad';
    const y = f.yield.trim();
    if (y && !(Number(y) > 0)) p.yield = 'crops.yield_bad';
    if (!/^-?\d+$/.test(f.sort.trim())) p.sort = 'crops.sort_bad';
    setProblems(p); setErr('');
    if (Object.keys(p).length) return;
    setBusy(true);
    const body = cropBody({
      code: crop?.code ?? code, name_en: f.name_en.trim(), name_ku: f.name_ku.trim() || null, color: f.color.toUpperCase(), category: f.category, season: f.season,
      yield_kg_per_dunam: y ? Number(y) : null, active: f.active, sort_order: Number(f.sort),
    });
    try {
      if (crop) {
        // gone meanwhile: say so instead of a bare 404
        if (!await fresh(crop.code)) { setErr(t('crops.gone')); return; }
        await api.put('/dashboard/crops/' + encodeURIComponent(crop.code), body);
      } else {
        try { await api.post('/dashboard/crops', body); }
        catch (e) {
          // a retry of a create that went through: the crop is there with exactly what we sent
          if (!(e instanceof ApiError && e.code === 'already_exists')) throw e;
          const there = await fresh(code);
          if (!there || there.name_en !== body.name_en || there.color !== body.color) throw e;
        }
      }
      invalidate('crops');
      toast(t('common.saved'), 'good');
      onClose();
    } catch (e) {
      const x = e as ApiError;
      setErr(x.code === 'already_exists' ? t('crops.code_taken') : errText(x));
      if (x.field) setProblems(pp => ({ ...pp, [x.field!]: 'v.needed' }));
    } finally { setBusy(false); }
  };

  const tx = (k?: string) => (k ? t(k) : undefined);
  return (
    <Modal title={crop ? t('crops.edit_title', { name: crop.name_en }) : t('crops.add')} onClose={onClose} wide foot={<>
      <button className="btn" onClick={onClose} disabled={busy}>{t('common.cancel')}</button>
      <button className="btn primary" onClick={save} disabled={busy}>{busy ? t('common.loading') : t('common.save')}</button>
    </>}>
      <div className="form-grid">
        <Field label={t('crops.f_code')} hint={crop ? t('crops.code_fixed') : t('crops.code_hint')} error={tx(problems.code)}>
          <input type="text" className="ltr-input" value={f.code} onChange={e => set('code', e.target.value.toLowerCase())} disabled={!!crop} maxLength={24} placeholder="sesame" dir="ltr" />
        </Field>
        <Field label={t('crops.f_color')} error={tx(problems.color)}>
          <span className="row" style={{ gap: 8 }}><input type="color" value={/^#[0-9a-fA-F]{6}$/.test(f.color) ? f.color : '#000000'} onChange={e => set('color', e.target.value)} style={{ width: 44, padding: 2 }} aria-label={t('crops.f_color')} />
            <input type="text" value={f.color} onChange={e => set('color', e.target.value)} maxLength={7} dir="ltr" style={{ width: 110 }} /></span>
        </Field>
        <Field label={t('crops.f_name_en')} error={tx(problems.name_en)}><input type="text" value={f.name_en} onChange={e => set('name_en', e.target.value)} maxLength={60} dir="ltr" /></Field>
        <Field label={t('crops.f_name_ku')}><input type="text" value={f.name_ku} onChange={e => set('name_ku', e.target.value)} maxLength={60} dir="rtl" /></Field>
        <Field label={t('crops.f_category')}><Select value={f.category} onChange={v => set('category', v as Category)} options={CATEGORIES.map(c => [c, t('crops.cat_' + c)] as [string, string])} /></Field>
        <Field label={t('crops.f_season')}><Select value={f.season} onChange={v => set('season', v as Season)} options={SEASONS.map(c => [c, t('crops.season_' + c)] as [string, string])} /></Field>
        <Field label={t('crops.f_yield')} hint={t('crops.yield_hint')} error={tx(problems.yield)}><input type="number" min={0} inputMode="decimal" value={f.yield} onChange={e => set('yield', e.target.value)} /></Field>
        <Field label={t('crops.f_sort')} hint={t('crops.sort_hint')} error={tx(problems.sort)}><input type="number" inputMode="numeric" value={f.sort} onChange={e => set('sort', e.target.value)} /></Field>
        <div className="full set">
          <div className="txt"><b>{t('crops.f_active')}</b><small>{t('crops.active_hint')}</small></div>
          <div className="ctl"><Switch on={f.active} onChange={v => set('active', v)} label={t('crops.f_active')} /></div>
        </div>
      </div>
      {err && <div style={{ marginTop: 10 }}><Note tone="danger">{err}</Note></div>}
    </Modal>
  );
}
