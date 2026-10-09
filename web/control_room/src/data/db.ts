// The collections. Each is loaded from the browser (or the sample data) the first time a page asks for it.
import { Collection, Doc } from './store';
import * as S from './seed';
import type {
  Farmer, Farm, Crop, Officer, Alert, Message, News, Listing, DistrictReading, Dam, Fire, Rule,
  DoctorQuestion, AnswerBankItem, AppText, Job, AppConfig, Settings,
} from './types';

export const db = {
  farmers: new Collection<Farmer>('farmers', S.seedFarmers),
  farms: new Collection<Farm>('farms', S.seedFarms),
  crops: new Collection<Crop>('crops', S.seedCrops),
  officers: new Collection<Officer>('officers', S.seedOfficers),
  alerts: new Collection<Alert>('alerts', S.seedAlerts),
  messages: new Collection<Message>('messages', S.seedMessages),
  news: new Collection<News>('news', S.seedNews),
  listings: new Collection<Listing>('listings', S.seedListings),
  readings: new Collection<DistrictReading>('readings', S.seedReadings),
  dams: new Collection<Dam>('dams', S.seedDams),
  fires: new Collection<Fire>('fires', S.seedFires),
  rules: new Collection<Rule>('rules', S.seedRules),
  questions: new Collection<DoctorQuestion>('questions', S.seedQuestions),
  answers: new Collection<AnswerBankItem>('answers', S.seedAnswerBank),
  appTexts: new Collection<AppText>('appTexts', S.seedAppTexts),
  jobs: new Collection<Job>('jobs', S.seedJobs),
  /** Site text overrides from the Texts page: id = "<lang>:<key>". */
  siteTexts: new Collection<{ id: string; text: string }>('siteTexts', () => []),
  app: new Doc<AppConfig>('app', S.seedAppConfig),
  settings: new Doc<Settings>('settings', S.seedSettings),
};
export type DB = typeof db;

export const PLACES = S.PLACES;
