# Kurdistan Region map demo

A live web map of the four governorates (Duhok, Erbil, Sulaymaniyah, Halabja) with their KRG districts and sub-districts, exact town locations, live coordinates and search.

## Run

```bash
cd web/map_demo
python3 -m http.server 8791
# open http://127.0.0.1:8791/
```

Opening `index.html` straight from the disk also works. It needs internet for the map tiles.

## What it does

- Real map tiles in WGS84 / Web Mercator, the same coordinates Google Maps uses: OpenStreetMap streets (default), Esri satellite, Esri streets, Esri topographic.
- Governorates in one bright colour each. Click one (map or list) and its colour gives way to its districts, with sub-district borders dashed. Zoom in and the fills fade so the street map shows through.
- Live cursor position: latitude and longitude to 6 decimals (about 0.1 m), degrees minutes seconds to 0.001", UTM 38S to the centimetre, metres per screen pixel, and which governorate, district and sub-district you are in (English and Sorani).
- Latitude / longitude rulers on the top and left edges, with a grid on the map. Steps refine with the zoom, from 1 degree down to 1 second. A searched or clicked point gets dashed crosshair lines to both rulers and its exact value marked in red. Turn it off in the list on the right.
- Click anywhere to drop a pin: coordinates, UTM, elevation (Open-Meteo, Copernicus DEM 90 m), Copy, Open in Google Maps, Open in OpenStreetMap.
- Search box (press `/`): governorates, districts, sub-districts, towns and dams in English or Sorani; coordinates as `36.191174, 44.009414`, `36°11'28.2"N 44°00'33.9"E`, `36 11 28.2 N 44 00 33.9 E` or `36°11.47'N 44°00.565'E` (longitude first also works); anything else goes to OpenStreetMap search inside the region. Pick a result and the map flies there and pins the exact point.
- My location: browser GPS with its accuracy circle.

## Data and how it was built

| File | What |
|---|---|
| `kri_map_data.js` | everything the page draws: governorates, districts, sub-districts, 66 towns, 2 dams |
| `build/kri_borders.kml` | source boundaries: Iraq CSO 2019 sub-districts via geoBoundaries (CC BY 4.0) |
| `build/geocode_towns.py` | finds each town on OpenStreetMap (Nominatim) and keeps it only if it falls in its own sub-district (12 km margin); result `build/towns.json`, raw answers `build/towns_cache.json` |
| `build/build_map_data.py` | regroups the sub-districts into KRG districts and governorates, adds Sorani names and colours, writes `kri_map_data.js` |

Rebuild (needs Python with shapely):
```bash
cd web/map_demo/build
python3 geocode_towns.py kri_borders.kml                  # uses towns_cache.json, so no new requests unless a name is new
python3 build_map_data.py kri_borders.kml towns.json ../kri_map_data.js
```

KRG grouping follows the KRG administrative maps of Duhok (2024), Erbil (2026) and Halabja: Akre, Shekhan and Bardarash are Duhok districts; Soran, Khalifan, Chuman, Sidakan, Mergasor, Harir, Pirmam, Taqtaq and Qushtapa are Erbil districts; Halabja governorate is Halabja (with Sirwan and Byara) and Khurmal; Shahrazur (Zarayan) is in Sulaymaniyah.

## Known limits

- Erbil's newest districts Khabat, Bnaslawa, Barhka and Ainkawa are inside the Erbil district shape: the CSO data has no separate polygons for them. Their towns are on the map.
- Halabja's Bamo sub-district is not drawn (no polygon in the source).
- Baadre and Arbat towns are left out: OpenStreetMap search did not return a settlement point for them.
- Taq Taq town uses OSM node 2488270116, which sits on the town but is tagged as the oil field.
- Some sub-districts have no Sorani name yet (Gnareen, Bebaz, Karmak, Mirka, Saruchik): English only until a Sorani speaker checks them.
- Boundary lines are as precise as the CSO 2019 source (tens of metres). Coordinates under the cursor and pins are exact for the map projection.
