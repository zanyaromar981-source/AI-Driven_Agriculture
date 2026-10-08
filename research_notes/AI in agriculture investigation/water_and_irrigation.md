# AI and Satellites for Agricultural Water: Proven Deployments, Numbers, Failures (focus: dry regions, Iraq/KRI)

Researched 2026-10-07. Tags used on each finding: **[OPERATIONAL]** = running in a government/agency workflow; **[PILOT]** = field trial or pilot with measured result; **[RESEARCH]** = paper/model only, no operational use found; **[CLAIM]** = vendor or programme self-reported, not independently verified. Country flags: **[IRQ] [KRI] [IRN] [TUR] [SYR] [JOR]** etc.

## Q1. Satellite evapotranspiration / water-productivity platforms in operation (WaPOR, OpenET, IrriSAT): who uses them for allocation or enforcement, and with what measured effect?

### Takeaway
These platforms are really in use, mainly for **water accounting, reporting and benchmarking**: Egypt's MWRI Water Accounting Unit, Iraq's Ministry of Water Resources, US groundwater agencies under SGMA, and Australian cotton growers. But I found **no published measured water-saving figure** from WaPOR or OpenET use. The hard enforcement examples (Spain) run on plain Landsat/Sentinel NDVI irrigated-area mapping, not "AI ET" platforms.

### Cited Findings
**FAO WaPOR (Water Productivity Open-access Portal)**
- [OPERATIONAL] The Government of Egypt uses WaPOR data in the Water Accounting Unit of the Ministry of Water Resources and Irrigation (MWRI). Rwanda uses it for Natural Capital Accounts. Farmer-facing apps built on WaPOR include IRWI (Egypt) and LARI-LEB (Lebanon, Bekaa), which tell farmers how much water is needed, crop health and predicted yield. No farmer counts or savings are reported — [FAO WaPOR / FutureWater summary](https://www.fao.org/in-action/remote-sensing-for-water-productivity/wapor-data); [FutureWater one-pager on WaPOR apps](https://www.futurewater.nl/wp-content/uploads/2022/02/OnePager-PFT-WaPORapps.pdf)
- [OPERATIONAL/ANALYTIC] WaPOR water-accounting studies exist for the Awash basin, Ethiopia (18.6 million people), the Litani basin, Lebanon (2010–2016), Kenya (2010–2021) and the Amman-Zarqa basin, Jordan (2018–2021). Amman-Zarqa availability is "highly responsive to precipitation" — [FAO water accounting](https://www.fao.org/in-action/remote-sensing-for-water-productivity/water-accounting/2/en); [Springer, Amman-Zarqa WA+](https://link.springer.com/article/10.1007/s40808-024-02159-0) **[JOR]**
- [PILOT] Bekaa Valley, Lebanon: FAO Investment Centre and the World Bank used WaPOR data to support irrigation management (released 22 Mar 2023). The page gives no farmer numbers, area or water savings — [FAO news](https://www.fao.org/in-action/remote-sensing-for-water-productivity/news-and-events/news/news-detail/WaPOR-data-used-in-the-Bekaa-Valley-Lebanon-to-support-irrigation/en)
- [PILOT] Tunisia: the IREY app ("Irrigation Reference to Enhance Yield") uses WaPOR data to send near-real-time irrigation alerts in wheat areas — [CGIAR/ICARDA search summary](https://cgspace.cgiar.org/items/33d86126-9d60-41fd-b9c1-42e0759ed457). See Q6 for the farmer acceptance figures.
- **[IRQ][KRI]** For WaPOR use in Iraq, see Q5.

**OpenET (US)**
- [OPERATIONAL] OpenET launched in 2021 covering 17 western states. Its FARMS tool covered 27 states in early 2025, and data covered all 48 contiguous states from 15 Dec 2025. It is run by a consortium of NASA, USGS, CSU Monterey Bay, EDF, DRI, Google Earth Engine and HabitatSeven — [NASA, Dec 2025](https://science.nasa.gov/blogs/science-news/2025/12/16/expanding-crucial-water-tracking)
- [OPERATIONAL] Named users: California's Central Delta and South Delta Water Agencies (water-use reporting), with "an increase in landowner reporting and significant cost savings for farmers" (not quantified); Gallo and Sun Pacific Farming (irrigation management); and water agencies in California, Nebraska, Oregon and Kansas (groundwater accounting) — [NASA, Dec 2025](https://science.nasa.gov/blogs/science-news/2025/12/16/expanding-crucial-water-tracking)
- [OPERATIONAL] The open-source groundwater accounting platform was built in 2018 by Rosedale-Rio Bravo Water Storage District (Kern County, CA) and EDF. It tracks landowner groundwater allocations against use in near real time under California's SGMA, and has since been expanded to other agencies with the California Water Data Consortium — [ESA project page](https://esassoc.com/projects/rrbwsd-water-accounting-platform/); [ESA 2021 announcement](https://esassoc.com/news-and-ideas/2021/05/new-partnership-announced-in-california-supporting-groundwater-sustainability/)
- Context, not an OpenET result: Diamond Valley, Nevada became the state's first Critical Management Area (Order 1264, 25 Aug 2015). The sustainable cap is 30,000 acre-feet but use was about 70,000 af (more than 2×). The Nevada Supreme Court reinstated its groundwater management plan on 16 Jun 2022 — [Nevada Independent](https://thenevadaindependent.com/article/in-diamond-valley-farmers-are-looking-to-protect-their-future-and-testing-the-limits-of-nevadas-water-laws); [Western States Water](https://westernstateswater.org/wp-content/uploads/2022/02/Fairbanks-DVGMP_Presentation_FINAL.pdf)

**IrriSAT (Australia, CSIRO)**
- [OPERATIONAL, free] A weather-based scheduling service that uses Sentinel-2 and Landsat on Google Earth Engine to get field-specific crop coefficients (Kc). It gives water-use deficits and 7-day forecasts of crop water use. **More than 1,500 growers and consultants are registered users.** Industry benchmarking found cotton water productivity can be **up to 6 bales/ha below potential** — [GrowAg / CRDC benchmarking project](https://www.growag.com/research-project/benchmarking-water-use-efficiency-and-crop-productivity-in-the-australian-cotton-industry); [Inside Cotton, IrriSAT pilot](https://www.insidecotton.com/node/55375/full)

### Inferences
- Across WaPOR, OpenET and IrriSAT, the proven value is **measurement and accounting** (who used how much, and where). Usage numbers are reported, but none of the sources found report water actually saved.
- An Iraqi or Kurdistan product could plug into WaPOR (already accepted by Iraq's ministry, see Q5) instead of building its own ET model.

### Gaps
- No peer-reviewed or agency figure for water saved from WaPOR or OpenET use was found.
- No case was found of OpenET used directly as legal evidence for a pumping penalty. Diamond Valley's OpenET use is not confirmed.

## Q2. AI irrigation scheduling and SMS/app advisory for smallholders in arid regions: measured water savings and yield effects?

### Takeaway
The best-documented smallholder system is the **satellite + SMS Irrigation Advisory Service (IAS) in Pakistan, India and Bangladesh** (University of Washington with PCRWR). It reached 100,000+ farmers at about **$10 per farmer per year**. Its savings figures (up to 85%) are programme-reported, not from a randomised trial. In MENA, advisory work is mostly **pilots** (Tunisia IREY, Egypt IRWI/AgriSAT, Lebanon LARI-LEB) with acceptance data but no measured savings. Vendor numbers (30–60%) are claims.

### Cited Findings
- [OPERATIONAL→SCALED, CLAIM-level outcomes] **Pakistan / South Asia IAS:** the Pakistan Council of Research in Water Resources asked for it in 2015. It started in 2016 with **700 farmers** in Pakistan and had grown to **100,000+ farmers in Pakistan, India and Bangladesh** by April 2021. Inputs are Landsat thermal (100 m), GRACE and MODIS weather data, with advice sent by SMS. It costs **about $10 per farmer per year** for governments. NASA reports **up to 85% water savings per dry season**, or 80 million m³ per typical irrigation district in India and 150 million m³ in Pakistan (programme claim; baseline not stated). Bangladesh was evaluating it for its national AMISDP advisory system for 2022 — [NASA, Apr 2021](https://science.nasa.gov/missions/landsat/south-asian-farmers-fine-tune-when-to-water-with-landsat)
- [PILOT, anecdotal] The same system is described as reaching **10,000 farmers** in Pakistan. In documented cases, advised farmers used **3 irrigations vs neighbours' 6–7**, with yields of **4,742 vs 4,149 kg/ha** (+14%) — [Global Innovation Exchange](https://waterdatachallenge.globalinnovationexchange.org/innovations/growing-more-less-using-satellites-and-cellphones-satellite-cellphone-irrigation)
- [CLAIM, unverified] An APCTT (UN-ESCAP) 2026 brief says India's "Satellite-based Irrigation Advisory Service (SIAS)" reaches **10+ million smallholders** with AI-driven SMS/app advice. I found no primary Indian government source; it may mix this up with general agro-advisory reach — [APCTT](https://apctt.org/sites/default/files/attachment/2026-02/CR%20Query%206%20Smart%20Irrigation.pdf)
- [PILOT] IFAD/IWMI piloted an interactive SMS water and weather service in Nubaria (Egypt), the Gash Delta (Sudan) and Arata Chufa (Ethiopia) (2014). No outcome numbers were found — [CGIAR WLE](https://wle.cgiar.org/thrive/2014/03/10/new-sms-service-connects-farmers-weather-and-water-information)
- [PILOT] Egypt's AgriSAT programme provides remote-sensing soil moisture, crop water requirement and weather data to farmers and extension agents. No measured outcomes were found — [ICARDA](https://icarda.org/research/featured/enhancing-food-security-arab-countries)
- [PILOT, not AI, useful benchmark] Egypt raised-bed wheat (ICARDA farmer fields, 2020): **−31% irrigation water, +32% grain yield, +98% water-use efficiency** — [ICARDA](https://icarda.org/research/featured/enhancing-food-security-arab-countries)
- [CLAIM] ICARDA says water-saving techniques saved **2.66 bcm in Morocco and 4.79 bcm in Egypt**. The method and dates are not given in the summary — [ICARDA](https://icarda.org/research/featured/enhancing-food-security-arab-countries)
- [CLAIM] CropX (Israel): "water savings alone is over 30%" — [Israel21c](https://archive.israel21c.org/how-an-israeli-startup-is-transforming-the-world-of-agriculture-one-drop-of-water-at-a-time/). Netafim (India): USD 85 m financing for precision irrigation for **35,000 farmers**, with an *expected* **~40%** saving in water and fertiliser — [KrASIA](https://kr-asia.com/netafim-secures-usd-85-million-to-provide-irrigation-solutions-to-35000-farmers-in-india). Both are vendor or press figures.
- [OPERATIONAL, not AI] **[KRI]** Drip irrigation was found "widely used" in site visits across Bazian, Sangaw, Halabja, Shahrazur, Erbil and Rovia in summer 2025. The NGO claims drip can raise yields by up to 50% and cut water use by up to 60% (generic claim) — [EPIC](https://enablingpeace.org/irrigation/). In Raparin, farmers adopted perforated "rain hose" lines during the 2025 drought — [Kurdistan24](https://www.kurdistan24.net/en/story/859439/innovative-irrigation-helps-kurdistan-farmers-combat-severe-drought)

### Inferences
- The proven model is satellite ET plus weather forecast, turned into "irrigate X mm on day Y" and sent by SMS, at a cost of around $10 per farmer per year. The hard part is getting farmers to act on it, not the AI.
- Field-level savings often do not become basin-level savings (see Q6, the efficiency paradox). Any pitch should say "less pumping and energy for the farmer", not "more water in the river", unless it comes with a cap.

### Gaps
- No randomised controlled trial of an AI/SMS irrigation advisory in MENA or Central Asia with published water and yield effects was found.
- No India SIAS primary source was found. Uzbekistan and Central Asia field-level advisory results were not found.

## Q3. AI/ML forecasting of reservoir inflow and dam levels for seasonal allocation: operational examples and skill, especially in transboundary snow/rain basins

### Takeaway
The clearest **operational** AI seasonal water-supply forecast is the US **NRCS M4** ensemble, used for western snow-fed irrigation supply outlooks. Central Asia has an operational ML forecasting effort (**SAPPHIRE**, Swiss-funded, in national hydromet services). For the **Tigris (Mosul Dam), I found only academic ML studies**, with no operational use by Iraq's ministry documented.

### Cited Findings
- [OPERATIONAL] **USDA NRCS M4 (Multi-Model Machine-Learning Metasystem):** averages six models, including the old regression model, and uses AutoML with SNOTEL snow and precipitation inputs. In 20 hindcast test cases across the western US and Alaska, out-of-sample **R² and RPSS improved by more than 50% on average and RMSE improved by 13%** against existing benchmarks. It was live-tested in the 2020 season and is now used in NRCS water supply forecasts — [Fleming et al. 2021, J. Hydrology (NRCS PDF)](https://nrcs-prod.azureedge.us/sites/default/files/2023-03/FlemingEtAl_JournalOfHydrology_2021_M4Testing.pdf); [Western States Water](https://westernstateswater.org/?p=17304)
- [OPERATIONAL/PILOT] **SAPPHIRE Central Asia** (Swiss SDC; implementer Hydrosolutions GmbH; **2022–2026; CHF 1.72 m**) co-develops and deploys open-source forecasting tools that blend classical and ML methods inside the national hydromet services of Kazakhstan, Kyrgyzstan, Tajikistan, Turkmenistan and Uzbekistan. TFT, TiDE and TSMixer models were tested for operational **10-day streamflow forecasts at 100+ gauges** (EGU 2026) — [Swiss FDFA fact sheet](https://www.eda.admin.ch/content/dam/countries/countries-content/uzbekistan/en/SAPPHIRE.pdf); [EGU26-6492](https://meetingorganizer.copernicus.org/EGU26/EGU26-6492.html) (Amu Darya / Syr Darya region)
- [RESEARCH] Deep-ensemble bias correction of GloFAS-ERA5 streamflow for snow-fed Syr Darya and Amu Darya gauges, motivated by global reanalysis losing accuracy in small glacier headwaters — [Water 2026](https://doi.org/10.3390/w18162055)
- [RESEARCH] **[IRQ]** Mosul Dam: CNN-LSTM and related deep models predicted dam water level using 1993–2006 data with the Iraqi Ministry of Water Resources and Mosul University (best MAE 0.087). An earlier ANN reservoir-operation study used 1990–2012 inflow, evaporation, rain, storage and outflow — [PMC 2024](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11622834/)

### Inferences
- In Iraq, most inflow (more than 70%, see Q5) is decided by upstream release decisions in Turkey and Iran, not by weather alone. Any inflow model for Mosul, Dukan or Darbandikhan therefore has a hard ceiling unless it uses upstream reservoir and release data. That is an argument for **satellite altimetry and reservoir-area monitoring of upstream dams** as model inputs.
- M4 shows the operational pattern: a multi-model ensemble with probabilistic output that supplements, rather than replaces, the agency's existing method.

### Gaps
- No operational AI inflow forecasting was found for the Nile, Indus or Tigris–Euphrates agencies, and no published skill scores for Iraqi or Turkish operational forecasts.

## Q4. Satellite detection of illegal wells / over-pumping / unauthorized irrigation and GRACE groundwater mapping: enforcement or policy use?

### Takeaway
**Spain is the strongest documented case of satellite → enforcement.** The Guadalquivir authority has run twice-yearly Landsat/Sentinel surveys since 2016, about 500 wells have been closed since 2015, and an EU Court ruling followed in 2021. **Jordan and Iran enforce on a large scale but by inspection and meters**, and satellite use there is mostly research. GRACE gave the headline Tigris–Euphrates groundwater loss but has not driven enforcement.

### Cited Findings
**Spain**
- [OPERATIONAL] Doñana: the Guadalquivir Hydrographic Confederation (CHG) has used Landsat 8 and Sentinel-2 to monitor illegal farming **twice a year (spring and autumn) since 2016**. Satellite results plus land-rights maps, drones and flow-meter checks feed a **7-year administrative and legal closure process**. **About 500 wells have been closed since 2015**. Penalties are up to 5 years in prison, and the Guardia Civil created water-theft units after 2019 — [HortiDaily](https://www.hortidaily.com/article/9877963/spain-tackles-illegal-water-extraction-in-fruit-growing-region/)
- [CONTEXT] WWF estimates for Doñana: **more than 1,000 illegal wells, 1,700 suspicious ponds and about 3,000 ha of illegal farms** — [HortiDaily](https://www.hortidaily.com/article/9877963/spain-tackles-illegal-water-extraction-in-fruit-growing-region/); [WWF 2021](https://updates.panda.org/court-ruling-brings-hope-for-spanish-wilderness)
- [POLICY] The **EU Court of Justice ruled on 24 Jun 2021** that Spain broke EU law by not accounting for illegal extractions in Doñana — [WWF](https://updates.panda.org/court-ruling-brings-hope-for-spanish-wilderness); [FreshPlaza](https://www.freshplaza.com/europe/article/9334061/european-court-of-justice-condemns-spain-for-not-protecting-donana-from-the-continuous-plundering-of-water/)
- [FAILURE / political pushback] The Andalusian parliament proposed a law to legalise existing illegal wells, and **UNESCO** expressed "utmost concern" and asked for a report by **1 Dec 2024** under threat of "in Danger" listing. A joint plan of about **US$1.59 bn** aims to reduce groundwater pressure — [UNESCO WHC decision](https://whc.unesco.org/en/decisions/8239); [Smart Water Magazine](https://smartwatermagazine.com/news/smart-water-magazine/eu-will-take-action-if-spain-allows-further-deterioration-donana-wetlands)
- [OPERATIONAL] La Mancha Oriental (9,962 km²): the Júcar Hydrographic Confederation and the irrigators' board (JCRMO) use NDVI-based remote sensing to detect unauthorised irrigated area and over-use against licensed volumes. Sanctions increased from 2000 to 2020, and illegal irrigation is now **<1% of all water use**. The method is described as "inexpensive" and "highly effective" — [COALA project](https://www.coalaproject.eu/blog/irrigation-non-compliance/)

**Jordan [JOR]**
- [OPERATIONAL, inspection-based] The Ministry of Water and Irrigation's campaign (since 2013) has sealed **1,063 illegal wells** and confiscated **62 drilling rigs**, and stopped **38,213** violations on networks and resources. Earlier totals were 780 wells in 2016 and 939 by Sep 2017. The exact date of the latest figure is unclear, probably about 2018–19. The sources do not say satellites were used — [Jordan Times](https://jordantimes.com/news/local/authorities-tackle-30000-violations-water-resources-2013); [YorkU archive](https://mideastenvironment.apps01.yorku.ca/?p=12732)

**Iran [IRN]**
- [CONTEXT] About **770,000 wells, of which only about 440,000 are licensed**. Officials cited about 220,000 unauthorised wells in 2017. About **14,000 illegal wells are sealed per year**; Isfahan sealed 7,635 over 16 years. Tavanir has installed **307,000 smart meters** on farm wells. **400+ plains** are "critical/prohibited" (Geological Survey of Iran, 2023). Note: these numbers come from the summary of several sources and are not individually verified — [Uni-Halle working paper 2025](https://www.opendata.uni-halle.de/bitstream/1981185920/121075/1/wp2025260.pdf); [IIASA 2025](https://pure.iiasa.ac.at/id/eprint/20876/); [Eghtesad Online](https://www.eghtesadonline.com/en/news/787582/isfahan-woes-worsened-by-illegal-water-wells-land-subsidence)
- [FAILURE/governance] Reporting alleges that some illegal wells are backed by military-linked actors, which limits enforcement — [Global Voices, Jul 2025](https://globalvoices.org/2025/07/28/is-irans-water-crisis-fueled-by-military-backed-illegal-wells/)
- [RESEARCH] Illegal-well detection in Bastam, Iran by fusing Landsat 8 and Sentinel-2 with GIS (academic; no enforcement link found) — [PMC 2025](https://pmc.ncbi.nlm.nih.gov/articles/PMC11846939/)

**Egypt**
- [OPERATIONAL, enforcement] Rice is restricted to allotted areas. Fines are **EGP 3,000–10,000 per feddan** or up to 6 months in prison. Rice grown outside the allotment was still **210,000–309,308 ha per year** over five years. Despite fines, harvested area "usually exceeds the MWRI limit significantly" — [USDA FAS](https://www.fas.usda.gov/data/egypt-egyptian-parliament-approves-prohibition-rice-cultivation-non-designated-areas); [Gulf News](https://gulfnews.com/world/mena/egypt-cracks-down-on-illegal-cultivation-amid-dam-row-1.79054409)

**GRACE**
- [RESEARCH, policy-relevant] **[IRQ][IRN][TUR][SYR]** Tigris–Euphrates–western Iran, 2003–2009: total water storage fell **143.6 km³** (−27.2 mm/yr). **Groundwater loss was 91.3 ± 10.9 km³** (about 60% of the total) — [Voss et al. 2013, WRR](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC3644870/)

### Inferences
- What works in enforcement is cheap irrigated-area mapping compared against the licence register, followed by legal follow-through. Spain shows that the satellite part is easy. The 7-year legal process and political legalisation pressure are the bottleneck.
- Egypt shows that detection plus fines without full political will leaves 200–300k ha of non-compliance a year.

### Gaps
- No source found whether Egypt's MWRI uses satellites specifically for the rice fines.
- No satellite-based well or over-pumping enforcement was found in Iraq/KRI, Jordan or Morocco.

## Q5. Iraq and Kurdistan Region: water-crisis facts and existing digital water projects

### Takeaway
Iraq went from an **80-year low in reserves (about 10 bcm, May 2025)** to **about 34 bcm (July 2026)** after a wet winter. Swings that large make seasonal forecasting and allocation the core problem. More than 70% of inflow comes from Turkey, Iran and Syria. FAO's **WaPOR II** is the only documented digital water-monitoring project in Iraq/KRI that the government uses, and it includes Erbil's **Shemamuk scheme (15,000 ha)**. I found **no published water-saving or yield result** from any digital water project in Iraq/KRI.

### Cited Findings
**Hydrology and crisis numbers**
- **[IRQ]** May 2025: reserves were **about 10 bcm against at least 18 bcm needed to start summer**, half of 2024's level and "lowest in 80 years" (MoWR spokesman Khaled Shamal). The planted area target was cut from **2.5 m to 1.5 m dunams (−40%)**. Causes given were low rain, low snowmelt and upstream dams in Turkey and Iran — [The National, 25 May 2025](https://www.thenationalnews.com/news/mena/2025/05/25/iraq-water-reserves-lowest-in-80-years/)
- **[IRQ]** July 2026: reserves were **about 34 bcm**. The ministry expects to draw down about 40%, to just over 20 bcm, before winter. **Mosul, Dukan and Darbandikhan were 70–80% full**, and **more than 70% of Iraq's inflows come from neighbouring countries** (Turkey first, then Iran and Syria). The article also cites a much lower prior-year pre-winter figure (extracted as about 5 bcm, "lowest since 1934"; verify before quoting) — [964media, 27 Jul 2026](https://en.964media.com/50317/)
- **[KRI]** Dukan, June 2025: **about 1.6 bcm of 7 bcm (about 24%)**, the lowest in about 20 years. Lake area shrank **56%** between May 2019 (full) and June 2025. Rainfall was **220 mm vs a typical 600 mm**, and upstream damming of the Little Zab in Iran was a secondary cause (AFP) — [Arab News / AFP, Jun 2025](https://www.arabnews.com/node/2605253/middle-east)
- **[KRI]** 3 May 2025: **Dukan hydropower was halted** (it had been running one unit at 40–45 MW) to protect Sulaimani drinking water. Darbandikhan was at about 70 MW, "far below capacity", and shut several hours a day — [Rudaw](https://www.rudaw.net/english/kurdistan/030520252)
- **[KRI]** Earlier context: in May 2024 Darbandikhan was within 25 cm of full, its highest since 2019. Dukan reached 57% by June 2024, and early-2025 rainfall was 188 mm, about one-third of 2024 — [Shafaq](https://shafaq.com/en/Report/Kurdistan-confronts-water-security-pressures-amid-climate-volatility). After the 2025–26 winter rains, 20+ Kurdistan dams overflowed (date not verified) — [Peregraf](https://peregraf.com/en/news/11264)
- **[TUR][IRQ]** Turkey pledged to raise releases by **420 m³/s** (Erdogan and Speaker al-Mashhadani, Jul 2025). Iraqi MPs say it was not fully honoured. There is a 10-year framework agreement (Apr 2024), and its implementation mechanism was signed in Nov 2025. Claim: Iraq receives **less than 40% of its historical share** — [Rudaw](https://www.rudaw.net/english/middleeast/06092025); [The New Region](https://thenewregion.com/posts/4587); [Shafaq](https://shafaq.com/en/Iraq/Iraq-Turkiye-finalize-framework-to-solve-water-crisis%20)
- **[IRQ]** Displacement: IOM recorded **31,001 families (186,006 people)** displaced by climate factors in **12 governorates** (Sep 2025), with 3 in 5 moving to cities. In Dhi Qar alone, **more than 10,500 families (about 60,000 people)** were displaced by Feb 2026 — [Shafaq report](https://shafaq.com/en/Report/Iraq-s-Displacement-From-war-driven-migration-to-climate-induced-mobility-pressures); [EUAA COI](https://www.euaa.europa.eu/country-origin-information-report/331-climate-change-induced-displacement); [IOM/ESCWA case study](https://unescwa.org/sites/default/files/event/materials/1.1%20Impact%20of%20Drought%20on%20Human%20Mobility-A%20Case%20Study%20from%20Iraq-Mateo%20Merchan-IOM.pdf)

**Digital and remote-sensing water projects in Iraq/KRI**
- [OPERATIONAL/PILOT] **FAO + Iraq Ministry of Water Resources, WaPOR II** (funded by the Netherlands): monitors land and water productivity in the **West Gharraf scheme (about 84,000 ha, Wasit/Thi Qar)** and the **Shemamuk scheme (15,000 ha, Erbil Plain)**, plus rainfed and irrigated areas in KRI (2023–24). **Ministry technicians produced WaPOR-based reports that government decision-makers approved.** Indicators include seasonal ET, biomass, groundwater abstraction and recharge — [UN Iraq](https://iraq.un.org/en/277583-fao-wapor-ii-project-continues-develop-capacities-and-increase-technical-follow); [FAO, Baghdad Water Week May 2025](https://www.fao.org/in-action/remote-sensing-for-water-productivity/news-and-events/news/news-detail/fao-highlights-data-driven-approaches-to-boost-irrigation-efficiency-during-baghdad-water-week/en); [MoWR–FAO agreement](https://iraq.un.org/en/194385-new-agreement-between-ministry-water-resources-and-fao-introduces-innovative-tools-monitor)
- [PILOT] **SRVALI** (Green Climate Fund, **USD 39 m**, launched 25 May 2025) for Karbala, Najaf and Muthanna — [FAO](https://www.fao.org/in-action/remote-sensing-for-water-productivity/news-and-events/news/news-detail/fao-highlights-data-driven-approaches-to-boost-irrigation-efficiency-during-baghdad-water-week/en)
- [CAPACITY] **[KRI]** In Feb 2026, FAO, Salahaddin University–Erbil and the KRG held a WaPOR seminar on water management and climate-smart agriculture — [FAO](https://www.fao.org/in-action/remote-sensing-for-water-productivity/news-and-events/news/news-detail/fao-and-salahaddin-university-hold-seminar-on-wapor-and-sustainable-water-management/en)
- [RESEARCH] Iraqi journal study of irrigation performance in Salah al-Din using WaPOR v3 and ArcGIS — [J. Water Resources (Iraq)](http://jwrg.gov.iq/index.php/jwrg/article/view/170)
- [INFRASTRUCTURE, not digital] **[KRI]** The KRG reports a water and wastewater project portfolio worth **more than USD 2 bn** — [Smart Water Magazine](https://smartwatermagazine.com/news/smart-water-magazine/kurdistan-region-advances-water-projects-worth-more-usd-2-billion)

### Inferences
- The year-to-year swing (about 10 bcm in 2025 to 34 bcm in 2026) shows that KRI's allocation problem is driven by forecasting and upstream conditions. A tool that combines **upstream reservoir monitoring** (satellite area or altimetry of Turkish and Iranian dams), snow cover, and WaPOR ET could serve both the MoWR and the KRG Ministry of Agriculture and Water Resources.
- WaPOR is already institutionally accepted in Baghdad and Erbil (Shemamuk). Building on it lowers adoption risk.

### Gaps
- No GIZ, JICA, World Bank, UNDP or USAID **digital** water project in KRI with published results was found. Searches surfaced only infrastructure and drip projects. A 2026 World Bank document (P515521) appeared in results but was not reviewed.
- No source was found for groundwater-decline rates or well counts in the Erbil and Sulaimani basins, or for exact Darbandikhan storage in 2025.
- KRG Ministry of Agriculture and Water Resources digital initiatives were not found in English sources.

## Q6. Known limits, documented failures, and why farmers do or don't follow AI water advice

### Takeaway
The biggest documented failure is the **irrigation-efficiency paradox**: subsidised "water-saving" tech, such as Morocco's 100%-subsidised drip, often **raises** total consumption unless extraction is capped. Adoption of scientific scheduling stays low even in rich countries (**13.1% of US irrigated farms use soil-moisture sensors**). Smallholders accept advice in principle (77% in Tunisia) but are limited by water-delivery uncertainty, trust and cost.

### Cited Findings
- [EVIDENCE] **Irrigation efficiency paradox** (Grafton, Steduto et al., *Science* 2018): higher efficiency "rarely reduces water consumption". Savings require basin water accounting, **a cap on extractions** and an understanding of irrigators' incentives — [IWMI WLE summary](https://archive.iwmi.org/wle/paradox-irrigation-efficiency-higher-efficiency-rarely-reduces-water-consumption/)
- [FAILURE] **Morocco:** drip is **100% subsidised**, yet total water consumption rose. Subsidised butane was diverted to pump groundwater, and irrigation expanded onto new land in the Berrechid plain (25-farm study) — [HAL/INRAE](https://hal.inrae.fr/hal-03671021); [IDEAS](https://ideas.repec.org/p/hal/journl/hal-05219597.html)
- [ADOPTION] **USDA 2023 Irrigation and Water Management Survey:** **13.1% of farms** use soil-moisture sensing devices (11.9% in 2018). "Condition of crop" and "feel of soil" remain the main methods — [NASS 2023 IWMS](https://nass.usda.gov/Publications/AgCensus/2022/Online_Resources/Farm_and_Ranch_Irrigation_Survey/iwms.txt); [Penn State Extension](https://extension.psu.edu/2023-agriculture-irrigation-and-water-management-survey)
- [ADOPTION BARRIERS] Rio Grande (TX) irrigator survey: barriers were **lack of weather data access, cost-effectiveness doubts, unfamiliarity, fear of reduced yields, and uncertainty about future water availability**, which made farmers reluctant to invest — [TWRI/AgriLife 2021](https://agrilifetoday.tamu.edu/2021/08/05/barriers-to-irrigation-technology-in-the-rio-grande/)
- [ADOPTION] **Tunisia:** only **54%** of farmers were satisfied with their water users' association, but **77% accepted using the proposed irrigation advisory service** (Water 2022) — [MDPI Water 14(22):3638](https://doi.org/10.3390/w14223638)
- [STRUCTURAL LIMIT] In canal systems, farmers often irrigate when water arrives, on the supplier's schedule, not when advised. Most farmers schedule "based on soil feel and crop condition or a schedule set by the water supplier" — [TWRI](https://twri.tamu.edu/news/2021/july/rio-grande-farmers-have-a-unique-relationship-with-irrigation-technology/)
- [FAILURE] Egypt's rice fines did not stop 210–309k ha per year of over-quota rice — [USDA FAS](https://www.fas.usda.gov/data/egypt-egyptian-parliament-approves-prohibition-rice-cultivation-non-designated-areas). Doñana legalisation pressure — [UNESCO](https://whc.unesco.org/en/decisions/8239). Iran's alleged military-backed wells — [Global Voices](https://globalvoices.org/2025/07/28/is-irans-water-crisis-fueled-by-military-backed-illegal-wells/)

### Inferences
- For a smallholder-facing tool in KRI, the realistic win is lower pumping and energy cost and protected yield in drought years. Claims of basin water "saved" need a cap or allocation rule behind them.
- Advice must fit how water actually arrives, whether canal rotation or a private well. In rotation systems the advice is about *how much* to apply, not *when*.

### Gaps
- No randomised or quasi-experimental study was found measuring compliance with AI irrigation advice among MENA or Kurdish farmers.
- No documented failure or abandonment case of a specific satellite irrigation advisory product was found, apart from general low adoption.
