// Alwa markets and their price boards (FRONTEND.md 9 Alwa): staff add, rename and remove markets and type
// the price of a crop at a market on a day. A typed price is what makes a listing "fair", "high" or "low".
// Routes: /dashboard/alwa/markets[/{slug}], /dashboard/alwa/markets/{slug}/prices[/{crop}/{day}].
import { useState } from 'react';
import { Plus, Pencil, Trash2, Tag } from 'lucide-react';
import { useI18n } from '../../../i18n';
import { useAuth } from '../../../auth/auth';
import { api, qs, ApiError } from '../../../api/client';
import { useApi, invalidate } from '../../../api/cache';
import { Modal, Field, Select, Switch, Confirm } from '../../../components/ui';
import { CropTag, StateBox, useErrorText } from '../../../components/domain';
import { CROPS } from '../../../data/crops';
import { newKey } from '../farms/common';
import { useAction } from '../inbox/useAction';

export interface Market { slug: string; name_en: string; name_ku: string }
interface Recorded { crop: string; day: string; fixed: boolean; market: string; price_iqd_per_kg: number; source: string; updated_at: string }
const today = () => { const d = new Date(); return new Date(d.getTime() - d.getTimezoneOffset() * 60000).toISOString().slice(0, 10); };

export function Markets() {
  const { t } = useI18n();
  const { can } = useAuth();
  const q = useApi<{ markets: Market[] }>('/dashboard/alwa/markets', ['alwa_prices'], { auth: true });
  const [edit, setEdit] = useState<Market | 'new' | null>(null);
  const markets = q.data?.markets ?? [];
  return (
    <div className="card">
      <div className="card-head">
        <span className="eyebrow">{t('alwa.boards')}</span>
        {can('alwa', 'create') && <button className="btn sm" onClick={() => setEdit('new')}><Plus />{t('alwa.m_add')}</button>}
      </div>
      <p className="muted small" style={{ marginTop: 0 }}>{t('alwa.boards_note')}</p>
      {q.error?.status === 403 && !q.data ? <StateBox kind="locked" />
        : q.error && !q.data ? <StateBox kind="error" />
          : q.loading ? <div className="sk-rows">{Array.from({ length: 4 }, (_, i) => <i key={i} className="sk" />)}</div>
            : !markets.length ? <StateBox kind="empty" title={t('alwa.m_none')} />
              : markets.map(m => <MarketBoard key={m.slug} market={m} onEdit={() => setEdit(m)} />)}
      {edit && <MarketForm market={edit === 'new' ? null : edit} onClose={() => setEdit(null)} />}
    </div>
  );
}

function MarketBoard({ market, onEdit }: { market: Market; onEdit: () => void }) {
  const { t, num, date, pick } = useI18n();
  const { can } = useAuth();
  const errText = useErrorText();
  const [entering, setEntering] = useState<Recorded | 'new' | null>(null);
  const [del, setDel] = useState<Recorded | 'market' | null>(null);
  const { busy, run } = useAction();
  const q = useApi<{ prices: Recorded[]; count: number }>('/dashboard/alwa/markets/' + market.slug + '/prices' + qs({ rows_per_page: 8 }), ['alwa_prices'], { auth: true });
  const rows = q.data?.prices ?? [];
  const removePrice = (p: Recorded) => run(async () => { await api.del(`/dashboard/alwa/markets/${market.slug}/prices/${p.crop}/${p.day}`); invalidate('alwa_prices'); }, t('common.deleted'));
  const removeMarket = () => run(async () => { await api.del('/dashboard/alwa/markets/' + market.slug); invalidate('alwa_prices'); }, t('common.deleted'));
  return (
    <div className="board">
      <div className="spread">
        <b>{pick(market.name_ku, market.name_en)}</b>
        <span className="row">
          {can('alwa', 'create') && <button className="btn sm primary" disabled={busy} onClick={() => setEntering('new')}><Tag />{t('alwa.p_enter')}</button>}
          {can('alwa', 'update') && <button className="btn sm" disabled={busy} onClick={onEdit} aria-label={t('alwa.m_rename')}><Pencil /></button>}
          {can('alwa', 'delete') && <button className="btn sm danger" disabled={busy} onClick={() => setDel('market')} aria-label={t('common.delete')}><Trash2 /></button>}
        </span>
      </div>
      {q.error && !q.data ? <p className="small muted">{errText(q.error)}</p>
        : !rows.length ? <p className="small muted">{q.loading ? '…' : t('alwa.p_none')}</p> : (
          <table className="t compact">
            <tbody>{rows.map(p => (
              <tr key={p.crop + p.day}>
                <td><CropTag code={p.crop} /></td>
                <td className="num"><b className="tabular">{num(p.price_iqd_per_kg)}</b> <span className="small muted">{t('common.iqd_kg')}</span></td>
                <td className="small muted">{date(p.day, 'date')}{p.fixed && <> · {t('alwa.p_fixed')}</>}</td>
                <td className="small muted" dir="auto">{p.source}</td>
                <td className="act">
                  {can('alwa', 'update') && <button className="btn sm" disabled={busy} onClick={() => setEntering(p)} aria-label={t('alwa.p_correct')}><Pencil /></button>}{' '}
                  {can('alwa', 'delete') && <button className="btn sm danger" disabled={busy} onClick={() => setDel(p)} aria-label={t('common.delete')}><Trash2 /></button>}
                </td>
              </tr>))}</tbody>
          </table>
        )}
      {entering && <PriceForm market={market} price={entering === 'new' ? null : entering} onClose={() => setEntering(null)} />}
      {del === 'market' && <Confirm title={t('alwa.m_del_title')} text={t('alwa.m_del_text')} okLabel={t('common.delete')} onOk={removeMarket} onClose={() => setDel(null)} />}
      {del && del !== 'market' && <Confirm title={t('alwa.p_del_title')} text={t('alwa.p_del_text')} okLabel={t('common.delete')} onOk={() => removePrice(del)} onClose={() => setDel(null)} />}
    </div>
  );
}

function MarketForm({ market, onClose }: { market: Market | null; onClose: () => void }) {
  const { t } = useI18n();
  const [slug, setSlug] = useState(market?.slug ?? '');
  const [en, setEn] = useState(market?.name_en ?? '');
  const [ku, setKu] = useState(market?.name_ku ?? '');
  const [key] = useState(newKey);
  const { busy, run } = useAction();
  const badSlug = !market && !/^[a-z0-9_-]{2,40}$/.test(slug);
  const ok = !badSlug && en.trim() && ku.trim();
  const save = () => run(async () => {
    if (market) await api.put('/dashboard/alwa/markets/' + market.slug, { name_en: en.trim(), name_ku: ku.trim() });
    else await api.post('/dashboard/alwa/markets', { slug, name_en: en.trim(), name_ku: ku.trim() }, key);
    invalidate('alwa_prices');
    onClose();
  }, t('common.saved'));
  return (
    <Modal title={market ? t('alwa.m_rename') : t('alwa.m_add')} onClose={onClose} foot={<>
      <button className="btn" onClick={onClose}>{t('common.cancel')}</button>
      <button className="btn primary" disabled={busy || !ok} onClick={save}>{t('common.save')}</button>
    </>}>
      <div className="form-grid">
        {!market && <Field label={t('alwa.m_slug')} hint={t('alwa.m_slug_hint')} error={slug && badSlug ? 'alwa.m_slug_bad' : undefined} full>
          <input type="text" dir="ltr" value={slug} onChange={e => setSlug(e.target.value.toLowerCase())} maxLength={40} />
        </Field>}
        <Field label={t('alwa.m_name_ku')}><input type="text" dir="rtl" value={ku} onChange={e => setKu(e.target.value)} maxLength={80} /></Field>
        <Field label={t('alwa.m_name_en')}><input type="text" dir="ltr" value={en} onChange={e => setEn(e.target.value)} maxLength={80} /></Field>
      </div>
    </Modal>
  );
}

/** Enter a new price (POST) or correct one (PUT). A POST that meets an existing price for that crop and
 *  day answers 409: the form then offers to correct it instead. */
function PriceForm({ market, price, onClose }: { market: Market; price: Recorded | null; onClose: () => void }) {
  const { t, pick } = useI18n();
  const [crop, setCrop] = useState(price?.crop ?? '');
  const [day, setDay] = useState(price?.day ?? today());
  const [value, setValue] = useState(price ? String(price.price_iqd_per_kg) : '');
  const [source, setSource] = useState(price?.source ?? '');
  const [fixed, setFixed] = useState(price?.fixed ?? false);
  const [exists, setExists] = useState(false);
  const [key] = useState(newKey);
  const { busy, run } = useAction();
  const v = Number(value);
  const ok = crop && /^\d{4}-\d{2}-\d{2}$/.test(day) && value.trim() !== '' && v > 0 && source.trim().length > 0;
  const put = () => api.put(`/dashboard/alwa/markets/${market.slug}/prices/${crop}/${day}`, { price_iqd_per_kg: v, source: source.trim(), fixed });
  const save = () => run(async () => {
    try {
      if (price || exists) await put();
      else await api.post(`/dashboard/alwa/markets/${market.slug}/prices`, { crop, day, price_iqd_per_kg: v, source: source.trim(), fixed }, key);
    } catch (e) {
      if (e instanceof ApiError && e.status === 409 && !price) { setExists(true); return; }
      throw e;
    }
    invalidate('alwa_prices', 'alwa_listings');
    onClose();
  }, t('common.saved'));
  return (
    <Modal title={(price ? t('alwa.p_correct') : t('alwa.p_enter')) + ': ' + pick(market.name_ku, market.name_en)} onClose={onClose} foot={<>
      <button className="btn" onClick={onClose}>{t('common.cancel')}</button>
      <button className="btn primary" disabled={busy || !ok} onClick={save}>{exists ? t('alwa.p_correct') : t('common.save')}</button>
    </>}>
      {exists && <div className="note warn" style={{ marginBottom: 10 }}>{t('alwa.p_exists')}</div>}
      <div className="form-grid">
        <Field label={t('alwa.crop')}>
          <Select value={crop} disabled={!!price} onChange={c => { setCrop(c); setExists(false); }} options={[['', '-'], ...CROPS.filter(c => c.code !== 'empty' && c.active !== false).map(c => [c.code, pick(c.ku, c.en)] as [string, string])]} />
        </Field>
        <Field label={t('alwa.p_day')}><input type="date" dir="ltr" value={day} disabled={!!price} max={today()} onChange={e => { setDay(e.target.value); setExists(false); }} /></Field>
        <Field label={t('alwa.p_price')} error={value && !(v > 0) ? 'alwa.p_bad' : undefined}><input type="number" dir="ltr" min={1} step="any" value={value} onChange={e => setValue(e.target.value)} /></Field>
        <Field label={t('alwa.p_source')} hint={t('alwa.p_source_hint')}><input type="text" value={source} onChange={e => setSource(e.target.value)} maxLength={200} /></Field>
        <div className="set full"><div className="txt"><b>{t('alwa.p_fixed')}</b><small>{t('alwa.p_fixed_sub')}</small></div><div className="ctl"><Switch on={fixed} onChange={setFixed} label={t('alwa.p_fixed')} /></div></div>
      </div>
    </Modal>
  );
}
