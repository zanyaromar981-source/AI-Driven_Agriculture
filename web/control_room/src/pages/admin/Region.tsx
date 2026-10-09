// Region data: dryness per district, the two dams, fires and earlier years. Numbers arrive from the data
// jobs; an admin can correct one by hand, and it stays marked "edited by hand" until the next job run.
import { useMemo, useState } from 'react';
import { Pencil, RotateCcw, CheckCheck, Info, Droplets, Flame, Undo2 } from 'lucide-react';
import { db, PLACES } from '../../data/db';
import { useRows } from '../../data/store';
import { downloadCsv } from '../../data/api';
import type { Dam, DistrictReading, Fire } from '../../data/types';
import { useI18n } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { PageHead, Card, Kpi, Pill, Note, Switch, Tabs, Field, Select, Modal, useToast, type Tone } from '../../components/ui';
import { DataTable, type Col } from '../../components/DataTable';
import { PairBars, VBars } from '../../components/charts';
import { DistrictMap, DRY_RAMP, ramp } from '../../components/DistrictMap';
import { usePlaceNames } from '../../components/domain';

type Tab = 'districts' | 'dams' | 'fires' | 'compare';

/** Dryness band from the two rule values (Rules page). */
function useBands() {
  const rules = useRows(db.rules);
  return useMemo(() => {
    const dry = rules.find(r => r.id === 'b_dry')?.value ?? 60, vdry = rules.find(r => r.id === 'b_vdry')?.value ?? 80;
    return { dry, vdry, band: (v: number): ['normal' | 'dry' | 'vdry', Tone] => (v >= vdry ? ['vdry', 'danger'] : v >= dry ? ['dry', 'warn'] : ['normal', 'good']) };
  }, [rules]);
}

export default function Region() {
  const { t } = useI18n();
  const [tab, setTab] = useState<Tab>('districts');
  return (
    <>
      <PageHead eyebrow={t('nav.g_fields')} title={t('region.title')} sub={t('region.sub')} />
      <div className="mb"><Note tone="info" icon={<Info />}>{t('region.how')}</Note></div>
      <Tabs value={tab} onChange={setTab} items={[['districts', t('region.tab_districts')], ['dams', t('region.tab_dams')], ['fires', t('region.tab_fires')], ['compare', t('region.tab_compare')]]} />
      {tab === 'districts' && <Districts />}
      {tab === 'dams' && <Dams />}
      {tab === 'fires' && <Fires />}
      {tab === 'compare' && <Compare />}
    </>
  );
}

function ManualPill({ manual }: { manual: boolean }) {
  const { t } = useI18n();
  return manual ? <Pill tone="gold" icon={<Pencil />}>{t('common.edited_by_hand')}</Pill> : <Pill>{t('common.from_job')}</Pill>;
}

function Districts() {
  const { t, num, ago } = useI18n();
  const rows = useRows(db.readings);
  const place = usePlaceNames();
  const { band, dry, vdry } = useBands();
  const [edit, setEdit] = useState<DistrictReading | null>(null);
  const [focus, setFocus] = useState<string | null>(null);
  const byId = useMemo(() => new Map(rows.map(r => [r.id, r])), [rows]);
  const govOf = useMemo(() => new Map(PLACES.districts.map(d => [d.en, d.gov])), []);
  const stats = useMemo(() => {
    let s = 0, manual = 0, vd = 0, d = 0;
    for (const r of rows) { s += r.dryness; if (r.manual) manual++; if (r.dryness >= vdry) vd++; else if (r.dryness >= dry) d++; }
    return { avg: rows.length ? s / rows.length : 0, manual, vd, d };
  }, [rows, dry, vdry]);
  const sel = focus ? byId.get(focus) : undefined;

  const cols: Col<DistrictReading>[] = [
    { key: 'dist', label: t('common.district'), cell: r => <b>{place.dist(r.id)}</b>, sort: r => place.dist(r.id) },
    { key: 'gov', label: t('common.governorate'), cell: r => place.gov(govOf.get(r.id) ?? ''), sort: r => govOf.get(r.id) ?? '' },
    { key: 'dry', label: t('region.dryness'), num: true, cell: r => { const [k, tone] = band(r.dryness); return <span className="row" style={{ justifyContent: 'flex-end', flexWrap: 'nowrap' }}><b>{num(r.dryness)}</b><Pill tone={tone}>{t('region.band_' + k)}</Pill></span>; }, sort: r => r.dryness },
    { key: 'green', label: t('region.greenness'), num: true, cell: r => num(r.greenness) + '%', sort: r => r.greenness },
    { key: 'rain', label: t('region.rain'), num: true, cell: r => num(r.rain), sort: r => r.rain },
    { key: 'ly', label: t('region.vs_last_year'), num: true, cell: r => { const d = r.dryness - r.lastYear; return <span style={{ color: d > 0 ? 'var(--danger)' : 'var(--good)' }}>{d > 0 ? '+' : ''}{num(d)}</span>; }, sort: r => r.dryness - r.lastYear },
    { key: 'upd', label: t('common.updated'), cell: r => <span className="small nowrap">{ago(r.updated)}</span>, sort: r => r.updated },
    { key: 'src', label: t('common.source'), optional: true, cell: r => <span className="small muted"><bdi>{r.source}</bdi></span> },
    { key: 'man', label: t('region.value_from'), cell: r => <ManualPill manual={r.manual} />, sort: r => (r.manual ? 0 : 1) },
    { key: 'pub', label: t('region.public'), cell: r => <span onClick={e => e.stopPropagation()}><Switch on={r.public} onChange={v => db.readings.patch(r.id, { public: v })} label={t('region.public')} /></span>, sort: r => (r.public ? 0 : 1) },
  ];
  const exportCsv = () => downloadCsv('district_readings', ['district', 'governorate', 'dryness', 'greenness_pct', 'rain_mm', 'last_year', 'updated', 'source', 'edited_by_hand', 'public'],
    rows.map(r => [r.id, govOf.get(r.id) ?? '', r.dryness, r.greenness, r.rain, r.lastYear, r.updated, r.source, r.manual ? 'yes' : 'no', r.public ? 'yes' : 'no']));

  return (
    <>
      <div className="grid g4 mb">
        <Kpi label={t('region.k_avg')} value={num(stats.avg)} note={t('region.k_avg_n')} />
        <Kpi label={t('region.band_vdry')} value={num(stats.vd)} note={t('region.k_from', { n: num(vdry) })} tone="danger" />
        <Kpi label={t('region.band_dry')} value={num(stats.d)} note={t('region.k_from', { n: num(dry) })} tone="warn" />
        <Kpi label={t('common.edited_by_hand')} value={num(stats.manual)} note={t('region.k_manual_n')} tone="gold" />
      </div>
      <div className="grid g-main mb">
        <Card title={t('region.map_title')}>
          <DistrictMap styleKey={rows.map(r => r.dryness).join(',')} focus={focus} onBack={() => setFocus(null)} onDistrict={setFocus}
            fill={d => { const r = byId.get(d); return r ? ramp(r.dryness, [30, 45, 60, 80], DRY_RAMP) : undefined; }} />
          <div className="legend">{DRY_RAMP.map((c, i) => <span key={c}><i className="dotc" style={{ background: c }} />{['< 30', '30 to 45', '45 to 60', '60 to 80', '80+'][i]}</span>)}</div>
        </Card>
        <Card title={sel ? place.dist(sel.id) : t('region.pick_district')}>
          {sel ? (
            <>
              <dl className="facts">
                <dt>{t('region.dryness')}</dt><dd>{num(sel.dryness)} · {t('region.band_' + band(sel.dryness)[0])}</dd>
                <dt>{t('region.greenness')}</dt><dd>{num(sel.greenness)}%</dd>
                <dt>{t('region.rain')}</dt><dd>{num(sel.rain)}</dd>
                <dt>{t('region.last_year')}</dt><dd>{num(sel.lastYear)}</dd>
                <dt>{t('common.updated')}</dt><dd>{ago(sel.updated)}</dd>
                <dt>{t('common.source')}</dt><dd><bdi>{sel.source}</bdi></dd>
                <dt>{t('region.value_from')}</dt><dd><ManualPill manual={sel.manual} /></dd>
              </dl>
              <div className="row mt"><button className="btn sm" onClick={() => setEdit(sel)}><Pencil />{t('region.edit')}</button>
                {sel.manual && <UndoManual onUndo={() => db.readings.patch(sel.id, { manual: false })} />}</div>
            </>
          ) : <p className="muted">{t('region.pick_hint')}</p>}
        </Card>
      </div>
      <Card>
        <DataTable id="readings" rows={rows} cols={cols} onRow={r => setEdit(r)} defaultSort={['dry', -1]} per={40}
          head={<button className="btn sm" onClick={exportCsv}>{t('common.export_csv')}</button>} />
      </Card>
      {edit && <EditReading r={edit} onClose={() => setEdit(null)} />}
    </>
  );
}

function UndoManual({ onUndo }: { onUndo: () => void }) {
  const { t } = useI18n();
  const toast = useToast();
  return <button className="btn sm" title={t('region.undo_hint')} onClick={() => { onUndo(); toast(t('region.undone'), 'good'); }}><RotateCcw />{t('region.undo')}</button>;
}

function useReason() {
  const [why, setWhy] = useState('');
  const [bad, setBad] = useState(false);
  return { why, setWhy, bad, ok: () => { const ok = why.trim().length >= 3; setBad(!ok); return ok; } };
}

function EditReading({ r, onClose }: { r: DistrictReading; onClose: () => void }) {
  const { t } = useI18n();
  const { me } = useAuth();
  const toast = useToast();
  const place = usePlaceNames();
  const [v, setV] = useState({ dryness: r.dryness, greenness: r.greenness, rain: r.rain });
  const reason = useReason();
  const [err, setErr] = useState('');
  const save = () => {
    if (!reason.ok()) return;
    if (v.dryness < 0 || v.dryness > 100 || v.greenness < 0 || v.greenness > 300 || v.rain < 0) { setErr('region.v_range'); return; }
    db.readings.patch(r.id, { ...v, manual: true, updated: new Date().toISOString(), source: `${me?.name ?? ''}: ${reason.why.trim()}` });
    toast(t('common.saved'), 'good'); onClose();
  };
  return (
    <Modal title={t('region.edit_title', { d: place.dist(r.id) })} onClose={onClose} foot={<>
      {r.manual && <span style={{ marginInlineEnd: 'auto' }}><UndoManual onUndo={() => { db.readings.patch(r.id, { manual: false }); onClose(); }} /></span>}
      <button className="btn" onClick={onClose}>{t('common.cancel')}</button><button className="btn primary" onClick={save}>{t('common.save')}</button></>}>
      <p className="muted" style={{ marginTop: 0 }}>{t('region.edit_sub')}</p>
      <div className="form-grid">
        <Field label={t('region.dryness')} hint="0 to 100"><input type="number" min={0} max={100} value={v.dryness} onChange={e => setV({ ...v, dryness: +e.target.value })} /></Field>
        <Field label={t('region.greenness')} hint="%"><input type="number" min={0} value={v.greenness} onChange={e => setV({ ...v, greenness: +e.target.value })} /></Field>
        <Field label={t('region.rain')}><input type="number" min={0} value={v.rain} onChange={e => setV({ ...v, rain: +e.target.value })} /></Field>
        <Field label={t('region.reason')} hint={t('common.required')} error={reason.bad ? 'region.v_reason' : undefined} full>
          <textarea rows={2} value={reason.why} onChange={e => reason.setWhy(e.target.value)} placeholder={t('region.reason_ph')} />
        </Field>
      </div>
      {err && <div className="mt"><Note tone="danger">{t(err)}</Note></div>}
    </Modal>
  );
}

function Dams() {
  const { t, num, ago, b, lang } = useI18n();
  const dams = useRows(db.dams);
  const monthFmt = useMemo(() => new Intl.DateTimeFormat(lang === 'ku' ? 'ckb-IQ' : 'en-GB', { month: 'short' }), [lang]);
  const [edit, setEdit] = useState<Dam | null>(null);
  return (
    <>
      <div className="grid g2">
        {dams.map(d => (
          <Card key={d.id}>
            <div className="spread"><h2 className="row"><Droplets style={{ color: 'var(--water)' }} />{b(d.name)}</h2><ManualPill manual={d.manual} /></div>
            <div className="tabular" style={{ font: '800 38px var(--display)', color: 'var(--water)', margin: '8px 0 6px' }}>{num(d.pct)}%</div>
            <div className="bar" style={{ height: 12 }}><i style={{ width: Math.min(100, d.pct) + '%', background: 'var(--water)' }} /></div>
            <dl className="facts mt">
              <dt>{t('region.volume')}</dt><dd><bdi>{t('region.volume_v', { v: num(d.volume, 2), c: num(d.capacity, 2) })}</bdi></dd>
              <dt>{t('region.year_ago')}</dt><dd><bdi>{num(d.yearAgo)}%</bdi> <span dir="ltr" style={{ color: d.pct >= d.yearAgo ? 'var(--good)' : 'var(--danger)' }}>({d.pct >= d.yearAgo ? '+' : ''}{num(d.pct - d.yearAgo)})</span></dd>
              <dt>{t('common.updated')}</dt><dd>{ago(d.updated)}</dd>
              <dt>{t('common.source')}</dt><dd><bdi>{d.source}</bdi></dd>
            </dl>
            <div className="eyebrow mt">{t('region.months')}</div>
            <VBars height={110} unit="%" max={100} items={d.history.map(h => ({ key: h.month, label: monthFmt.format(new Date(h.month + '-15')), value: h.pct, color: 'var(--water)' }))} />
            <div className="row mt"><button className="btn sm" onClick={() => setEdit(d)}><Pencil />{t('region.edit_dam')}</button>
              {d.manual && <UndoManual onUndo={() => db.dams.patch(d.id, { manual: false })} />}</div>
          </Card>
        ))}
      </div>
      {edit && <EditDam d={edit} onClose={() => setEdit(null)} />}
    </>
  );
}

function EditDam({ d, onClose }: { d: Dam; onClose: () => void }) {
  const { t, b } = useI18n();
  const { me } = useAuth();
  const toast = useToast();
  const [pct, setPct] = useState(d.pct), [vol, setVol] = useState(d.volume);
  const reason = useReason();
  const [err, setErr] = useState('');
  const save = () => {
    if (!reason.ok()) return;
    if (pct < 0 || pct > 110 || vol < 0 || vol > d.capacity * 1.1) { setErr('region.v_range'); return; }
    const month = new Date().toISOString().slice(0, 7);
    const history = d.history.some(h => h.month === month) ? d.history.map(h => (h.month === month ? { month, pct } : h)) : [...d.history.slice(1), { month, pct }];
    db.dams.patch(d.id, { pct, volume: vol, history, manual: true, updated: new Date().toISOString(), source: `${me?.name ?? ''}: ${reason.why.trim()}` });
    toast(t('common.saved'), 'good'); onClose();
  };
  return (
    <Modal title={t('region.edit_title', { d: b(d.name) })} onClose={onClose} foot={<><button className="btn" onClick={onClose}>{t('common.cancel')}</button><button className="btn primary" onClick={save}>{t('common.save')}</button></>}>
      <p className="muted" style={{ marginTop: 0 }}>{t('region.edit_sub')}</p>
      <div className="form-grid">
        <Field label={t('region.pct_full')} hint="%"><input type="number" min={0} max={110} value={pct} onChange={e => setPct(+e.target.value)} /></Field>
        <Field label={t('region.volume')} hint={t('region.bn_m3')}><input type="number" step="0.01" min={0} value={vol} onChange={e => setVol(+e.target.value)} /></Field>
        <Field label={t('region.reason')} hint={t('common.required')} error={reason.bad ? 'region.v_reason' : undefined} full>
          <textarea rows={2} value={reason.why} onChange={e => reason.setWhy(e.target.value)} placeholder={t('region.reason_ph')} />
        </Field>
      </div>
      {err && <div className="mt"><Note tone="danger">{t(err)}</Note></div>}
    </Modal>
  );
}

const CONF_TONE: Record<Fire['confidence'], Tone> = { low: '', nominal: 'warn', high: 'danger' };
const CONF_COLOR: Record<Fire['confidence'], string> = { low: '#E3A35A', nominal: '#D9682F', high: '#B23A2E' };

function Fires() {
  const { t, num, date } = useI18n();
  const fires = useRows(db.fires);
  const place = usePlaceNames();
  const toast = useToast();
  const open = fires.filter(f => !f.checked).length, near = fires.reduce((a, f) => a + (f.nearFarms > 0 && !f.checked ? 1 : 0), 0);
  const points = useMemo(() => fires.map(f => ({ id: f.id, lat: f.lat, lon: f.lon, color: f.checked ? '#8A978F' : CONF_COLOR[f.confidence], radius: f.checked ? 5 : 8, label: place.dist(f.dist) })), [fires, place]);
  const toggle = (f: Fire) => { db.fires.patch(f.id, { checked: !f.checked }); toast(t(f.checked ? 'region.unchecked' : 'region.checked_ok'), 'good'); };
  const cols: Col<Fire>[] = [
    { key: 'at', label: t('region.seen'), cell: f => date(f.at, 'datetime'), sort: f => f.at },
    { key: 'dist', label: t('common.district'), cell: f => <b>{place.dist(f.dist)}</b>, sort: f => f.dist },
    { key: 'pt', label: t('region.point'), cell: f => <span className="mono ltr">{f.lat.toFixed(2)}, {f.lon.toFixed(2)}</span> },
    { key: 'conf', label: t('region.confidence'), cell: f => <Pill tone={CONF_TONE[f.confidence]}>{t('region.conf_' + f.confidence)}</Pill>, sort: f => f.confidence },
    { key: 'sat', label: t('region.satellite'), cell: f => f.satellite },
    { key: 'near', label: t('region.near_farms'), num: true, cell: f => (f.nearFarms ? <b style={{ color: 'var(--danger)' }}>{num(f.nearFarms)}</b> : num(0)), sort: f => f.nearFarms },
    { key: 'chk', label: t('common.status'), cell: f => <button className="btn sm" onClick={e => { e.stopPropagation(); toggle(f); }}>{f.checked ? <><Undo2 />{t('region.checked')}</> : <><CheckCheck />{t('region.mark_checked')}</>}</button>, sort: f => (f.checked ? 1 : 0) },
  ];
  return (
    <>
      <div className="grid g3 mb">
        <Kpi label={t('region.k_fires')} value={num(fires.length)} note={t('region.k_fires_n')} icon={<Flame />} />
        <Kpi label={t('region.k_open')} value={num(open)} note={t('region.k_open_n')} tone={open ? 'warn' : 'good'} />
        <Kpi label={t('region.k_near')} value={num(near)} note={t('region.k_near_n')} tone={near ? 'danger' : 'good'} />
      </div>
      <div className="grid g-main">
        <Card title={t('region.fire_map')}><DistrictMap size="sm" styleKey="fires" fill={() => '#EEF0E8'} points={points} /></Card>
        <Card><Note tone="info" icon={<Info />}>{t('region.fires_how')}</Note></Card>
      </div>
      <Card className="mt"><DataTable id="fires" rows={fires} cols={cols} defaultSort={['at', -1]} /></Card>
    </>
  );
}

function Compare() {
  const { t, num } = useI18n();
  const rows = useRows(db.readings);
  const place = usePlaceNames();
  const years = useMemo(() => Object.keys(rows[0]?.byYear ?? {}).sort(), [rows]);
  const [a, setA] = useState(() => years[years.length - 2] ?? '');
  const [b, setB] = useState(() => years[years.length - 1] ?? '');
  const [gov, setGov] = useState('');
  const govOf = useMemo(() => new Map(PLACES.districts.map(d => [d.en, d.gov])), []);
  const list = useMemo(() => rows.filter(r => !gov || govOf.get(r.id) === gov).map(r => ({ key: r.id, label: place.dist(r.id), a: r.byYear[a] ?? 0, b: r.byYear[b] ?? 0 }))
    .sort((x, y) => (y.b - y.a) - (x.b - x.a)), [rows, gov, a, b, govOf, place]);
  const avg = (k: 'a' | 'b') => (list.length ? list.reduce((s, r) => s + r[k], 0) / list.length : 0);
  const yearAvg = useMemo(() => years.map(y => ({ key: y, label: y, value: rows.length ? rows.reduce((s, r) => s + (r.byYear[y] ?? 0), 0) / rows.length : 0 })), [rows, years]);
  const yo = years.map(y => [y, y] as [string, string]);
  return (
    <>
      <div className="filters">
        <Select value={a} onChange={setA} options={yo} aria-label={t('region.year_a')} />
        <span className="muted">{t('region.against')}</span>
        <Select value={b} onChange={setB} options={yo} aria-label={t('region.year_b')} />
        <Select value={gov} onChange={setGov} options={[['', t('common.all_govs')], ...PLACES.governorates.map(g => [g.en, place.gov(g.en)] as [string, string])]} />
      </div>
      <div className="grid g3 mb">
        <Kpi label={t('region.avg_in', { y: a })} value={num(avg('a'))} />
        <Kpi label={t('region.avg_in', { y: b })} value={num(avg('b'))} />
        <Kpi label={t('region.change')} value={(avg('b') - avg('a') > 0 ? '+' : '') + num(avg('b') - avg('a'))} tone={avg('b') > avg('a') ? 'danger' : 'good'} note={t('region.change_n')} />
      </div>
      <div className="grid g-main">
        <Card title={t('region.compare_title')}><PairBars rows={list} a={a} b={b} max={100} /></Card>
        <Card title={t('region.year_avgs')}>
          <VBars height={140} items={yearAvg.map(y => ({ ...y, color: y.key === a ? 'var(--ink-3)' : y.key === b ? 'var(--brand)' : '#BFDCC8' }))} max={100} />
          <p className="muted small">{t('region.compare_note')}</p>
        </Card>
      </div>
    </>
  );
}
