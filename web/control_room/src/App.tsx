// Routes. Public View page at "/", staff sign in at "/login" and work under "/admin" (design 00).
// Every page is its own chunk. The intro plays only on the first visit (empty cache).
import { lazy, Suspense, useEffect, useState } from 'react';
import { HashRouter, Navigate, Route, Routes } from 'react-router-dom';
import { I18nProvider } from './i18n';
import { AuthProvider, RequireStaff } from './auth/auth';
import { ToastProvider, Skeleton } from './components/ui';
import { Preloader, introWanted } from './motion/Preloader';
import { AdminLayout } from './layouts/AdminLayout';
import { startVersionWatch } from './api/cache';
import { Link } from 'react-router-dom';
import { StateBox } from './components/domain';
import { useI18n } from './i18n';

/** An unknown /admin/... address: say so instead of landing on the public page. */
function AdminNotFound() {
  const { t } = useI18n();
  return <section className="card"><StateBox kind="empty" title={t('state.notfound_title')} text={t('state.notfound_text')} action={<Link className="btn" to="/admin">{t('state.notfound_back')}</Link>} /></section>;
}

const View = lazy(() => import('./pages/view/ViewPage'));
const Login = lazy(() => import('./pages/Login'));
const Overview = lazy(() => import('./pages/admin/Overview'));
const Farms = lazy(() => import('./pages/admin/Farms'));
const Staff = lazy(() => import('./pages/admin/Staff'));
const Roles = lazy(() => import('./pages/admin/Roles'));
const Crops = lazy(() => import('./pages/admin/Crops'));
const Region = lazy(() => import('./pages/admin/Region'));
const Alerts = lazy(() => import('./pages/admin/Alerts'));
const Inbox = lazy(() => import('./pages/admin/Inbox'));
const Alwa = lazy(() => import('./pages/admin/Alwa'));
const AppControl = lazy(() => import('./pages/admin/AppControl'));
const Rules = lazy(() => import('./pages/admin/Rules'));
const Jobs = lazy(() => import('./pages/admin/Jobs'));
const SettingsPage = lazy(() => import('./pages/admin/Settings'));
const SupportLetter = lazy(() => import('./pages/print/SupportLetter'));
const GovReport = lazy(() => import('./pages/print/GovReport'));
const CropReport = lazy(() => import('./pages/print/CropReport'));

export function App() {
  const [intro, setIntro] = useState(introWanted);
  useEffect(() => startVersionWatch(), []);
  return (
    <I18nProvider>
      <AuthProvider>
        <ToastProvider>
          <HashRouter>
            <Suspense fallback={<div style={{ padding: 24 }}><Skeleton /></div>}>
              <Routes>
                <Route path="/" element={<View />} />
                <Route path="/login" element={<Login />} />
                <Route path="/admin" element={<RequireStaff><AdminLayout /></RequireStaff>}>
                  <Route index element={<Overview />} />
                  <Route path="farms" element={<Farms />} />
                  <Route path="staff" element={<Staff />} />
                  <Route path="roles" element={<Roles />} />
                  <Route path="crops" element={<Crops />} />
                  <Route path="region" element={<Region />} />
                  <Route path="alerts" element={<Alerts />} />
                  <Route path="inbox" element={<Inbox />} />
                  <Route path="alwa" element={<Alwa />} />
                  <Route path="app" element={<AppControl />} />
                  <Route path="rules" element={<Rules />} />
                  <Route path="jobs" element={<Jobs />} />
                  <Route path="settings" element={<SettingsPage />} />
                  <Route path="*" element={<AdminNotFound />} />
                </Route>
                <Route path="/print/letter/:farmerId" element={<RequireStaff><SupportLetter /></RequireStaff>} />
                <Route path="/print/government" element={<RequireStaff><GovReport /></RequireStaff>} />
                <Route path="/print/crops" element={<RequireStaff><CropReport /></RequireStaff>} />
                <Route path="*" element={<Navigate to="/" replace />} />
              </Routes>
            </Suspense>
          </HashRouter>
          {intro && <Preloader onDone={() => setIntro(false)} />}
        </ToastProvider>
      </AuthProvider>
    </I18nProvider>
  );
}
