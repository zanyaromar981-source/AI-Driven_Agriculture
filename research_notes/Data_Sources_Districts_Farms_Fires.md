# Data sources: districts, farms, fires, prices, SMS, boundaries

Checked 2026-10-09. "Live" = called today with no key, for 35.56N 45.43E (Sulaymaniyah city), 35.35N 45.60E (Sharazur farmland) or 36.19N 44.01E (Erbil). "Read" = read on the provider page today, not called. "Unverified" = from memory or a search snippet only. Nothing was signed up for.

## Short answer

1. **Districts: can be met.** Keep Open-Meteo ERA5 for rain. Add CHIRPS v3 as a second rain source with ready-made SPI, MODIS NDVI (ORNL), WaPOR v3 for evapotranspiration, ERA5-Land for soil moisture and snow depth. All keyless. No study was found that ranks CHIRPS, IMERG and ERA5-Land against each other over Kurdistan, so "which validates best" stays open.
2. **Farms: mostly.** Soil: SoilGrids (modelled, 250 m, slow API, fetch once and store). Greenness: Sentinel-2 via Planetary Computer or Earth Search, keyless. Forecast: Open-Meteo, 16 days. **Groundwater at farm scale: no.** Only a 25 km modelled percentile exists.
3. **Fires: can be met** with FIRMS, about 3 hours behind the satellite. Flares: build your own mask from repeat detections. VIIRS Nightfire is not free for a non-academic team.
4. **Wholesale prices: no source exists.** WFP and FAO carry retail only, monthly, three Kurdistan cities. Alwa prices must be collected by hand.
5. **Sign-in codes: can be met, but SMS is expensive** (about 0.40 to 0.50 USD each). WhatsApp authentication is listed at about 0.008 USD. Start with WhatsApp, SMS as fallback.
6. **Boundaries: can be met** with OCHA COD-AB (CC BY-IGO, levels 0 to 3). It has 20 districts in the three Kurdistan governorates, not 33.

## 1. Per district

| Source | Gives | Resolution | History | Latency | Access, auth | Free limit | Paid | Verified how |
|---|---|---|---|---|---|---|---|---|
| Open-Meteo archive (ERA5) | Daily rain | 25 km, daily | From 1940 | Values returned through today | `archive-api.open-meteo.com/v1/archive`, none | 10,000 calls/day, 5,000/hour, 600/min, non-commercial only | Standard 1M calls/month. Price not shown on page: unverified. Archive needs Professional | Live: 1940-01-02 = 5.0 mm, 2026-10-07 = 1.7 mm. Limits: read |
| Open-Meteo archive, `models=era5_land` | Soil moisture 4 layers, snow depth | 10 km, daily | From 1950 (unverified, see open questions) | 6 days (last value 2026-10-03) | Same | Same | Same | Live: soil 0 to 7 cm 0.263 m3/m3. Snow depth 1.45 m at 36.8N 44.9E (3,032 m) on 2026-01-20. `snowfall_sum` and `et0` came back null with this model |
| CHIRPS v3 (CHC) | Rain, satellite plus gauges | 5.5 km. Daily, pentad, dekad, month | From 1981 | Preliminary 2 days after each 5-day period. Final in 3rd week of next month | GeoTIFF at `data.chc.ucsb.edu/products/CHIRPS/v3.0/`, none | None stated | Free | Live: March 2026 = 242.6 mm Sharazur, 185.1 mm Erbil. Aug 2026 = 0.0. Licence unverified |
| CHC SPI from CHIRPS3 | SPI already computed, 1 to 21 pentad windows | 5.5 km, every 5 days | Normal period 1996 to 2025 | Latest file: pentad 55 of 2026, preliminary | `data.chc.ucsb.edu/products/SPI/CHIRPS3/`, none | None | Free | Live: listing and README read. No pixel sampled |
| NASA GRACE-DA drought indicators | Root-zone and surface soil moisture percentile, groundwater percentile | 25 km, weekly | From 2003 | 4 days (latest 2026-10-05) | GeoTIFF at `nasagrace.unl.edu/globaldata/current/`, none | None | Free | Live: root-zone 82nd percentile, groundwater 9th at Sharazur |
| NASA POWER | Rain, soil wetness, temperature | About 50 km, daily | From 1981 (unverified) | 3 days | `power.larc.nasa.gov/api/temporal/daily/point`, none | Unverified | Free | Live: root wetness 0.35, last value 2026-10-06 |
| MODIS NDVI, ORNL subset API | Greenness | 250 m, 16 days | 2000-02-18 to 2026-09-14 | About 25 days | `modis.ornl.gov/rst/api/v1/MOD13Q1/subset`, none | Unverified | Free | Live: 0.139 city 2026-09-14, 0.675 Sharazur 2026-04-07. Normal: compute from the 26 years. VIIRS there stops 2024-05-24 |
| FAO ASIS | Vegetation health, agricultural stress, drought intensity | 1 km, dekadal | Unverified | Unverified | Catalog `data.apps.fao.org/gismgr/api/v2/catalog/workspaces/ASIS/mapsets`, none | Unverified | Free | Catalog live. Raster download returned 403. Not usable as tested |
| FAO WaPOR v3 level 2 | Actual evapotranspiration, also relative soil moisture, biomass | 100 m, dekadal | From 2018 | About 5 weeks (latest 2026-08-D3) | Catalog `.../workspaces/WAPOR-3/mapsets`, files on `storage.googleapis.com/fao-gismgr-wapor-3-data`, none | None | Free | Live: 0.6 mm/day Sharazur, 0.8 Erbil. Iraq is covered at 100 m. Licence unverified |
| Copernicus GDO | SPI, soil moisture anomaly, combined drought indicator | Unverified | Unverified | Unverified | Download page `drought.emergency.copernicus.eu/tumbo/gdo/download/` | Unverified | Free | Page loads (200). A guessed WMS path returned 502. No machine route confirmed |
| IMERG, SMAP, ASCAT, ESA CCI, MODIS snow cover | Rain, soil moisture, snow cover | 10 km, 9 to 36 km, 12 to 25 km, 25 km, 500 m | 2000, 2015, 2007, 1978, 2000 | Hours to days | NASA Earthdata login, EUMETSAT H SAF or Copernicus CDS account, all free | Unverified | Free | Unverified, not called. ORNL replied "No SPL3SMP_E data available" for Sharazur. NASA GIBS WMS lists `MODIS_Terra_NDSI_Snow_Cover` keyless, pictures only |

Validation: Aziz, Omer and Khayyat (2026, Passer Journal 8(1)) compared CHIRPS with the Erbil station: R2 0.942, NSE 0.91, PBIAS 5.6% (abstract read). A northern Iraq study on 11 stations (2000 to 2014) reports CHIRPS SPI correlation 0.64 to 0.87 with bias 1.05 to 1.81, meaning it over-reads (search snippet, citation unverified). No local validation was found for ERA5-Land, IMERG, any soil moisture product, NDVI or WaPOR. FEWS NET does not appear to monitor Iraq (unverified).

## 2. Per farm

| Source | Gives | Resolution | History | Latency | Access, auth | Free limit | Paid | Verified how |
|---|---|---|---|---|---|---|---|---|
| SoilGrids v2 (ISRIC) | Clay, sand, pH, organic carbon by depth, with uncertainty | 250 m, static | One map | None | `rest.isric.org/soilgrids/v2.0/properties/query`, none | Fair use, number unverified | Free, CC BY 4.0 (unverified) | Live: Sharazur clay 40.3%, sand 20.4%, pH 7.4, organic carbon 14.4 g/kg. Calls took 1 s, 67 s, and one timed out at 40 s. City point returned null |
| OpenLandMap | Soil and terrain layers | 250 m to 1 km | Static | None | STAC catalog on `s3.eu-central-1.wasabisys.com/stac/openlandmap/`, none | Unverified | Free | Catalog live. No pixel sampled |
| HWSD v2, Iraq national soil map (Buringh 1960) | Soil units | 1 km, paper scale | Static | None | File download | n/a | Free | Unverified |
| Sentinel-2 L2A, Planetary Computer | Band values at a point | 10 m, 2 to 3 days here | Unverified start | Same day | STAC `planetarycomputer.microsoft.com/api/stac/v1/search`, point `.../api/data/v1/item/point/{lon},{lat}`, none | Unverified | Free | Live: 2026-10-04 scene, B04 2832, B08 3480 in 1.3 s. Subtract 1000 from each first: NDVI 0.15 |
| Sentinel-2 L2A, Earth Search (AWS) | Cloud-optimised GeoTIFFs | 10 m | Unverified start | Published 3.4 hours after overpass | STAC `earth-search.aws.element84.com/v1/search`, none | Unverified | Free | Live: scenes 09-29, 10-01, 10-04. B04 1832, B08 2480 read remotely |
| Copernicus Data Space Statistical API | Mean NDVI per polygon, server side | 10 m | Unverified | Same day | Free account, OAuth | 10,000 processing units and 10,000 requests/month, 300/min | Commercial tiers | Read (quota page). Not called |
| Open-Meteo forecast | Hourly and daily weather, reference evapotranspiration | About 10 to 25 km | n/a | Updated several times a day | `api.open-meteo.com/v1/forecast`, none | As above | As above | Live: 16 days to 2026-10-24, max 27.7 C today |
| ECMWF open data | Raw IFS forecast | 25 km, to 360 hours | Last 4 days online | 2026-10-09 00z run present at 08:47 UTC | `data.ecmwf.int/forecasts/`, GRIB2, none | Unverified | Free | Live listing. Licence unverified |
| Tomorrow.io | Forecast | n/a | n/a | n/a | API key | Unverified | Unverified | Pricing page returned 403: unverified |
| Groundwater: GRACE-DA (table 1) | Modelled percentile, not depth | 25 km | From 2003 | 4 days | Keyless | None | Free | Live. District-scale hint only |

Agronomic warnings: no ready-made service was found. Derive them from forecast thresholds (frost, heat, wind, rain).

## 3. Fires

| Source | Gives | Resolution | History | Latency | Access, auth | Free limit | Paid | Verified how |
|---|---|---|---|---|---|---|---|---|
| FIRMS regional CSV | Detections, brightness, radiative power, day or night | VIIRS 375 m, MODIS 1 km | 24 hours and 7 days | Newest MODIS point 06:12 UTC, file fetched 08:47 UTC | `firms.modaps.eosdis.nasa.gov/data/active_fire/noaa-20-viirs-c2/csv/J1_VIIRS_C2_Russia_Asia_24h.csv`, none | None seen | Free | Live: Iraq is in the `Russia_Asia` file. 7 days, NOAA-20: 914 detections in the region box, 398 at night |
| FIRMS area API | Same, by box and date | Same | 1 to 5 days per call | Same | `/api/area/csv/[MAP_KEY]/[SOURCE]/[west,south,east,north]/[days]`. Free MAP_KEY by email | 5,000 transactions per 10 min | Free | Read. Live with a fake key: 400 "Invalid MAP_KEY." |
| LSA SAF FRP-PIXEL, Meteosat IODC | Fire radiative power | 3 km, every 15 min | From 2017 | About 20 min (snippet, unverified) | `datalsasaf.lsasvcs.ipma.pt`, free registration | Unverified | Free, CC BY 4.0 | Read. Middle East listed in coverage. MTG version (1 km, 10 min) lists Europe, Africa, South America only |
| VIIRS Nightfire (EOG) | Night combustion sources with temperature, separates flares | 750 m, nightly | From 2012 (unverified) | About a day (unverified) | Account plus signed licence | Free for academic researchers only | Non-profit and commercial: paid yearly, price by email | Read |

Flares: in the 7-day file one 1 km cell near 35.53N 44.34E (Kirkuk oil fields) had 23 detections, others 11 to 16. A cell that burns on many separate days is a flare. Build the mask from FIRMS history. Small stubble fires can be missed by a 3 km geostationary pixel (unverified).

## 4. Prices

| Source | Gives | Resolution | History | Latency | Access, auth | Free limit | Paid | Verified how |
|---|---|---|---|---|---|---|---|---|
| WFP food prices, HDX | Retail only. Erbil, Dohuk, Sulaimaniyah. Tomatoes, potatoes, onions, cucumbers, eggplants, wheat flour, rice, meat, eggs and others | One market per city, monthly | 2012 to 2026-08-15 | About 7 weeks | CSV from `data.humdata.org/dataset/wfp-food-prices-for-iraq`, none | None | Free, CC BY-IGO | Live: 61,825 rows, 0 wholesale. Dohuk potatoes 1,000 IQD/kg |
| FAO GIEWS FPMA | Same WFP series, fewer crops | Same | 2012 to 2026-08 | Same | `fpma.fao.org/giews/v4/price_module/api/v1/FpmaSerie/`, none | None | Free | Live: 72 Iraq series, all retail |
| KRSO | Yearly tonnage through wholesale centres, local against imported, per governorate | Yearly | 2014, 2015 seen | Years | PDF | n/a | Free | Live: 2015 release read. No prices. Its source is the Ministry of Agriculture and Water Resources |

## 5. Sign-in codes

| Source | Gives | Price to Iraq | Sender rules | Verified how |
|---|---|---|---|---|
| Twilio | SMS | 0.5030 USD per segment, plus 0.001 per failed message | Letters-only sender name. Asiacell needs it registered since 2026-07-01, 5 days. Numeric senders fail on all three. No replies | Read 2026-10-09 |
| Plivo | SMS | Asiacell 0.3954, Zain 0.4528, Korek 0.4459 USD | Not stated | Read 2026-10-09. A search snippet showed Zain at 0.1998: conflicting |
| Vonage, Infobip, local gateways, operators direct | SMS | No public price confirmed | Unverified | Unverified (Vonage page returned 403). Reseller figures of 0.09 to 0.28 EUR are snippets only |
| WhatsApp authentication template | Code inside WhatsApp | 0.0079 USD per delivered message from 2026-10-01, was 0.0091 | Verified Meta business, approved template, user must have WhatsApp | Meta page read: Iraq has its own rates from 2026-10-01. The figures come from a reseller blog quoting Meta's rate card |

The 0.1280 USD "authentication-international" rate applies only to a business based outside Iraq that sends over 750,000 messages in 30 days (Meta page, read).

## 6. Boundaries

| Source | Gives | Licence | Verified how |
|---|---|---|---|
| OCHA COD-AB Iraq (HDX) | Levels 0 to 3: 18 governorates, 101 districts, 294 sub-districts. Kurdistan: 20 districts (Sulaymaniyah 10, Erbil 6, Duhok 4), 64 sub-districts | CC BY-IGO, publish with credit | Live: GeoJSON downloaded and counted |
| geoBoundaries gbOpen IRQ ADM2 | Same 101 districts, simplified | CC BY 3.0 IGO | Live: API and file. ADM3 returns 404 |
| GADM | Levels 0 to 2 | "Redistribution or commercial use is not allowed without prior permission" | Read. Do not publish |

## What cannot be had

- Groundwater depth or trend per farm or per district: no public feed. Published well surveys exist as papers (Erbil, 54 wells, 2004 to 2016, falls of about 1 m a year: snippet, unverified), not as data.
- Wholesale alwa prices: none anywhere. Someone must record crop, grade, unit and low and high price at each alwa every market day.
- A flare mask free for a ministry: none confirmed.
- Soil measured on the farm: every soil source here is a global model.
- A tested ranking of rain products over the Zagros.
- The team's 33 districts as an open file.

## Open questions

1. Does Open-Meteo count a Ministry dashboard as non-commercial? Ask them, and ask the paid price.
2. Which 33 districts? Can they be built by merging the 64 COD sub-districts, and is Halabja separate (COD has it inside Sulaymaniyah)?
3. Will the Ministry share its wholesale-centre records and any well records?
4. Does the Directorate of Meteorology share gauge data, so rain products can be ranked locally?
5. Real SMS cost and delivery per operator: test Twilio and Plivo with a few numbers each. Ask Korek and Asiacell for a direct business SMS price.
6. What share of farmers have WhatsApp?
7. Open-Meteo ERA5-Land returned null rain for 1950 and 1951: confirm the start year before computing normals.
8. FAO ASIS and Copernicus GDO: find a download route that works from a server.
