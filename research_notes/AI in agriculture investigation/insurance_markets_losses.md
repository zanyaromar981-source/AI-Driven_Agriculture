# AI and Satellite Data on the Money and Market Side of Agriculture (insurance, credit, subsidies, prices, post-harvest), with an Iraq/Kurdistan focus

Research date: 2026-10-07. Labels used below: **[OPERATIONAL]** = running at national/commercial scale with reported results; **[EVALUATED]** = peer-reviewed or independent evaluation; **[PILOT]** = limited trial; **[VENDOR/SELF-REPORTED]** = figures from the implementer itself, not independently checked; **[UNVERIFIED]** = seen only in a search summary or a low-quality page.

## 1. Satellite/AI index insurance at scale (India PMFBY, IBLI/KLIP, ACRE, Pula, Morocco, Turkey) and credit scoring

### Takeaway
Satellite-based insurance now pays real money at national scale (India PMFBY, Zambia FISP via Pula, Kenya KLIP, Mexico CADENA), but satellites are mostly used as one input mixed with field crop-cutting, not as the sole payout trigger. The best-measured evidence shows high basis risk: Kenya's IBLI left buyers with about 69% of their original risk, and Zambia's weather index covered only about 30% of farmers' historical losses. Take-up also collapses without subsidies. Successful schemes are government-paid or bundled with seed or credit.

### Cited Findings

**India: PMFBY (world's largest crop insurance scheme) [OPERATIONAL]**
- Official Lok Sabha answer (4 Aug 2026), covering 2021-22 to 2025-26: **6,300.97 lakh (≈630 million) farmer applications** enrolled under PMFBY and RWBCIS. Rajasthan had 1,881.54 lakh, Madhya Pradesh 824.82 lakh and Maharashtra 778.92 lakh. — [Lok Sabha Unstarred Q 2644, MoA&FW, 04-08-2026](https://sansad.in/getFile/lsapps/loksabhaquestions/annex/188/AU2644_TsQCG4.pdf)
- Same answer, same 5 years: premiums were **Rs 18,351.69 cr from farmers, Rs 66,177.87 cr from states and Rs 55,738.64 cr from the Centre**. Insurers refunded **Rs 15,552.29 cr** to governments under the "cup-and-cap" and profit-sharing Alternate Risk Transfer Models introduced from Kharif 2023. **Claims reported: Rs 91,318.21 cr. Claims paid: Rs 86,731.59 cr.** — [Lok Sabha Q 2644](https://sansad.in/getFile/lsapps/loksabhaquestions/annex/188/AU2644_TsQCG4.pdf)
- Farmer premium caps: 2% of sum insured (Kharif), 1.5% (Rabi) and 5% (commercial/horticulture). Rules require claims to be settled within **21 days** of claim calculation on the national portal and receipt of subsidy. A **12% penalty** has been auto-levied on late insurers since Kharif 2024 and on states that pay late since Kharif 2025. State escrow accounts are mandatory from Kharif 2025. — [Lok Sabha Q 2644](https://sansad.in/getFile/lsapps/loksabhaquestions/annex/188/AU2644_TsQCG4.pdf)
- Technology stack named by the ministry:
  - **CCE-Agri app**, which captures crop-cutting experiment (CCE) data
  - the **Digiclaim** module (from Kharif 2022), which links the insurance portal to the public finance system for direct claim transfer
  - the **CLAP** crop-loss app, mandatory in all states, for localized and post-harvest losses
  - **YES-TECH**, a remote-sensing yield estimate given a **mandatory 30% weight** for paddy and wheat from Kharif 2023, with soybean added from Kharif 2024

  Source: [Lok Sabha Q 2644](https://sansad.in/getFile/lsapps/loksabhaquestions/annex/188/AU2644_TsQCG4.pdf)
- YES-TECH was implemented in **10 states**. In **7 states**, Kharif 2023 claims were paid on a YES-TECH basis (Lok Sabha statement, 21 Dec 2024). — [Global Agriculture](https://www.global-agriculture.com/india-region/can-yes-tech-technology-in-pmfby-provide-accurate-crop-assessments/)
- Since 2016-17 (all years to 2024-25): **Rs 1,92,477 cr** in claims paid, benefiting **23.22 crore farmer applications**. — [PIB note, as summarized in search results](https://www.pib.gov.in/PressNoteDetails.aspx?NoteId=155010&ModuleId=3&reg=3&lang=1)
- **Failure, delays:** the PMFBY dashboard showed **no Kharif 2024 claim settled as of Feb 2025**, more than four months after harvest. A 2025 IIM study found settlement delays of **up to 14 months**. The ministry blames states releasing their premium subsidy late (Assam, Bihar, AP, Telangana, MP, Rajasthan, WB, Gujarat). — [Ground Report](https://www.groundreport.in/groundreport/crop-insurance-delays-farmers-left-waiting-amid-climate-challenges-8766785/); [IASGyan](https://www.iasgyan.in/daily-current-affairs/pradhan-mantri-fasal-bima-yojana-systemic-failures)
- **Failure, satellite yields disputed:** Madhya Pradesh gave YES-TECH **100% weight** for select crops (soybean, wheat, paddy) from Kharif 2023, against the national 30% rule. A Centre for Science and Environment (CSE) field study in Sehore, Vidisha and Bhopal found farmers dissatisfied with delayed and inadequate payouts. Reporting also notes a "wide gap" between CCE and YES-TECH yields in Maharashtra districts, but gives no numbers. — [Down To Earth](https://www.downtoearth.org.in/agriculture/can-technology-make-indias-crop-insurance-payouts-more-accurate); [Down To Earth – Beyond the Lens](https://www.downtoearth.org.in/agriculture/beyond-the-lens)

**Kenya/Ethiopia: IBLI and KLIP (livestock, NDVI-based)**
- [EVALUATED] Jensen, Barrett and Mude (2016, *Am. J. Agric. Econ.* 98:1450–69), using 8 semi-annual seasons of panel data: IBLI cut exposure to covariate (shared) high-loss risk by an average of **63%**. Because individual herd losses are large, however, buyers were still left with **69% of their original risk** from high-loss events. The authors urge caution in promoting index insurance. — [ILRI](https://www.ilri.org/knowledge/publications/index-insurance-quality-and-basis-risk-evidence-northern-kenya); [IDEAS/RePEc](https://ideas.repec.org/a/oup/ajagec/v98y2016i5p1450-1469..html)
- [EVALUATED] After the 2011 drought payout (Janzen & Carter), insured households were **22–36 percentage points less likely** to expect to sell off assets. They were **27–36 pp less likely** to expect to cut meals. IBLI launched Jan 2010 in Marsabit and expanded to Isiolo and Wajir in Kenya and Borana in Ethiopia, reaching **4,000+ pastoralists** in the evaluated phase. The Ethiopia trigger fires when cumulative NDVI deviation falls below the **15th percentile**. — [CGIAR CSA Guide](https://csaguide.cgiar.org/csa/index-based-livestock-insurance-ibli-for-nomadic-pastoralists-in-northern-kenya-and-southern-ethiopia)
- The Kenya contract pays when predicted area livestock mortality reaches **15%**. The index was revised in 2015 so it pays earlier, before drought impacts peak. Payouts go via M-Pesa. — [BASIS/UC Davis policy brief](https://basis.ucdavis.edu/publication/policy-brief-assessing-impacts-livestock-insurance-kenya); [ResearchGate IBLI](https://www.researchgate.net/publication/383052145_Index-based_livestock_insurance)
- **Failure, take-up collapse:** in Isiolo, Takaful's IBLI sales fell from **2,653 households (2018) to 549 (2019)**, and many counties have recorded low purchases since 2019. A 2025 survey found **80%** knew IBLI but only **33%** of those bought it. — [Boresha IBLI technical brief](https://boreshahoa.org/wp-content/uploads/2022/03/Boresha-IBLI-Technical-Brief-Final-web.pdf); [ScienceDirect 2025, "Uninsured Pastoralists"](https://www.sciencedirect.com/science/article/pii/S1550742425001083)
- [OPERATIONAL] The government-funded **KLIP** pays insurers premiums on behalf of **18,000+ vulnerable households**, with 100% premium support for 5 Tropical Livestock Units (TLU) each. Payouts go to every registered pastoralist in an area when satellite NDVI forage falls below a threshold. — [UNFCCC KLIP paper](https://unfccc.int/sites/default/files/resource/Aligning%20KLIP%20to%20UNFCCC%20De-risking%20strategies%20to%20enhance%20adaptation%20finance.pdf); [UC Davis KLIP slides](https://i4.ucdavis.edu/sites/g/files/dgvnsk466/files/files/event/S8%20Richard%20Kyuma-Kenya%20Livestock%20Insurance%20Program.pdf)

**ACRE Africa (Kenya, Rwanda, Tanzania) [VENDOR/SELF-REPORTED + early impact study]**
- **1.7 million+ contracts** and **8.5 million beneficiaries**. The flagship **Replanting Guarantee** is bundled with each maize seed bag: the farm is geotagged and watched by satellite for 21 days after planting, and the payout goes to M-Pesa. — [GIIF/ACRE](https://www.globalinclusiveinsuranceforum.org/news/acre-africa-protecting-rural-africa-through-creative-partnerships-and-technology)
- 2013 snapshot:
  - **187,467 farmers**: Kenya 67,607, Rwanda 115,550, Tanzania 4,310
  - **$12.3M** sum insured, but only **$370,405** paid out
  - **97%** had insurance-linked loans: **177,782 farmers received $8.4M** in credit

  A 2012 impact study found insured farmers invested **19% more** and earned **16% more**. — [ACRE partner profile (GIIF)](https://www.indexinsuranceforum.org/sites/default/files/1505755-PartnerProfiles_ACRE.pdf)

**Pula / Zambia FISP (largest African crop index payouts)**
- [VENDOR/SELF-REPORTED] Pula reports **22 million farmers insured** and **$133.9M total payouts**. — [IFC profile of Pula, 2024](https://www.ifc.org/content/dam/ifc/doc/2024/pula-ifc-2024.pdf); [Pula About](https://www.pula-advisors.com/about)
- [OPERATIONAL] Zambia, 2023/24 drought season: **over 800 million kwacha** paid to **500,071 smallholders** under the government Farmer Input Support Programme (FISP), the largest crop insurance payout in Zambia's history. The payout event was 11 June 2025. Insurers were ZSIC, PICZ, Savenda and Madison, working with the Ministry of Agriculture. — [Pula](https://www.pula-advisors.com/post/zambia-s-largest-crop-index-insurance-payout-marks-historic-milestone-for-smallholder-farmers)
- Other Zambia figures don't match one another, so the totals should not be added:
  - Pula elsewhere says **$39M** was paid in 2024 to over 1 million insured farmers. — [Pula About](https://www.pula-advisors.com/about)
  - The Insurer (Jan 2025) headlined "Record **$34.4mn** payouts triggered" on the national scheme. — [The Insurer](https://prod.theinsurer.com/parametric-insurer/news/record-344mn-payouts-triggered-on-zambias-national-agriculture-insurance-scheme-2025-01-08)
- How the products work: Area Yield Index Insurance compares end-of-season yield in an agro-ecological zone with its historical average. Satellite imagery replaces some field auditors. — [CGAP–Pula](https://www.cgap.org/news/cgap-and-pula-partner-to-bring-satellite-based-insurance-to-farmers)
- **[EVALUATED] Basis risk:** IRI (Columbia) found Zambia FISP's satellite-rainfall weather index covered only about **30% of farmers' reported historical risks**. Adding a yield-audit component would raise this to **50–60%**. — [IRI Zambia FISP yield audit 2023](https://fist.iri.columbia.edu/publications/docs/zambia_fisp_yieldaudit_2023)

**Documented failure: African Risk Capacity, Malawi 2016 (satellite-rainfall sovereign drought index)**
- The Africa RiskView model first estimated only **20,594 people** affected, too few to trigger a payout. Meanwhile, Malawi had declared a national emergency (April 2016), WFP estimated **6.5 million** people hungry, and the response cost **$395M**. On review, ARC found farmers had switched to a crop variety the model wasn't calibrated for. After recalibration the estimate became about **2 million** affected. **$8.1M** was agreed in Nov 2016 and released in **Jan 2017, nine months after the emergency**. ActionAid called it "too little, too late." — [Climate Home News](https://www.climatechangenews.com/2017/05/25/g7-backed-insurance-little-late-malawi-drought-report-finds/); [Artemis](https://www.artemis.bm/news/2016/11/17/african-risk-capacity-in-8m-parametric-insurance-payout-to-malawi/); [ReliefWeb/ActionAid](https://reliefweb.int/report/malawi/wrong-model-resilience-how-g7-backed-drought-insurance-failed-malawi-and-what-we-must)

**Mexico CADENA (government-paid index insurance replacing ad hoc relief) [OPERATIONAL]**
- Started 2003. The state or federal government pays the premium, which is how near-national coverage was reached. By 2013 it insured **6 million+ hectares**. The weather index covers rainfed staple farmers with under 20 ha in three crop phases. — [World Bank CADENA brief](https://documents1.worldbank.org/curated/en/124521468287160777/pdf/881000BRI0P1300urance04Pager0Cadena.pdf); [FERDI impact evaluation](https://ferdi.fr/publications/weather-indexed-insurance-and-productivity-of-small-scale-farmers-an-impact-evaluation-of-mexico-s-cadena-program)
- [EVALUATED] A payout led to more hectares of maize sown the following year. — [FERDI](https://ferdi.fr/publications/weather-indexed-insurance-and-productivity-of-small-scale-farmers-an-impact-evaluation-of-mexico-s-cadena-program)

**Morocco: Mamda "multirisque climatique" (nearby region, cereals/drought) [OPERATIONAL]**
- Created in 2011 by agreement with the state. It covers cereals and legumes against drought, excess water, frost, hail, wind and sandstorms. The target was 300,000 ha in 2011/12, rising to **1 million ha by 2015**, and the **1.2 billion MAD** in guaranteed indemnities corresponds to 1M ha (Jan 2016). Indemnities follow a ministry calamity declaration and, in some provinces, a Mamda investigation. — [TelQuel 2016](https://telquel.ma/2016/01/04/secheresse-12-milliards-dirhams-les-agriculteurs_1476299); [FNH](https://www.fnh.ma/article/actualite-economique/Assurance%20agricole:%20la%20multirisque,%20un%20bouclier%20contre%20les%20al%C3%A9as%20climatiques)
- The MAMDA-MCMA group insures **320,000+ farmers** (2026, aggregator page). — [wafir.ma](https://wafir.ma/en/guide/multirisque-agricole-mamda-mcma-maroc-2026-assurance-exploitations)

**Turkey: TARSIM (neighbor of Iraq/KRI) [OPERATIONAL]**
- A state-supported pool set up by Law 5363 (2005), operating since 2006, with **27 member insurers** and a government premium subsidy of **up to 70%**. — [GIIF blog](https://www.globalinclusiveinsuranceforum.org/blog/tarsims-role-advancing-agricultural-insurance-turkiye); [MAPFRE Re paper](https://app.mapfre.com/ccm/content/documentos/mapfrere/fichero/en/subsidised-agricultural-insurance-in-Turkey.pdf)
- **Village-Based Drought Yield Insurance** (Köy Bazlı Kuraklık Verim Sigortası) covers dryland wheat, barley, rye, oats, triticale, chickpea and lentils:
  - The village's actual yield is measured on **reference plots chosen by TARSIM** at harvest.
  - Phenology observers visit during the season.
  - Farmers don't need to file individual damage notices.

  This is an area-yield index on field plots, a direct model for rainfed wheat in KRI. — [TARSIM general conditions 2024 (PDF)](https://www.tarsim.gov.tr/staticweb/krm-web/mevzuatlar/genel-sartlar/2024/kbkvs-genel-sartlar.pdf)

**Farmer credit scoring with satellite/ML**
- [VENDOR/SELF-REPORTED] **Apollo Agriculture** (Kenya, Zambia) runs machine-learning models on satellite data to infer farm traits such as expected yield, combined with credit-bureau data where available, to score credit. Maize input loans are **KES 15,000–24,000 (~$115–180)** on an 8-month schedule with a balloon payment after harvest. — [GSMA](https://www.gsma.com/solutions-and-impact/connectivity-for-good/mobile-for-development/blog/ai-driven-smallholder-farmer-lending-in-africa-insights-from-apollo-agriculture/)
- [UNVERIFIED] Apollo has "500,000+ farmers, ~$120M cumulative loans, default ~6% by March 2026". Seen only in a search summary from a low-quality template site; don't cite as fact. — [search-result source](https://businessmodelcanvastemplate.com/products/apollo-agriculture-swot-analysis)
- Insurance-linked credit: ACRE's 2013 figures above (177,782 farmers, $8.4M in loans). — [ACRE profile](https://www.indexinsuranceforum.org/sites/default/files/1505755-PartnerProfiles_ACRE.pdf)

### Inferences
- PMFBY's 5-year claims paid (Rs 86,732 cr) against gross premium (Rs 140,268 cr) give a loss ratio of about 62%. Net of the Rs 15,552 cr refunded under the risk-transfer models, it is about 70%. Calculated from the [Lok Sabha Q 2644](https://sansad.in/getFile/lsapps/loksabhaquestions/annex/188/AU2644_TsQCG4.pdf) numbers. Claims paid were about 95% of claims reported (86,732 / 91,318).
- Every large-scale success has the government paying all or most of the premium: PMFBY (up to 98%), KLIP (100%), CADENA (100%), Zambia FISP (premium bundled into the input subsidy), TARSIM (up to 70%). Voluntary retail index insurance (IBLI commercial sales) shows take-up collapsing. For Iraq/KRI, any scheme would realistically need to be bundled with the state wheat-purchase or seed programs.
- Satellites are mostly blended with field data, not used alone: YES-TECH at 30%, TARSIM's reference plots, and Pula/IRI pairing satellite with crop cuts. The one place a state set satellite weight to 100% (Madhya Pradesh) drew farmer complaints.

### Gaps
- No published YES-TECH accuracy statistic (e.g., RMSE versus CCE yields) was found. CSE and DTE describe a "wide gap" but give no numbers.
- No evidence was found that TARSIM uses satellite imagery operationally for loss assessment or fraud detection. The Turkish conditions document describes field reference plots only.
- No evidence was found that Mamda's multirisk product uses a satellite index.
- Apollo's default rate and scale are not independently verified. SatSure and Indian bank satellite credit scoring were not researched.
- No total payout figures were found for IBLI/KLIP across years.

## 2. Satellite checks of government farm subsidies and drought compensation (EU CAP Area Monitoring System, India fraud, satellite-based drought declarations)

### Takeaway
The EU has switched from checking a 5% sample of claims by field visit to watching 100% of claimed land with Sentinel-1/2. The EU says this corrected over 3 million hectares of claims in 2024. However, no published euro figure for administrative savings was found. India uses satellite vegetation indices as one formal trigger for drought declarations, which unlock relief money. India's biggest documented insurance fraud (Maharashtra, 2024-25) was caught by cross-checking applications, and no source credits satellites.

### Cited Findings

**EU CAP: checks by monitoring, then the Area Monitoring System (AMS) [OPERATIONAL]**
- Since 2018, EU paying agencies may use Sentinel data, geotagged photos and drones in place of field visits ("checks by monitoring"). The first agency started in **May 2018 in Foggia, Italy**. In **2019, 15 paying agencies** in Belgium, Denmark, Italy, Malta and Spain used it for some schemes, and another 13 agencies in 8 member states planned to start in 2020. — [ECA Special Report 04/2020 press release](https://www.eca.europa.eu/lists/ecadocuments/insr20_04/insr_new_technologies_in_agri-monitoring_en.pdf)
- Area-based aid makes up **almost 80%** of EU funding for agriculture and rural development. In 2019, **no** paying agency used checks by monitoring for environmental or climate conditions, partly because some can't be checked with Sentinel alone. Agencies feared the Commission would challenge satellite-based decisions. — [ECA SR 04/2020](https://www.eca.europa.eu/lists/ecadocuments/insr20_04/insr_new_technologies_in_agri-monitoring_en.pdf)
- The AMS became mandatory from **1 Jan 2023** and had to be fully operational in all member states by **1 Jan 2024**. — [Council document ST-12971-2022](https://data.consilium.europa.eu/doc/document/ST-12971-2022-INIT/en/pdf)
- It replaces the old protocol of on-the-spot field checks on **at least 5%** of applications with monitoring of **100%** of area claims. Ireland, for example, ended 5% inspections for its basic income support and areas-of-natural-constraint (BISS/ANC) schemes in 2023 and relies on **Sentinel-1 radar** because of cloud cover. — [Irish Farmers Journal, Jan 2023](https://www.farmersjournal.ie/satellite-inspections-every-farmer-to-be-monitored-for-biss-and-anc-in-2023-742991)
- **"In 2024 alone, over 3 million hectares were corrected thanks to AMS."** The EU's integrated administration and control system (IACS) covers more than 90% of EU agricultural land. (European Commission, DG AGRI, 19 Sep 2025.) — [DG AGRI news](https://agriculture.ec.europa.eu/media/news/satellite-tech-and-smart-data-take-root-europes-farming-future-2025-09-19_en)
- Parcels get traffic-light flags (conclusive, doubtful or not payable). Doubtful parcels go to follow-up, such as a geotagged-photo request or a field visit. — [Tools4CAP AMS brief, Nov 2024](https://www.tools4cap.eu/wp-content/uploads/2024/11/BR3-Area-Monitoring-System.pdf)
- [VENDOR] Sinergise, which builds AMS systems, estimates farmer claims contain about **2–3% errors**, which is the business case for monitoring every parcel. — [Sinergise/Sentinel Hub blog](https://medium.com/sentinel-hub/area-monitoring-concept-effc2c262583)
- [EVALUATED] Hungary ran nationwide Sentinel-2 monitoring of its single area payment and greening subsidies. — [MDPI Remote Sensing 2022](https://www.mdpi.com/2072-4292/14/16/3917)

**India: fraud in subsidized crop insurance**
- [OPERATIONAL] Maharashtra (2024-25 cycle): **5.9 lakh bogus applicants** detected. The state and Centre had already paid **Rs 478.5 cr** in premium on them. **4.14 lakh** claims (about **2.5%** of applications) were rejected as fraudulent. Fake claims covered crops never sown, government land, petrol stations and religious sites. **96 Common Service Centres** were suspended for filing false applications to earn fees. The state estimates **Rs 80 cr** was saved, and potential exposure in a calamity year was **Rs 6,000 cr**. The state later scrapped its Re 1 premium scheme. — [GKToday](https://www.gktoday.in/maharashtra-uncovers-fraud-within-pmfby/); [DT Next](https://www.dtnext.in/news/national/irregularities-found-in-crop-insurance-scheme-shrines-shown-as-farmlands-minister-820126)
- The ministry's stated anti-leakage tools are integrating **state land records** with the national insurance portal, letting insurers witness crop-cutting experiments, the CCE-Agri app and YES-TECH. — [Lok Sabha Q 2644](https://sansad.in/getFile/lsapps/loksabhaquestions/annex/188/AU2644_TsQCG4.pdf)

**Governments paying drought relief or compensation from satellite maps**
- [OPERATIONAL] India's **Manual for Drought Management (2016)** uses two triggers:
  - **Trigger 1 (mandatory):** rainfall deficit or the standardized precipitation index (SPI), plus dry spells.
  - **Trigger 2:** at least 3 of 4 impact indicators, one being **remote sensing**: NDVI/NDWI deviation or the Vegetation Condition Index (VCI), from MODIS 250 m fortnightly data in the national NADAMS system run by the NRSC.

  A declaration releases state and national disaster relief funds (SDRF/NDRF) and enables crop-insurance claims and loan restructuring. — [PreventionWeb – Manual 2016](https://preventionweb.net/quick/46916); [NRSC drought monitoring slides](https://saarc-sdmc.gujarat.gov.in/sites/default/files/programmes_doc_upload/SDMC-Day-3-EO-data-DroughtMonitoring-India-NRSC.pdf); [SuperKalam explainer](https://superkalam.com/current-affairs/articles/drought-declaration-criteria-india-ndrf)
- [OPERATIONAL] **Kenya KLIP** and **Mexico CADENA** are government budget lines that pay automatically from satellite NDVI or weather indices instead of ad hoc relief (see Section 1).

### Inferences
- The EU model, in which every claimed parcel is checked by Sentinel and only flagged ones get a photo or visit, transfers directly to a "claimed wheat area vs. satellite-observed wheat area" check. The relevant check in Iraq/KRI is whether area quotas per dunam are being claimed on land that was never sown (see Section 5).
- The Maharashtra pattern (non-farm land and crops never sown) is exactly what a Sentinel-2 crop-presence check catches. No source says Maharashtra used satellites, though.

### Gaps
- No quantified euro savings or field-inspection-hour reductions for the AMS were found in an official source. The ECA 2020 full report probably has cost figures, but only the press release was readable.
- No error or false-flag rate was found for AMS signals, such as the share of parcels flagged as inconclusive.
- No source was found on how Maharashtra's fraud was technically detected.
- No independent study was found on how accurate India's satellite drought triggers are.

## 3. AI crop price/market forecasting used by governments or farmers

### Takeaway
India has the most documented government price forecasting, but the published, evaluated systems are classical statistical models (ARIMA/GARCH), not modern AI. They report about 80–90%+ "accuracy" for cereals, pulses and oilseeds, much worse for onion, potato and tomato, and only vague evidence of farmer income gains. Newer "AI" claims (India's consumer-affairs price forecasts 30–60 days ahead) have no published error metrics.

### Cited Findings
- [EVALUATED, government research network] ICAR-NIAP Policy Paper 34 (Saxena et al., 2019) covers the NAIP/ICAR market-intelligence network of state agricultural universities, which issued pre-sowing and pre-harvest price forecasts. Accuracy is derived from MAPE (mean absolute percentage error):
  - **Cereals: about 90%.** Maize was **77–89%** at the Dhule market.
  - **Pulses: over 80%** in 2014–15, lower for pre-sowing in 2016.
  - **Oilseeds: over 90%** for pre-harvest forecasts.
  - **Cotton: 90%.**
  - **Vegetables: lowest**, because onion, potato and tomato prices swing wildly, though cabbage, chilli and green pea were over 90%.
  - **Fruits: over 80%**, except pear, cherry and pineapple. Mango was **60%** in Uttar Pradesh in 2016.

  The models were ARIMA, SARIMA, GARCH, E-GARCH and VAR. — [NIAP Policy Paper 34](https://ncap.res.in/pdf/publication/policy-paper/Market%20Intelligence%20in%20India.pdf)
- The same paper cites earlier claims of forecasts for **34 crops at "90 to 100 per cent accuracy"** (Acharya 2017). It says adopters of market advisories had higher income than non-adopters (NAIP 2014), without giving a magnitude. At least **30 farmers per commodity per state** were tracked. Forecasts went out via newspapers, SMS/voice SMS (IFFCO Kisan), radio, TV and farmer fairs. — [NIAP Policy Paper 34](https://ncap.res.in/pdf/publication/policy-paper/Market%20Intelligence%20in%20India.pdf)
- [OPERATIONAL, accuracy unpublished] The Price Monitoring Cell of India's Department of Consumer Affairs collects daily prices for **22 essential commodities**, now from about **550 centres**. It uses an "algorithm-based" model to predict prices **30, 45 and 60 days ahead**, described as "with a lot of accuracy," and has a pulses retail-price model built on benchmark mandi and import prices. — [Mint via PressReader, 28 Mar 2024](https://www.pressreader.com/india/mint-hyderabad/20240328/281621015340366); [Rajya Sabha PQ, 3 Dec 2024](https://rsdebate.nic.in/bitstream/123456789/754436/1/PQ_266_03122024_U847_p272_p272.pdf)
- [OPERATIONAL] TNAU's Domestic and Export Market Intelligence Cell (DEMIC, Tamil Nadu) has issued price forecasts since **Nov 2004**, about **150 forecasts in ~6 years**. They use 15–20 years of price history, futures prices and trader surveys. No accuracy statistics are published. — [TNAU DEMIC](https://agritech.tnau.ac.in/govt_schemes_services/govt_serv_schems_nadp_tnau_11_12_Market.html)

### Inferences
- For a hackathon, government-grade price forecasting in India is simple time-series modeling plus wide distribution (SMS). The value is in reach and timing more than in model sophistication.
- In Iraq, the wheat price is set administratively (Section 5), so price forecasting matters little for wheat. It matters more for vegetables, where Indian experience shows accuracy is worst.

### Gaps
- USDA WASDE forecast accuracy, Kenyan systems, FEWS NET price projections and private AI price apps were not researched within the tool budget.
- No evaluation was found of the Department of Consumer Affairs model's error rate or policy use, such as when buffer-stock releases are triggered.

## 4. AI in post-harvest losses and storage (silos, cold chains): measured impact

### Takeaway
Measured post-harvest gains come overwhelmingly from physical technology: hermetic storage, solar cold rooms and cleaning equipment. AI and IoT add-ons have mostly vendor or review-level claims. The best independent study found in this search (solar cold storage in Nigeria, 7 treated vs. 7 control markets) used no AI.

### Cited Findings
- About **14%** of the world's food is lost between harvest and retail, from on-farm activities, storage and transport (FAO SOFA 2019). — [Global Agriculture/FAO](https://www.globalagriculture.org/?p=7825)
- [EVALUATED, no AI] Solar-powered cold storage in Nigeria, comparing 7 markets with units against 7 controls, with 251 market agents:
  - The value of losses as a share of gross revenue fell by **about 11%**.
  - Sales volumes and revenues rose by **about 70%**, and sale prices by **up to 20%**.
  - Each unit cost **$40,000**, earns about **$8,000 a year** in net profit and pays back in **about 10 years**.

  (Gustafson, *Agricultural Economics*, July 2023.) — [IFPRI Food Security Portal](https://ssa.foodsecurityportal.org/node/2484)
- [VENDOR/SELF-REPORTED] Ecozen's Ecofrost solar cold rooms in India are app-controlled for temperature and humidity. The company's own research claims farmer incomes rose **50–100%**. — [Futures Centre](https://www.thefuturescentre.org/signal/solar-powered-cold-rooms-improve-profits-for-farmers/)
- [OPERATIONAL, no AI] A project of India's National Centre for Cold-chain Development (2016-17) cut kinnow (mandarin) losses by **76%**. — [Down To Earth (via search result)](https://www.downtoearth.org.in/amp/story/food/sustainable-cold-chains-can-address-climate-food-crises-unep-fao-report-85966)
- [EVALUATED review, mostly non-AI] Kumar & Kalita (2017) report that improved storage practices and new technologies (hermetic etc.) cut storage losses by about **98%**, regardless of crop or storage period. — [PMC5296677](https://pmc.ncbi.nlm.nih.gov/articles/PMC5296677/)
- [REVIEW-LEVEL CLAIMS, AI/IoT] A 2025 critical review cites an IoT and AI grain-storage monitor in Brazil that cut spoilage by **10%** but struggled with rural connectivity. It also cites "AI-driven cold storage" achieving **60%** less perishable loss. Primary studies were not checked. — [ScienceDirect review 2025](https://www.sciencedirect.com/science/article/pii/S2666188825008470)
- [MODELING] Grain stored in poor conditions loses **0.5–18%** dry matter. An IoT and machine-learning framework (temperature, humidity, moisture and CO₂ sensors) has been proposed to predict and prevent spoilage. — [Springer 2026](https://link.springer.com/article/10.1007/s10751-026-02774-6)
- Relevant KRI non-AI example: a new **wheat-cleaning facility in Shekhan** was reported to end long-standing grain rejections at silos. — [Kurdistan24](https://www.kurdistan24.net/en/story/920652)

### Inferences
- An AI/satellite project that claims to "reduce post-harvest loss" would have weak evidence behind it. Stronger angles are forecasting harvest volume so silos and trucks can be planned (KRI silos are overwhelmed in record years, Section 5) and rejection or quality prediction at intake.

### Gaps
- No independently evaluated AI-specific post-harvest intervention with a measured loss-reduction percentage was found.
- No figures were found on Iraqi or KRI silo storage losses.

## 5. Iraq/Kurdistan: state wheat purchase, payments, compensation, fraud and smuggling, insurance, satellite use

### Takeaway
Iraq buys wheat from farmers at a heavily subsidized administered price, recently cut from 850,000 to 700,000 IQD/t within the plan and 500,000 outside it. Quotas are set per dunam: 750–900 kg/dunam in central and southern Iraq versus only 88.5 kg/dunam allocated to KRI in 2026. Payments to KRI farmers have run years late, with hundreds of billions of IQD outstanding. The large gap between market and silo prices has fed documented fraud: merchant wheat sold under farmers' names, bribes at silos, and imported or smuggled wheat (including via KRI from Iran, Turkey and Syria) passed off as local. This dispute is central to Baghdad–Erbil quota fights. No source found shows Iraq or the KRG using satellites to verify wheat areas or claims, and no functioning crop insurance was found.

### Cited Findings

**Prices and quotas (2024–2026)**
- 2025/26 season: wheat "within the plan" is bought at **700,000 IQD/t (~$534)** and "outside the plan" at **500,000 IQD/t (~$381)**. Yield norms used to cap purchases are **900 kg/dunam** for modern irrigation, **750 kg/dunam** for flood irrigation and **~300 kg/dunam** for rainfed land. The Council of Ministers approved settling arrears for 2024–25 and 2025–26 through the budget, with Trade Bank of Iraq guarantees. — [Milling MEA](https://millingmea.com/iraq-sets-new-wheat-prices-and-secures-seeds-for-2025-to-2026-season/); [UkrAgroConsult](https://ukragroconsult.com/en/news/iraq-sets-wheat-purchase-prices-and-prepares-for-harvest-season/)
- The previous price was **850,000 IQD/t**. The cut (−150,000 for registered farmers, −350,000 for unregistered) triggered protests in Baghdad (Tahrir Square), Kirkuk and Diyala in **May 2026**. Farmers demand quotas above 1 t/dunam and payment of arrears. — [Kurdistan24, 10 May 2026](https://www.kurdistan24.net/en/story/913197/iraqi-farmers-protest-wheat-price-cuts-in-baghdad-and-kirkuk)
- **2026 KRI allocation:**
  - **292,000 t** within the plan at 700k IQD and 108,000 t outside the plan at 500k, for a **400,000 t** total.
  - By province: Sulaymaniyah 150,209 t, Erbil 122,680 t, Duhok 116,083 t, Halabja 11,028 t.
  - Baghdad's national target: **4.5 Mt**.
  - KRI introduced **mandatory online registration with SMS delivery slots** and on-site verification; a missed slot means re-registering.

  Source: [Kurdistan24, 13 Jun 2026](https://www.kurdistan24.net/en/story/919617/kurdistan-region-upgrades-wheat-procurement-strategy-as-record-harvest-challenges-local-supply-chains)
- **KRG protest (15 Apr 2026):** the KRI quota equals **88.5 kg/dunam**, against **750–900 kg/dunam** in central and southern Iraq.
  - KRI wheat area is **3.3 million dunams**, with output expected above **2.5 Mt** after 600–1,000 mm of rain.
  - The KRG asked Baghdad to buy **1.25 Mt** (50%).
  - KRI purchases fell from **700,000 t (2024)** to **400,000 t (2025)** to **292,000 t within plan (2026)**.
  - Baghdad's planned purchase from central and southern provinces is **3.8 Mt**.

  Source: [Kurdistan24, 19 Apr 2026](https://www.kurdistan24.net/en/story/908985/kurdistan-region-condemns-baghdads-wheat-policy-as-unjust-toward-farmers)
- KRSO statistics, 2022-23: **4,409,376 dunams** of wheat, **68,704** winter-crop farmers, **499,633 t** delivered to KRI silos in 2023. — [Kurdistan Region Statistics Office](https://krso.gov.krd/en/indicator/agriculture)
- KRI has **11 operational silos** with about **1 Mt/yr** capacity (2024). — [Kurdistan24 2024](https://www.kurdistan24.net/en/story/395285/KRG-Begins-Wheat-Collection-from-Farmers-for-2024)
- Federal side: **78 silos and storage sites** were readied for the 2025/26 season. — [Milling MEA](https://millingmea.com/?p=2039861)
- **Conflicting data:** Al Jazeera Arabic (Apr 2025) quotes the government claiming **6.4 Mt** of 2024-25 production against **5–5.2 Mt** of consumption, and a price of "450,000 IQD/t". That price conflicts with the 850,000 IQD/t reported elsewhere for the same period and is likely an error. Critics say Iraq still imports wheat to **blend at about 30%**. — [Al Jazeera Arabic, 4 Apr 2025](https://www.aljazeera.net/ebusiness/2025/4/4/%D8%A5%D8%B9%D9%84%D8%A7%D9%86-%D8%A7%D9%84%D8%A7%D9%83%D8%AA%D9%81%D8%A7%D8%A1-%D8%A7%D9%84%D8%B0%D8%A7%D8%AA%D9%8A-%D8%A7%D9%84%D8%B9%D8%B1%D8%A7%D9%82%D9%8A-%D9%84%D9%84%D8%AD%D9%86%D8%B7%D8%A9)
- **Nationwide:** Iraq received over **5 Mt** of wheat by 19 July 2025. — [Rudaw, 14 Oct 2025](https://www.rudaw.net/english/kurdistan/14102025)

**Delayed payments to KRI farmers**
- 2025 harvest: **400,000 t** bought from KRI at **850,000 IQD/t**, worth over **300 bn IQD (~$229M)**. By 14 Oct 2025 Baghdad had sent only **72.5 bn IQD** in two instalments (57 bn and then 15.5 bn). — [Rudaw, 14 Oct 2025](https://www.rudaw.net/english/kurdistan/14102025)
- The prime minister authorized **284 bn IQD (~$217M)** for KRI farmers' delayed wheat payments. Reported date is uncertain: one summary says Nov 2024, while the page header shows 2026. More than **141 bn IQD** from the 2025 campaign was reported unpaid. — [Shafaq News](https://shafaq.com/en/Kurdistan/Iraq-allocates-284B-dinars-to-Kurdish-farmers-amid-wheat-payment-delays)
- Earlier (Feb 2017): KRI farmers had been unpaid for 3 years, with a debt of **902 bn IQD**. Baghdad's compensation package (~**900 bn IQD**) paid KRI farmers **17%** versus **80%** for farmers elsewhere in Iraq. — [Rudaw, 28 Feb 2017](https://rudaw.net/english/kurdistan/280220172)
- Federal payments are legally due within **72 hours** of delivery. In 2020, crops sold at the end of April were still unpaid in early June. — [Al Jazeera, Jun 2020](https://www.aljazeera.com/economy/2020/6/10/rampant-corruption-scorches-iraqs-grain-farmers)

**Fraud and smuggling in wheat sales**
- In 2020 the silo price was **$466, $391 and $308/t** by grade, against a wholesale market price of about **$354/t (425,000 IQD)**. Trucks paid bribes of **1–2 million IQD ($800–1,600)** to pass quality control. Farmers sell to wholesalers, who "swap" the grain into silos at the preferential price. The Grain Board runs about **50 silos**, holds about **$2 bn** of grain, rejects about **10%** of deliveries and had about 10 staff under investigation. — [Al Jazeera, Jun 2020](https://www.aljazeera.com/economy/2020/6/10/rampant-corruption-scorches-iraqs-grain-farmers)
- A parliamentary report (MP Hamid al-Mosawi, Apr 2020) found merchants selling wheat under farmers' names, manipulated grading, and "collusion between the traders, smugglers and corrupted employees" to sell **wheat imported from the Kurdistan Region and neighbouring countries** to silos as local produce. — [Al Jazeera, Jun 2020](https://www.aljazeera.com/economy/2020/6/10/rampant-corruption-scorches-iraqs-grain-farmers)
- Baghdad has accused Erbil of **mislabeling wheat from Syria, Turkey and Iran** as KRI-grown, which it used to justify refusing to buy or compensate KRI crops (2017). — [Rudaw, 28 Feb 2017](https://rudaw.net/english/kurdistan/280220172); [Rudaw 2016 (tougher regulations)](https://www.iraqoilreport.com/daily-brief/official-tougher-iraqi-regulations-impact-kurdish-wheat-farmers-18714/)
- Recent arrests by Iraq's Federal Integrity Commission (dates not given on the pages):
  - the head and two members of the **Amarah (Maysan) silo** intake committee, for unauthorized fees charged to farmers
  - the deputy director of the **Al-Ishaqi (Salahaddin) grain complex**, for taking money to accept or reclassify wheat

  Source: [964media](https://en.964media.com/49319/); [Hathalyoum](https://en.hathalyoum.net/articles/299870)
- Earlier silo staff were investigated for **forging transport documents** (2016), and a former Grain Board chief was investigated in a graft case. — [Grainews](https://www.grainews.ca/daily/iraq-grain-board-chief-to-be-investigated-in-graft-case)

**Drought, compensation and insurance status**
- The KRG's **Law No. 4 of 2008** obliges the Ministry of Agriculture to compensate owners for crops damaged by natural disasters. — [FAOLEX/UNEP LEAP](https://leap.unep.org/en/countries/iq/national-legislation/law-no-4-2008-protection-and-development-agricultural-production)
- Nineveh: rainfall in the latest season was about **100 mm** against 500 mm the year before, and **43.3 mm** in Mosul at its lowest. Wheat output fell about **1 Mt** year on year. In Sinjar a loss-assessment **committee** is measuring damage while farmers wait for compensation. The Grain Trading Company director says the budget is tight. — [Jummar, Dec 2025](https://jummar.media/en/2025/12/08/bread-has-become-a-farmers-sorrow-drought-shrinking-farmland-and-abandoned-harvests-in-nineveh/); [Rudaw, May 2025](https://www.rudaw.net/english/middleeast/iraq/27052025)
- Iraq's state insurers (National Insurance Co., Iraqi General Insurance and the state reinsurer) are named in academic work on agricultural insurance. No operating crop insurance product for wheat farmers was found. — [Iraqi Academic Scientific Journals](https://iasj.rdd.edu.iq/journals/uploads/2024/12/14/69c7cb53312e2f7d6bb3887a76105df3.pdf)
- Satellite use: FAO GIEWS and others use NDVI to track Iraqi crop conditions. Satellite chlorophyll mapping showed irrigated crop area in **April 2022 down 50%** from April 2020. These are monitoring uses, not tied to payments. — [FAO GIEWS](https://openknowledge.fao.org/server/api/core/bitstreams/8f2c69c8-ef69-4aa3-94a8-72aa9c5870ed/content); [Kermap](https://kermap.com/en/satellite-imagery-climate-change-monitoring-drought-iraq/)
- A search summary of Arabic sources claimed the Iraqi Ministry of Agriculture used satellites to track crop growth and soil. The Al Jazeera article that was fetched contained no satellite information, so the claim is **[UNVERIFIED]**. — [Al-Rasheed Media, May 2025](https://www.alrasheedmedia.com/2025/05/08/624018/)

### Inferences
- A clear need exists. Purchase quotas are expressed per dunam and per farming method, Baghdad disputes how much KRI actually grows, and fraud works by pushing non-local or merchant grain through farmer quotas. A Sentinel-2 map of **sown wheat area and expected yield per registered farmer or village** would supply the evidence missing in all three disputes: area claimed vs. sown, KRI production totals, and plausible delivery per dunam. The EU AMS (100% parcel screening, follow-up only on flagged parcels) and India's YES-TECH (satellite blended with crop cuts) are tested templates.
- A TARSIM-style village drought-yield index for rainfed KRI wheat, possibly paid from the existing compensation budget under Law 4/2008, is the nearest workable insurance model. Malawi (ARC) and Madhya Pradesh show the main risks: a mis-calibrated crop model or full reliance on satellite can produce wrong or late payouts.
- Rainfed yield norms (~300 kg/dunam) and KRI's 88.5 kg/dunam quota are far below KRI's claimed output (2.5 Mt from 3.3M dunams, about 760 kg/dunam). That gap is exactly what independent satellite yield estimates could arbitrate.

### Gaps
- No evidence was found of the KRG or Iraqi ministries (Agriculture, or Trade/Grain Board) using satellite imagery to verify wheat areas, procurement claims or drought compensation.
- No recent (2023–2026) documented case of Iranian or Syrian wheat smuggled into silos with quantities was found. Evidence is 2016–2020 era allegations and parliamentary findings.
- No data was found on KRI drought compensation amounts actually paid in 2021–2025, or on any crop insurance product sold in Iraq.
- The date of the 284 bn IQD disbursement is ambiguous across sources.
