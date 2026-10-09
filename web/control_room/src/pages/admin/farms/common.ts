// Shapes and helpers for the farmers, farms, letter and report pages (backend/API.md, API 1.6.0).
import { CROP } from '../../../data/crops';

export type FarmerLang = 'ku' | 'kmr' | 'ar' | 'en';
export interface Farmer {
  id: string; phone: string; name?: string | null; lang: FarmerLang; farms_count: number;
  gender?: 'male' | 'female' | null; birth_year?: number | null; village?: string | null;
  governorate?: string | null; zone_slug?: string | null; sub_zone_slug?: string | null;
  notes?: string | null; blocked: boolean; created_at: string; updated_at: string;
}
export interface CropArea { crop: string; dunam: number }
export interface FarmSummary {
  id: string; name: string; area_dunam: number; crops: CropArea[]; centroid: { lat: number; lon: number };
  governorate?: string | null; zone_slug?: string | null; sub_zone_slug?: string | null; created_at: string; owner_phone?: string;
}
export interface FarmDetail extends FarmSummary { outline?: { lat: number; lon: number }[]; owner_phone?: string; updated_at?: string }

export interface LetterFarm { id: string; name: string; area_dunam: number; crops: CropArea[]; governorate?: string | null; zone_slug?: string | null; sub_zone_slug?: string | null }
export interface IssuedLetter {
  number: string; purpose: string; lang: 'ku' | 'en'; created_at: string;
  issued_by: { id: string; name: string };
  farmer: { id: string; name?: string | null; phone: string; gender?: 'male' | 'female' | null; birth_year?: number | null; village?: string | null; governorate?: string | null; zone_slug?: string | null; sub_zone_slug?: string | null };
  farms: LetterFarm[];
  totals: { farms: number; dunam: number; crops: CropArea[] };
}
export interface LetterRecord { number: string; purpose: string; lang: 'ku' | 'en'; created_at: string; farmer_id: string; farmer_name?: string | null; staff_id: string }

/**
 * Expected harvest in tonnes from the server's yield for this crop (GET /v1/crops, yield_kg_per_dunam).
 * null when the server has no yield for it: the site never invents one, the cell shows "-".
 */
export const expectedTonnes = (crop: string, dunam: number): number | null => {
  const y = CROP.get(crop)?.yieldKgPerDunam;
  return y ? y * dunam / 1000 : null;
};
/** Sum of known expected harvests; null when none of the crops has a yield. */
export const sumTonnes = (rows: { crop: string; dunam: number }[]) => {
  const v = rows.map(r => expectedTonnes(r.crop, r.dunam)).filter((x): x is number => x != null);
  return v.length ? v.reduce((a, b) => a + b, 0) : null;
};

/** Fresh key per create form, reused on retries of the same form (FRONTEND.md 3). */
export const newKey = () => (typeof crypto !== 'undefined' && 'randomUUID' in crypto ? crypto.randomUUID() : Date.now().toString(36) + Math.random().toString(36).slice(2));

/** Excel-friendly CSV (UTF-8 with BOM so Kurdish shows correctly). */
export function downloadCsv(name: string, header: string[], rows: (string | number | null | undefined)[][]) {
  const cell = (v: string | number | null | undefined) => { const s = v == null ? '' : String(v); return /[",\n]/.test(s) ? `"${s.replace(/"/g, '""')}"` : s; };
  const text = '﻿' + [header, ...rows].map(r => r.map(cell).join(',')).join('\r\n');
  const a = document.createElement('a');
  a.href = URL.createObjectURL(new Blob([text], { type: 'text/csv;charset=utf-8' }));
  a.download = name + '.csv'; a.click();
  setTimeout(() => URL.revokeObjectURL(a.href), 1000);
}

/** Normalise an Iraqi mobile number to +9647XXXXXXXXX. */
export function normPhone(p: string) {
  let s = p.replace(/[\s-]/g, '');
  if (s.startsWith('00964')) s = '+' + s.slice(2);
  if (s.startsWith('07')) s = '+964' + s.slice(1);
  if (s.startsWith('9647')) s = '+' + s;
  return s;
}
export const phoneOk = (p: string) => /^\+9647\d{9}$/.test(p);
