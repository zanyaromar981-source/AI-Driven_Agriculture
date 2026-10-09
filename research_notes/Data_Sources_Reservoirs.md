# Data sources for reservoir fullness: Dukan and Darbandikhan

Checked 9 October 2026. "Live" means a keyless call made that day. "Read" means a web page only.

## Answer

For the demo this week, keep the Sentinel-2 pipeline on Planetary Computer and back-fill history from the Global Water Watch (GWW) API, which is keyless and returned surface area for both dams from 1985 to 27 May 2026 (it is 4.5 months stale, so it is history, not the live number). Next, fix the real weakness, the area to volume curve: register free at DAHITI for the 52 Dukan altimetry levels (2002 to 2010) and pair them with same-date areas, and cut a second curve from SRTM, which was flown in February 2000 when both lakes were very low. Then add Sentinel-1 RTC from Planetary Computer for cloudy winter passes, and SWOT (level plus area, free) as an independent check. Pay for nothing yet: Sentinel Hub Exploration (about 300 EUR per year) only if the free 10,000 processing units run out, and Planet only if someone needs daily 3 m images.

## Comparison

| Source | Gives | Both dams? | History | Refresh | Access, auth | Free limit | Paid | Verified |
|---|---|---|---|---|---|---|---|---|
| Sentinel-2 L2A, Planetary Computer | 10 m optical | Yes | 21 Aug 2015 on | 2 to 3 days here | STAC + data API, anonymous | No stated quota, throttled | None | Live: newest 7 Oct 2026 |
| Sentinel-2, CDSE STAC/OData | Same, earliest copy | Yes | 2015 on | Published 2.9 h after pass | Search keyless, download needs free account | 12 TB/month, 4 connections | None | Live |
| CDSE Sentinel Hub (Process, Statistical API) | Server-side index statistics | Yes | 2015 on | Same | OAuth client, free account | 10,000 PU and 50,000 requests/month | Exploration 300 EUR/yr, Basic 999 EUR/yr (unverified) | Read |
| Sentinel-2, AWS Earth Search | COGs | Yes | 2015 on | 3.3 h after pass | Keyless STAC, public S3 | None stated | None | Live |
| Sentinel-1 RTC, Planetary Computer | 10 m radar, terrain corrected | Yes | 10 Oct 2014 on | 1 to 5 days (1C, 1D) | Anonymous STAC and token | None stated | None | Live: water fraction computed |
| Landsat 5/7/8/9 C2 L2 | 30 m optical | Yes | 12 Apr 1984 on | 8 days, about 1 day latency | PC, AWS, USGS STAC keyless | None | None | Live |
| HLS v2 (L30, S30) | 30 m harmonised | Yes | PC: 2017 and 2020 on | 2 to 3 days | PC anonymous; NASA needs Earthdata login | None | None | Live (search) |
| Google Earth Engine | All of the above plus JRC | Yes | 1984 on | Same | Google account + Cloud project | 150 EECU-hours/month (Community) | Commercial plans, rate not read | Read |
| Planet PlanetScope | 3 m daily | Yes (unverified) | 2016 on (unverified) | Daily | API key | Students: 3,000 km2/month, 30 day delay | 15,000 USD/yr Research | Read |
| Global Water Watch | Area time series | Yes, ids 89638 and 88978 | 1985 to 27 May 2026 | Irregular, stale | Keyless REST | None stated | None | Live |
| CLMS Lake Water Level | Altimetry level | No. Mosul, Hamrin, Tharthar area only | 1992 or 2019 on | 10 to 27 days | CDSE STAC keyless search | Free | None | Live |
| CLMS Water Bodies 100 m | Monthly water mask | Yes (global raster) | Oct 2020 on | Monthly, 5 weeks late | CDSE STAC, S3 needs account | Free | None | Live (search) |
| JRC Global Surface Water | Occurrence, yearly and monthly masks | Yes | 1984 to 2020/21 | Not updated | PC anonymous, GEE | Free | None | Live (PC item) |
| DAHITI | Level, some areas and volumes | Dukan only, level 2002 to 2010 | 52 points | Dukan not updated | Free account, API key | Free | None | Read (API returned 403 without key) |
| Hydroweb.next (Theia) | Level, area, volume for 399 lakes | Unverified | Up to 30 years | 3 days after pass | Free Theia account, API key | Free | None | Read; server reset keyless calls |
| SWOT LakeSP via Hydrocron | Level and area per lake | Unverified, passes exist | 2023 on | About 21 day cycle, 1 to 3 passes | Hydrocron keyless, needs lake_id | Free | None | Live (API reachable, id unknown) |
| G-REALM (USDA) | Altimetry level | Unverified | n/a | n/a | Site shows "IPAD retired" | n/a | n/a | Live: retired page |
| GloLakes | Storage for 27,000 lakes | Unverified | 1984 on | 3 to 91 days | NCI download | Free, CC BY 4.0 | None | Read |

## 1. Imagery

- Planetary Computer: `POST https://planetarycomputer.microsoft.com/api/stac/v1/search`. Live results for the Dukan box: `sentinel-2-l2a` 2015-08-21 to 2026-10-07, `sentinel-1-rtc` and `sentinel-1-grd` from 2014-10-10, `landsat-c2-l2` from 1984-04-12 (Landsat 5) to 2026-10-01, `hls2-l30` from 2017, `hls2-s30` from 2020. `/api/sas/v1/token/{collection}` gave tokens without a key, RTC included.
- Sentinel-1 test: `POST /api/data/v1/item/statistics?collection=sentinel-1-rtc&item=S1C_IW_GRDH_1SDV_20261008T145956_..._rtc&expression=(vv<0.02)*1&asset_as_band=true` over the Dukan box returned HTTP 200, mean 0.264, about 231 km2 before removing radar shadow. The server-side approach works for radar too. Mask slopes with a DEM.
- CDSE: `https://stac.dataspace.copernicus.eu/v1/search` and the OData catalogue answered keyless. Scene of 7 Oct 07:48 UTC was published 10:42 UTC. Sentinel-1 of 9 Oct 03:01 was published 05:24. Quotas read at https://documentation.dataspace.copernicus.eu/Quotas.html. The Statistical API needs an OAuth client from a free account, not tested.
- AWS `https://earth-search.aws.element84.com/v1/search` and USGS `https://landsatlook.usgs.gov/stac-server/search` are keyless.
- HLS on NASA CMR: 5,355 S30 and 3,136 L30 granules, newest 7 Oct 2026. Download needs an Earthdata login.
- Earth Engine: noncommercial tiers since 27 April 2026 (150, 1,000 or 100,000 EECU-hours per month), read at https://developers.google.com/earth-engine/guides/noncommercial_tiers.
- Planet: tiers read at https://planet.com/markets/education-and-research/. The free tier is non-commercial personal research use; ask before putting it on a public dashboard. Sentinel Hub commercial prices came from a search snippet (unverified).
- Licence: Sentinel data is free and open with attribution ("contains modified Copernicus Sentinel data [year]"). Landsat and HLS are public domain. Both suit a public dashboard.

## 2. Ready-made products

- Global Water Watch, https://api.globalwaterwatch.earth (OpenAPI at `/openapi.json`). `POST /reservoir/geometry` with a polygon found Dukan (id 89638, GRanD 4463) and Darbandikhan (id 88978, GRanD 4649, unnamed). `GET /reservoir/{id}/ts/surface_water_area` returned 2,226 and 1,143 points, 1985 to 2026-05-27; the monthly variable has 497 points each. Dukan ranges up to 239 km2, Darbandikhan up to 107 km2. Single dates are noisy (Dukan 236, 129, 239 km2 within five days): use the monthly series or a median filter. `/hypsometry` returned an empty list for both, so no volume. Licence not read (unverified).
- CLMS Lake Water Level (`clms_wl-lakes_global_vector_daily_v2_geojson`): zero items for both dams. Five stations in Iraq, including Mosul (42.9 E, 36.68 N, from 1992, last 1 Oct 2026) and Hamrin (45.01 E, 34.2 N, from 2019).
- DAHITI: Dukan is target 38593, https://dahiti.dgfi.tum.de/en/38593/: 479.385 to 509.809 m, no area, volume or hypsometry. Darbandikhan was not found (unverified).
- GloLakes: https://doi.org/10.25914/K8ZF-6G46, paper https://essd.copernicus.org/articles/16/201/2024/. Keyed by HydroLAKES id: Dukan 1362, Darbandikhan 1376 (ids from GWW).
- Static archives, coverage unverified: GRS (1999 to 2018) https://doi.org/10.1038/s41467-023-38843-5, ReaLSAT https://doi.org/10.1038/s41597-022-01449-5.
- BlueDot Observatory: no evidence it is still maintained (unverified).

## 3. Area to volume

- Empirical curve for Dukan: DAHITI levels (2002 to 2010) against same-date Landsat or GWW area. Monthly area in that period spans 86 to 234 km2, almost the full range.
- DEM bathymetry: GWW shows Dukan at about 103 km2 and Darbandikhan at about 31 km2 in February 2000, when SRTM was flown. SRTM therefore maps most of the active storage band of both lakes. Compute area and volume per metre above the SRTM water line. Copernicus DEM (2010 to 2015) cross-checks it.
- Published geometry: Hassan et al. 2017, "Bathymetry and siltation rate for Dokan Reservoir, Iraq", https://doi.org/10.1111/lre.12173 (2014 survey, full text not read). Rashid 2023, https://doi.org/10.31026/j.eng.2023.12.05: Dukan capacity at 511.78 m is 6,924 to 7,008 million m3 against 8,000 originally. Faris et al. 2021 for Darbandikhan operation, https://doi.org/10.18280/ijdne.160401 (curve presence unverified).
- Global curves: GRDL https://doi.org/10.1029/2023WR035781, ReGeom https://doi.org/10.1029/2017WR022040, GLOBathy https://doi.org/10.1038/s41597-022-01132-9, GeoDAR https://doi.org/10.5194/essd-14-1869-2022. Coverage of the two dams unverified.
- Current levels: SWOT (`https://soto.podaac.earthdatacloud.nasa.gov/hydrocron/v1/timeseries?feature=PriorLake&feature_id=...`, lake_id from the Prior Lake Database on Hydroweb) and ICESat-2 ATL13 (97 granules over the Dukan box, newest 29 June 2026).

## 4. Ground truth

| Date | Figure | Source |
|---|---|---|
| 8 May 2024 | Darbandikhan 25 cm below full, capacity 3 billion m3, last full 2019 | Shafaq, dam director |
| 21 Jun 2025 | Dukan 1.6 billion m3, 24% of 7; area down 56% since 28 May 2019 | AFP via Al-Monitor |
| 22 Mar 2026 | Darbandikhan overflowed; Dukan up 17 m since 1 January; Duhok 41.5 of 52 million m3 | Rudaw |
| 27 Mar 2026 | Dukan up 21 m, over 3.5 of 7.2 billion m3, "nearly 60%", 13 m below maximum | Kurdistan24 |

Cross-check done: GWW gives Dukan 238 km2 in late May 2019 and 117 km2 in late May 2025, a 51% drop against the reported 56%. Anchor pairs for the curve: about 112 km2 for 1.6 billion m3, about 179 km2 for roughly 3.5 billion m3. No public feed of daily levels from the Iraqi ministry or KRG was found (unverified). Capacity itself is quoted as 6.97, 7.0 and 7.2 for Dukan, and 2.6 and 3.0 for Darbandikhan.

## 12-hourly job

1. Search PC STAC for Sentinel-2, Sentinel-1 RTC and Landsat items newer than the last stored date, per reservoir polygon.
2. Optical: MNDWI statistics as now, skip scenes with cloud over the polygon. Radar: VV threshold with a slope mask.
3. Store one row per scene: date, sensor, area, cloud fraction. Take a 15-day rolling median.
4. Convert area to level and volume with the curve; publish percent full with an error band and the scene date.
5. Weekly: pull SWOT, compare, flag drift over 10%.
6. On failure, show the last good value and its age.

## Open questions

- Which capacity is the 100% line for each dam?
- Will the dam directorates share daily levels, even one season?
- SWOT lake_id for both dams (needs a free Theia or Earthdata account).
- Does the Hassan 2017 paper include the full table?
- Is GWW paused or only late?
