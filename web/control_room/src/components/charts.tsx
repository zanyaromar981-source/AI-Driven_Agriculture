// Bar charts as plain HTML (no chart library): they mirror for Kurdish by themselves, print well,
// and draw in O(n) for n bars.
import type { ReactNode } from 'react';
import { useI18n } from '../i18n';

export interface BarItem { key: string; label: ReactNode; value: number; color?: string; note?: ReactNode }

/** Horizontal bars, longest first is up to the caller. */
export function HBars({ items, max, unit = '', digits = 0, showShare }: { items: BarItem[]; max?: number; unit?: string; digits?: number; showShare?: boolean }) {
  const { num } = useI18n();
  const m = max ?? Math.max(1, ...items.map(i => i.value));
  const sum = items.reduce((a, i) => a + i.value, 0) || 1;
  return (
    <div role="list">
      {items.map(i => (
        <div className="hbar" role="listitem" key={i.key}>
          <span style={{ minWidth: 0, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{i.label}</span>
          <div className="bar"><i style={{ width: (i.value / m * 100) + '%', background: i.color ?? 'var(--brand)' }} /></div>
          <span className="nowrap tabular end"><b>{num(i.value, digits)}</b>{unit && <span className="muted small"> {unit}</span>}
            {showShare && <span className="muted small"> · {num(i.value / sum * 100, 1)}%</span>}{i.note}</span>
        </div>
      ))}
    </div>
  );
}

/** Vertical bars (columns) with the value on top; for months or years. */
export function VBars({ items, height = 160, unit = '', digits = 0, max }: { items: BarItem[]; height?: number; unit?: string; digits?: number; max?: number }) {
  const { num } = useI18n();
  const m = max ?? Math.max(1, ...items.map(i => i.value));
  return (
    <div style={{ display: 'flex', alignItems: 'flex-end', gap: 6, height: height + 40, paddingTop: 4 }} role="list">
      {items.map(i => (
        <div key={i.key} role="listitem" style={{ flex: 1, minWidth: 0, display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 4, fontSize: 11 }} title={`${num(i.value, digits)} ${unit}`}>
          <span className="tabular" style={{ fontWeight: 700, color: 'var(--ink-2)' }}>{num(i.value, digits)}</span>
          <b style={{ width: '100%', maxWidth: 46, height: Math.max(2, i.value / m * height), background: i.color ?? 'var(--brand)', borderRadius: '4px 4px 2px 2px', transition: 'height .5s var(--ease)' }} />
          <span className="muted" style={{ whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', maxWidth: '100%' }}>{i.label}</span>
        </div>
      ))}
    </div>
  );
}

/** Two values side by side per row (this year against another year). */
export function PairBars({ rows, a, b, colorA = 'var(--ink-3)', colorB = 'var(--brand)', max }: {
  rows: { key: string; label: ReactNode; a: number; b: number }[]; a: string; b: string; colorA?: string; colorB?: string; max?: number;
}) {
  const { num } = useI18n();
  const m = max ?? Math.max(1, ...rows.flatMap(r => [r.a, r.b]));
  return (
    <div>
      <div className="legend" style={{ marginBottom: 8 }}><span><i className="dotc" style={{ background: colorA }} />{a}</span><span><i className="dotc" style={{ background: colorB }} />{b}</span></div>
      {rows.map(r => (
        <div key={r.key} className="hbar" style={{ alignItems: 'center' }}>
          <span style={{ minWidth: 0, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{r.label}</span>
          <div style={{ display: 'grid', gap: 3 }}>
            <div className="bar" style={{ height: 8 }}><i style={{ width: r.a / m * 100 + '%', background: colorA }} /></div>
            <div className="bar" style={{ height: 8 }}><i style={{ width: r.b / m * 100 + '%', background: colorB }} /></div>
          </div>
          <span className="tabular small end nowrap">{num(r.a)} · <b>{num(r.b)}</b></span>
        </div>
      ))}
    </div>
  );
}

/** A small line of points (sparkline) as SVG, for histories. */
export function Spark({ values, width = 220, height = 48, color = 'var(--water)' }: { values: number[]; width?: number; height?: number; color?: string }) {
  if (values.length < 2) return null;
  const lo = Math.min(...values), hi = Math.max(...values), span = hi - lo || 1;
  const pts = values.map((v, i) => `${(i / (values.length - 1)) * width},${height - 4 - ((v - lo) / span) * (height - 8)}`).join(' ');
  return (
    <svg width="100%" viewBox={`0 0 ${width} ${height}`} preserveAspectRatio="none" style={{ display: 'block', height }} aria-hidden="true">
      <polyline points={pts} fill="none" stroke={color} strokeWidth="2" strokeLinejoin="round" strokeLinecap="round" vectorEffect="non-scaling-stroke" />
    </svg>
  );
}
