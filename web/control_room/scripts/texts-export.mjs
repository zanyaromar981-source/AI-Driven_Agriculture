// Write every English text of the site into one Excel file for the Kurdish translator.
//   npm run texts:export              -> Desktop/Jutyar_Translation/jutyar_texts.xlsx
//   npm run texts:export -- <file>    -> another place
// Kurdish already written (in the Excel file or in src/i18n/ku) is kept. The old file is copied to a
// dated backup before it is replaced.
import ExcelJS from 'exceljs';
import { copyFileSync, existsSync, mkdirSync } from 'node:fs';
import { dirname } from 'node:path';
import { readLang, textsFile, pageOf } from './texts-common.mjs';

const file = textsFile(process.argv[2]);
const en = readLang('en'), ku = readLang('ku');

// Kurdish from an earlier version of the Excel file wins over the repo files (it is the newest work)
const kept = { ...ku };
if (existsSync(file)) {
  const old = new ExcelJS.Workbook(); await old.xlsx.readFile(file);
  const ws = old.getWorksheet('Texts');
  ws?.eachRow((row, i) => {
    if (i === 1) return;
    const key = String(row.getCell(1).value ?? '').trim(), k = String(row.getCell(4).value ?? '').trim();
    if (key && k) kept[key] = k;
  });
  const stamp = new Date().toISOString().slice(0, 16).replace(/[-:T]/g, '');
  copyFileSync(file, file.replace(/\.xlsx$/, `_backup_${stamp}.xlsx`));
}

const wb = new ExcelJS.Workbook();
wb.creator = 'Jutyar';
const help = wb.addWorksheet('How to', { properties: { tabColor: { argb: 'FF1E7A5A' } } });
help.columns = [{ width: 110 }];
[
  'Jutyar website texts',
  '',
  '1. Open the sheet "Texts". Every row is one text that appears on the website.',
  '2. Column C is the English text. Write the Kurdish (Sorani) in column D, on the same row.',
  '3. Keep words in curly brackets exactly as they are, for example {n} or {name}. The site puts a number or a name there.',
  '   Example: English "{n} min ago"  ->  Kurdish "{n} خولەک لەمەوبەر"',
  '4. Column B says where the text appears. Empty Kurdish cells show the English text on the site until they are written.',
  '5. Do not change column A (the key) and do not change the order of the columns. You may sort or filter.',
  '6. Save the file. Then send it back (or run "npm run texts:import" in web/control_room) and the site uses your Kurdish.',
  '',
  'Admins can also change any text later, inside the site: Admin > Texts and languages.',
].forEach((l, i) => { const c = help.getCell(i + 1, 1); c.value = l; if (i === 0) c.font = { bold: true, size: 16, color: { argb: 'FF1E7A5A' } }; });

const ws = wb.addWorksheet('Texts', { views: [{ state: 'frozen', ySplit: 1 }] });
ws.columns = [
  { header: 'Key (do not change)', key: 'key', width: 34 },
  { header: 'Where it appears', key: 'page', width: 34 },
  { header: 'English', key: 'en', width: 70 },
  { header: 'Kurdish (Sorani): write here', key: 'ku', width: 70 },
  { header: 'Done', key: 'done', width: 8 },
];
const head = ws.getRow(1);
head.font = { bold: true, color: { argb: 'FFFFFFFF' } };
head.fill = { type: 'pattern', pattern: 'solid', fgColor: { argb: 'FF123A2C' } };
head.height = 22;
let missing = 0;
for (const key of Object.keys(en).sort((a, b) => a.localeCompare(b))) {
  const k = kept[key] ?? '';
  if (!k) missing++;
  const r = ws.addRow({ key, page: pageOf(key), en: en[key], ku: k, done: { formula: `IF(LEN(D${ws.rowCount + 1})>0,"✓","")` } });
  r.getCell('en').alignment = { wrapText: true, vertical: 'top' };
  r.getCell('ku').alignment = { wrapText: true, vertical: 'top', horizontal: 'right', readingOrder: 'rtl' };
  r.getCell('ku').font = { name: 'Noto Sans Arabic', size: 12 };
  r.getCell('ku').fill = { type: 'pattern', pattern: 'solid', fgColor: { argb: k ? 'FFE3F0EA' : 'FFFFF8E6' } };
  r.getCell('key').font = { color: { argb: 'FF8A978F' }, size: 9 };
  r.getCell('page').font = { color: { argb: 'FF5E6E64' }, size: 10 };
  r.getCell('done').alignment = { horizontal: 'center' };
}
ws.autoFilter = { from: 'A1', to: 'E1' };

mkdirSync(dirname(file), { recursive: true });
await wb.xlsx.writeFile(file);
console.log(`${Object.keys(en).length} texts written to ${file}. Kurdish missing: ${missing}.`);
