// Routes. The public View page is "/", admins sign in at "/login" and work under "/admin".
// Every page is its own chunk, loaded when first opened.
import { lazy, Suspense, useState } from 'react';
import { HashRouter, Navigate, Route, Routes } from 'react-router-dom';
import { I18nProvider } from './i18n';
import { AuthProvider, RequireAdmin } from './auth/auth';
import { ToastProvider, Skeleton } from './components/ui';
import { Preloader, introWanted } from './motion/Preloader';
import { Cursor } from './motion/Cursor';
import { AdminLayout } from './layouts/AdminLayout';

const View = lazy(() => import('./pages/view/ViewPage'));
const Login = lazy(() => import('./pages/Login'));
const Overview = lazy(() => import('./pages/admin/Overview'));
const Farms = lazy(() => import('./pages/admin/Farms'));
const Officers = lazy(() => import('./pages/admin/Officers'));
const Crops = lazy(() => import('./pages/admin/Crops'));
const Region = lazy(() => import('./pages/admin/Region'));
const Alerts = lazy(() => import('./pages/admin/Alerts'));
const Inbox = lazy(() => import('./pages/admin/Inbox'));
const Doctor = lazy(() => import('./pages/admin/Doctor'));
const Alwa = lazy(() => import('./pages/admin/Alwa'));
const Rules = lazy(() => import('./pages/admin/Rules'));
const AppControl = lazy(() => import('./pages/admin/AppControl'));
const Texts = lazy(() => import('./pages/admin/Texts'));
const Jobs = lazy(() => import('./pages/admin/Jobs'));
const SettingsPage = lazy(() => import('./pages/admin/Settings'));
const SupportLetter = lazy(() => import('./pages/print/SupportLetter'));
const GovReport = lazy(() => import('./pages/print/GovReport'));
const CropReport = lazy(() => import('./pages/print/CropReport'));

export function App() {
  const [intro, setIntro] = useState(introWanted);
  return (
    <I18nProvider>
      <AuthProvider>
        <ToastProvider>
          <HashRouter>
            <Suspense fallback={<div style={{ padding: 24 }}><Skeleton /></div>}>
              <Routes>
                <Route path="/" element={<View />} />
                <Route path="/login" element={<Login />} />
                <Route path="/admin" element={<RequireAdmin><AdminLayout /></RequireAdmin>}>
                  <Route index element={<Overview />} />
                  <Route path="farms" element={<Farms />} />
                  <Route path="officers" element={<Officers />} />
                  <Route path="crops" element={<Crops />} />
                  <Route path="region" element={<Region />} />
                  <Route path="alerts" element={<Alerts />} />
                  <Route path="inbox" element={<Inbox />} />
                  <Route path="doctor" element={<Doctor />} />
                  <Route path="alwa" element={<Alwa />} />
                  <Route path="rules" element={<Rules />} />
                  <Route path="app" element={<AppControl />} />
                  <Route path="texts" element={<Texts />} />
                  <Route path="jobs" element={<Jobs />} />
                  <Route path="settings" element={<SettingsPage />} />
                </Route>
                <Route path="/print/farmer/:id" element={<RequireAdmin><SupportLetter /></RequireAdmin>} />
                <Route path="/print/government" element={<RequireAdmin><GovReport /></RequireAdmin>} />
                <Route path="/print/crops" element={<RequireAdmin><CropReport /></RequireAdmin>} />
                <Route path="*" element={<Navigate to="/" replace />} />
              </Routes>
            </Suspense>
          </HashRouter>
          <Cursor />
          {intro && <Preloader onDone={() => setIntro(false)} />}
        </ToastProvider>
      </AuthProvider>
    </I18nProvider>
  );
}
