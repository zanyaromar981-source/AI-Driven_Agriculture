// Copy the real KRG borders from web/map_demo/kri_map_data.js into public/data/kri_map.json.
// Only the four governorates are kept. The site fetches this file once, when a map is first shown.
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const src = fileURLToPath(new URL('../../map_demo/kri_map_data.js', import.meta.url));
const out = fileURLToPath(new URL('../public/data/kri_map.json', import.meta.url));
const raw = readFileSync(src, 'utf8');
const K = JSON.parse(raw.slice(raw.indexOf('=') + 1).trim().replace(/;\s*$/, ''));
const GOVS = new Set(['Duhok', 'Erbil', 'Sulaymaniyah', 'Halabja']);
const keep = fc => ({ type: 'FeatureCollection', features: fc.features.filter(f => GOVS.has(f.properties.gov)) });
const data = {
  source: K.source,
  governorates: keep(K.governorates),
  districts: keep(K.districts),
  subdistricts: keep(K.subdistricts),
  dams: K.dams.filter(d => GOVS.has(d.gov)),
};
writeFileSync(out, JSON.stringify(data));
console.log(`kri_map.json: ${data.governorates.features.length} governorates, ${data.districts.features.length} districts, ${data.subdistricts.features.length} sub-districts`);

// The place list (names and centres only, a few kB) for selects, filters and sample data, without the map.
const places = {
  governorates: data.governorates.features.map(({ properties: p }) => ({ en: p.en, ku: p.ku, c: p.c })),
  districts: data.districts.features.map(({ properties: p }) => ({ en: p.en, ku: p.ku, gov: p.gov, c: p.c })),
  subdistricts: data.subdistricts.features.map(({ properties: p }) => ({ en: p.en, ku: p.ku, gov: p.gov, dist: p.dist, c: p.c })),
};
writeFileSync(fileURLToPath(new URL('../src/data/places.json', import.meta.url)), JSON.stringify(places, null, 1));
console.log('places.json written');
