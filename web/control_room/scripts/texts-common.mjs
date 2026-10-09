// Shared by texts-export and texts-import: where the files are and how a page name reads.
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

export const I18N = fileURLToPath(new URL('../src/i18n/', import.meta.url));

/** The Desktop folder (OneDrive moves it on many Windows machines), or a path given on the command line. */
export function textsFile(arg) {
  if (arg) return arg;
  const home = process.env.USERPROFILE || process.env.HOME;
  const desk = [join(home, 'OneDrive', 'Desktop'), join(home, 'Desktop')].find(existsSync) ?? home;
  return join(desk, 'Jutyar_Translation', 'jutyar_texts.xlsx');
}

export function readLang(lang) {
  const dir = join(I18N, lang), out = {};
  if (!existsSync(dir)) return out;
  for (const f of readdirSync(dir).filter(f => f.endsWith('.json')).sort()) {
    const ns = f.replace('.json', '');
    const obj = JSON.parse(readFileSync(join(dir, f), 'utf8'));
    for (const [k, v] of Object.entries(obj)) out[ns + '.' + k] = v;
  }
  return out;
}

/** Where each group of texts appears, in plain words for the translator. */
export const PAGE = {
  common: 'Everywhere (buttons, words used on many pages)', nav: 'Admin menu (sidebar)', intro: 'Intro screen (loading)', login: 'Admin sign-in page',
  table: 'Tables (every list)', search: 'Search box', level: 'Farm status names', v: 'Form error messages', map: 'Maps',
  view: 'Public page (map, water, fires, prices)', overview: 'Admin: Overview', farms: 'Admin: Farmers and farms', letter: 'Support letter (printed)',
  govreport: 'Government report (printed)', crops: 'Admin: Crop register', cropreport: 'Crop report (printed)', region: 'Admin: Region data',
  alerts: 'Admin: Alerts', inbox: 'Admin: Inbox and news bar', doctor: 'Admin: The Doctor (AI)', alwa: 'Admin: Alwa market', rules: 'Admin: Rules',
  appctl: 'Admin: App control', texts: 'Admin: Texts and languages', jobs: 'Admin: Data jobs', settings: 'Admin: Settings', officers: 'Admin: Admins',
};
export const pageOf = key => PAGE[key.split('.')[0]] ?? key.split('.')[0];
