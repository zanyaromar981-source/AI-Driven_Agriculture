// Workers for hire: the cards people put up in the app (name, phone, cost), with search, an availability
// filter and Remove. With a staff token (VITE_STAFF_TOKEN) the rows are the server's
// (GET and DELETE /v1/dashboard/workers); without one they are sample rows kept in this browser.
import { useEffect, useMemo, useState } from 'react';
import { Trash2, Info } from 'lucide-react';
import { db, PLACES } from '../../data/db';
import { useRows } from '../../data/store';
import { STAFF_TOKEN, deleteWorker, fetchWorkers } from '../../data/backend';
import type { Worker } from '../../data/types';
import { useI18n } from '../../i18n';
import { Card, Confirm, Note, PageHead, Pill, Select, useDebounced, useToast } from '../../components/ui';
import { DataTable, type Col } from '../../components/DataTable';
import { Phone, usePlaceNames } from '../../components/domain';

const LIVE = !!STAFF_TOKEN;
/** district slug ("chamchamal") -> the English name places.json uses */
const DIST_BY_SLUG = new Map(PLACES.districts.map(d => [d.en.toLowerCase().replace(/\s+/g, '-'), d.en]));

export default function Workers() {
  const { t, num, ago } = useI18n();
  const toast = useToast();
  const place = usePlaceNames();
  const sample = useRows(db.workers);
  const [q, setQ] = useState(''), [avail, setAvail] = useState('');
  const dq = useDebounced(q.trim(), LIVE ? 350 : 200);
  const [live, setLive] = useState<Worker[] | null>(null);
  const [failed, setFailed] = useState('');
  const [del, setDel] = useState<Worker | null>(null);

  useEffect(() => {
    if (!LIVE) return;
    let on = true;
    fetchWorkers(dq, avail).then(rows => { if (on) { setLive(rows); setFailed(''); } }, e => { if (on) { setLive([]); setFailed(e instanceof Error ? e.message : String(e)); } });
    return () => { on = false; };
  }, [dq, avail]);

  const rows = useMemo(() => {
    if (LIVE) return live ?? [];
    const s = dq.toLowerCase();
    return sample.filter(w => (!avail || String(w.available) === avail) && (!s || w.name.toLowerCase().includes(s) || w.note.toLowerCase().includes(s)));
  }, [live, sample, dq, avail]);

  const remove = (w: Worker) => {
    if (!LIVE) { db.workers.remove(w.id); toast(t('workers.removed', { name: w.name }), 'good'); return; }
    deleteWorker(w.id).then(
      () => { setLive(x => x && x.filter(r => r.id !== w.id)); toast(t('workers.removed', { name: w.name }), 'good'); },
      e => toast(t('workers.remove_failed', { code: e instanceof Error ? e.message : String(e) }), 'danger'));
  };

  const cols: Col<Worker>[] = [
    { key: 'name', label: t('workers.name'), sort: w => w.name.toLowerCase(), cell: w => <b><bdi>{w.name}</bdi></b> },
    { key: 'phone', label: t('workers.phone'), cell: w => (w.phone ? <Phone value={w.phone} /> : <span className="muted">-</span>) },
    { key: 'cost', label: t('workers.cost'), sort: w => (w.per === 'hour' ? w.cost * 8 : w.cost), cell: w => <span className="nowrap">{t('workers.cost_' + w.per, { n: num(w.cost) })}</span> },
    { key: 'note', label: t('workers.note'), cell: w => (w.note ? <bdi>{w.note}</bdi> : <span className="muted">-</span>) },
    { key: 'district', label: t('common.district'), sort: w => w.zone, cell: w => (w.zone ? place.dist(DIST_BY_SLUG.get(w.zone) ?? w.zone) : <span className="muted">-</span>) },
    { key: 'available', label: t('workers.available'), sort: w => (w.available ? 0 : 1), cell: w => (w.available ? <Pill tone="good">{t('workers.available')}</Pill> : <Pill>{t('workers.paused')}</Pill>) },
    { key: 'updated', label: t('common.updated'), sort: w => w.updated, cell: w => <span className="nowrap">{ago(w.updated)}</span> },
    { key: 'act', label: '', className: 'act', cell: w => <button className="btn sm danger" onClick={() => setDel(w)}><Trash2 />{t('workers.remove')}</button> },
  ];

  return (
    <>
      <PageHead eyebrow={t('nav.g_people')} title={t('workers.title')} sub={t('workers.sub')} />
      <div className="mb">
        {failed ? <Note tone="danger" icon={<Info />}>{t('workers.load_failed', { code: failed })}</Note>
          : LIVE ? <Note tone="info" icon={<Info />}>{t('workers.live_note')}</Note>
            : <Note tone="warn" icon={<Info />}>{t('workers.sample_note')}</Note>}
      </div>
      <Card>
        <div className="filters">
          <input type="search" className="grow" value={q} onChange={e => setQ(e.target.value)} placeholder={t('workers.search')} />
          <Select value={avail} onChange={setAvail} options={[['', t('workers.any_avail')], ['true', t('workers.only_avail')], ['false', t('workers.only_paused')]]} aria-label={t('workers.available')} />
        </div>
        <DataTable id="workers" rows={rows} cols={cols} defaultSort={['updated', -1]} empty={LIVE && !live ? t('workers.loading') : undefined}
          head={<>{!LIVE && <Pill tone="warn">{t('common.sample')}</Pill>}<span className="muted small">{t('workers.count', { n: num(rows.length) })}</span></>} />
      </Card>
      {del && <Confirm title={t('workers.remove_q', { name: del.name })} text={t('workers.remove_text')} okLabel={t('workers.remove')} onClose={() => setDel(null)} onOk={() => remove(del)} />}
    </>
  );
}
