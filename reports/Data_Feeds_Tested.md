# Free data feeds for the Farm Doctor, tested live on 2026-10-08

All tested by curl/python from this machine. "Keyless" = no account at all. Ranked by how much the data changes a real decision.

## Works now

| # | Source | Gives | Decision it enables | Coverage / freq / res | Licence / key | Effort |
|---|---|---|---|---|---|---|
| 1 | **Open-Meteo Flood API** (GloFAS v4) `flood-api.open-meteo.com/v1/flood` | river flow m³/s, 10-day forecast + spread, history from 1984 | flood warning for riverside fields and pumps; irrigation water availability | KRI; daily; 5 km. River cells: Greater Zab 35.975N/43.325E (174 m³/s today), Little Zab below Dukan 35.925N/44.925E (26), Sirwan below Darbandikhan 35.025N/45.625E (41), Tigris at Faysh Khabur 37.025N/42.325E (102). Little Zab peaked 835 m³/s in Mar 2019 | CC-BY-4.0, keyless | 3 h |
| 2 | **Open-Meteo Air Quality** (CAMS) `air-quality-api.open-meteo.com` | PM10, PM2.5, dust, hourly, 5 days ahead | dust-storm alerts: delay spraying/harvest, shelter livestock | KRI; hourly; ~40 km. Slemani today: PM10 up to 192, dust 276 µg/m³ (an active dust event) | CC-BY-4.0, keyless | 1 h |
| 3 | **Open-Meteo extras** (same forecast API) | soil moisture in 5 layers (0–81 cm), FAO ET0, VPD, soil temperature; ERA5-Land archive with snow depth (1.5 m at 36.6N/45.0E in Feb 2026) | irrigation scheduling (ET0 × Kc), "sow after this rain?", livestock heat stress (THI Erbil next 48 h = 76) | KRI; hourly; 9–11 km | CC-BY-4.0, keyless | 2 h |
| 4 | **FAO WaPOR v3** COGs on Google Cloud `storage.googleapis.com/fao-gismgr-wapor-3-data/DATA/WAPOR-3/MAPSET/<mapset>/WAPOR-3.<mapset>.2026-09-D2.tif` | 100 m actual ET, relative soil moisture, biomass water productivity, every 10 days; 5 km daily rain | which fields are irrigated / under-watered; Ministry water accounting | KRI; dekadal; 100 m; latest 2026-09-D3 (1–2 week lag). Erbil plain: ET mean 1.3, max 8.9 mm/day (irrigated plots), soil moisture 33% | **CC-BY-NC-SA (non-commercial only)**, keyless | 4 h |
| 5 | **Sentinel-1 radar** via Planetary Computer STAC + keyless SAS sign | 10 m radar backscatter through clouds | flood extent after storms; field wetness change (sowing, trafficability); flooded-field claims | KRI; 2–3 passes per 12 days; scenes of 2–3 Oct over Erbil (5-day lag) | CC-BY-4.0, keyless | 6–8 h |
| 6 | **Landsat 8/9 thermal** (PC `landsat-c2-l2`, asset `lwir11`, K = DN × 0.00341802 + 149) | 30 m surface temperature | per-field heat stress at wheat flowering (April); water-stress index with ET0 | 8-day revisit, clear days; 23 Sep scene: 46 °C Erbil plain; ~7-day lag | public domain, keyless | 4 h |
| 7 | **Sentinel-3** on PC (OLCI FAPAR/chlorophyll 300 m daily; SLSTR surface temp 1 km daily; 10-day NDVI 1 km) | daily district-scale crop condition | crop condition in cloudy weeks when Sentinel-2 fails; district drought flags | KRI; daily; latest 2 Oct | Copernicus free, keyless | 4 h |
| 8 | **SoilGrids REST** `rest.isric.org/soilgrids/v2.0/properties/query` | pH, clay, sand, organic carbon, N, CEC by depth | irrigation interval from water-holding; soil context for the doctor | global; static (2020); 250 m. Erbil plain: clay 38%, pH 7.4, SOC 18 g/kg | CC-BY-4.0, keyless | 1 h |
| 9 | **WOAH WAHIS API** `wahis.woah.org/api/v1/pi/event/filtered-list` body `{"countries":[113]}` | official animal-disease events in Iraq (28 since 2022: bird flu Jan 2026, rabies, pullorum…) | herders/vets: quarantine, vaccination, market movement | national; weeks lag | WOAH terms, keyless | 2 h |
| 10 | **Plant-disease image sets** | Halabja pomegranate (Zenodo 15856012, 2,178 originals); Kurdistan watermelon (Mendeley 4dyy62bvfp, 2026); NUST wheat rust (Zenodo 21672928); Wheat Disease Small (Zenodo 7307816); sunn-pest species (Zenodo 15260200); Macrobot barley/wheat (Zenodo 13734021); PlantDoc (2.6k field images) | reference images for the doctor's vision | metadata verified, not downloaded | CC-BY-4.0 (IP102 research only) | 3 h |
| 11 | **Sorani speech** | STT: Google Chirp 2 `ckb-IQ` ($0.016/min); open models hawzhin/Kurdish-OmniASR-7B (Apache-2.0, WER 13.9%), rzgar/whisper-large-v3-sorani-kurdish-ckb-v2, razhan/whisper-small-ckb. Plain Whisper has NO Kurdish. TTS: Google none; open aranemini/central-kurdish-tts (CC-BY-NC-ND), razhan/mms-tts-ckb, RevgeAI/vekol-tts-ckb-edge (CC-BY-NC) | voice in/out | audio quality not tested | see left | 2–4 h |
| 12 | **WorldCereal 2021** Zenodo 7875105 | 10 m winter-cereal, cropland and irrigation maps | sown-area and irrigated-area baseline | global zips 18–20 GB, clip to KRI; not downloaded | CC-BY-4.0, keyless | 4–6 h |
| 13 | **GRACE-FO CSR mascons** `download.csr.utexas.edu/outgoing/grace/RL0603_mascons/` | monthly water-storage anomaly | Ministry groundwater trend only (KRI ≈ 1–2 mascons) | ~300 km effective; 2-month lag; file updated 2026-10-07 | free, keyless | 3 h |
| 14 | **Snow cover** NASA GIBS WMS `MODIS_Terra_NDSI_Snow_Cover` (image only; PNG over the Dukan catchment OK); NSIDC MOD10A1/VNP10A1F current to 2026-10-06 (free login) | daily 500 m snow | spring water outlook for Dukan/Darbandikhan | PC `modis-10A1-061` stale since 2025-06-25 | GIBS keyless; NSIDC free key | 2–4 h |
| 15 | **PubChem REST** | active ingredient → hazard pictograms and H-codes | label reader: hazard and protective gear | global | public domain, keyless | 2 h |
| 16 | **NASA Earthdata (free login)**: IMERG Early rain (6-h lag), SMAP soil moisture 9 km (2-day lag), VIIRS surface temp | rain nowcast; soil-moisture truth | KRI | free registration | 3 h each |
| 17 | **GDACS API** `gdacs.org/gdacsapi/api/events/geteventlist/SEARCH?eventlist=FL,DR&country=Iraq` | flood/drought event alerts (Mar 2026 flood) | Ministry situational alert | national; event-based | free, keyless | 1 h |

Crop calendar: FAO's crop-calendar API does not cover Iraq. FAO GIEWS brief (2026-09-01): winter-cereal harvest mid-May to July 2026, Iraq wheat 5.6 Mt (+34% vs 5-yr average). Sowing months must be hard-coded from KRI practice (15 Nov–5 Dec best window).

## Did NOT work / caveats
- CHIRPS-GEFS rain forecasts: stale since 2026-06-30.
- Planetary Computer: IMERG ends 2021; MODIS snow stale since June 2025; MODIS surface temperature lags 13 days.
- FAO EMPRES-i (animal disease) and FAO Locust Hub: no public API.
- Copernicus Global Land (FAPAR/LAI): no keyless path; use Sentinel-3 instead.
- EPPO pest database: needs a free token; EU pesticide API gone; **Iraq/KRG banned-pesticide list: no machine-readable source** (the label reader needs it typed in by hand).
- FAOSTAT API blocked (bulk ZIP works); FAO price tool has no API.
- Not tested: GSMaP, FAO ASIS, Google Earth Engine (needs account), CABI, Sorani audio quality.

Scratch scripts: session scratchpad `kri/`.
