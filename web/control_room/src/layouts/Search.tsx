// Search pages and farmers (name or phone, on the server). Press "/" anywhere in the admin.
import { useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { User } from 'lucide-react';
import { useI18n } from '../i18n';
import { useAuth } from '../auth/auth';
import { useApi } from '../api/cache';
import { qs } from '../api/client';
import { Modal, useDebounced } from '../components/ui';
import { Phone } from '../components/domain';
import { NAV_FLAT } from './nav';

interface FarmerRow { id: string; name?: string | null; phone: string }

export function SearchBox({ onClose }: { onClose: () => void }) {
  const { t } = useI18n();
  const { can } = useAuth();
  const nav = useNavigate();
  const [q, setQ] = useState('');
  const dq = useDebounced(q.trim(), 250);
  const pages = useMemo(() => (dq ? NAV_FLAT.filter(n => (!n.needs || can(n.needs)) && t('nav.' + n.key).toLowerCase().includes(dq.toLowerCase())) : []), [dq, t, can]);
  const farmers = useApi<{ farmers: FarmerRow[] }>(dq.length >= 2 && can('farmers') ? '/dashboard/farmers' + qs({ q: dq, rows_per_page: 8 }) : null, ['farmers'], { auth: true });
  const go = (path: string) => { onClose(); nav(path); };
  const list = farmers.data?.farmers ?? [];
  return (
    <Modal title={t('search.open')} onClose={onClose}>
      <input type="search" autoFocus value={q} onChange={e => setQ(e.target.value)} placeholder={t('search.placeholder')} style={{ minHeight: 46, fontSize: 16 }} />
      <div style={{ maxHeight: 380, overflowY: 'auto', marginTop: 8 }}>
        {dq && !pages.length && !list.length && !farmers.loading && <div className="empty">{t('search.none')}</div>}
        {pages.map(p => <button key={p.key} className="list-item click" style={{ width: '100%', border: 0, background: 'none' }} onClick={() => go(p.path)}><p.icon /><b>{t('nav.' + p.key)}</b></button>)}
        {list.map(f => <button key={f.id} className="list-item click" style={{ width: '100%', border: 0, background: 'none' }} onClick={() => go('/admin/farms?farmer=' + f.id)}><User /><span>{f.name || t('farms.no_name')} · <Phone value={f.phone} /></span></button>)}
      </div>
    </Modal>
  );
}
