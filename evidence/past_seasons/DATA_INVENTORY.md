# Data inventory (started 2026-10-07, 00:30) — everything downloaded for the Oct 8–9 build

Root: `evidence/past_seasons/backtest_data/` (raw downloads, never edit) · scripts: `evidence/past_seasons/backtest/`

| # | Data | Source | Where | Status |
|---|---|---|---|---|
| 1 | Daily rain, mean temp, ET0, soil moisture 0–100 cm + 28–100 cm, 1990-09 → 2026-10-05, 16 zones (farmland points) | ERA5 via Open-Meteo archive (free, no key) | `openmeteo/<Zone>.json` | ✅ done |
| 2 | Same weather for the two reservoir points | Open-Meteo | `openmeteo/_reservoir_points_Dukan_Darbandikhan.json` | ✅ done |
| 3 | Satellite greenness (NDVI 250 m, every 16 days), Feb–May 2000–2025, 8 km box per zone | NASA MODIS MOD13Q1 via ORNL web service | `modis/<Zone>_<year>.json` | ✅ all 16 zones (416 files) |
| 4 | Reservoir water area: NDVI 40 km box over Dukan and Darbandikhan, ALL composites 2000–2026 | MODIS via ORNL | `reservoirs/<Dam>_<year>[abc].json` | ✅ 159 files (Dukan_2026b failed) — ⚠ does not measure lake area correctly, see note below |
| 5 | Full-year greenness curves, all 16 zones (Jan, Oct–Dec, Jun–Sep), 2000–2025 | MODIS via ORNL | `modis_full/<Zone>_<year>_<jan|autumn|summer>.json` | ✅ done (jan 400, autumn 416, summer 416) |
| 6 | 2026 greenness so far, all 16 zones | MODIS via ORNL | `modis_full/<Zone>_2026_[ab].json` | ✅ done |
| 7 | El Niño / La Niña index (ONI) 1950 → Sep 2026 + NOAA's current statement | NOAA CPC | `enso/oni.ascii.txt`, `enso/ensodisc.html` | ✅ done |
| 8 | 16-day weather forecast, 16 zones (sample from 2026-10-07; REFETCH on demo day) | Open-Meteo forecast (free) | `forecast/forecast16_2026-10-07.json` | ✅ sample |
| 9 | Governorate + district boundaries (Iraq ADM1, ADM2, simplified GeoJSON) | geoBoundaries 2022 | `boundaries/*.geojson` | ✅ done |
| 10 | 12 climate indices 1950 → Sep 2026 (NAO, AO, AMO, IOD/DMI, Niño3.4, SOI, TNA, QBO, PDO, CPC teleconnections, Eurasian snow cover) | NOAA PSL, NOAA CPC, NCEI, Rutgers | `climate_indices/*.txt` | ✅ done, tested in `backtest/signals_test.py` — only ENSO is useful |
| 11 | CHIRPS v2 monthly rain per zone (±0.04° box), 1981-01 → 2026-09 (Sep prelim) | UCSB CHC (IRI now needs login) | `chirps/chirps_monthly_zones.csv`, raw tiles in `chirps/raw/` | ✅ done, tested: no gain |
| 12 | NOAA STAR Vegetation Health (VCI/TCI/VHI), cropland, weekly, Duhok/Erbil/Sulaymaniyah/Kirkuk/Ninawa/Diyala, 1982 → 2026 w39 (gap: most of spring 2004) | NOAA STAR | `vhi/vhi_weekly_provinces.csv` | ✅ done, tested: no gain |
| 13 | **Official wheat & barley area/production/yield per governorate** (Erbil, Sulaymaniyah, Duhok, KRI total 1970–2023; Garmiyan 2013+, Halabja 2017+) | KRSO PDF (April 2025) | `harvest_stats/krso_wheat_barley_governorates.csv` (+ raw PDF/text, FEWS NET file) | ✅ done; caveats: 10 rows yield ≠ prod/area, Garmiyan 2022=2023 duplicate, 2019+ rounded estimates |
| 14 | Extra ERA5 daily (tmax, tmin, snow, rain, rain hours, soil 0–7/7–28 cm, VPD, radiation) | Open-Meteo | `openmeteo_extra/<Zone>.json` | ✅ 16/16 zones; tested: no gain |
| 15 | 7 seasonal forecast models (NMME + ECMWF SEAS5), area-mean precip, starts Sep–Dec 1991–2026 incl. Oct 2026 forecast | IRI Data Library OPeNDAP (keyless) | `seasonal_models/*.dods`, `urls.txt` | ✅ done; tested: second opinion only |
| 16 | **Greenness map of the whole region** (NDVI 250 m, every 16 days, Feb 2000 → Jul 2026, 592 dates, sinusoidal 2048×2560 px window of tile h21v05) | NASA MODIS MOD13Q1 v061 via Microsoft Planetary Computer (keyless COG reads) | `modis_grid/MOD13Q1_<YYYYDDD>.i16.z` + `_grid_info.json` (how to find a lon/lat) | ✅ done 2026-10-08 00:59 (592/592; 20 dates needed a retry); matches ORNL zone pixels 1089/1089 |
| 17 | Sea temperature, monthly area mean: East Mediterranean, whole Mediterranean, Persian Gulf, Jan 1981 → Sep 2026 | NOAA ERSST v5 via IRI Data Library data.tsv (keyless) | `sea_temp/*.tsv` | ✅ done 2026-10-08; tested: no gain over El Niño |

Zone points: `backtest/zones.json` (16 farmland points near each town, hand-picked). Processed outputs: `backtest/out_late/`, `backtest/out_peak/` (features.csv, results.json).

## Not downloaded, and why
- **Soil (SoilGrids):** API answers but returns no layers → still paused. HWSD v2 is a 1+ GB package needing a database tool → skip for the hackathon; crop fit uses rain history + FAO crop water needs only.
- **NASA FIRMS fires:** needs a personal map key (free, by e-mail). Only if fire alerts stay in scope.
- **Sentinel-2 (10 m) / Earth Engine:** needs the team's Earth Engine approval. MODIS 250 m covers everything above until then.
- **Land surface temperature (MOD11A2):** possible from the same ORNL service, ~800 more calls; not needed by the backtest model → optional later.
- **Dam level ground truth:** nothing machine-readable online; validate the reservoir curve against the known facts (Dukan ~24% June 2025, near full spring 2026, Darbandikhan overflow March 2026).

## How to resume the download if it stops
`cd evidence/past_seasons/backtest && python3 -I fetch_all.py zones.json ../backtest_data` — it skips files already present. One request at a time; parallel requests make the NASA server return errors.

**2026-10-07 warning:** reservoir boxes (`reservoirs/`) do not measure lake area correctly with MOD13Q1 NDVI (water mostly not negative / fill pattern inconsistent; 2025 shows more water than 2019). Do not use for the lake without a proper water method (Landsat/Sentinel-2 NDWI or JRC Global Surface Water in Earth Engine).

**2026-10-07 15:20:** full NASA download queue finished (`backtest/fetch_all.log`: 1609 jobs, 1 failed = Dukan_2026b). New test data from helpers: `planted_area/`, `fires/`, `farmland_loss/`, `dam_water/` (see BACKTEST_RESULTS Investigation 6).
