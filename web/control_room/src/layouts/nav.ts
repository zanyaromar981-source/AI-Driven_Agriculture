// The admin pages in sidebar order (design: 06 to 20). A page shows only when the staff member can
// read its resource (FRONTEND.md 8); null = every signed-in staff member.
import {
  LayoutDashboard, Users, IdCard, ShieldCheck, Wheat, Map, BellRing, Inbox, Store, Smartphone, SlidersHorizontal,
  Activity, Settings, type LucideIcon,
} from 'lucide-react';
import type { Resource } from '../api/types';

export interface NavItem { key: string; path: string; icon: LucideIcon; needs: Resource | null }
export const NAV: { group: string; items: NavItem[] }[] = [
  { group: '', items: [{ key: 'overview', path: '/admin', icon: LayoutDashboard, needs: null }] },
  { group: 'people', items: [
    { key: 'farms', path: '/admin/farms', icon: Users, needs: 'farmers' },
    { key: 'staff', path: '/admin/staff', icon: IdCard, needs: 'staff' },
    { key: 'roles', path: '/admin/roles', icon: ShieldCheck, needs: 'roles' },
  ] },
  { group: 'fields', items: [
    { key: 'crops', path: '/admin/crops', icon: Wheat, needs: 'crops' },
    { key: 'region', path: '/admin/region', icon: Map, needs: 'zones' },
  ] },
  { group: 'act', items: [
    { key: 'alerts', path: '/admin/alerts', icon: BellRing, needs: null },
    { key: 'inbox', path: '/admin/inbox', icon: Inbox, needs: 'messages' },
    { key: 'alwa', path: '/admin/alwa', icon: Store, needs: 'alwa' },
  ] },
  { group: 'app', items: [
    { key: 'app', path: '/admin/app', icon: Smartphone, needs: 'app' },
    { key: 'rules', path: '/admin/rules', icon: SlidersHorizontal, needs: 'rules' },
  ] },
  { group: 'system', items: [
    { key: 'jobs', path: '/admin/jobs', icon: Activity, needs: 'jobs' },
    { key: 'settings', path: '/admin/settings', icon: Settings, needs: null },
  ] },
];
export const NAV_FLAT = NAV.flatMap(g => g.items);
