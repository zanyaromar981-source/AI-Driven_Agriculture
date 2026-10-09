// The 4 governorates, 33 districts and 72 sub-districts (same as the server, FRONTEND.md 3).
// The server names a district by slug: its English name in lower case with hyphens.
import placesJson from './places.json';

export interface Place { en: string; ku: string; c: [number, number] }
export interface District extends Place { gov: string; slug: string }
export interface SubDistrict extends Place { gov: string; dist: string; slug: string }

export const slugify = (en: string) => en.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
const raw = placesJson as { governorates: Place[]; districts: Omit<District, 'slug'>[]; subdistricts: Omit<SubDistrict, 'slug'>[] };
export const GOVERNORATES = raw.governorates.map(g => ({ ...g, slug: slugify(g.en) }));
export const DISTRICTS: District[] = raw.districts.map(d => ({ ...d, slug: slugify(d.en) }));
export const SUBDISTRICTS: SubDistrict[] = raw.subdistricts.map(s => ({ ...s, slug: slugify(s.en) }));
export const DISTRICT_BY_SLUG = new Map(DISTRICTS.map(d => [d.slug, d]));
export const DISTRICT_BY_EN = new Map(DISTRICTS.map(d => [d.en, d]));
export const GOV_BY_NAME = new Map(GOVERNORATES.flatMap(g => [[g.en, g], [g.slug, g]] as [string, typeof g][]));
export const SUB_BY_SLUG = new Map(SUBDISTRICTS.map(s => [s.slug, s]));
