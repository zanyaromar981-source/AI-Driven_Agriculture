// The Jutyar logo: 21 grain rays around a centre (app/lib/widgets/grain_sun.dart).
const RAYS = Array.from({ length: 21 }, (_, i) => (i * 360 / 21).toFixed(3));

export function GrainSun({ size = 26, draw = false }: { size?: number; draw?: boolean }) {
  return (
    <svg className={'sun' + (draw ? ' draw' : '')} width={size} height={size} viewBox="0 0 100 100" aria-hidden="true">
      <g className="rays">
        {RAYS.map((r, i) => (
          <path key={i} className="ray" style={{ ['--i' as string]: i }} d="M50 30A30 30 0 0 1 50 7A30 30 0 0 1 50 30Z" transform={`rotate(${r} 50 50)`} />
        ))}
      </g>
      <circle className="core" cx="50" cy="50" r="14" />
    </svg>
  );
}
