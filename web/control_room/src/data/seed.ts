// Sample data, the same every time (seeded random). Used only when the browser has no saved data yet.
// Real place names come from places.json (the KRG borders in web/map_demo). Everything else is invented.
import placesJson from './places.json';
import type {
  Farmer, Farm, Crop, Officer, Alert, Message, News, Listing, DistrictReading, Dam, Fire, Rule,
  DoctorQuestion, AnswerBankItem, AppText, Job, AppConfig, Settings, District, SubDistrict, Place, Bi,
} from './types';

export const PLACES = placesJson as { governorates: Place[]; districts: District[]; subdistricts: SubDistrict[] };

let seed = 20261009;
const rnd = () => { seed |= 0; seed = seed + 0x6D2B79F5 | 0; let t = Math.imul(seed ^ seed >>> 15, 1 | seed); t = t + Math.imul(t ^ t >>> 7, 61 | t) ^ t; return ((t ^ t >>> 14) >>> 0) / 4294967296; };
const pick = <T,>(a: readonly T[]) => a[Math.floor(rnd() * a.length)];
function pickW<T>(a: readonly T[], w: readonly number[]): T {
  let s = 0; for (const x of w) s += x;
  let r = rnd() * s;
  for (let i = 0; i < a.length; i++) { r -= w[i]; if (r <= 0) return a[i]; }
  return a[a.length - 1];
}
const gauss = () => { let u = 0, v = 0; while (!u) u = rnd(); while (!v) v = rnd(); return Math.sqrt(-2 * Math.log(u)) * Math.cos(2 * Math.PI * v); };
const round = (n: number, d = 1) => Math.round(n * 10 ** d) / 10 ** d;
const bi = (en: string, ku = ''): Bi => ({ en, ku });

const NOW = Date.now();
const iso = (msAgo: number) => new Date(NOW - msAgo).toISOString();
const DAY = 864e5, MIN = 6e4;

// ---------- crops ----------
export const seedCrops = (): Crop[] => ([
  ['wheat', 'Wheat', 'گەنم', '#E0B13A', 'cereal', 'winter', 450],
  ['barley', 'Barley', 'جۆ', '#C8B560', 'cereal', 'winter', 350],
  ['chickpea', 'Chickpea', 'نۆک', '#D9B88A', 'legume', 'winter', 150],
  ['lentil', 'Lentil', 'نیسک', '#B5835A', 'legume', 'winter', 120],
  ['tomato', 'Tomato', 'تەماتە', '#D9483B', 'vegetable', 'summer', 5000],
  ['cucumber', 'Cucumber', 'خەیار', '#6DB352', 'vegetable', 'summer', 4000],
  ['potato', 'Potato', 'پەتاتە', '#A9784A', 'vegetable', 'summer', 3500],
  ['onion', 'Onion', 'پیاز', '#B46FA8', 'vegetable', 'summer', 4000],
  ['watermelon', 'Watermelon', 'شووتی', '#EF7C8E', 'fruit', 'summer', 4000],
  ['grape', 'Grape', 'ترێ', '#7E57C2', 'fruit', 'perennial', 2000],
  ['pomegranate', 'Pomegranate', 'هەنار', '#C2185B', 'fruit', 'perennial', 2500],
  ['olive', 'Olive', 'زەیتوون', '#7D8B3A', 'oil', 'perennial', 600],
  ['sunflower', 'Sunflower', 'گوڵەبەڕۆژە', '#F5C518', 'oil', 'summer', 250],
] as const).map(([id, en, ku, color, category, season, y]) => ({ id, name: bi(en, ku), color, category, season, yieldKgPerDunam: y, active: true, notes: '' }));

const CROP_IDS = ['wheat', 'barley', 'chickpea', 'lentil', 'tomato', 'cucumber', 'potato', 'onion', 'watermelon', 'grape', 'pomegranate', 'olive', 'sunflower'];
const CROP_W = [45, 17, 4, 2, 7, 4, 5, 3, 3, 3, 1.5, 3, 1];
export const AVG_PRICE: Record<string, number> = {
  wheat: 800, barley: 450, chickpea: 1750, lentil: 1500, tomato: 750, cucumber: 600, potato: 500, onion: 400,
  watermelon: 300, grape: 1250, pomegranate: 1000, olive: 1500, sunflower: 1000,
};

// ---------- people ----------
const MALE: [string, string][] = [['Karwan', 'کاروان'], ['Rebaz', 'ڕێباز'], ['Dilshad', 'دڵشاد'], ['Hemin', 'هێمن'], ['Azad', 'ئازاد'], ['Sherko', 'شێرکۆ'], ['Bakhtiyar', 'بەختیار'],
  ['Sirwan', 'سیروان'], ['Hawre', 'هاوڕێ'], ['Kamaran', 'کامەران'], ['Omed', 'ئومێد'], ['Aram', 'ئارام'], ['Jamal', 'جەمال'], ['Salar', 'سالار'], ['Hiwa', 'هیوا'],
  ['Barzan', 'بارزان'], ['Nawzad', 'نەوزاد'], ['Mahmud', 'مەحموود'], ['Ahmad', 'ئەحمەد'], ['Ali', 'عەلی'], ['Hassan', 'حەسەن'], ['Kawa', 'کاوە'], ['Goran', 'گۆران'],
  ['Shwan', 'شوان'], ['Rizgar', 'ڕزگار'], ['Saman', 'سامان'], ['Zana', 'زانا']];
const FEMALE: [string, string][] = [['Shilan', 'شیلان'], ['Nazdar', 'نازدار'], ['Hevi', 'هیڤی'], ['Shirin', 'شیرین'], ['Lana', 'لانە'], ['Avin', 'ئاڤین'], ['Gulala', 'گوڵاڵە'],
  ['Sozan', 'سۆزان'], ['Nasrin', 'نەسرین'], ['Bahar', 'بەهار'], ['Rozhan', 'ڕۆژان'], ['Kazhal', 'کەژاڵ'], ['Tavga', 'تاڤگە'], ['Parwin', 'پەروین']];
const FATHER: [string, string][] = [...MALE, ['Aziz', 'عەزیز'], ['Omar', 'عومەر'], ['Salih', 'ساڵح'], ['Mustafa', 'مستەفا'], ['Rasul', 'ڕەسوڵ'], ['Qadir', 'قادر'], ['Hama', 'حەمە'], ['Karim', 'کەریم']];
const VILLAGES = ['Qalat', 'Kani Spi', 'Girda Rasha', 'Bnaslawa', 'Tuwi Malik', 'Sarkan', 'Kani Goma', 'Bestansur', 'Zarayan', 'Bakrajo', 'Tasluja', 'Hajiawa', 'Sitak', 'Gopala', 'Kani Watman', 'Darashakran'];
const FARM_NAMES = ['کێڵگەی سەرەوە', 'زەوی باوکم', 'کێڵگەی تەماتە', 'کێڵگەی جۆ', 'زەوی ڕووبار', 'کێڵگەی نۆک', 'کێڵگەی خوارەوە', 'زەوی گوند', 'باخی ترێ',
  'کێڵگەی نوێ', 'کێڵگەی پەتاتە', 'باخی زەیتوون', 'زەوی گەورە', 'کێڵگەی بچووک', 'زەوی کانی', 'کێڵگەی گەنم', 'زەوی دایکم', 'کێڵگەی شووتی'];
const PREFIX = ['750', '770', '751', '773', '771', '772', '780', '782'];
// where farms are dense (farm land), the rest get a small weight
const DIST_W: Record<string, number> = { Makhmur: 9, Erbil: 8, Chamchamal: 8, Sharazur: 6, Kalar: 6, Qushtapa: 5, Sumel: 5, Shekhan: 4, Koya: 4, Ranya: 3, Halabja: 3, Dukan: 3,
  Sulaymaniyah: 3, Akre: 2.5, Darbandikhan: 2, Duhok: 2, Zakho: 2, Bardarash: 2, Taqtaq: 1.5, Pshdar: 1.5, Shaqlawa: 1.2, Khurmal: 1 };

interface Generated { farmers: Farmer[]; farms: Farm[] }
let GEN: Generated | null = null;
function generate(): Generated {
  if (GEN) return GEN;
  const D = PLACES.districts, W = D.map(d => DIST_W[d.en] ?? 0.4);
  const subsBy = new Map<string, SubDistrict[]>();
  for (const s of PLACES.subdistricts) { const k = s.dist; if (!subsBy.has(k)) subsBy.set(k, []); subsBy.get(k)!.push(s); }
  const farmers: Farmer[] = [], farms: Farm[] = [];
  for (let i = 0; i < 1284; i++) {
    const d = pickW(D, W), subs = subsBy.get(d.en) ?? [], s = subs.length ? pick(subs) : null;
    const female = rnd() < 0.14, f = pick(female ? FEMALE : MALE), fa = pick(FATHER);
    farmers.push({
      id: String(1000 + i), name: bi(`${f[0]} ${fa[0]}`, `${f[1]} ${fa[1]}`),
      phone: '+964' + pick(PREFIX) + String(Math.floor(rnd() * 1e7)).padStart(7, '0'),
      gender: female ? 'female' : 'male', birthYear: rnd() < 0.8 ? 1950 + Math.floor(rnd() * 52) : null,
      gov: d.gov, dist: d.en, sub: s ? s.en : d.en, village: pick(VILLAGES), status: 'active',
      joined: iso(Math.floor(rnd() * 160 + 1) * DAY), notes: '',
    });
  }
  for (let i = 0; i < 2031; i++) {
    const owner = farmers[i < farmers.length ? i : Math.floor(rnd() * farmers.length)];
    const subs = subsBy.get(owner.dist) ?? [];
    const s = rnd() < 0.8 ? subs.find(x => x.en === owner.sub) : (subs.length ? pick(subs) : undefined);
    const c = (s ?? D.find(x => x.en === owner.dist)!).c;
    const area = round(Math.min(400, Math.max(0.4, Math.exp(1.9 + 0.95 * gauss()))));
    const main = pickW(CROP_IDS, CROP_W);
    const crops = [{ crop: main, dunam: area }];
    if (rnd() < 0.3) {
      const second = pickW(CROP_IDS.filter(x => x !== main), CROP_W.filter((_, k) => CROP_IDS[k] !== main));
      const a = round(area * (0.55 + rnd() * 0.4)); crops[0].dunam = a; crops.push({ crop: second, dunam: round(area - a) });
    }
    const irrigation = pickW(['rainfed', 'irrigated', 'mixed'] as const, [58, 30, 12]);
    const created = Math.floor(rnd() * 150 + 1) * DAY;
    farms.push({
      id: String(2400 - i), farmerId: owner.id, name: pick(FARM_NAMES),
      gov: owner.gov, dist: owner.dist, sub: s ? s.en : owner.sub, village: owner.village,
      lat: round(c[0] + (rnd() - 0.5) * 0.12, 5), lon: round(c[1] + (rnd() - 0.5) * 0.12, 5),
      area, crops, irrigation,
      water: irrigation === 'rainfed' ? 'rain' : pickW(['well', 'river', 'canal', 'spring'] as const, [50, 22, 20, 8]),
      ownership: pickW(['owned', 'rented', 'shared'] as const, [72, 20, 8]),
      level: pickW(['normal', 'watch', 'alarm', 'none'] as const, [70, 14, 3, 13]),
      created: iso(created), updated: iso(Math.floor(rnd() * created)), notes: '',
    });
  }
  farmers.slice(40, 42).forEach(f => { f.status = 'blocked'; });
  GEN = { farmers, farms };
  return GEN;
}
export const seedFarmers = () => generate().farmers;
export const seedFarms = () => generate().farms;

export const seedOfficers = (): Officer[] => ([
  ['Karwan Aziz', 'karwan.aziz@jutyar.krd', '+9647501112233', 'Head of the Control Room'],
  ['Shilan Ahmed', 'shilan.ahmed@jutyar.krd', '+9647701234567', 'Agricultural engineer'],
  ['Dlovan Omar', 'dlovan.omar@jutyar.krd', '+9647509876543', 'Water planner'],
  ['Hevi Mustafa', 'hevi.mustafa@jutyar.krd', '+9647504445566', 'Field officer, Duhok'],
  ['Rebaz Salih', 'rebaz.salih@jutyar.krd', '+9647712223344', 'Field officer, Erbil'],
  ['Nazdar Hassan', 'nazdar.hassan@jutyar.krd', '+9647706667788', 'Data analyst'],
] as const).map(([name, email, phone, jobTitle], i) => ({
  id: 'o' + (i + 1), name, email, phone, jobTitle, active: i !== 5, created: iso((200 - i * 20) * DAY), lastSeen: i === 5 ? null : iso((i * 37 + 12) * MIN),
}));

export const seedAlerts = (): Alert[] => [
  { id: 'a4', type: 'weather', title: bi('Frost on Sunday night'), body: bi('Night temperature can fall to -3 °C on Sunday. Pick ripe tomatoes and cucumbers on Saturday and cover young plants.'), status: 'draft', by: 'Shilan Ahmed', created: iso(2 * 60 * MIN), sent: null },
  { id: 'a3', type: 'weather', title: bi('Dust from Tuesday'), body: bi('Dust is expected on Tuesday and Wednesday. Do not spray on these days.'), status: 'sent', by: 'Karwan Aziz', created: iso(3 * DAY), sent: iso(3 * DAY) },
  { id: 'a2', type: 'water', title: bi('Canal water on a schedule'), body: bi('Canal water is shared by schedule this month. Ask your water user group for your day.'), status: 'sent', by: 'Dlovan Omar', created: iso(7 * DAY), sent: iso(7 * DAY) },
  { id: 'a1', type: 'general', title: bi('Good sowing rain from Thursday'), body: bi('More than 20 mm of rain is expected from Thursday. It is a good time to sow wheat and barley.'), status: 'sent', by: 'Rebaz Salih', created: iso(15 * DAY), sent: iso(15 * DAY) },
];

const MSG: [Message['kind'], string, string][] = [
  ['report', 'Yellow stripes on wheat', 'گەڵای گەنمەکەم هێڵی زەردی تێدایە.'],
  ['question', 'When to sow wheat', 'کەی گەنم بچێنم؟'],
  ['report', 'Insects on young barley', 'مێروو لەسەر جۆی گەنجە.'],
  ['complaint', 'Canal water did not come', 'ئاوی کەناڵەکە نەهات.'],
  ['report', 'Hail broke tomato plants', 'تەرزە تەماتەکانی شکاند.'],
  ['request', 'Need a support letter for the bank', 'نامەیەکی پشتگیریم پێویستە بۆ بانک.'],
  ['question', 'Price of wheat this season', 'نرخی گەنم ئەمساڵ چەندە؟'],
  ['report', 'Fire near the field edge', 'ئاگر لە نزیک کێڵگەکە.'],
  ['report', 'Flood after the storm', 'لافاو کێڵگەکەی گرت.'],
  ['question', 'How to save water on tomatoes', 'چۆن ئاو بۆ تەماتە کەم بکەمەوە؟'],
  ['other', 'Thank you for the frost message', 'سوپاس بۆ ئاگادارکردنەوەی سەرما.'],
];
export const seedMessages = (): Message[] => {
  const { farms } = generate();
  return Array.from({ length: 46 }, (_, i) => {
    const f = farms[Math.floor(rnd() * farms.length)], m = MSG[i % MSG.length];
    const state = i < 9 ? 'new' : pickW(['read', 'replied', 'closed'] as const, [2, 5, 4]);
    const at = iso((i < 9 ? i * 47 + 5 : i * 600 + 300) * MIN);
    return {
      id: 'm' + (900 - i), farmerId: f.farmerId, farmId: rnd() < 0.85 ? f.id : null, kind: m[0], subject: m[1], text: m[2], photos: m[0] === 'report' ? 1 + Math.floor(rnd() * 3) : 0,
      at, state, reply: state === 'replied' || state === 'closed' ? 'سڵاو، سوپاس بۆ نامەکەت. ئەندازیاری کشتوکاڵ پەیوەندیت پێوە دەکات.' : '',
      repliedBy: state === 'replied' || state === 'closed' ? 'Shilan Ahmed' : '', repliedAt: state === 'replied' || state === 'closed' ? at : null,
    };
  });
};

export const seedNews = (): News[] => ([
  ['Dukan dam is 92% full, Darbandikhan 63%', 'both'],
  ['Wheat sells for about 800 IQD per kg in the Alwa market', 'both'],
  ['Frost is likely in Penjwen and Pshdar on Sunday night', 'both'],
  ['2,031 farms are registered in the Jutyar app', 'public'],
  ['9 new messages from farmers wait in the inbox', 'admin'],
  ['Good sowing rain is expected from Thursday in Erbil and Makhmur', 'both'],
] as const).map(([en, where], i) => ({ id: 'n' + (i + 1), text: bi(en), where, active: true, order: i }));

export const seedListings = (): Listing[] => {
  const { farms } = generate();
  return Array.from({ length: 186 }, (_, i) => {
    const f = farms[Math.floor(rnd() * farms.length)], crop = f.crops[0].crop, avg = AVG_PRICE[crop] ?? 700;
    const state = pickW(['open', 'sold', 'expired', 'cancelled'] as const, [55, 32, 9, 4]);
    const price = Math.round(avg * (0.88 + rnd() * 0.3) / 25) * 25;
    const kg = Math.round(Math.max(200, f.crops[0].dunam * (crop === 'wheat' || crop === 'barley' ? 380 : 900) * (0.3 + rnd() * 0.6)) / 50) * 50;
    const posted = Math.floor(rnd() * 40) * DAY + Math.floor(rnd() * 600) * MIN;
    return {
      id: String(700 - i), farmerId: f.farmerId, farmId: f.id, crop, kg, price,
      soldPrice: state === 'sold' ? Math.round(price * (0.92 + rnd() * 0.08) / 25) * 25 : null,
      quality: pickW(['A', 'B', 'C'] as const, [40, 45, 15]), gov: f.gov, dist: f.dist,
      posted: iso(posted), closes: new Date(NOW - posted + 14 * DAY).toISOString(), state,
      offers: state === 'open' ? Math.floor(rnd() * 6) : 1 + Math.floor(rnd() * 8), views: 10 + Math.floor(rnd() * 400), photos: Math.floor(rnd() * 5),
      description: '',
    };
  });
};

export const seedReadings = (): DistrictReading[] => PLACES.districts.map((d, i) => {
  const base = 35 + ((i * 37) % 50), byYear: Record<string, number> = {};
  for (let y = 2018; y <= 2026; y++) byYear[y] = Math.max(5, Math.min(98, Math.round(base + gauss() * 14)));
  byYear['2026'] = base;
  return {
    id: d.en, dryness: base, greenness: Math.round(120 - base * 0.7 + gauss() * 5), rain: Math.round(Math.max(0, 60 - base * 0.5 + gauss() * 8)),
    lastYear: byYear['2025'], byYear, updated: iso(6 * DAY + 5 * 60 * MIN), source: 'Sentinel-2 + CHIRPS', manual: false, public: true,
  };
});

export const seedDams = (): Dam[] => {
  const months = Array.from({ length: 12 }, (_, k) => { const d = new Date(NOW); d.setMonth(d.getMonth() - 11 + k); return d.toISOString().slice(0, 7); });
  const hist = (from: number, to: number) => months.map((month, k) => ({ month, pct: Math.round(from + (to - from) * (k / 11) + Math.sin(k) * 4) }));
  return [
    { id: 'dukan', name: bi('Dukan', 'دوکان'), pct: 92, volume: 6.4, capacity: 6.97, yearAgo: 31, history: hist(31, 92), updated: iso(DAY), source: 'KRG Directorate of Dams + Sentinel-2 lake area', manual: false },
    { id: 'darbandikhan', name: bi('Darbandikhan', 'دەربەندیخان'), pct: 63, volume: 1.9, capacity: 3.0, yearAgo: 42, history: hist(42, 63), updated: iso(DAY), source: 'KRG Directorate of Dams + Sentinel-2 lake area', manual: false },
  ];
};

export const seedFires = (): Fire[] => ([
  ['Soran', 36.65, 44.52, 'high', 'VIIRS', 0], ['Amedi', 37.09, 43.48, 'nominal', 'VIIRS', 0], ['Penjwen', 35.62, 45.94, 'nominal', 'MODIS', 2],
  ['Kalar', 34.62, 45.31, 'high', 'VIIRS', 1], ['Makhmur', 35.78, 43.58, 'low', 'MODIS', 3], ['Chamchamal', 35.53, 44.83, 'nominal', 'VIIRS', 0],
] as const).map(([dist, lat, lon, confidence, satellite, nearFarms], i) => ({ id: 'f' + (i + 1), at: iso((i * 190 + 40) * MIN), dist, lat, lon, confidence, satellite, nearFarms, checked: false }));

export const seedRules = (): Rule[] => ([
  ['frost', 'weather', 'Frost night', 'A night is frost (watch) when the lowest temperature is at or below this.', 0, '°C', -10, 5],
  ['hard_frost', 'weather', 'Hard frost night', 'A night is hard frost (alarm) when the lowest temperature is at or below this.', -2, '°C', -15, 0],
  ['heat', 'weather', 'Heat day', 'A day is a heat day when the highest temperature is at or above this.', 31, '°C', 25, 45],
  ['rain', 'weather', 'Heavy rain day', 'A day is heavy rain when rain in that day is at or above this.', 12, 'mm', 5, 60],
  ['sowing', 'weather', 'Sowing rain', 'Rain in 3 days at or above this means the soil is wet enough to sow.', 20, 'mm', 5, 60],
  ['dust', 'weather', 'Dust', 'Dust warning when PM10 is at or above this.', 150, 'µg/m³', 50, 500],
  ['b_dry', 'dryness', 'Dry band starts at', 'Districts with a dryness index from this value are shown as Dry.', 60, '', 30, 90],
  ['b_vdry', 'dryness', 'Very dry band starts at', 'Districts with a dryness index from this value are shown as Very dry.', 80, '', 50, 99],
  ['weak', 'field', 'Weak field', 'A farm is on Watch when its greenness is below this share of normal.', 85, '%', 50, 100],
  ['alarm_sq', 'field', 'Field alarm', 'A farm is on Alarm when its greenness is below this share of normal.', 70, '%', 30, 95],
  ['cloud', 'field', 'Skip cloudy pictures', 'A satellite picture is skipped when more of the farm than this is under cloud.', 30, '%', 0, 90],
] as const).map(([id, group, en, meaning, value, unit, min, max]) => ({
  id, group, name: bi(en), meaning: bi(meaning), value, unit, min, max,
  usedBy: group === 'weather' ? 'weather_planner' : group === 'dryness' ? 'dryness_job' : 'field_eye', history: [],
}));

const TOPICS: [string, string, string, string, Message['kind'] | null][] = [
  ['yellow_leaves', 'wheat', 'The tips of my wheat leaves are yellow.', 'Likely short of nitrogen after the heavy rain. Ask an officer before adding fertilizer.', null],
  ['insects', 'barley', 'Small insects on my young barley.', 'Likely aphids. Check 10 plants in different places; if most have them, call your plant protection office.', null],
  ['stem_rot', 'tomato', 'Black spots at the bottom of tomato stems.', 'Cannot tell: stem rot or frost damage. An officer will look at the photos.', null],
  ['leaf_curl', 'grape', 'Grape leaves are curling.', 'Unsure: leafroll virus or water stress. An officer will look at it.', null],
  ['sowing_time', 'wheat', 'When should I sow wheat?', 'Wait for about 20 mm of rain in 3 days. That is likely from Thursday in your area.', null],
  ['irrigation', 'tomato', 'How often should I water tomatoes now?', 'Every 4 to 5 days in this weather, early in the morning.', null],
  ['weeds', 'chickpea', 'Many weeds between my chickpeas.', 'Remove them by hand before they flower. The Doctor does not name products.', null],
];
export const seedQuestions = (): DoctorQuestion[] => {
  const { farms } = generate();
  return Array.from({ length: 64 }, (_, i) => {
    const t = TOPICS[Math.floor(rnd() * TOPICS.length)];
    const pool = farms.filter(f => f.crops.some(c => c.crop === t[1]));
    const f = pool.length ? pool[Math.floor(rnd() * pool.length)] : farms[i];
    const conf = t[0] === 'stem_rot' || t[0] === 'leaf_curl' ? 'unsure' : pickW(['sure', 'likely'] as const, [1, 2]);
    const rated = i > 10 && rnd() < 0.5;
    return {
      id: 'q' + (5000 - i), farmerId: f.farmerId, farmId: f.id, crop: t[1], topic: t[0], question: t[2], answer: t[3], confidence: conf,
      at: iso(Math.floor(rnd() * 13 * DAY)), rating: rated ? (rnd() < 0.8 ? 'good' : 'bad') : null, correction: '', ratedBy: rated ? 'Shilan Ahmed' : '',
    };
  });
};

export const seedAnswerBank = (): AnswerBankItem[] => ([
  ['sowing_time', 'wheat', 'When should I sow wheat?', 'Sow after about 20 mm of rain in 3 days, from late October to early December.'],
  ['yellow_leaves', 'wheat', 'Why are my wheat leaves yellow?', 'After heavy rain the soil can lose nitrogen. Yellow stripes can also be rust. Send a photo so an officer can tell.'],
  ['irrigation', 'tomato', 'How often should I water tomatoes?', 'In hot weather every 4 to 5 days, early in the morning. Water the soil, not the leaves.'],
  ['frost', '', 'What should I do before frost?', 'Pick ripe fruit before the night, cover young plants, and water the soil in the afternoon.'],
] as const).map(([topic, crop, q, a], i) => ({ id: 'b' + (i + 1), topic, crop, question: bi(q), answer: bi(a), approved: i < 3, by: 'Shilan Ahmed', updated: iso((i + 2) * DAY) }));

export const seedAppTexts = (): AppText[] => ([
  ['home.waiting_picture', 'Home', 'Waiting for the first satellite picture'],
  ['home.weak_line', 'Home', 'Weak in the {where} corner'],
  ['signin.wrong_code', 'Sign in', 'Wrong code, try again'],
  ['farm.saved_offline', 'Add farm', 'Saved on your phone. It will be sent when you have internet.'],
  ['report.seen', 'Reports', 'Seen by an officer'],
  ['plan.nothing', 'This week', 'Nothing to act on in the next 10 days'],
  ['alwa.price_today', 'Alwa', 'Today\'s average price'],
  ['alwa.post', 'Alwa', 'Put a crop on sale'],
  ['doctor.ask', 'Doctor', 'Ask about your field'],
  ['doctor.unsure', 'Doctor', 'The Doctor is not sure. An officer will answer you.'],
] as const).map(([id, screen, en]) => ({ id, screen, text: bi(en), checked: false, updated: iso(5 * DAY) }));

export const seedJobs = (): Job[] => {
  const strip = (s: string) => [...s].map(c => (c === 'o' ? 'ok' : c === 'p' ? 'late' : c === 'x' ? 'failed' : 'none') as Job['days'][number]);
  return ([
    ['satellite', 'Satellite pictures (Sentinel-2)', 'satellite', 'Every day 05:00', 5.2, 18.8, '1,845 of 2,031 farms read, 186 cloudy', 'ok', 'oooooooooooooo'],
    ['weather', 'Weather planner', 'cloud-sun', 'Every 6 hours', 4.7, 1.3, '2,031 plans, 3 alarms', 'ok', 'oooooooooooooo'],
    ['fires', 'Fires (NASA FIRMS)', 'flame', 'Every 3 hours', 1.7, 1.3, '6 fires, 3 near farms', 'ok', 'oooooooooooooo'],
    ['dams', 'Dam levels', 'waves', 'Every day 07:00', 27.7, -3.7, 'No reading today: the source page did not answer', 'late', 'oooooooooooopx'],
    ['dryness', 'Dryness by district', 'sprout', 'Monthly, day 3', 6 * 24 + 5, 24 * 24, '33 districts, 72 sub-districts', 'ok', '-----o-------o'],
    ['alwa', 'Alwa average prices', 'store', 'Every day 08:00', 2.7, 21.3, '13 crops from the app listings', 'ok', 'ooooooopoooooo'],
  ] as const).map(([id, en, icon, every, lastH, nextH, result, state, s]) => ({
    id, name: bi(en), icon, every, last: iso(lastH * 60 * MIN), next: new Date(NOW + nextH * 60 * MIN).toISOString(), result, state, days: strip(s),
  }));
};

export const seedAppConfig = (): AppConfig => ({
  id: 'app', latestVersion: '1.0.3', minVersion: '1.0.2',
  versions: [{ version: '1.0.3', share: 80, released: iso(DAY) }, { version: '1.0.2', share: 15, released: iso(3 * DAY) }, { version: '1.0.0', share: 5, released: iso(5 * DAY) }],
  updateMessage: bi('A new version of Jutyar is ready. Please update to keep using the app.'),
  maintenance: false, maintenanceMessage: bi('Jutyar is paused for a short time. Your farms are safe on your phone.'), maintenanceFrom: '02:00', maintenanceUntil: '03:00',
  announcementOn: false, announcement: bi(''),
  features: { add_farm: true, walk_mode: true, satellite: true, doctor: true, reports: true, alwa: false, plan: true, push: true },
  limits: { farmsPerPhone: 20, maxFarmDunam: 1000, minCorners: 3, maxCorners: 50, gpsMeters: 15 },
  helpPhone: '+964 750 000 0000',
});

export const seedSettings = (): Settings => ({
  id: 'settings',
  orgName: bi('Ministry of Agriculture and Water Resources, Kurdistan Region', 'وەزارەتی کشتوکاڵ و سەرچاوەکانی ئاو، هەرێمی کوردستان'),
  helpPhone: '+964 750 000 0000', defaultLang: 'ku', kurdishDigits: false, areaUnit: 'dunam',
  publicView: { map: true, water: true, fires: true, compare: true, market: true, farmTotals: true },
  letterSigner: bi('Karwan Aziz', 'کاروان عەزیز'), letterSignerTitle: bi('Head of the Control Room', ''), letterPrefix: 'JTY', sampleBanner: true,
});
