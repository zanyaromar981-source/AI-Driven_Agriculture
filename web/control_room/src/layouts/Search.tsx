// Search pages, farmers (name or phone) and farms (number or name). Press "/" anywhere in the admin.
// Cost: one pass over farmers and farms per settled query (typing is debounced), at most 8 hits each.
import { useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Map as MapIcon, User } from 'lucide-react';
import { useI18n } from '../i18n';
import { db } from '../data/db';
import { Modal, useDebounced } from '../components/ui';
import { NAV_FLAT } from './nav';

export function SearchBox({ onClose }: { onClose: () => void }) {
  const { t, b } = useI18n();
  const nav = useNavigate();
  const [q, setQ] = useState('');
  const dq = useDebounced(q.trim().toLowerCase(), 150);
  const hits = useMemo(() => {
    if (!dq) return null;
    const digits = dq.replace(/\D/g, '');
    const pages = NAV_FLAT.filter(n => t('nav.' + n.key).toLowerCase().includes(dq));
    const farmers = [], farms = [];
    for (const f of db.farmers.all()) {
      if (farmers.length >= 8) break;
      if (f.name.en.toLowerCase().includes(dq) || f.name.ku.includes(dq) || (digits.length >= 3 && f.phone.includes(digits))) farmers.push(f);
    }
    for (const f of db.farms.all()) {
      if (farms.length >= 8) break;
      if (f.id === dq || f.name.includes(dq)) farms.push(f);
    }
    return { pages, farmers, farms };
  }, [dq, t]);
  const go = (path: string) => { onClose(); nav(path); };
  return (
    <Modal title={t('search.open')} onClose={onClose}>
      <input type="search" autoFocus value={q} onChange={e => setQ(e.target.value)} placeholder={t('search.placeholder')} style={{ minHeight: 44, fontSize: 15 }} />
      <div style={{ maxHeight: 380, overflowY: 'auto', marginTop: 8 }}>
        {hits && !hits.pages.length && !hits.farmers.length && !hits.farms.length && <div className="empty">{t('search.none')}</div>}
        {hits?.pages.map(p => <a key={p.key} className="list-item" onClick={() => go(p.path)} href={'#' + p.path}><p.icon /><b>{t('nav.' + p.key)}</b></a>)}
        {hits?.farmers.map(f => <a key={f.id} className="list-item" onClick={() => go('/admin/farms?farmer=' + f.id)} href={'#/admin/farms?farmer=' + f.id}><User /><span>{b(f.name)} <span className="muted small ltr">{f.phone}</span></span></a>)}
        {hits?.farms.map(f => <a key={f.id} className="list-item" onClick={() => go('/admin/farms?farm=' + f.id)} href={'#/admin/farms?farm=' + f.id}><MapIcon /><span>{t('search.farm')} #{f.id} · <span className="ku-text">{f.name}</span></span></a>)}
      </div>
    </Modal>
  );
}
