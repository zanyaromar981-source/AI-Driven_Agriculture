// The admin frame (design 06 to 20): top bar, news bar, the page, and the sidebar on the inline-start
// side (right in Kurdish). The sidebar can collapse to icons (tablet default) and becomes a sheet on
// phones. Only pages the staff member may read are listed.
import { Suspense, useEffect, useState } from 'react';
import { NavLink, Outlet, Link, useLocation, useNavigate } from 'react-router-dom';
import { Menu, X, ChevronsLeft, LogOut, Globe, Search, ChevronDown, UserRound } from 'lucide-react';
import { useI18n } from '../i18n';
import { useAuth } from '../auth/auth';
import { prefs } from '../data/store';
import { useApi } from '../api/cache';
import { GrainSun } from '../motion/GrainSun';
import { Ticker } from '../motion/Ticker';
import { Skeleton } from '../components/ui';
import { NAV } from './nav';
import { LangSwitch } from './LangSwitch';
import { SearchBox } from './Search';

const isTablet = () => matchMedia('(max-width: 1024px)').matches;
const initials = (n: string) => n.split(/\s+/).map(s => s[0]).join('').slice(0, 2).toUpperCase();

export function AdminLayout() {
  const { t } = useI18n();
  const { me, can, signOut } = useAuth();
  const nav = useNavigate();
  const loc = useLocation();
  const counts = useApi<{ new: number }>(can('messages') ? '/dashboard/messages/counts' : null, ['messages'], { auth: true });
  const inboxNew = counts.data?.new ?? 0;
  const [mini, setMini] = useState<boolean>(() => prefs.get('side.mini', isTablet()));
  const [open, setOpen] = useState(false);
  const [menu, setMenu] = useState(false);
  const [search, setSearch] = useState(false);

  useEffect(() => { setOpen(false); setMenu(false); window.scrollTo(0, 0); }, [loc.pathname]);
  useEffect(() => {
    const k = (e: KeyboardEvent) => {
      if (e.key === '/' && !/input|textarea|select/i.test((document.activeElement as HTMLElement)?.tagName)) { e.preventDefault(); setSearch(true); }
    };
    addEventListener('keydown', k); return () => removeEventListener('keydown', k);
  }, []);
  const toggleMini = () => setMini(m => { prefs.set('side.mini', !m); return !m; });
  if (!me) return null;
  const roles = me.roles.map(r => r.name).join(' · ');

  return (
    <>
      <header className="top">
        <button className="icon-btn menu-btn" onClick={() => setOpen(true)} aria-label={t('common.menu')}><Menu /></button>
        <Link to="/admin" className="logo"><span className="mark"><GrainSun size={26} /></span><span className="word-ku">جوتیار</span><span className="word-en-full">Jutyar</span><span className="tag">{t('common.control_room')}</span></Link>
        <div className="top-end">
          <button className="search-btn" onClick={() => setSearch(true)} aria-label={t('search.open')}><Search /><span>{t('search.placeholder')}</span><kbd>/</kbd></button>
          <LangSwitch />
          <Link to="/" className="icon-btn globe-btn" title={t('common.view_site')} aria-label={t('common.view_site')}><Globe /></Link>
          <button className="who" onClick={e => { e.stopPropagation(); setMenu(m => !m); }} aria-expanded={menu}>
            <span className="avatar">{initials(me.name)}</span>
            <span className="who-text"><b>{me.name}</b><small>{roles}</small></span>
            <ChevronDown size={14} />
          </button>
          {menu && (
            <div className="menu" onClick={e => e.stopPropagation()}>
              <Link to="/admin/settings"><UserRound /><span className="small">{me.email}</span></Link>
              <hr />
              <button onClick={() => { signOut(); nav('/login'); }}><LogOut />{t('common.sign_out')}</button>
            </div>
          )}
        </div>
      </header>
      <Ticker />
      <div className="shell" onClick={() => setMenu(false)}>
        {open && <div className="overlay" style={{ zIndex: 320 }} onClick={() => setOpen(false)} />}
        <nav className={'side' + (mini ? ' mini' : '') + (open ? ' open' : '')} aria-label={t('common.menu')}>
          <div className="side-head"><span className="logo"><span className="mark"><GrainSun size={24} /></span>جوتیار</span><button className="icon-btn" onClick={() => setOpen(false)} aria-label={t('common.close')}><X /></button></div>
          <div className="side-scroll">
            {NAV.map(g => {
              const items = g.items.filter(i => !i.needs || can(i.needs));
              if (!items.length) return null;
              return (
                <div key={g.group || 'home'}>
                  {g.group && <h6>{t('nav.g_' + g.group)}</h6>}
                  {items.map(i => {
                    const label = t('nav.' + i.key), badge = i.key === 'inbox' ? inboxNew : 0;
                    return (
                      <NavLink key={i.key} to={i.path} end={i.path === '/admin'} className={({ isActive }) => 'nav' + (isActive ? ' on' : '')} data-tip={label}>
                        <i.icon /><span className="label">{label}</span>
                        {badge > 0 && <><span className="badge pill danger">{badge}</span><span className="mini-dot" /></>}
                      </NavLink>
                    );
                  })}
                </div>
              );
            })}
          </div>
          <div className="side-foot">
            <button onClick={toggleMini} aria-label={mini ? t('common.expand') : t('common.collapse')}><ChevronsLeft className="collapse-ico" /><span className="label">{t('common.collapse')}</span></button>
          </div>
        </nav>
        <main className="content" id="main">
          <Suspense fallback={<Skeleton />}>
            <div className="page-enter" key={loc.pathname}><Outlet /></div>
          </Suspense>
        </main>
      </div>
      {search && <SearchBox onClose={() => setSearch(false)} />}
    </>
  );
}
