// Fires tab: satellite fire detections (not confirmed fires, FRONTEND.md 11), paged by the server, with
// a district and status filter, the points on the map, and a status change (PUT sends the whole fire).
import { useMemo, useState } from 'react';
import { useI18n } from '../../../i18n';
import { useAuth } from '../../../auth/auth';
import { useApi, invalidate } from '../../../api/cache';
import { api, ApiError, qs } from '../../../api/client';
import { DataTable, type Col } from '../../../components/DataTable';
import { DistrictMap } from '../../../components/DistrictMap';
import { StateBox, usePlace, usePlaceOptions, useErrorText } from '../../../components/domain';
import { Pill, Select, useToast, type Tone } from '../../../components/ui';
import type { DashFire } from './types';

const STATUSES = ['active', 'spreading', 'under_control', 'out'] as const;
const TONE: Record<DashFire['status'], Tone> = { active: 'danger', spreading: 'danger', under_control: 'warn', out: 'good' };

export function Fires() {
  const { t, num, date, ago } = useI18n();
  const { can } = useAuth();
  const place = usePlace();
  const opts = usePlaceOptions('');
  const toast = useToast();
  const errText = useErrorText();
  const [page, setPage] = useState(1);
  const [status, setStatus] = useState('');
  const [zone, setZone] = useState('');
  const [busy, setBusy] = useState<string | null>(null);
  const path = '/dashboard/fires' + qs({ page, rows_per_page: 25, status, zone_slug: zone });
  const q = useApi<{ fires: DashFire[]; count: number }>(path, ['fires'], { auth: true });
  const rows = q.data?.fires ?? [];
  const points = useMemo(() => rows.map(f => ({ id: f.id, lat: f.lat, lon: f.lon, color: f.status === 'out' ? '#7A8A80' : '#E0533F', label: place.dist(f.zone_slug) + ' · ' + date(f.detected_at, 'datetime') })), [rows, place, date]);

  const setFireStatus = async (f: DashFire, s: DashFire['status']) => {
    if (s === f.status) return;
    setBusy(f.id);
    try {
      const { id: _i, external_id: _e, updated_at: _u, ...body } = f;
      await api.put(`/dashboard/fires/${f.id}`, { ...body, status: s });
      invalidate('fires');
      toast(t('common.saved'), 'good');
    } catch (e) { toast(e instanceof ApiError ? errText(e) : t('err.generic'), 'danger'); } finally { setBusy(null); }
  };

  if (q.error && !q.data) return <StateBox kind="error" action={<button className="btn" onClick={q.reload}>{t('common.retry')}</button>} />;

  const cols: Col<DashFire>[] = [
    { key: 'when', label: t('region.c_seen'), cell: f => <div><b>{ago(f.detected_at)}</b><div className="muted small">{date(f.detected_at, 'datetime')}</div></div> },
    { key: 'zone', label: t('region.c_district'), cell: f => place.dist(f.zone_slug) || <bdi>{f.place_en}</bdi> },
    { key: 'pt', label: t('region.c_point'), cell: f => <span className="ltr mono">{f.lat.toFixed(3)}, {f.lon.toFixed(3)}</span>, optional: true },
    { key: 'near', label: t('region.c_near'), num: true, cell: f => (f.farms_within_5km == null ? '-' : num(f.farms_within_5km)) },
    { key: 'st', label: t('common.status'), cell: f => (can('fires', 'update')
      ? <Select value={f.status} disabled={busy === f.id} onChange={v => setFireStatus(f, v as DashFire['status'])} options={STATUSES.map(s => [s, t('region.fs_' + s)])} />
      : <Pill tone={TONE[f.status]}>{t('region.fs_' + f.status)}</Pill>) },
    { key: 'src', label: t('common.source'), cell: f => <bdi className="muted small rg-src" title={f.source}>{f.source}</bdi>, optional: true },
  ];

  return (
    <div className="stack">
      <section className="card">
        <div className="card-head"><span className="eyebrow">{t('region.fires_map')}</span><span className="muted small">{t('region.fires_note')}</span></div>
        <DistrictMap size="sm" styleKey="fires" fill={() => '#E7EAE3'} points={points} />
      </section>
      <section className="card">
        <DataTable id="region-fires" rows={rows} cols={cols} loading={q.loading}
          head={<div className="filters" style={{ margin: 0 }}>
            <Select value={zone} onChange={v => { setZone(v); setPage(1); }} options={[['', t('common.all_dists')], ...opts.dists]} />
            <Select value={status} onChange={v => { setStatus(v); setPage(1); }} options={[['', t('region.all_status')], ...STATUSES.map(s => [s, t('region.fs_' + s)] as [string, string])]} />
          </div>}
          server={{ total: q.data?.count ?? 0, page, onPage: setPage }} />
      </section>
    </div>
  );
}
