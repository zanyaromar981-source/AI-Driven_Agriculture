// Every record the Control Room shows. One file so the future Supabase tables can follow it 1:1.

export type Id = string;
export type Lang = 'ku' | 'en';
/** A text in both languages. Kurdish may be empty until a native speaker writes it. */
export interface Bi { en: string; ku: string }

export interface Place { en: string; ku: string; c: [number, number] }
export interface District extends Place { gov: string }
export interface SubDistrict extends Place { gov: string; dist: string }

export type Gender = 'male' | 'female';
export type RecordStatus = 'active' | 'blocked';

export interface Farmer {
  id: Id;
  name: Bi;
  phone: string;            // +9647XXXXXXXXX
  gender: Gender;
  birthYear: number | null;
  gov: string; dist: string; sub: string;  // English place names, the keys of places.json
  village: string;
  status: RecordStatus;
  joined: string;           // ISO date
  notes: string;
}

export type Irrigation = 'rainfed' | 'irrigated' | 'mixed';
export type WaterSource = 'rain' | 'well' | 'river' | 'canal' | 'spring';
export type Ownership = 'owned' | 'rented' | 'shared';
export type FieldLevel = 'normal' | 'watch' | 'alarm' | 'none';

export interface FarmCrop { crop: string; dunam: number }

export interface Farm {
  id: Id;
  farmerId: Id;
  name: string;
  gov: string; dist: string; sub: string;
  village: string;
  lat: number; lon: number;
  area: number;             // dunam
  crops: FarmCrop[];
  irrigation: Irrigation;
  water: WaterSource;
  ownership: Ownership;
  level: FieldLevel;        // from the satellite job
  created: string; updated: string;
  notes: string;
}

export type CropCategory = 'cereal' | 'vegetable' | 'fruit' | 'legume' | 'oil' | 'fodder' | 'other';
export type CropSeason = 'winter' | 'summer' | 'perennial';

export interface Crop {
  id: Id;                   // the code, e.g. "wheat"
  name: Bi;
  color: string;
  category: CropCategory;
  season: CropSeason;
  yieldKgPerDunam: number;  // typical, used for the expected harvest in reports
  active: boolean;
  notes: string;
}

export interface Officer {
  id: Id;
  name: string;
  email: string;
  phone: string;
  jobTitle: string;
  active: boolean;
  created: string;
  lastSeen: string | null;
}

export type AlertType = 'general' | 'weather' | 'pest' | 'water' | 'market' | 'other';
export interface Alert {
  id: Id;
  type: AlertType;
  title: Bi;
  body: Bi;
  status: 'draft' | 'sent';
  by: string;
  created: string;
  sent: string | null;
}

export type MessageKind = 'question' | 'report' | 'complaint' | 'request' | 'other';
export type MessageState = 'new' | 'read' | 'replied' | 'closed';
export interface Message {
  id: Id;
  farmerId: Id;
  farmId: Id | null;
  kind: MessageKind;
  subject: string;
  text: string;
  photos: number;
  at: string;
  state: MessageState;
  reply: string;
  repliedBy: string;
  repliedAt: string | null;
}

/** One line of the moving news bar. */
export interface News {
  id: Id;
  text: Bi;
  where: 'both' | 'public' | 'admin';
  active: boolean;
  order: number;
}

export type ListingState = 'open' | 'sold' | 'expired' | 'cancelled';
export interface Listing {
  id: Id;
  farmerId: Id;
  farmId: Id | null;
  crop: string;
  kg: number;
  price: number;            // IQD per kg the farmer asks
  soldPrice: number | null; // IQD per kg of the deal
  quality: 'A' | 'B' | 'C';
  gov: string; dist: string;
  posted: string;
  closes: string;
  state: ListingState;
  offers: number;
  views: number;
  photos: number;
  description: string;
}

export interface DistrictReading {
  id: Id;                   // district English name
  dryness: number;          // 0 to 100, higher is drier
  greenness: number;        // % of normal
  rain: number;             // mm since 1 Oct
  lastYear: number;         // dryness the same month last year
  byYear: Record<string, number>; // dryness per year, for Compare years
  updated: string;
  source: string;
  manual: boolean;          // edited by hand until the next job run
  public: boolean;          // shown on the public View page
}

export interface Dam {
  id: Id;
  name: Bi;
  pct: number;              // % of full
  volume: number;           // billion m3
  capacity: number;
  yearAgo: number;          // % a year ago
  history: { month: string; pct: number }[];
  updated: string;
  source: string;
  manual: boolean;
}

export interface Fire {
  id: Id;
  at: string;
  dist: string;
  lat: number; lon: number;
  confidence: 'low' | 'nominal' | 'high';
  satellite: 'VIIRS' | 'MODIS';
  nearFarms: number;
  checked: boolean;
}

export interface Rule {
  id: Id;
  group: 'weather' | 'dryness' | 'field';
  name: Bi;
  meaning: Bi;
  value: number;
  unit: string;
  min: number; max: number;
  usedBy: string;           // where the number is applied
  history: { value: number; by: string; at: string; why: string }[];
}

export interface DoctorQuestion {
  id: Id;
  farmerId: Id;
  farmId: Id | null;
  crop: string;
  topic: string;            // problem code, e.g. yellow_leaves
  question: string;
  answer: string;
  confidence: 'sure' | 'likely' | 'unsure';
  at: string;
  rating: 'good' | 'bad' | null;
  correction: string;
  ratedBy: string;
}

export interface AnswerBankItem {
  id: Id;
  topic: string;
  crop: string;             // crop code or "" for any crop
  question: Bi;
  answer: Bi;
  approved: boolean;
  by: string;
  updated: string;
}

export interface AppText {
  id: Id;                   // key used by the mobile app, e.g. home.waiting_picture
  screen: string;
  text: Bi;
  checked: boolean;         // read by a native speaker
  updated: string;
}

export interface Job {
  id: Id;
  name: Bi;
  icon: string;
  every: string;
  last: string;
  next: string;
  result: string;
  state: 'ok' | 'late' | 'failed' | 'off';
  days: ('ok' | 'late' | 'failed' | 'none')[]; // last 14 days, oldest first
}

export interface AppConfig {
  id: 'app';
  latestVersion: string;
  minVersion: string;
  versions: { version: string; share: number; released: string }[];
  updateMessage: Bi;
  maintenance: boolean;
  maintenanceMessage: Bi;
  maintenanceFrom: string; maintenanceUntil: string;
  announcementOn: boolean;
  announcement: Bi;
  features: Record<string, boolean>;
  limits: { farmsPerPhone: number; maxFarmDunam: number; minCorners: number; maxCorners: number; gpsMeters: number };
  helpPhone: string;
}

export interface Settings {
  id: 'settings';
  orgName: Bi;
  helpPhone: string;
  defaultLang: Lang;
  kurdishDigits: boolean;   // ١٢٣ instead of 123 when the site is in Kurdish
  areaUnit: 'dunam' | 'hectare' | 'm2';
  publicView: { map: boolean; water: boolean; fires: boolean; compare: boolean; market: boolean; farmTotals: boolean };
  letterSigner: Bi;         // name under the support letter
  letterSignerTitle: Bi;
  letterPrefix: string;     // e.g. JTY
  sampleBanner: boolean;
}
