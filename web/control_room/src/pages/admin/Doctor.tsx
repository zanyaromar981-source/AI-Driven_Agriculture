// The Doctor (AI): review its answers, keep a bank of approved answers written by people, see where the
// same problem is asked about many times (an early warning), and read the rules it can never break.
// Cost: the problems map counts the last 14 days in one pass over the questions.
import { useMemo, useState } from 'react';
import { ThumbsUp, ThumbsDown, Plus, Pencil, Trash2, Lock, ShieldCheck, Ban, Calculator, HelpCircle, BookOpenCheck } from 'lucide-react';
import { useI18n } from '../../i18n';
import { useAuth } from '../../auth/auth';
import { db } from '../../data/db';
import { useRows } from '../../data/store';
import { newId, nowIso } from '../../data/api';
import type { AnswerBankItem, DoctorQuestion } from '../../data/types';
import { Card, Confirm, Drawer, Field, Kpi, Modal, Note, PageHead, Pill, Select, Switch, Tabs, useToast, type Tone } from '../../components/ui';
import { DataTable, type Col } from '../../components/DataTable';
import { CropTag, useCrops, usePlaceNames } from '../../components/domain';
import { DistrictMap, ramp } from '../../components/DistrictMap';

const TOPICS = ['yellow_leaves', 'insects', 'stem_rot', 'leaf_curl', 'sowing_time', 'irrigation', 'weeds', 'frost'];
const CONF_TONE: Record<DoctorQuestion['confidence'], Tone> = { sure: 'good', likely: 'warn', unsure: 'danger' };
const HEAT = ['#F3EFD9', '#F2D49B', '#E8A85F', '#D2703F', '#A63A26'];
type TabK = 'questions' | 'bank' | 'map' | 'rules';

export default function Doctor() {
  const { t, num } = useI18n();
  const questions = useRows(db.questions);
  const [tab, setTab] = useState<TabK>('questions');
  const stats = useMemo(() => {
    let unrated = 0, bad = 0, unsure = 0, good = 0;
    for (const q of questions) { if (!q.rating) unrated++; else if (q.rating === 'bad') bad++; else good++; if (q.confidence === 'unsure') unsure++; }
    return { unrated, bad, unsure, good };
  }, [questions]);
  return (
    <>
      <PageHead eyebrow={t('nav.g_act')} title={t('nav.doctor')} sub={t('doctor.sub')} />
      <div className="grid g4 mb">
        <Kpi label={t('doctor.k_questions')} value={num(questions.length)} note={t('doctor.k_questions_n')} icon={<HelpCircle />} />
        <Kpi label={t('doctor.k_unrated')} value={num(stats.unrated)} note={t('doctor.k_unrated_n')} tone={stats.unrated ? 'warn' : ''} />
        <Kpi label={t('doctor.k_unsure')} value={num(stats.unsure)} note={t('doctor.k_unsure_n')} tone={stats.unsure ? 'danger' : ''} />
        <Kpi label={t('doctor.k_good')} value={stats.good + stats.bad ? num(stats.good / (stats.good + stats.bad) * 100) + '%' : '-'} note={t('doctor.k_good_n', { n: num(stats.good + stats.bad) })} tone="good" />
      </div>
      <Tabs value={tab} onChange={setTab} items={[['questions', t('doctor.tab_questions')], ['bank', t('doctor.tab_bank')], ['map', t('doctor.tab_map')], ['rules', t('doctor.tab_rules')]]} />
      {tab === 'questions' && <Questions rows={questions} />}
      {tab === 'bank' && <Bank />}
      {tab === 'map' && <Problems rows={questions} />}
      {tab === 'rules' && <HardRules />}
    </>
  );
}

function Questions({ rows }: { rows: DoctorQuestion[] }) {
  const { t, date, b } = useI18n();
  const { me } = useAuth();
  const toast = useToast();
  const [filter, setFilter] = useState<'all' | 'unrated' | 'bad' | 'unsure'>('all');
  const [open, setOpen] = useState<DoctorQuestion | null>(null);
  const [fix, setFix] = useState('');
  const list = useMemo(() => rows.filter(q => filter === 'all' || (filter === 'unrated' ? !q.rating : filter === 'bad' ? q.rating === 'bad' : q.confidence === 'unsure')), [rows, filter]);
  const cols: Col<DoctorQuestion>[] = [
    { key: 'at', label: t('doctor.c_date'), cell: q => date(q.at, 'short'), sort: q => q.at },
    { key: 'crop', label: t('doctor.c_crop'), cell: q => <CropTag id={q.crop} />, sort: q => q.crop },
    { key: 'topic', label: t('doctor.c_topic'), cell: q => t('doctor.topic_' + q.topic), sort: q => q.topic },
    { key: 'question', label: t('doctor.c_question'), cell: q => <bdi>{q.question}</bdi> },
    { key: 'answer', label: t('doctor.c_answer'), cell: q => <span className="small ink2"><bdi>{q.answer}</bdi></span>, optional: true },
    { key: 'conf', label: t('doctor.c_conf'), cell: q => <Pill tone={CONF_TONE[q.confidence]}>{t('doctor.conf_' + q.confidence)}</Pill>, sort: q => q.confidence },
    { key: 'rating', label: t('doctor.c_rating'), cell: q => q.rating ? <Pill tone={q.rating === 'good' ? 'good' : 'danger'}>{t('doctor.r_' + q.rating)}</Pill> : <span className="muted small">{t('doctor.r_none')}</span>, sort: q => q.rating ?? '' },
  ];
  const rate = (r: 'good' | 'bad') => {
    if (!open) return;
    db.questions.patch(open.id, { rating: r, correction: fix.trim(), ratedBy: me?.name ?? '' });
    toast(t('doctor.rated'), 'good'); setOpen(null);
  };
  const farmer = open ? db.farmers.get(open.farmerId) : undefined;
  return (
    <Card>
      <div className="chips mb">
        {(['all', 'unrated', 'bad', 'unsure'] as const).map(k => <button key={k} className={'chip' + (filter === k ? ' on' : '')} onClick={() => setFilter(k)}>{t('doctor.f_' + k)}</button>)}
      </div>
      <DataTable id="doctor-q" rows={list} cols={cols} onRow={q => { setOpen(q); setFix(q.correction); }} defaultSort={['at', -1]} selected={open?.id} />
      {open && (
        <Drawer title={<bdi>{open.question}</bdi>} eyebrow={t('doctor.topic_' + open.topic)} onClose={() => setOpen(null)}>
          <div className="row mb"><CropTag id={open.crop} /><Pill tone={CONF_TONE[open.confidence]}>{t('doctor.conf_' + open.confidence)}</Pill><span className="muted small">{date(open.at, 'datetime')}</span></div>
          <div className="eyebrow">{t('doctor.the_answer')}</div>
          <div className="note info" style={{ margin: '6px 0 14px' }}><span dir="auto">{open.answer}</span></div>
          {farmer && <p className="small muted">{t('doctor.asked_by', { name: b(farmer.name) })}{open.farmId ? ' · ' + t('doctor.farm') + ' #' + open.farmId : ''}</p>}
          <Field label={t('doctor.correction')} hint={t('doctor.correction_hint')}>
            <textarea rows={4} dir="auto" value={fix} onChange={e => setFix(e.target.value)} />
          </Field>
          {open.rating && <p className="small muted">{t('doctor.rated_by', { by: open.ratedBy, r: t('doctor.r_' + open.rating) })}</p>}
          <div className="foot">
            <button className="btn danger" onClick={() => rate('bad')}><ThumbsDown />{t('doctor.rate_bad')}</button>
            <button className="btn primary" onClick={() => rate('good')}><ThumbsUp />{t('doctor.rate_good')}</button>
          </div>
          <p className="muted small">{t('doctor.rate_note')}</p>
        </Drawer>
      )}
    </Card>
  );
}

const blankAnswer = (): AnswerBankItem => ({ id: '', topic: TOPICS[0], crop: '', question: { en: '', ku: '' }, answer: { en: '', ku: '' }, approved: false, by: '', updated: '' });

function Bank() {
  const { t, b, ago } = useI18n();
  const { me } = useAuth();
  const toast = useToast();
  const crops = useCrops();
  const items = useRows(db.answers);
  const [edit, setEdit] = useState<AnswerBankItem | null>(null);
  const [del, setDel] = useState<AnswerBankItem | null>(null);
  const [err, setErr] = useState('');
  const sorted = useMemo(() => [...items].sort((a, z) => a.topic.localeCompare(z.topic)), [items]);
  const save = () => {
    if (!edit) return;
    if ((!edit.question.ku.trim() && !edit.question.en.trim()) || (!edit.answer.ku.trim() && !edit.answer.en.trim())) { setErr('doctor.bank_need'); return; }
    db.answers.put({ ...edit, id: edit.id || newId('b'), by: me?.name ?? '', updated: nowIso() });
    setEdit(null); setErr(''); toast(t('common.saved'), 'good');
  };
  const E = edit;
  return (
    <div className="stack">
      <Note tone="brand" icon={<BookOpenCheck />}>{t('doctor.bank_note')}</Note>
      <Card title={t('doctor.bank_title', { n: items.length })} extra={<button className="btn primary sm" onClick={() => setEdit(blankAnswer())}><Plus />{t('doctor.bank_add')}</button>}>
        {sorted.map(a => (
          <div key={a.id} className="list-item" style={{ alignItems: 'flex-start' }}>
            <span className={'ico ' + (a.approved ? 'good' : 'warn')}><BookOpenCheck /></span>
            <div style={{ flex: 1, minWidth: 0 }}>
              <div className="row"><Pill>{t('doctor.topic_' + a.topic)}</Pill>{a.crop ? <CropTag id={a.crop} /> : <span className="muted small">{t('doctor.any_crop')}</span>}
                <Pill tone={a.approved ? 'good' : 'warn'}>{t(a.approved ? 'doctor.approved' : 'doctor.not_approved')}</Pill></div>
              <b style={{ display: 'block', marginTop: 4 }}><bdi>{b(a.question)}</bdi></b>
              <div className="small ink2" dir="auto">{b(a.answer)}</div>
              <div className="muted tiny">{a.by} · {ago(a.updated)}{!a.answer.ku && ' · ' + t('common.ku_missing')}</div>
            </div>
            <div className="row" style={{ flexWrap: 'nowrap', flex: 'none' }}>
              <Switch on={a.approved} onChange={v => db.answers.patch(a.id, { approved: v, updated: nowIso(), by: me?.name ?? a.by })} label={t('doctor.approved')} />
              <button className="btn sm icon" onClick={() => setEdit({ ...a })} aria-label={t('common.edit')}><Pencil /></button>
              <button className="btn sm icon danger" onClick={() => setDel(a)} aria-label={t('common.delete')}><Trash2 /></button>
            </div>
          </div>
        ))}
        {!items.length && <div className="empty">{t('doctor.bank_empty')}</div>}
      </Card>
      {E && (
        <Modal wide title={t(E.id ? 'doctor.bank_edit' : 'doctor.bank_add')} onClose={() => { setEdit(null); setErr(''); }}
          foot={<><button className="btn" onClick={() => { setEdit(null); setErr(''); }}>{t('common.cancel')}</button><button className="btn primary" onClick={save}>{t('common.save')}</button></>}>
          <div className="form-grid">
            <Field label={t('doctor.c_topic')}><Select value={E.topic} onChange={v => setEdit({ ...E, topic: v })} options={TOPICS.map(k => [k, t('doctor.topic_' + k)] as [string, string])} /></Field>
            <Field label={t('doctor.c_crop')}><Select value={E.crop} onChange={v => setEdit({ ...E, crop: v })} options={[['', t('doctor.any_crop')], ...crops.list.map(c => [c.id, b(c.name)] as [string, string])]} /></Field>
            <Field label={t('doctor.q_ku')} full><input type="text" dir="rtl" lang="ckb" className="ku-text" value={E.question.ku} onChange={e => setEdit({ ...E, question: { ...E.question, ku: e.target.value } })} /></Field>
            <Field label={t('doctor.a_ku')} full><textarea rows={3} dir="rtl" lang="ckb" className="ku-text" value={E.answer.ku} onChange={e => setEdit({ ...E, answer: { ...E.answer, ku: e.target.value } })} /></Field>
            <Field label={t('doctor.q_en')} full><input type="text" dir="ltr" value={E.question.en} onChange={e => setEdit({ ...E, question: { ...E.question, en: e.target.value } })} /></Field>
            <Field label={t('doctor.a_en')} full><textarea rows={3} dir="ltr" value={E.answer.en} onChange={e => setEdit({ ...E, answer: { ...E.answer, en: e.target.value } })} /></Field>
            <Field label={t('doctor.approved')}><Switch on={E.approved} onChange={v => setEdit({ ...E, approved: v })} /></Field>
          </div>
          {err && <div className="mt"><Note tone="danger">{t(err)}</Note></div>}
        </Modal>
      )}
      {del && <Confirm title={t('doctor.bank_delete')} text={<bdi>{b(del.question)}</bdi>} okLabel={t('common.delete')} onClose={() => setDel(null)} onOk={() => { db.answers.remove(del.id); toast(t('common.deleted')); }} />}
    </div>
  );
}

function Problems({ rows }: { rows: DoctorQuestion[] }) {
  const { t, num, lang } = useI18n();
  const pn = usePlaceNames();
  const [topic, setTopic] = useState('');
  const [days, setDays] = useState('14');
  const data = useMemo(() => {
    const since = Date.now() - +days * 864e5;
    const byDist = new Map<string, { n: number; topics: Map<string, number> }>(), byTopic = new Map<string, number>();
    let total = 0;
    for (const q of rows) {
      if (+new Date(q.at) < since) continue;
      byTopic.set(q.topic, (byTopic.get(q.topic) ?? 0) + 1);
      if (topic && q.topic !== topic) continue;
      const dist = (q.farmId ? db.farms.get(q.farmId)?.dist : undefined) ?? db.farmers.get(q.farmerId)?.dist;
      if (!dist) continue;
      let d = byDist.get(dist); if (!d) { d = { n: 0, topics: new Map() }; byDist.set(dist, d); }
      d.n++; d.topics.set(q.topic, (d.topics.get(q.topic) ?? 0) + 1); total++;
    }
    const top = [...byDist].sort((a, z) => z[1].n - a[1].n);
    return { byDist, byTopic, top, total, max: top[0]?.[1].n ?? 0 };
  }, [rows, topic, days]);
  const stops = [1, 2, 4, 7];
  return (
    <div className="stack">
      <Note tone="info">{t('doctor.map_note')}</Note>
      <div className="filters">
        <div className="chips" style={{ flex: '1 1 auto' }}>
          <button className={'chip' + (!topic ? ' on' : '')} onClick={() => setTopic('')}>{t('doctor.all_topics')}</button>
          {TOPICS.map(k => <button key={k} className={'chip' + (topic === k ? ' on' : '')} onClick={() => setTopic(k)}>{t('doctor.topic_' + k)} <span className="muted">{num(data.byTopic.get(k) ?? 0)}</span></button>)}
        </div>
        <Select value={days} onChange={setDays} options={[['7', t('doctor.days', { n: 7 })], ['14', t('doctor.days', { n: 14 })], ['30', t('doctor.days', { n: 30 })]]} />
      </div>
      <div className="grid g-main">
        <Card title={t('doctor.map_title', { n: num(data.total) })}>
          <DistrictMap styleKey={topic + days + lang + data.total} fill={d => { const n = data.byDist.get(d)?.n ?? 0; return n ? ramp(n, stops, HEAT) : undefined; }} />
          <div className="legend">{HEAT.slice(1).map((c, i) => <span key={c}><i className="dotc" style={{ background: c }} />{t('doctor.band_' + i)}</span>)}</div>
        </Card>
        <Card title={t('doctor.top_districts')}>
          {data.top.slice(0, 10).map(([d, v]) => {
            const main = [...v.topics].sort((a, z) => z[1] - a[1])[0];
            const hot = v.n >= 4;
            return (
              <div key={d} className="list-item">
                <span className={'ico ' + (hot ? 'danger' : 'warn')}><b className="tabular">{num(v.n)}</b></span>
                <div style={{ flex: 1, minWidth: 0 }}><b>{pn.dist(d)}</b><div className="muted small">{t('doctor.mostly', { topic: t('doctor.topic_' + main[0]), n: num(main[1]) })}</div></div>
                {hot && <Pill tone="danger">{t('doctor.look')}</Pill>}
              </div>
            );
          })}
          {!data.top.length && <div className="empty">{t('doctor.map_empty')}</div>}
        </Card>
      </div>
    </div>
  );
}

function HardRules() {
  const { t } = useI18n();
  const rules: [JSX.Element, string][] = [[<Calculator key="1" />, 'doses'], [<Ban key="2" />, 'brands'], [<ShieldCheck key="3" />, 'numbers'], [<HelpCircle key="4" />, 'unsure'], [<Lock key="5" />, 'privacy']];
  return (
    <Card>
      <Note tone="warn" icon={<Lock />}>{t('doctor.rules_note')}</Note>
      <div className="mt">
        {rules.map(([ic, k]) => (
          <div key={k} className="list-item" style={{ alignItems: 'flex-start' }}>
            <span className="ico brand">{ic}</span>
            <div style={{ flex: 1 }}><b>{t('doctor.rule_' + k)}</b><div className="muted small">{t('doctor.rule_' + k + '_why')}</div></div>
            <Pill tone="dark" icon={<Lock />}>{t('doctor.fixed')}</Pill>
          </div>
        ))}
      </div>
    </Card>
  );
}
