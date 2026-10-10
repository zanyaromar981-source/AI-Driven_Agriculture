// Small pieces that know about Jutyar data: crop tags, field status, place names, option lists.
import { useMemo } from 'react';
import { db, PLACES } from '../data/db';
import { useRows } from '../data/store';
import { distByName, govByName, isCrop, subByKey } from '../data/api';
import type { FieldLevel } from '../data/types';
import { useI18n } from '../i18n';
import { Pill, type Tone } from './ui';

export function useCrops() {
  const crops = useRows(db.crops);
  // list and active hold crops only: the other marketplace products (eggs, sheep ...) cannot be grown on a farm
  return useMemo(() => ({ list: crops.filter(isCrop), byId: new Map(crops.map(c => [c.id, c])), active: crops.filter(c => c.active && isCrop(c)) }), [crops]);
}

export function CropTag({ id }: { id: string }) {
  const { b } = useI18n();
  const c = db.crops.get(id);
  return <span className="nowrap"><span className="dotc" style={{ background: c?.color ?? '#ccc' }} />{c ? b(c.name) : id}</span>;
}

const LEVEL_TONE: Record<FieldLevel, Tone> = { normal: 'good', watch: 'warn', alarm: 'danger', none: '' };
export function LevelPill({ level }: { level: FieldLevel }) {
  const { t } = useI18n();
  return <Pill tone={LEVEL_TONE[level]}>{t('level.' + level)}</Pill>;
}

/** Place names in the current language. */
export function usePlaceNames() {
  const { nm } = useI18n();
  return useMemo(() => ({
    gov: (en: string) => nm(govByName.get(en)) || en,
    dist: (en: string) => nm(distByName.get(en)) || en,
    sub: (dist: string, en: string) => nm(subByKey.get(dist + '|' + en)) || en,
  }), [nm]);
}

/** [value, label] lists for selects. */
export function usePlaceOptions(gov: string, dist: string) {
  const { nm } = useI18n();
  return useMemo(() => ({
    govs: PLACES.governorates.map(g => [g.en, nm(g)] as [string, string]),
    dists: PLACES.districts.filter(d => !gov || d.gov === gov).map(d => [d.en, nm(d)] as [string, string]),
    subs: PLACES.subdistricts.filter(s => s.dist === dist).map(s => [s.en, nm(s)] as [string, string]),
  }), [gov, dist, nm]);
}

/** +9647501234567 -> +964 750 123 4567, always left to right. */
export function Phone({ value }: { value: string }) {
  const p = value.replace(/\s/g, '');
  const pretty = p.length === 14 ? `${p.slice(0, 4)} ${p.slice(4, 7)} ${p.slice(7, 10)} ${p.slice(10)}` : value;
  return <span className="ltr tabular">{pretty}</span>;
}
