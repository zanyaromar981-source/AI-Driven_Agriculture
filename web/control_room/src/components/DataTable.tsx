// One table for every list: sort by a column, pages of N rows, choose which columns to show
// (remembered per table in this browser), rows turn into cards on phones.
// Cost: sorting is O(n log n) and runs only when the rows or the sort change; only one page is drawn.
import { useMemo, useState, type ReactNode } from 'react';
import { ChevronLeft, ChevronRight, Columns3, ArrowUpDown, ArrowUp, ArrowDown } from 'lucide-react';
import { prefs } from '../data/store';
import { useI18n } from '../i18n';
import { Modal, Switch } from './ui';

export interface Col<T> {
  key: string;
  label: string;
  cell: (r: T) => ReactNode;
  sort?: (r: T) => string | number;
  num?: boolean;
  /** hidden until the admin turns it on */
  optional?: boolean;
  className?: string;
}

export function DataTable<T extends { id: string }>({ id, rows, cols, onRow, per = 25, empty, selected, head, defaultSort }: {
  id: string; rows: T[]; cols: Col<T>[]; onRow?: (r: T) => void; per?: number; empty?: string; selected?: string | null;
  head?: ReactNode; defaultSort?: [string, 1 | -1];
}) {
  const { t, num } = useI18n();
  const [sort, setSort] = useState<[string, 1 | -1] | null>(defaultSort ?? null);
  const [page, setPage] = useState(0);
  const [hidden, setHidden] = useState<string[]>(() => prefs.get('cols.' + id, cols.filter(c => c.optional).map(c => c.key)));
  const [chooser, setChooser] = useState(false);

  const sorted = useMemo(() => {
    if (!sort) return rows;
    const c = cols.find(x => x.key === sort[0]);
    if (!c?.sort) return rows;
    const key = c.sort, dir = sort[1];
    // decorate, sort, undecorate: the key is computed once per row, not once per comparison
    return rows.map(r => [key(r), r] as const).sort((a, b) => (a[0] < b[0] ? -dir : a[0] > b[0] ? dir : 0)).map(x => x[1]);
  }, [rows, sort, cols]);

  const pages = Math.max(1, Math.ceil(sorted.length / per));
  const pg = Math.min(page, pages - 1);
  const view = sorted.slice(pg * per, pg * per + per);
  const shown = cols.filter(c => !hidden.includes(c.key));
  const toggleSort = (c: Col<T>) => c.sort && setSort(s => (s?.[0] === c.key ? (s[1] === 1 ? [c.key, -1] : null) : [c.key, 1]));
  const setHid = (h: string[]) => { setHidden(h); prefs.set('cols.' + id, h); };

  return (
    <div>
      <div className="spread" style={{ marginBottom: 8 }}>
        <div className="row">{head}</div>
        <button className="btn sm" onClick={() => setChooser(true)}><Columns3 />{t('table.columns')}</button>
      </div>
      <div className="table-wrap">
        <table className="t cards">
          <thead><tr>{shown.map(c => (
            <th key={c.key} className={(c.num ? 'num ' : '') + (c.sort ? 'sortable' : '')} onClick={() => toggleSort(c)} aria-sort={sort?.[0] === c.key ? (sort[1] === 1 ? 'ascending' : 'descending') : undefined}>
              <span className="row" style={{ gap: 4, flexWrap: 'nowrap', justifyContent: c.num ? 'flex-end' : undefined }}>{c.label}
                {c.sort && (sort?.[0] === c.key ? (sort[1] === 1 ? <ArrowUp size={12} /> : <ArrowDown size={12} />) : <ArrowUpDown size={12} style={{ opacity: .35 }} />)}</span>
            </th>))}</tr></thead>
          <tbody>
            {view.map(r => (
              <tr key={r.id} className={(onRow ? 'click ' : '') + (selected === r.id ? 'sel' : '')} onClick={onRow ? () => onRow(r) : undefined}
                tabIndex={onRow ? 0 : undefined} onKeyDown={onRow ? e => { if (e.key === 'Enter') onRow(r); } : undefined}>
                {shown.map(c => <td key={c.key} data-label={c.label} className={(c.num ? 'num ' : '') + (c.className ?? '')}>{c.cell(r)}</td>)}
              </tr>))}
          </tbody>
        </table>
      </div>
      {!rows.length && <div className="empty">{empty ?? t('table.empty')}</div>}
      {rows.length > per && (
        <div className="pager">
          <span className="muted small">{t('table.page', { a: num(pg + 1), b: num(pages), n: num(rows.length) })}</span>
          <div className="row">
            <button className="btn sm" disabled={pg === 0} onClick={() => setPage(pg - 1)}><ChevronLeft className="flip-rtl" />{t('table.prev')}</button>
            <button className="btn sm" disabled={pg >= pages - 1} onClick={() => setPage(pg + 1)}>{t('table.next')}<ChevronRight className="flip-rtl" /></button>
          </div>
        </div>
      )}
      {chooser && (
        <Modal title={t('table.columns')} onClose={() => setChooser(false)} foot={<button className="btn primary" onClick={() => setChooser(false)}>{t('common.done')}</button>}>
          {cols.map(c => (
            <div className="set" key={c.key}><div className="txt"><b>{c.label}</b></div>
              <Switch on={!hidden.includes(c.key)} onChange={on => setHid(on ? hidden.filter(h => h !== c.key) : [...hidden, c.key])} label={c.label} /></div>))}
        </Modal>
      )}
    </div>
  );
}
