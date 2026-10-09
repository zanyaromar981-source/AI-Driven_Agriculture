// The Kurdistan Region map: 33 districts coloured by a value, governorate borders on top, names in the
// current language. Clicking a district can zoom into its sub-districts. Points (farms, fires, dams) can
// be drawn on top. Leaflet and the borders file are loaded the first time any map is shown, then cached.
import { useEffect, useRef, useState } from 'react';
import type * as Leaflet from 'leaflet';
import 'leaflet/dist/leaflet.css';
import { useI18n } from '../i18n';

interface GeoProps { en: string; ku: string; gov: string; dist?: string }
type FC = GeoJSON.FeatureCollection<GeoJSON.MultiPolygon | GeoJSON.Polygon, GeoProps>;
export interface KriMap { governorates: FC; districts: FC; subdistricts: FC; dams: { en: string; ku: string; lat: number; lon: number; dist: string }[] }

let libP: Promise<typeof Leaflet> | null = null;
let dataP: Promise<KriMap> | null = null;
export const loadLeaflet = () => (libP ??= import('leaflet').then(m => (m as unknown as { default: typeof Leaflet }).default ?? m));
export const loadKri = () => (dataP ??= fetch(new URL('data/kri_map.json', document.baseURI).href).then(r => r.json() as Promise<KriMap>));

export interface MapPoint { id: string; lat: number; lon: number; color?: string; radius?: number; label?: string; onClick?: () => void }

export function DistrictMap({ fill, styleKey, onDistrict, focus, onBack, points, size = '', tiles = 'osm', showSubs = true }: {
  /** colour for a district (English name); undefined = grey */
  fill: (dist: string) => string | undefined;
  /** change this when the colours change, so the map repaints */
  styleKey: string | number;
  onDistrict?: (dist: string) => void;
  /** a district to zoom into, showing its sub-districts */
  focus?: string | null;
  onBack?: () => void;
  points?: MapPoint[];
  size?: '' | 'sm' | 'xs';
  tiles?: 'osm' | 'none';
  showSubs?: boolean;
}) {
  const { nm, t, lang } = useI18n();
  const el = useRef<HTMLDivElement>(null);
  const st = useRef<{ L?: typeof Leaflet; map?: Leaflet.Map; dl?: Leaflet.GeoJSON; sub?: Leaflet.GeoJSON; pts?: Leaflet.LayerGroup; data?: KriMap; home?: Leaflet.LatLngBounds }>({});
  const [ready, setReady] = useState(false);
  const fillRef = useRef(fill); fillRef.current = fill;
  const clickRef = useRef(onDistrict); clickRef.current = onDistrict;

  // create once
  useEffect(() => {
    let dead = false;
    Promise.all([loadLeaflet(), loadKri()]).then(([L, data]) => {
      if (dead || !el.current) return;
      const map = L.map(el.current, { zoomSnap: 0.25, scrollWheelZoom: false, attributionControl: true, zoomControl: true });
      if (tiles === 'osm') L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', { maxZoom: 18, opacity: 0.35, attribution: '© OpenStreetMap' }).addTo(map);
      const home = L.geoJSON(data.governorates as GeoJSON.GeoJsonObject).getBounds();
      map.fitBounds(home, { padding: [8, 8] });
      st.current = { L, map, data, home };
      setReady(true);
    });
    return () => { dead = true; st.current.map?.remove(); st.current = {}; };
  }, [tiles]);

  // districts: (re)draw when colours, language or focus change
  useEffect(() => {
    const { L, map, data } = st.current; if (!ready || !L || !map || !data) return;
    st.current.dl?.remove(); st.current.sub?.remove();
    const dl = L.geoJSON(data.districts as GeoJSON.GeoJsonObject, {
      style: f => {
        const p = f!.properties as GeoProps, c = fillRef.current(p.en);
        const dim = focus && focus !== p.en;
        return { color: '#fff', weight: 1, fillColor: c ?? '#E3E6DE', fillOpacity: dim ? 0.25 : 0.88 };
      },
      onEachFeature: (f, layer) => {
        const p = f.properties as GeoProps;
        if (!focus) layer.bindTooltip(nm(p), { permanent: true, direction: 'center', className: 'map-label' });
        layer.on('click', () => clickRef.current?.(p.en));
        layer.on('mouseover', () => (layer as Leaflet.Path).setStyle({ weight: 2.5, color: '#14201A' }));
        layer.on('mouseout', () => (layer as Leaflet.Path).setStyle({ weight: 1, color: '#fff' }));
      },
    }).addTo(map);
    L.geoJSON(data.governorates as GeoJSON.GeoJsonObject, { style: { color: '#14201A', weight: 2, opacity: 0.55, fill: false }, interactive: false }).addTo(dl);
    st.current.dl = dl;
    if (focus && showSubs) {
      const subs = { ...data.subdistricts, features: data.subdistricts.features.filter(f => f.properties.dist === focus) };
      const sub = L.geoJSON(subs as GeoJSON.GeoJsonObject, {
        style: { color: '#14201A', weight: 1.2, dashArray: '3 3', fill: false },
        onEachFeature: (f, layer) => layer.bindTooltip(nm(f.properties as GeoProps), { permanent: true, direction: 'center', className: 'map-label' }),
        interactive: false,
      }).addTo(map);
      st.current.sub = sub;
      const target = data.districts.features.find(f => f.properties.en === focus);
      if (target) map.flyToBounds(L.geoJSON(target as GeoJSON.GeoJsonObject).getBounds(), { padding: [20, 20], duration: 0.6 });
    } else if (st.current.home) {
      map.flyToBounds(st.current.home, { padding: [8, 8], duration: 0.5 });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [ready, styleKey, focus, lang]);

  // Kurdish: zoom buttons on the right, so the "Whole region" button (inline end) never covers them
  useEffect(() => {
    const { map } = st.current; if (!ready || !map) return;
    map.zoomControl?.setPosition(lang === 'ku' ? 'topright' : 'topleft');
  }, [ready, lang]);

  // points on top
  useEffect(() => {
    const { L, map } = st.current; if (!ready || !L || !map) return;
    st.current.pts?.remove();
    if (!points?.length) return;
    const g = L.layerGroup();
    for (const p of points) {
      const m = L.circleMarker([p.lat, p.lon], { radius: p.radius ?? 6, color: '#fff', weight: 1.5, fillColor: p.color ?? '#C2452A', fillOpacity: 0.95 });
      if (p.label) m.bindTooltip(p.label);
      if (p.onClick) m.on('click', p.onClick);
      g.addLayer(m);
    }
    g.addTo(map); st.current.pts = g;
  }, [ready, points]);

  return (
    // isolate: Leaflet's controls (z-index 800) and the back button stay under the sticky header
    <div style={{ position: 'relative', isolation: 'isolate' }}>
      <div ref={el} className={'map ' + size} role="img" aria-label={t('map.aria')}>{!ready && <i className="sk block" style={{ minHeight: '100%' }} />}</div>
      {focus && onBack && (
        <button className="btn sm" style={{ position: 'absolute', top: 10, insetInlineEnd: 10, zIndex: 500 }} onClick={onBack}>{t('map.back')}</button>
      )}
    </div>
  );
}

/** Pick a colour from a ramp: stops are the upper bounds of each band. */
export const ramp = (v: number, stops: number[], cols: string[]) => cols[stops.findIndex(s => v < s)] ?? cols[cols.length - 1];
export const GREEN_RAMP = ['#EAF2EC', '#BFDCC8', '#7FBC93', '#3E9461', '#0F5B4B'];
export const GOLD_RAMP = ['#F3EFD9', '#E6DA9C', '#D6BE5A', '#B8932A', '#7E5F12'];
export const DRY_RAMP = ['#2E8B5B', '#8DC28F', '#E9DFA6', '#E3A35A', '#B23A2E'];
