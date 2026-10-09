// The crop codes the server accepts today (backend/API.md). Names and colours for the site; when the
// server's crops table arrives (GET /v1/crops, FRONTEND.md 14) the site reads them from there instead.
export interface CropInfo { code: string; en: string; ku: string; color: string }
export const CROPS: CropInfo[] = [
  { code: 'wheat', en: 'Wheat', ku: 'گەنم', color: '#E0B13A' },
  { code: 'barley', en: 'Barley', ku: 'جۆ', color: '#C8B560' },
  { code: 'chickpea', en: 'Chickpea', ku: 'نۆک', color: '#D9B88A' },
  { code: 'tomato', en: 'Tomato', ku: 'تەماتە', color: '#D9483B' },
  { code: 'cucumber', en: 'Cucumber', ku: 'خەیار', color: '#6DB352' },
  { code: 'potato', en: 'Potato', ku: 'پەتاتە', color: '#A9784A' },
  { code: 'onion', en: 'Onion', ku: 'پیاز', color: '#B46FA8' },
  { code: 'watermelon', en: 'Watermelon', ku: 'شووتی', color: '#EF7C8E' },
  { code: 'grape', en: 'Grape', ku: 'ترێ', color: '#7E57C2' },
  { code: 'olive', en: 'Olive', ku: 'زەیتوون', color: '#7D8B3A' },
  { code: 'sunflower', en: 'Sunflower', ku: 'گوڵەبەڕۆژە', color: '#F5C518' },
  { code: 'pomegranate', en: 'Pomegranate', ku: 'هەنار', color: '#C2185B' },
  { code: 'okra', en: 'Okra', ku: 'بامیە', color: '#5E8C3A' },
  { code: 'eggplant', en: 'Eggplant', ku: 'باینجان', color: '#5B3A7A' },
  { code: 'pepper', en: 'Pepper', ku: 'بیبەر', color: '#C0392B' },
  { code: 'apple', en: 'Apple', ku: 'سێو', color: '#D35454' },
  { code: 'empty', en: 'Not planted', ku: 'نەچێنراو', color: '#D9D6CC' },
];
export const CROP = new Map(CROPS.map(c => [c.code, c]));
