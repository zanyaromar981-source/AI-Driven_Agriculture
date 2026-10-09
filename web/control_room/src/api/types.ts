// Shapes shared by several pages, copied from backend/API.md (API 1.6.0). Page-specific shapes live
// next to their page. Field names stay snake_case, as the server sends them.

export type Band = 'much_greener' | 'greener' | 'normal' | 'dry' | 'very_dry';
export type Resource = 'zones' | 'dams' | 'outlooks' | 'water' | 'fires' | 'alwa' | 'farmers' | 'farms' | 'insights' | 'staff' | 'roles' | 'briefs' | 'crops' | 'rules' | 'messages' | 'app' | 'jobs';
export type Action = 'create' | 'read' | 'update' | 'delete';
export interface Permission { resource: Resource; action: Action }
export interface RoleRef { id: string; name: string }
export interface Staff { id: string; email: string; name: string; active: boolean; job_title?: string | null; phone?: string | null; roles: RoleRef[]; created_at: string; updated_at: string }
export interface SignedIn { token: string; staff: Staff; permissions: Permission[] }
export interface Me { staff: Staff; permissions: Permission[] }

export interface ZoneOverview { slug: string; name_en: string; name_ku: string; governorate: string; band?: Band | null; dryness?: number | null; rank?: number | null; change_vs_last_year?: number | null; water_need?: number | null; nitrogen_hold?: boolean | null }
export interface RegionOverview { month: string; summary: { average_dryness?: number | null; change_vs_last_year?: number | null; driest: string[]; nitrogen_hold: string[]; zones_with_data: number }; zones: ZoneOverview[] }
export interface SubZoneDryness { slug: string; name_en: string; name_ku: string; band?: Band | null; dryness?: number | null }
export interface ZoneDetail {
  slug: string; name_en: string; name_ku: string; governorate: string; month: string;
  reading?: { band: Band; dryness: number; rank: number; rank_of: number; source: string; updated_at: string; greenness_pct_vs_normal?: number | null; rain_pct_of_normal?: number | null; water_need?: number | null; nitrogen_hold: boolean; best_crops: string[] } | null;
  history: { year: number; dryness: number }[]; sub_zones: SubZoneDryness[];
}
export interface RegionCompare { month: number; year: number; with: number; region: { year: number; average_dryness: number }[]; zones: { slug: string; name_en: string; name_ku: string; dryness?: number | null; dryness_with?: number | null; change?: number | null }[] }

export interface Dam { slug: string; name_en: string; name_ku: string; capacity_bn_m3: number; latest?: { day: string; pct_full: number; volume_bn_m3?: number | null; lake_area_km2?: number | null; farm_supply_bn_m3?: number | null; source: string } | null; year_ago?: { day: string; pct_full: number } | null }
export interface DamHistory { slug: string; readings: { day: string; pct_full: number; volume_bn_m3?: number | null }[] }

export type FireStatus = 'active' | 'spreading' | 'under_control' | 'out';
export interface Fire { id: string; lat: number; lon: number; zone_slug?: string | null; place_en?: string | null; place_ku?: string | null; detected_at: string; status: FireStatus; source: string; area_ha?: number | null; farms_within_5km?: number | null; farmers_alerted?: number | null; wind_kmh?: number | null }
export interface Fires { hours: number; fires: Fire[]; summary: { active: number; under_control: number; area_ha: number; farms_within_5km: number; farmers_alerted: number; zones: string[] } }

export interface BriefPoint { level: 'info' | 'watch' | 'alarm'; text_en: string; text_ku: string }
export interface Brief { day: string; scope: string; headline_en: string; headline_ku: string; summary_en: string; summary_ku: string; points: BriefPoint[]; sources: { title: string; url: string }[]; author: string; generated_at: string; updated_at: string }

export interface StatsCrop { crop: string; dunam: number; farms: number; farmers?: number }
export interface StatsArea { slug: string; name_en: string; name_ku: string; governorate?: string | null; zone_slug?: string | null; farmers: number; farms: number; dunam: number; crops: StatsCrop[] }
export interface FarmStats { as_of: string; totals: { farmers: number; farms: number; dunam: number }; by_governorate: StatsArea[]; by_zone: StatsArea[]; by_sub_zone?: StatsArea[]; by_crop: (StatsCrop & { farmers: number })[] }

export interface Paged { count: number; page: number; rows_per_page: number }
