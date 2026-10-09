// Crop names and colours come from the server (GET /v1/crops, topic crops; FRONTEND.md 4). This list is
// only the fallback while that answer is not loaded yet or the server cannot be reached. Yields are
// never invented: they come from the server and are often null.
export interface CropInfo { code: string; en: string; ku: string; color: string; yieldKgPerDunam?: number | null; active?: boolean }
export const FALLBACK_CROPS: CropInfo[] = [
  { code: 'wheat', en: 'Wheat', ku: 'گەنم', color: '#E0B13A' }, { code: 'barley', en: 'Barley', ku: 'جۆ', color: '#C8B560' },
  { code: 'tomato', en: 'Tomato', ku: 'تەماتە', color: '#D9483B' }, { code: 'cucumber', en: 'Cucumber', ku: 'خەیار', color: '#6DB352' },
  { code: 'potato', en: 'Potato', ku: 'پەتاتە', color: '#A9784A' }, { code: 'onion', en: 'Onion', ku: 'پیاز', color: '#B46FA8' },
  { code: 'watermelon', en: 'Watermelon', ku: 'شووتی', color: '#EF7C8E' }, { code: 'grape', en: 'Grape', ku: 'ترێ', color: '#7E57C2' },
  { code: 'olive', en: 'Olive', ku: 'زەیتوون', color: '#7D8B3A' }, { code: 'sunflower', en: 'Sunflower', ku: 'گوڵەبەڕۆژە', color: '#F5C518' },
  { code: 'chickpea', en: 'Chickpea', ku: 'نۆک', color: '#D9B88A' }, { code: 'pomegranate', en: 'Pomegranate', ku: 'هەنار', color: '#C2185B' },
  { code: 'okra', en: 'Okra', ku: 'بامیە', color: '#5E8C3A' }, { code: 'eggplant', en: 'Eggplant', ku: 'باینجان', color: '#5B3A7A' },
  { code: 'pepper', en: 'Pepper', ku: 'بیبەر', color: '#C0392B' }, { code: 'apple', en: 'Apple', ku: 'سێو', color: '#D35454' },
];
const EMPTY: CropInfo = { code: 'empty', en: 'Not planted', ku: 'نەچێنراو', color: '#D9D6CC' };

/** Live table, replaced when /v1/crops answers (see useCrops in components/domain.tsx). */
export let CROPS: CropInfo[] = FALLBACK_CROPS;
export let CROP = new Map([...FALLBACK_CROPS, EMPTY].map(c => [c.code, c]));
export function setServerCrops(list: { code: string; name_en: string; name_ku?: string | null; color: string; yield_kg_per_dunam?: number | null; active?: boolean }[]) {
  CROPS = list.map(c => ({ code: c.code, en: c.name_en, ku: c.name_ku || FALLBACK_CROPS.find(f => f.code === c.code)?.ku || c.name_en, color: c.color, yieldKgPerDunam: c.yield_kg_per_dunam ?? null, active: c.active }));
  CROP = new Map([...CROPS, EMPTY].map(c => [c.code, c]));
}
