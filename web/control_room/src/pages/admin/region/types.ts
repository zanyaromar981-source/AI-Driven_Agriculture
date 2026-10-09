// Shapes used by Overview, Region data and Alerts (backend/API.md, API 1.6.0).
import type { Band } from '../../../api/types';

export interface JobStatus {
  job: string; name_en: string; name_ku?: string | null; every_hours?: number | null;
  last_run?: { started_at: string; finished_at?: string | null; ok?: boolean | null; rows?: number | null; message?: string | null } | null;
  last_ok?: string | null; next_due?: string | null; state: 'ok' | 'late' | 'failed' | 'never';
  last_14_days: ('ok' | 'late' | 'failed' | null)[]; message?: string | null;
}
export interface ZoneReading {
  zone_slug: string; month: string; dryness: number; band: Band; rain_pct_of_normal?: number | null; greenness_pct_vs_normal?: number | null;
  water_need?: number | null; nitrogen_hold: boolean; best_crops: string[]; source: string; updated_at: string;
}
export interface DamRef { slug: string; name_en: string; name_ku: string; capacity_bn_m3: number }
export interface DamReading { day: string; pct_full: number; volume_bn_m3?: number | null; lake_area_km2?: number | null; farm_supply_bn_m3?: number | null; source: string; updated_at: string }
export interface DashFire {
  id: string; external_id: string; lat: number; lon: number; zone_slug?: string | null; place_en?: string | null; place_ku?: string | null;
  detected_at: string; area_ha?: number | null; wind_kmh?: number | null; wind_direction?: string | null; status: 'active' | 'spreading' | 'under_control' | 'out';
  farms_within_5km?: number | null; farmers_alerted?: number | null; source: string; updated_at: string;
}
export interface Outlooks { season: string; issued: string; counts: { good: number; normal: number; bad: number }; zones: { zone_slug: string; outlook: 'good' | 'normal' | 'bad'; confidence_pct: number; reason_en?: string | null; reason_ku?: string | null }[]; track_record?: { method: string; seasons_right: number; seasons_tested: number } | null }
export interface WaterPlan { season: string; entries: { rank: number; zone_slug: string; need: number; urgent: boolean; dam_slug?: string | null; send_million_m3?: number | null; note_en?: string | null; note_ku?: string | null }[]; totals: { planned_million_m3: number; urgent_zones: number; by_dam: { dam_slug: string; planned_million_m3: number; zones: number }[] } }

/** A reading changed by hand carries this mark at the start of its source, so it shows as such. */
export const HAND = 'Hand: ';
export const isHand = (source: string) => source.startsWith(HAND);
