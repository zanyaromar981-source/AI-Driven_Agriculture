// Read the Kurdish from the translator's Excel file into src/i18n/ku/<page>.json.
//   npm run texts:import              -> from Desktop/Jutyar_Translation/jutyar_texts.xlsx
//   npm run texts:import -- <file>
// Only filled cells are taken. A key that is not in the English files is reported and skipped.
// A row whose {placeholders} differ from the English is reported, because the site would show it wrong.
import ExcelJS from 'exceljs';
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { I18N, readLang, textsFile } from './texts-common.mjs';

const file = textsFile(process.argv[2]);
const en = readLang('en');
const wb = new ExcelJS.Workbook(); await wb.xlsx.readFile(file);
const ws = wb.getWorksheet('Texts');
if (!ws) { console.error('No sheet called "Texts" in ' + file); process.exit(1); }

const byNs = {}, unknown = [], badVars = [];
const vars = s => [...String(s).matchAll(/\{(\w+)\}/g)].map(m => m[1]).sort().join(',');
const text = v => (v && typeof v === 'object' && 'richText' in v ? v.richText.map(r => r.text).join('') : String(v ?? '')).trim();
let n = 0;
ws.eachRow((row, i) => {
  if (i === 1) return;
  const key = text(row.getCell(1).value), ku = text(row.getCell(4).value);
  if (!key || !ku) return;
  if (!(key in en)) { unknown.push(key); return; }
  if (vars(en[key]) !== vars(ku)) badVars.push(`${key}: English has {${vars(en[key])}}, Kurdish has {${vars(ku)}}`);
  const dot = key.indexOf('.'), ns = key.slice(0, dot);
  (byNs[ns] ??= {})[key.slice(dot + 1)] = ku;
  n++;
});
mkdirSync(join(I18N, 'ku'), { recursive: true });
for (const [ns, obj] of Object.entries(byNs)) writeFileSync(join(I18N, 'ku', ns + '.json'), JSON.stringify(obj, null, 2) + '\n');
console.log(`${n} Kurdish texts imported from ${file} into ${Object.keys(byNs).length} files.`);
if (unknown.length) console.log(`Skipped ${unknown.length} unknown keys: ${unknown.slice(0, 10).join(', ')}${unknown.length > 10 ? ' ...' : ''}`);
if (badVars.length) console.log(`Check the {words} in ${badVars.length} rows:\n  ` + badVars.join('\n  '));
