// The admin pages, in sidebar order. Labels are i18n keys "nav.<key>".
import {
  LayoutDashboard, Users, ShieldCheck, Wheat, Map, BellRing, Inbox, Stethoscope, Store, SlidersHorizontal,
  Smartphone, Languages, Activity, Settings, type LucideIcon,
} from 'lucide-react';

export interface NavItem { key: string; path: string; icon: LucideIcon }
export const NAV: { group: string; items: NavItem[] }[] = [
  { group: '', items: [{ key: 'overview', path: '/admin', icon: LayoutDashboard }] },
  { group: 'people', items: [{ key: 'farms', path: '/admin/farms', icon: Users }, { key: 'officers', path: '/admin/officers', icon: ShieldCheck }] },
  { group: 'fields', items: [{ key: 'crops', path: '/admin/crops', icon: Wheat }, { key: 'region', path: '/admin/region', icon: Map }] },
  { group: 'act', items: [{ key: 'alerts', path: '/admin/alerts', icon: BellRing }, { key: 'inbox', path: '/admin/inbox', icon: Inbox }, { key: 'doctor', path: '/admin/doctor', icon: Stethoscope }, { key: 'alwa', path: '/admin/alwa', icon: Store }] },
  { group: 'app', items: [{ key: 'rules', path: '/admin/rules', icon: SlidersHorizontal }, { key: 'app', path: '/admin/app', icon: Smartphone }, { key: 'texts', path: '/admin/texts', icon: Languages }] },
  { group: 'system', items: [{ key: 'jobs', path: '/admin/jobs', icon: Activity }, { key: 'settings', path: '/admin/settings', icon: Settings }] },
];
export const NAV_FLAT = NAV.flatMap(g => g.items);
