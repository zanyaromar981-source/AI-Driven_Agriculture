// Texts and languages: every word on this website, and the texts of the app, in Kurdish and English.
// Edits here are saved as overrides (db.siteTexts) and win over the text files.
// Cost: the list of ~800 keys is built once per change of the overrides; search runs once per settled
// query (debounced) in one pass; only one page of 25 rows is drawn.
import { useMemo, useState } from 'react';
import { Download, FileSpreadsheet, RotateCcw, Search, Plus, Pencil, Trash2, CheckCircle2 } from 'lucide-react';
import { EN, KU, useI18n } from '../../i18n';
import { db } from '../../data/db';
import { useRows } from '../../data/store';
import { downloadCsv, nowIso } from '../../data/api';
import type { AppText } from '../../data/types';
import { Card, Confirm, Field, Kpi, Modal, Note, PageHead, Pill, Select, Switch, Tabs, useDebounced, useToast } from '../../components/ui';
import { DataTable, type Col } from '../../components/DataTable';

type Tab = 'site' | 'app';

export default function Texts() {
  const { t } = useI18n();
  const [tab, setTab] = useState<Tab>('site');
  return (
    <>
      <PageHead eyebrow={t('nav.g_app')} title={t('nav.texts')} sub={t('texts.sub')} />
      <div className="mb"><Note tone="good" icon={<FileSpreadsheet />}>{t('texts.sheet_note')}</Note></div>
      <Tabs value={tab} onChange={setTab} items={[['site', t('texts.tab_site')], ['app', t('texts.tab_app')]]} />
      {tab === 'site' ? <SiteTexts /> : <AppTexts />}
    </>
  );
}

const PER = 25;
const KEYS = Object.keys(EN).sort();

function SiteTexts() {
  const { t, num } = useI18n();
  const toast = useToast();
  const over = useRows(db.siteTexts);
  const [q, setQ] = useState('');
  const [ns, setNs] = useState('');
  const [missing, setMissing] = useState(false);
  const [page, setPage] = useState(0);
  const dq = useDebounced(q.trim().toLowerCase());

  // every key with its file and override values, rebuilt only when overrides change
  const rows = useMemo(() => {
    const o = new Map(over.map(r => [r.id, r.text]));
    return KEYS.map(k => {
      const enO = o.get('en:' + k) ?? '', kuO = o.get('ku:' + k) ?? '';
      return { key: k, ns: k.slice(0, k.indexOf('.')), enFile: EN[k], kuFile: KU[k] ?? '', enO, kuO, ku: kuO || KU[k] || '' };
    });
  }, [over]);
  const namespaces = useMemo(() => [...new Set(rows.map(r => r.ns))], [rows]);
  const written = useMemo(() => rows.reduce((n, r) => n + (r.ku ? 1 : 0), 0), [rows]);

  const list = useMemo(() => rows.filter(r =>
    (!ns || r.ns === ns) && (!missing || !r.ku) &&
    (!dq || r.key.toLowerCase().includes(dq) || (r.enO || r.enFile).toLowerCase().includes(dq) || r.ku.includes(dq))), [rows, ns, missing, dq]);
  const pages = Math.max(1, Math.ceil(list.length / PER));
  const pg = Math.min(page, pages - 1);
  const view = list.slice(pg * PER, pg * PER + PER);

  const save = (lang: 'en' | 'ku', key: string, value: string) => {
    const id = lang + ':' + key, v = value.trim(), cur = db.siteTexts.get(id)?.text ?? '';
    if (v === cur) return;
    if (v) db.siteTexts.put({ id, text: v }); else db.siteTexts.remove(id);
    toast(t('common.saved'), 'good');
  };
  const reset = (key: string) => { db.siteTexts.remove('en:' + key); db.siteTexts.remove('ku:' + key); toast(t('texts.reset_done')); };
  const exportCsv = () => downloadCsv('jutyar_website_texts', ['key', 'page', 'English', 'Kurdish'], rows.map(r => [r.key, r.ns, r.enO || r.enFile, r.ku]));

  return (
    <>
      <div className="grid g3 mb">
        <Kpi label={t('texts.total')} value={num(rows.length)} />
        <Kpi label={t('texts.ku_written')} value={num(written)} tone="good" note={num(written / rows.length * 100, 0) + '%'} />
        <Kpi label={t('texts.ku_missing')} value={num(rows.length - written)} tone={rows.length - written ? 'warn' : 'good'} note={t('texts.ku_missing_note')} />
      </div>
      <Card>
        <div className="filters">
          <div className="grow" style={{ position: 'relative' }}>
            <input type="search" value={q} onChange={e => { setQ(e.target.value); setPage(0); }} placeholder={t('texts.search')} style={{ paddingInlineStart: 34 }} />
            <Search size={16} style={{ position: 'absolute', insetInlineStart: 10, top: 11, color: 'var(--ink-3)' }} />
          </div>
          <Select value={ns} onChange={v => { setNs(v); setPage(0); }} options={[['', t('texts.all_pages')], ...namespaces.map(n => [n, n] as [string, string])]} />
          <span className="row small" style={{ gap: 6 }}><Switch on={missing} onChange={v => { setMissing(v); setPage(0); }} label={t('texts.only_missing')} />{t('texts.only_missing')}</span>
          <button className="btn" onClick={exportCsv}><Download />{t('common.export_csv')}</button>
        </div>
        <div className="table-wrap">
          <table className="t cards">
            <thead><tr><th>{t('texts.key')}</th><th>{t('common.english')}</th><th>{t('common.kurdish')}</th><th /></tr></thead>
            <tbody>
              {view.map(r => (
                <tr key={r.key}>
                  <td data-label={t('texts.key')} style={{ maxWidth: 200 }}><span className="mono small ltr" style={{ wordBreak: 'break-all' }}>{r.key}</span></td>
                  <td data-label={t('common.english')} style={{ minWidth: 220 }}>
                    <textarea key={'en' + r.enO} dir="ltr" rows={1} defaultValue={r.enO} placeholder={r.enFile} onBlur={e => save('en', r.key, e.target.value)}
                      style={{ minHeight: 38 }} aria-label={t('common.english') + ' ' + r.key} />
                  </td>
                  <td data-label={t('common.kurdish')} style={{ minWidth: 220 }}>
                    <textarea key={'ku' + r.kuO} dir="rtl" className="ku-text" rows={1} defaultValue={r.kuO} placeholder={r.kuFile || t('common.ku_missing')}
                      onBlur={e => save('ku', r.key, e.target.value)} style={{ minHeight: 38, borderColor: r.ku ? undefined : 'var(--warn)' }} aria-label={t('common.kurdish') + ' ' + r.key} />
                  </td>
                  <td data-label="" className="act">
                    {(r.enO || r.kuO) && <button className="btn sm icon" title={t('texts.reset')} aria-label={t('texts.reset')} onClick={() => reset(r.key)}><RotateCcw /></button>}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        {!list.length && <div className="empty">{t('table.empty')}</div>}
        <div className="pager">
          <span className="muted small">{t('table.page', { a: num(pg + 1), b: num(pages), n: num(list.length) })} · {t('texts.blur_saves')}</span>
          <div className="row">
            <button className="btn sm" disabled={pg === 0} onClick={() => setPage(pg - 1)}>{t('table.prev')}</button>
            <button className="btn sm" disabled={pg >= pages - 1} onClick={() => setPage(pg + 1)}>{t('table.next')}</button>
          </div>
        </div>
      </Card>
    </>
  );
}

const blank = (): AppText => ({ id: '', screen: '', text: { en: '', ku: '' }, checked: false, updated: nowIso() });

function AppTexts() {
  const { t, ago } = useI18n();
  const toast = useToast();
  const rows = useRows(db.appTexts);
  const [q, setQ] = useState('');
  const dq = useDebounced(q.trim().toLowerCase());
  const [edit, setEdit] = useState<{ row: AppText; isNew: boolean } | null>(null);
  const [del, setDel] = useState<AppText | null>(null);
  const list = useMemo(() => rows.filter(r => !dq || r.id.toLowerCase().includes(dq) || r.text.en.toLowerCase().includes(dq) || r.text.ku.includes(dq) || r.screen.toLowerCase().includes(dq)), [rows, dq]);

  const cols: Col<AppText>[] = [
    { key: 'key', label: t('texts.key'), cell: r => <span className="mono small ltr">{r.id}</span>, sort: r => r.id },
    { key: 'screen', label: t('texts.screen'), cell: r => r.screen, sort: r => r.screen },
    { key: 'en', label: t('common.english'), cell: r => <bdi dir="ltr">{r.text.en}</bdi> },
    { key: 'ku', label: t('common.kurdish'), cell: r => r.text.ku ? <bdi dir="rtl" className="ku-text">{r.text.ku}</bdi> : <Pill tone="warn">{t('common.ku_missing')}</Pill> },
    { key: 'checked', label: t('texts.checked'), cell: r => <Switch on={r.checked} label={t('texts.checked')} onChange={v => { db.appTexts.patch(r.id, { checked: v, updated: nowIso() }); toast(t('common.saved'), 'good'); }} />, sort: r => (r.checked ? 1 : 0) },
    { key: 'updated', label: t('common.updated'), cell: r => ago(r.updated), sort: r => r.updated, optional: true },
    { key: 'act', label: '', className: 'act', cell: r => <>
      <button className="btn sm icon" aria-label={t('common.edit')} onClick={e => { e.stopPropagation(); setEdit({ row: r, isNew: false }); }}><Pencil /></button>{' '}
      <button className="btn sm icon danger" aria-label={t('common.delete')} onClick={e => { e.stopPropagation(); setDel(r); }}><Trash2 /></button></> },
  ];
  const checked = rows.filter(r => r.checked).length;
  return (
    <Card>
      <p className="muted" style={{ marginTop: 0 }}>{t('texts.app_sub')}</p>
      <DataTable id="appTexts" rows={list} cols={cols} defaultSort={['key', 1]}
        head={<>
          <input type="search" value={q} onChange={e => setQ(e.target.value)} placeholder={t('texts.search')} style={{ width: 240 }} />
          <button className="btn primary sm" onClick={() => setEdit({ row: blank(), isNew: true })}><Plus />{t('texts.add')}</button>
          <span className="muted small row" style={{ gap: 4 }}><CheckCircle2 size={14} />{t('texts.checked_count', { n: checked, all: rows.length })}</span>
        </>} />
      {edit && <EditAppText row={edit.row} isNew={edit.isNew} onClose={() => setEdit(null)} />}
      {del && <Confirm title={t('texts.del_title')} text={t('texts.del_text', { key: del.id })} okLabel={t('common.delete')}
        onOk={() => { db.appTexts.remove(del.id); toast(t('common.deleted')); }} onClose={() => setDel(null)} />}
    </Card>
  );
}

function EditAppText({ row, isNew, onClose }: { row: AppText; isNew: boolean; onClose: () => void }) {
  const { t } = useI18n();
  const toast = useToast();
  const [r, setR] = useState(row);
  const [err, setErr] = useState<Record<string, string>>({});
  const save = () => {
    const e: Record<string, string> = {};
    const id = r.id.trim();
    if (!/^[a-z0-9_]+(\.[a-z0-9_]+)+$/.test(id)) e.id = 'texts.key_bad';
    else if (isNew && db.appTexts.has(id)) e.id = 'texts.key_taken';
    if (!r.text.en.trim() && !r.text.ku.trim()) e.text = 'v.needed';
    setErr(e);
    if (Object.keys(e).length) return;
    db.appTexts.put({ ...r, id, screen: r.screen.trim(), text: { en: r.text.en.trim(), ku: r.text.ku.trim() }, updated: nowIso() });
    toast(t('common.saved'), 'good'); onClose();
  };
  return (
    <Modal title={isNew ? t('texts.add') : t('texts.edit')} onClose={onClose} foot={<>
      <button className="btn" onClick={onClose}>{t('common.cancel')}</button><button className="btn primary" onClick={save}>{t('common.save')}</button></>}>
      <div className="form-grid">
        <Field label={t('texts.key')} hint={t('texts.key_hint')} error={err.id}>
          <input type="text" className="ltr-input" value={r.id} disabled={!isNew} onChange={e => setR({ ...r, id: e.target.value })} />
        </Field>
        <Field label={t('texts.screen')}><input type="text" value={r.screen} onChange={e => setR({ ...r, screen: e.target.value })} /></Field>
        <Field label={t('common.kurdish')} full error={err.text}><textarea dir="rtl" className="ku-text" value={r.text.ku} onChange={e => setR({ ...r, text: { ...r.text, ku: e.target.value } })} /></Field>
        <Field label={t('common.english')} full><textarea dir="ltr" value={r.text.en} onChange={e => setR({ ...r, text: { ...r.text, en: e.target.value } })} /></Field>
        <div className="full row"><Switch on={r.checked} onChange={v => setR({ ...r, checked: v })} label={t('texts.checked')} /><span>{t('texts.checked_long')}</span></div>
      </div>
    </Modal>
  );
}
