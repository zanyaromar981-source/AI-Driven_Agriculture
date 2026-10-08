# AI for Pest/Disease Early Warning and Direct-to-Farmer Advisory: Evidence Base (compiled 2026-10-07)

Evidence labels used below: **[RCT]** randomized trial; **[Eval]** independent or structured evaluation, non-randomized; **[Impl]** numbers reported by the implementing organisation (including its own randomized evaluations); **[Vendor/Gov claim]** marketing, press or government statement with no published method; **[Lab]** model accuracy on a held-out dataset, not on farms. Middle East items are marked **FLAG-ME**.

## 1. Pest/disease early warning: desert locust (eLocust3), Ethiopia wheat rust, fall armyworm (FAMEWS), Sunn pest and wheat rust in the Middle East. What was the measured impact?

### Takeaway
The big wins are digital field-data capture combined with expert or model forecasting and government-funded control. FAO credits the 2020–21 locust response with 4.5 Mt of crop losses averted, and the Ethiopian wheat-rust system reached about 275k farmers by SMS. Neither has a counterfactual impact estimate. The locust figures are FAO's own modelled estimates, the Ethiopian wheat-rust evidence is anecdotal or confounded, and FAMEWS and the Middle East Sunn-pest/rust work have no published impact evaluation. Smartphone data capture by non-specialists (eLocust3m) had data-quality problems that were never fixed.

### Cited Findings
**Desert locust 2019–2021 (Horn of Africa and Yemen)**
- [Impl] Jan 2020–Dec 2021: nearly **2.3 million ha** treated by ground and air. FAO estimates **4.5 million tonnes of crop losses averted**, worth **USD 1.77 billion** in cereal and milk, plus **900 million litres of milk** saved and food secured for **41.5 million people**. Donors gave **USD 230.5 million** to the appeal, up to **20 aircraft** flew at once, and **>1.4 million locations were surveyed**. — [FAO Desert locust crisis 2020–21](https://www.fao.org/emergencies/crisis/desertlocust/en/); [FAO OpenKnowledge](https://openknowledge.fao.org/items/275b2f08-58ba-4f2c-8ef0-452f97dc1980)
- [Impl] An earlier version of the same FAO figure, cited in press coverage (around 2021), said **36.6 million people** protected and **USD 1.56 billion** in losses avoided. The headline numbers rose as reporting was updated, so they are model estimates rather than measured outcomes. — [Freethink, citing FAO](https://www.freethink.com/energy/locust-swarms-east-africa)
- [Eval] FAO Office of Evaluation real-time evaluation, three phases, final report April 2022:
  - Surveillance was "broadly successful", but significant data gaps remained in **Ethiopia, Eritrea and Yemen**. Causes named: weak national engagement in Somalia, internet outages in Ethiopia and conflict in Yemen.
  - **eLocust3g** (dedicated GPS device) was slow to roll out but produced high-quality data.
  - The smartphone versions **eLocust3m and eLocust3w "presented significant problems of data quality which have not been overcome."**
  - The evaluation issued 27 priority recommendations.
  - Sources: [PreventionWeb summary](https://recovery.preventionweb.net/publication/real-time-evaluation-faos-response-desert-locust-upsurge-2020-2021); [FAO RTE Phase I](https://openknowledge.fao.org/items/742d7d58-7dc7-455d-9083-306a8828f23f/full); [ALNAP Phase III listing](https://alnap.org/help-library/resources/real-time-evaluation-of-faos-response-to-desert-locust-upsurge-2020-2021-phase-3/)
- [Impl] PlantVillage (Penn State) built eLocust3m for the 2020 swarms: photo-based locust ID with automatic GPS and algorithmic photo checks. A **52-member scout team in northern Kenya** fed real-time data to county officers and aerial spray teams. — [Freethink](https://www.freethink.com/energy/locust-swarms-east-africa); [Penn State](https://news.engr.psu.edu/2020/shen-chaopeng-locust-infestation-app.aspx)

**Ethiopia wheat rust early warning and advisory system (EWAS): Cambridge, UK Met Office, EIAR, ATA, CIMMYT; running since 2015**
- [Impl/peer-reviewed] What the system does: phone-based field surveys, Met Office weather forecasts, spore-dispersal modelling (NAME) and environmental suitability, combined into an automated **7-day forecast** with about a **3-week window** for farmers to buy and apply fungicide. — [Allen-Sader et al. 2019, Environ. Res. Lett.](https://pmc.ncbi.nlm.nih.gov/articles/PMC7680955)
- [Impl/peer-reviewed] Reach and use (same paper):
  - SMS alerts reached about **275,000 smallholders and 10,000 development agents** in 2017–18.
  - About **150,000 more** were reached through regional channels in Arsi, West Arsi and Bale in 2018.
  - The system helped officials **allocate a limited fungicide stock in 2017 and 2018**.
- [Impl/peer-reviewed] What the paper does not show:
  - No yield saved or area protected is quantified, and the evidence of success is "remarkably positive" but anecdotal.
  - Formal validation was only planned for 2019–20.
  - Survey coverage follows surveyors' routes, not the pathogen's spread.
  - Counterfactuals are hard to establish.
  - Source: [Allen-Sader et al. 2019](https://pmc.ncbi.nlm.nih.gov/articles/PMC7680955)
- [Eval] CIMMYT impact assessment: **>1,000 households in 17 districts across Oromia and Amhara**. It found positive effects on fungicide-use behaviour and rust awareness. The authors say they cannot fully separate the EWAS effect from broader rust-control efforts. I could not retrieve the full report (HTTP 403), so I have no effect sizes. — [CGIAR publication page](https://www.cgiar.org/research/publication/wheat-rust-early-warning-and-advisory-system-in-ethiopia-impact-assessment-in-two-major-wheat-growing-regional-states)
- [Journalistic/anecdotal] 2021: field surveys found a new yellow-rust strain under "super optimal" weather. Early warning led to rapid government fungicide deployment, followed by a "bumper" harvest. — [MIT Technology Review, Dec 2024](https://www.technologyreview.com/2024/12/26/1108531/international-surveillance-wheat-crop-rust-infections/)

**Fall armyworm: FAMEWS**
- [Impl] FAMEWS is FAO's free Android app for recording standardized scouting and pheromone-trap data. The data feed FAO's global FAW maps. — [IPPC/FAO](https://www.ippc.int/en/about/core-activities/capacity-development/guides-and-training-materials/contributed-resource-detail/faw-monitoring-and-early-warning-system-famews)
- [Eval/research] A community-based monitoring and forecasting initiative ran in 6 East African countries (Ethiopia, Kenya, Tanzania, Uganda, Rwanda, Burundi) with **650+ community focal persons**. Analysis of the pooled data found that cropping system, crop diversity in rotation and control method all affected trap and larval counts. This is research output; no farmer impact was measured. — [PMC8195398](https://pmc.ncbi.nlm.nih.gov/articles/PMC8195398)

**India: AI pest surveillance and trap counting**
- [Gov claim] National Pest Surveillance System (NPSS), launched 15 Aug 2024:
  - Farmers upload photos and AI image recognition returns an advisory.
  - Covers **61 crops** (advisories for 15 major crops) in **6 languages**.
  - About **30,000 users** at launch; the stated aim is **146 million farmers**.
  - No evaluation found.
  - Source: [Drishti IAS summary (secondary)](https://www.drishtiias.com/daily-updates/daily-news-analysis/national-pest-surveillance-system-npss/print_manually)
- [Vendor claim] Wadhwani AI CottonAce, pink bollworm:
  - Counts pests from phone photos of pheromone traps and advises whether and when to spray.
  - Used by about **15,000 farmers** in Maharashtra, Gujarat and Telangana in Kharif 2020.
  - Reported **+11% yield and +20–25% income "over the previous year"**, with pesticide cost cut "up to 25%". This is a before/after comparison with no control group.
  - Sources: [Krishi Jagran](https://krishijagran.com/agriculture-world/cotton-farmers-in-india-fight-against-bollworm-with-the-help-of-artificial-intelligence-report/?amp=1); [Wadhwani AI](https://www.wadhwaniai.org/?p=2069)

**Middle East: wheat rust and Sunn pest (FLAG-ME: Iraq/Kurdistan, Syria, Iran, Turkey)**
- **FLAG-ME (Iraq)** [Gov/agency report, 2010] BGRI/ICARDA reported a major outbreak of a virulent new yellow (stripe) rust strain across Middle East wheat regions, including **northern Iraq**. Badly hit fields can lose **35–50%** of yield, and the worst cases approach total loss. — [USDA FAS IPAD 2010](https://ipad.fas.usda.gov/highlights/2010/06/Middle%20East)
- **FLAG-ME (Kurdistan Region, Sulaimani)** [News, 4 Apr 2024] Rust spread for about a month through the Raparin administration, where about **180,000 dunams** of wheat are grown.
  - One farmer reported a loss of about **450 million IQD on 40 dunams**, expecting under 40 t instead of over 300 t. As printed, that tonnage per dunam is implausible and is probably a reporting error.
  - Raparin's head of plant protection said the worst-hit crops were those planted before sporadic rains.
  - **No forecast or early warning is mentioned.**
  - Source: [Rudaw](https://rudaw.net/english/kurdistan/04042024)
- **FLAG-ME (Turkey, Iran, Syria)** [Secondary] Sunn pest (*Eurygaster integriceps*):
  - Infests over 15 million ha across West and Central Asia, with major outbreaks about once every 7 years.
  - Grain loss is estimated at **50–90% in wheat** and **20–30% in barley** when unmanaged.
  - Source: [Wikipedia](https://en.wikipedia.org/wiki/Eurygaster_integriceps)
- **FLAG-ME** [Research] Forecasting work exists but nothing has been evaluated:
  - A Sunn pest forecasting network in Iran (Amir-Maafi 2011).
  - Fat-content-based population forecasting in Southeast Anatolia, Turkey.
  - Sources: [UVM Sunn pest research](https://www.uvm.edu/~entlab/sunnpest/Research.html); [AREEO journal (Iran)](https://journals.areeo.ac.ir/article_119130.html)

### Inferences
- The locust and wheat-rust successes depended on government and donor capacity to act: aircraft, pesticide stocks, fungicide allocation. Forecasting without a delivery chain (Raparin 2024) appears to give no benefit.
- "AI" contributes little to the proven pest early-warning cases. The evidence is for digitised scouting plus physics, epidemiology and expert forecasting. No found evaluation attributes outcomes to an ML model.
- The eLocust3m/3w finding is directly relevant to any crowd- or farmer-sourced pest reporting in Kurdistan: plan for data validation from the start.
- Ethiopia's wheat-rust architecture (surveys, weather model, spore dispersal, SMS/IVR to farmers through the national hotline) is the closest transferable template for yellow rust in Iraq/Kurdistan. It is the same crop and disease family, with a similar smallholder-wheat context.

### Gaps
- No counterfactual (RCT or quasi-experimental) estimate of yield or loss avoided for any pest early-warning system. CIMMYT's Ethiopia effect sizes could not be retrieved (HTTP 403).
- No FAMEWS usage totals (records or countries) or impact evaluation found.
- No operational, evaluated Sunn pest or yellow-rust early-warning system found for Iraq, Kurdistan, Syria, Iran, Turkey or Jordan.

## 2. Phone-photo AI diagnosis (PlantVillage Nuru, Plantix, etc.): field vs claimed accuracy, real users, evidence on yields or pesticide use

### Takeaway
Lab accuracy of about 99% routinely collapses on real-world photos, to around 31% in the best-known case. The best field-validated app, Nuru, scored 65% on cassava: better than extension agents (40–58%) and farmers (18–31%), but far below lab claims. Plantix has real scale (30M+ downloads), yet there is no RCT showing yield or pesticide effects for any photo-diagnosis app. The only "impact" figures are vendor before/after numbers or cross-sectional surveys.

### Cited Findings
- [Lab] Mohanty, Hughes and Salathé (2016): a CNN trained on PlantVillage (54,306 images, 14 crops, 26 diseases) scored **99.35%** on held-out test images, but **31.4%** on images from other sources taken under different conditions. Training images were single leaves on uniform backgrounds. — [arXiv 1604.03169](https://arxiv.org/pdf/1604.03169)
- [Field test] Ramcharan et al. (PlantVillage, cassava): a mobile CNN deployed on phones lost about **32% of its F1 score** on real-world images with pronounced symptoms, mainly through lower recall. Outdoor lighting extremes were a named cause. — [arXiv 1805.08692](https://arxiv.org/pdf/1805.08692)
- [Field eval] Mrisho et al. 2020 (Frontiers in Plant Science; IITA, East Africa) tested PlantVillage Nuru on cassava mosaic disease, cassava brown streak disease and green mite damage:
  - Nuru: **65%** accuracy (2020).
  - Extension agents: **40–58%**.
  - Farmers: **18–31%**.
  - Nuru rose to **74–88%** when six leaves per plant were assessed.
  - Two weeks of Nuru use gave extension workers only a slight improvement in their own diagnostic skill.
  - Source: [Frontiers](https://www.frontiersin.org/articles/10.3389/fpls.2020.590889/full); [PMC7775399](https://pmc.ncbi.nlm.nih.gov/articles/PMC7775399/)
- [Impl] Plantix (PEAT, Germany/India): **30+ million downloads** and **>100 million images analysed** as of 3 May 2024. No accuracy or impact data were given in that release. — [ICRISAT pressroom](https://pressroom.icrisat.org/icrisat-and-plantix-celebrate-a-decade-of-collaboration)
- [Impl, via search summary, not verified on page] Plantix had **10 million users between Feb 2022 and Feb 2023, 6.3 million active, 80% in India**. It claims about 90% accuracy and covers 500+ threats on 50 crops. — [FAO Agritech Observatory: Plantix](https://agritechobservatory.review.fao.org/en/plantix-crop-doctor); [GSMA Plantix case study](https://www.gsma.com/solutions-and-impact/connectivity-for-good/mobile-for-development/programme/agritech/detecting-and-managing-crop-pests-and-diseases-with-ai-insights-from-plantix/) (HTTP 403 when fetched)
- [Small field test] Nigeria: Plantix scored **90–100%** on major pest and disease symptoms of maize, okra, cassava and plantain, and **100%** on healthy plants. This is a small, single study. — [AJOL, Ife J. Sci.](https://ajol.info/index.php/ijs/article/view/247309)
- [Pilot eval, documented failure] Vietnam (CGIAR): Plantix **failed to identify or distinguish early signs** of some pests and diseases on young rice, and farmers had trust issues. Farmers still **over-used pesticides** despite knowing label doses, so the app alone did not change spraying behaviour. — [CGSpace](https://cgspace.cgiar.org/server/api/core/bitstreams/94ce0440-677e-4f88-9059-5dcb53d3c8bd/content)
- [Cross-sectional survey, weak design] Andhra Pradesh, 416 farmers, March 2026, not peer-reviewed: AI app (Plantix) users reported **23% lower pesticide use** and **17% higher productivity** than non-users. Adoption tracked phone access and digital skills, so self-selection is likely. — [Zenodo](https://zenodo.org/records/19178160)
- **FLAG-ME (Arabic, North Africa and Gulf)** [Gov/agency claims, no evaluation found]:
  - Egypt's government "Hudhud" Arabic AI assistant (Dec 2021) does photo diagnosis. — [Daily News Egypt](https://www.dailynewsegypt.com/2021/12/07/government-launches-ai-enabled-system-to-enhance-agriculture-process/)
  - ICBA and the University of Barcelona's "Dr. Nabat" app targets Tunisia, Egypt and the UAE (announced Dec 2021). — [ICBA press release](https://iiqp.biosaline.org/sites/default/files/pr_mobile-app-for-agriculture_9-dec-21_final-en.pdf)
  - Plantix was launched in Tunisia. — [Kapitalis](https://kapitalis.com/anbaa-tounes/?p=136733)

### Inferences
- A realistic field-accuracy planning number for a photo-diagnosis model is about 65–85%, with gains from multiple images per plant (Nuru). Vendor "90%+" figures should be treated as best-case.
- Diagnosis does not equal behaviour change: the Vietnam pilot shows pesticide over-use persisting even with the app. Photo-AI is better framed as triage or decision support than as a yield intervention.
- The comparison that matters is AI vs the farmer's realistic alternative (18–31% farmer accuracy, 40–58% extension agent), not AI vs a lab expert.

### Gaps
- No RCT found measuring yield, income or pesticide-use effects of Nuru, Plantix or any photo-diagnosis app.
- Plantix retention and churn data, and any independent field-accuracy audit at scale, were not found.
- Plantix's support for Arabic, Turkish or Kurdish could not be verified. A 33-language list including Arabic and Turkish came from an App Store listing ("Plantix- Plant Leaf Identifier", id6450135619) that appears to be a different, unaffiliated app.
- No Middle East field-accuracy study for wheat or barley disease photo-AI was found.

## 3. AI chatbots and voice/SMS advisory in local languages (Farmer.Chat, Kisan e-Mitra, PxD RCTs, Gooey/Jugalbandi, Ethiopia 8028, Kenya/Nigeria): what do RCTs and evaluations show?

### Takeaway
Pre-LLM digital advice (SMS, IVR voice, hotlines, video) has solid RCT evidence. Adoption of recommended inputs rises about 22–23%, yields rise about 4–6% on average, and the cost per farmer is very low, so benefit-cost ratios of 10:1 or more are plausible. About half of individual studies find no significant yield or income effect. LLM chatbots (Farmer.Chat and others) have **no published RCT on yields or income yet**. Their evidence so far is usage counts, answer-quality audits (about 75% of queries answered) and self-reported surveys.

### Cited Findings
**Meta-analyses (pre-LLM digital advisory)**
- [Meta-analysis of RCTs] Fabregas, Kremer and Schilbach, *Science*, Dec 2019: digital agricultural information raised yields by **4%** and the odds of adopting recommended inputs by **22%**. Benefits likely exceed transmission costs "by an order of magnitude". — [Agrinatura summary](https://agrinatura-eu.eu/realizing-the-potential-of-digital-development-the-case-of-agricultural-advice/)
- [RCTs] Six SMS-extension RCTs in Kenya and Rwanda (run by KALRO, One Acre Fund, and PAD with IPA):
  - Covered **128,000 farmers** and promoted agricultural lime, plus fertilizer in four of the programmes.
  - Pooled result: **1.22× odds** of adopting the recommended practice (95% CI 1.16–1.29).
  - Source: [ATAI: SMS extension and farmer behavior, six RCTs](https://www.atai-research.org/sms-extension-and-farmer-behavior-lessons-from-six-rcts-in-east-africa/)
- [Meta-analysis] Beach, Milliken, Franzen and Lapidus, *Global Food Security* 2025, 20 RCT and quasi-experimental studies:
  - Pooled effects:
    - Fertilizer adoption **+23%** (CI +6% to +40%).
    - Improved seed **+11%** (not significant).
    - Yield **+6%** (CI +2% to +9%).
    - Income **+6%** (CI +2% to +9%).
  - Video showed the strongest effects (7 studies). SMS evidence was limited and mixed.
  - Many individual studies were null:
    - **7 of 13 yield studies** non-significant.
    - **5 of 9 income studies** non-significant.
    - **3 studies** showed negative fertilizer-adoption effects (−2% to −4%).
  - Coverage: 15 studies from Sub-Saharan Africa, 4 from India, 1 from Cambodia, and **none from the Middle East**.
  - Reported costs: Digital Green Ethiopia **$3–6 per adopting farmer**; Ghana audio programme **3:1 benefit-cost**.
  - Source: [PMC12167173](https://pmc.ncbi.nlm.nih.gov/articles/PMC12167173/)

**Individual RCTs: voice/IVR (PxD lineage)**
- [RCT, documented null] Cole and Fernando, *Economic Journal* 2021: Avaaj Otalo IVR push calls plus a question hotline, **1,200 cotton farmers in 40 villages** in Gujarat.
  - Farmers switched information sources and adopted the recommended inputs.
  - **There was "no systematic evidence of gains in yields or profitability".**
  - Willingness to pay was below the per-farmer cost of running the study service, but likely above cost at scale.
  - Conflict: the Beach 2025 meta-analysis cites "$10 returned per $1" for Avaaj Otalo, apparently from an earlier working-paper version.
  - Source: [Cole and Fernando, EJ (ATAI PDF)](https://www.atai-research.org/wp-content/uploads/2020/06/ueaa084.pdf)
- [Impl, randomized evaluation] PxD Ama Krushi, Odisha: two-way IVR (push advisories plus inbound hotline) run with the Odisha Department of Agriculture. The randomized evaluation covered **13,675 rice farmers in 2021–2023**.
  - Rice production: **+100 kg per farmer per season**, worth about **US$39**.
  - Severe crop loss (>50% of the crop): **−10%**.
  - Losses from pests, disease and weather: **−25%**.
  - In excess-rainfall areas: harvest **+9.4%**, and profit **+$30–48 per farmer**, equal to **14–30% of seasonal profit**.
  - Benefit-cost: **$12–19 per $1**, or $13 per $1 at evaluation time.
  - Scale: about **6.9–7 million farmers** on about a **$1M annual budget**, from which PxD projects $38 per $1.
  - Sources: [PxD 2024 annual report, Odisha](https://precisiondev.org/2024-annual-report/customized-digital-advice-can-help-farmers-manage-crop-loss-and-weather-shocks-evidence-from-pxds-work-in-odisha/); [PxD registry](https://registry.precisiondev.org/registry_entry/impact-evaluation-of-digital-extension-platform/)

**Ethiopia 8028 hotline (ATA, government-run IVR and SMS)**
- [Impl] More than **3 million farmers** used the hotline in its first months (Amhara, Oromia, Tigray, SNNPR). It later reported **5.7 million users and about 50 million calls**, and it pushed **wheat stem rust warnings** to registered farmers. — [The New Humanitarian](https://www.thenewhumanitarian.org/node/255002); [Ethiopian Embassy](https://ethiopianembassy.be/?p=2646)
- [RCT, A/B tests on the platform] Gender and narrator findings:
  - Women are **under 25%** of 8028 users (sample of 25,357 users).
  - A **female agronomist narrator** produced the best listening but the **lowest pick-up**. A male agronomist got the highest pick-up but lower listening.
  - Source: [PxD registry: narrator experiment](https://registry.precisiondev.org/registry_entry/using-male-female-and-agronomist-narrators-for-push-call-service/)
- [RCT, A/B tests on the platform] Menu friction: removing a profile-save prompt raised content access by **1 percentage point**, about **5,000 extra messages a month**. — [PxD registry: experiment 104](https://registry.precisiondev.org/registry_entry/experiment-104-do-not-ask-to-add-crop-soil-altitude-in-profie/)

**LLM chatbots**
- [Impl, answer-quality audit] Digital Green Farmer.Chat (built with Gooey.AI), arXiv Oct 2024:
  - Scale: **15,000+ farmers** across Kenya, India, Ethiopia and Nigeria, and **300,000+ queries**. Kenya had 8,805 active users (4,076 women) and 225,500 queries, about 29 queries per user.
  - Languages: Swahili, Amharic, Hausa, Hindi, Odia, Telugu and English.
  - Answer quality:
    - **75% of queries answered.** Of the **25% unanswered**: 66% were content gaps, 23% out of scope and 11% unsupported crops.
    - About **80%** of answers rated highly faithful.
    - Relevance: **67%** high, 15% medium, **18% low**.
    - Context precision: 71%.
  - Engagement and satisfaction:
    - **35% of users generated 80% of queries.**
    - Satisfaction was measured on only **22 survey respondents** (16 satisfied).
  - Source: [arXiv 2409.08916](https://arxiv.org/html/2409.08916v2)
- [Impl/self-commissioned surveys] Farmer.Chat in India by 2026:
  - **About 1 million users** and **3 million+ queries**, with women **about 45%** of users.
  - Cost **Rs 33 per farmer per year**, against **Rs 3,300** for in-person advisory.
  - 60 Decibels survey: about **60%** of active users act on the advice, and **91%** report more confidence.
  - Earlier IDinsight study: **67%** of active users applied advice.
  - Source: [Rural Voice](https://eng.ruralvoice.in/digital-green-ai-farming-assistant-crosses-10-lakh-users)
- [Impl] Digital Green cost estimates per farmer: about **$0.35 with the LLM tool**, **$3.50 with community video** and **$35 with traditional extension**. In FY23, cost per farmer reached was **$5.00** and cost per adoption **$6.50**. — [Agency Fund](https://theagencyfund.substack.com/p/bridging-data-divides-at-scale-through); [Digital Green 2023 Annual Report](https://digitalgreen.org/wp-content/uploads/2024/03/2023-Annual-Report.pdf)
- [Eval in progress] IFPRI and Digital Green are running structured user testing of Farmer.Chat in India and Kenya (Phase II, 2025–2027). Results are unpublished.
  - Early qualitative findings: voice is critical for less-educated farmers, and trust grows when advice matches farmers' experience.
  - Barriers named: connectivity, unavailable inputs, scepticism about AI and digital literacy.
  - Source: [Rural21](https://www.rural21.com/english/news/detail/article/testing-ai-advisory-services-insights-from-farmerchat-in-india-and-kenya.html)
- [Research, documented failure modes]
  - LLM agricultural answers can be wrong on **planting time, seed rate, and fertilizer rate and timing**, or too generic for resource-poor smallholders ("apply fertilizer appropriately" instead of "120 kg urea/ha at 21 and 45 days"). — [Can Tho Univ. J. Sci.](https://ctujs.ctu.edu.vn/index.php/ctujs/article/download/2240/869/12493) (via search summary)
  - Fine-tuning on expert-curated Bihar crop data substantially improved fact recall and F1. — [arXiv 2603.03294](https://arxiv.org/pdf/2603.03294v2)
- [Gov claim] Kisan e-Mitra (India) is a **PM-Kisan scheme grievance and FAQ bot**, not an agronomy advisor. It runs on the Bhashini language stack, in **11 languages** by voice or text.
  - Claim: **>3 million grievances resolved for >290,000 farmers** since Feb 2024. That is about 10 per farmer, an odd ratio, so treat it with caution.
  - Source: [Apolitical case study](https://apolitical.co/en/navigator/case-studies/kisan-e-mitra-indias-voice-enabled-ai-support-for-farmers); [IndiaAI](https://indiaai.gov.in/article/exploring-pradhan-mantri-kisan-ai-chatbot)
- [Vendor claim] Gooey.AI Farmer.AI in Malawi and Kenya: **1,800+ farmers** and 17,000 messages. — [Gooey.AI](https://gooey.ai/farmer-ai)
- Jugalbandi (Microsoft and IIT Madras) is a WhatsApp bot for government-scheme information. I found **no agricultural impact evaluation**. — [The Statesman](https://www.thestatesman.com/technology/microsoft-iit-madras-ai-chatbot-helps-villagers-access-govt-services-through-phones-1503184574.html)

### Inferences
- Implied delivery costs:
  - Ama Krushi: about **$0.15 per farmer per year** ($1M ÷ 6.9M farmers).
  - Farmer.Chat India: Rs 33, about $0.40, per farmer per year.
  - At these costs, even a 1–2% yield gain clears the cost bar, which is why the meta-analyses find favourable benefit-cost even though many individual studies are null.
- The proven channel is **voice/IVR with push plus pull, run with government**: Ama Krushi, 8028, and the Kenyan SMS programmes with KALRO. LLM chatbots are newer and unproven on outcomes. The honest framing is "LLM front-end on a proven extension model."
- A 25% unanswered rate and 18% low-relevance answers (Farmer.Chat) are the realistic baseline for an LLM advisory on curated content. A new-language deployment such as Kurdish would likely do worse at first.

### Gaps
- No RCT with yield or income outcomes for any LLM farmer chatbot (Farmer.Chat, Kisan e-Mitra, Jugalbandi, Gooey) found as of Oct 2026.
- No Nigeria-specific advisory RCT retrieved. Nigeria appears only as a Farmer.Chat deployment and a Plantix accuracy study.
- The full text of Ama Krushi's randomized evaluation (authors, peer review status) was not retrieved. Its figures are PxD-reported.

## 4. Weather/forecast-based advice for planting dates and inputs (India monsoon-onset AI forecasts 2022–2025, Kenya, Ethiopia): measured decision changes and gains

### Takeaway
This is the strongest new AI-in-agriculture evidence. A Telangana cluster-RCT (2022) showed that an accurate long-range monsoon-onset forecast changed planting area, crop choice and inputs in the predicted directions, and raised food consumption by 7%. In 2025 India's Ministry of Agriculture sent AI-blended (NeuralGCM + ECMWF AIFS) onset forecasts to 38.8 million farmers. A government survey found 31–52% adjusted planting decisions. Welfare gains at scale are projected, not measured.

### Cited Findings
- [RCT] Burlig, Jina, Kelley, Lane and Sahai, May 2025 (AEARCTR-0008846), Telangana, 2022 season. Setup:
  - **250 villages** were randomized to control, forecast or index insurance. The forecast arm was 497 households in 100 villages.
  - The forecast came from the Potsdam Institute, issued about **40 days before onset**, and was disseminated through ICRISAT as a trusted messenger.
  - Forecast accuracy: within one week in each of the prior 10 years, and about **73%** overall in validation.
  - Source: [Burlig et al. 2025 (ATAI PDF)](https://www.atai-research.org/wp-content/uploads/2025/08/20250528_BJKLS_forecasts.pdf)
- [RCT] Results (same paper):
  - Beliefs: **26% closer** to the forecast date (p=0.031).
  - Farmers whose prior was early, and who were therefore told the season would be worse, cut **land cultivated by 22%** (p=0.003) and had **25% lower output** (p=0.039). Their farm profit fell **$400, or 40%** (p=0.089), worse than the input changes imply.
  - Farmers whose prior was late, and who were therefore told the season would be better:
    - **+21% land** (p=0.061).
    - **+31% input spending** (p=0.017).
    - **+33% likelihood of planting cash crops** (p=0.005).
  - Welfare, pooled across all farmers: **+7% per-capita food consumption** (p=0.040) and a **+0.06 SD** welfare index (p=0.048). That is between emergency loans (0.02 SD) and irrigation access (0.11 SD).
  - Caveat: heavy **July floods**, outside the forecast's scope, broke the link between inputs and profits. Only **46%** of farmers were unaffected by floods.
  - The forecast had behavioural effects comparable to index insurance at far lower cost.
- [Gov/Impl] Kharif 2025 scale-up (India MoA&FW with the Development Innovation Lab-India):
  - Model: NeuralGCM + ECMWF AIFS + 125 years of IMD rainfall data.
  - Forecasts were sent by **SMS through the M-Kisan portal to 38,845,214 farmers in 13 states**, in Hindi, Odia, Marathi, Bangla and Punjabi.
  - A government survey found **31–52% of farmers adjusted planting decisions**, mainly land preparation and sowing timing, as well as crop and input choice.
  - Sources: [Global Agriculture](https://www.global-agriculture.com/india-region/ai-monsoon-alerts-influence-sowing-decisions-of-up-to-52-farmers-government-survey-finds/); [Free Press Journal](https://www.freepressjournal.in/tech/govt-deploys-ai-driven-monsoon-forecasts-pest-detection-and-chatbot-tools-to-boost-crop-productivity-and-farmer-welfare)
- [Impl] About **1 million more farmers in Odisha** were reached by voice message. Michael Kremer claims **$100+ return per $1** of government spending, which is a projection, not a measured estimate. — [UChicago Human-Centered Weather Forecasts](https://humancenteredforecasts.climate.uchicago.edu/forecasting-the-onset-of-the-indian-monsoon/)
- [Preprint, forecast skill] Aitken, Kremer et al., arXiv March 2026. The blended AI model, compared with static climatology and pooled over lead times up to 4 weeks, achieved:
  - **5–10%** better Brier score.
  - **20–25%** better RPS.
  - **+3–5 percentage points** AUC.
  - At a 1-week lead, about **15%** better Brier score than the "evolving-expectations" baseline (25% better than static climatology).
  - In 2025 it skilfully predicted the anomalous **early-summer dry period** (false onset).
  - Design lesson: benchmark against what farmers already know, or the forecast's value is exaggerated. Probabilistic, not deterministic, forecasts let heterogeneous farmers tailor decisions.
  - Source: [arXiv 2603.07893](https://arxiv.org/pdf/2603.07893v2)
- [Impl/RCT] Ama Krushi's weather-sensitive gains came in **excess-rainfall areas**: +9.4% harvest and −21% severe crop loss in inadequate-rainfall areas in year 2. — [PxD](https://precisiondev.org/2024-annual-report/customized-digital-advice-can-help-farmers-manage-crop-loss-and-weather-shocks-evidence-from-pxds-work-in-odisha/)
- Ethiopia: weather forecasts (UK Met Office) power the wheat-rust EWAS (see Q1). — [Allen-Sader et al. 2019](https://pmc.ncbi.nlm.nih.gov/articles/PMC7680955)

### Inferences
- Forecast value is two-sided. Farmers told "worse than you think" rationally shrink their planting, so output falls for them while welfare rises. A Kurdistan pilot should therefore measure decisions and welfare, not just yield.
- The pieces that made India work transfer directly to rain-fed wheat and barley in Kurdistan, where the onset of autumn rains drives sowing date:
  - An open AI weather model (NeuralGCM or AIFS) blended with long local rain-gauge history.
  - Probabilistic messages.
  - A trusted messenger.
  - A government SMS channel.

### Gaps
- No measured yield or income effect at the 38-million scale; only survey-reported decision change.
- No Kenya or Ethiopia RCT on AI weather-based planting advice retrieved. The cited Ghana and Kenya short-range forecast studies (Fosu et al. 2018; Rudder and Viviano 2024) were not read.
- No evaluated rainy-season-onset forecast service for Iraq, Syria, Iran or Turkey found.

## 5. Language: agricultural AI in Kurdish (Sorani/Kurmanji) or Arabic for Iraq; state of Kurdish speech and text AI for a farmer chatbot

### Takeaway
**No agricultural AI service in Kurdish was found anywhere.** Iraq's only identified digital extension tool is FAO's Arabic-language Al-Rafidain app (June 2023), with no published usage or evaluation. Kurdish language technology is usable but thin:
- Google Translate has supported Sorani since May 2022.
- ChatGPT scores about 70% on Sorani multiple-choice questions.
- Kurmanji speech recognition reaches 10.5% word error rate after fine-tuning on only about 68 hours of data.
- Badini and Hawrami have almost nothing.

### Cited Findings
- **FLAG-ME (Iraq)** [Agency launch] FAO and Iraq's Ministry of Agriculture launched the **"Al-Rafidain for Agricultural Extension"** app in **June 2023** for farmers across Iraq. It covers crops and livestock, with disease reporting, market prices and contact with experts. No user numbers or evaluation were found. — [UN Iraq press release](https://iraq.un.org/en/node/237050)
- **FLAG-ME (Arabic, regional)** Egypt "Hudhud" (2021), ICBA "Dr. Nabat" (2021–22) and Plantix Tunisia are all unevaluated (see Q2). — [Daily News Egypt](https://www.dailynewsegypt.com/2021/12/07/government-launches-ai-enabled-system-to-enhance-agriculture-process/); [ICBA](https://iiqp.biosaline.org/sites/default/files/pr_mobile-app-for-agriculture_9-dec-21_final-en.pdf)
- **FLAG-ME (Kurdish text)** Google Translate added **Sorani (Central Kurdish) in May 2022**, built largely from about 1.5 million data items gathered by volunteer Bokan Hassan over 7 years. — [Rudaw](https://www.rudaw.net/english/kurdistan/110520224); [Al-Monitor](https://al-monitor.com/originals/2022/05/google-translate-adds-sorani-kurdish)
- **FLAG-ME (Kurdish LLM)** [Peer-reviewed, Univ. of Human Development, Sulaimani, 2025] ChatGPT on 50 Sorani multiple-choice questions, 4 accounts × 10 cycles: about **70% accuracy**, with **significant variation across accounts**. It was weak on specialised and culture-specific content. — [UHD Journal of Science and Technology](https://journals.uhd.edu.iq/index.php/uhdjst/article/view/1616)
- **FLAG-ME** [Dataset] KurdishMCQ, the first large Sorani multiple-choice benchmark, has **17,233 questions** from school materials. — [Mendeley Data](https://data.mendeley.com/datasets/z8z28jhszp/1)
- **FLAG-ME (Kurmanji speech)** [arXiv, Oct 2024] Whisper v3 fine-tuned on about **68 h** of validated Common Voice 18.0 Kurmanji reached **WER 10.5% / CER 5.7%**. Prior wav2vec2 XLSR-53 on the same data scored about **16% WER**. The authors note Sorani has more written resources than Kurmanji. — [arXiv 2410.16330](https://arxiv.org/html/2410.16330v1)
- **FLAG-ME (Badini speech)** [arXiv, Aug 2025] On about **15 h** of children's-book narration by 6 speakers, wav2vec2 reached **82.67% accuracy** against **53.17% for Whisper-small**. Speech-to-text exists for Sorani but **not for Badini (about 2M speakers) or Hawrami**. — [arXiv 2508.09957](https://arxiv.org/abs/2508.09957)
- **FLAG-ME (Sorani speech)** [Research] Existing Sorani speech-recognition corpora are narrow:
  - BD-4SK-ASR covers grade 1–3 school vocabulary only. — [arXiv 1911.13087](https://arxiv.org/pdf/1911.13087v1)
  - The KSS dataset is **18,799 recordings of 500 isolated words** (>96% accuracy), not conversational speech. — [UHD Journal](https://journals.uhd.edu.iq/index.php/uhdjst/article/download/968/901?inline=1)
- **FLAG-ME** [Via search summary; page returned 403] Mozilla reports the first Sorani speech-recognition model was trained on about **122 h** of Common Voice Sorani (QuartzNet), and stresses the scarcity of Kurdish voice data. — [Mozilla Foundation](https://www.mozillafoundation.org/nl/blog/keeping-kurdish-alive/)

### Inferences
- A Sorani text chatbot is feasible today: Google Translate plus a frontier LLM, or a direct LLM at about 70% general MCQ accuracy. Agronomic accuracy in Sorani is unmeasured and likely lower, because LLMs are weak on specialised Kurdish content.
- Voice matters most for low-literacy farmers (IFPRI/Digital Green; Ethiopia 8028). Kurdish speech recognition is the bottleneck, especially for rural dialects such as Badini and Hawrami and for agricultural vocabulary. Plan for push voice messages (recorded or TTS) plus menu-based IVR before free-form speech.
- No Kurdish-language competitor exists in agri-advisory, but there is no Kurdish-language evidence base either. Any claims would rest on non-Kurdish analogues.

### Gaps
- Off-the-shelf Whisper's support level and baseline WER for Sorani were not verified.
- No published evaluation of Kurdish TTS quality for rural listeners. A KurFemTTS corpus exists on Mendeley but was not read.
- No usage numbers for Al-Rafidain or any Iraqi Ministry of Agriculture / KRG agriculture digital channel.
- No evidence found on Jordan, Syria or Turkey Arabic/Turkish agri-chatbots with evaluations.

## 6. Why many agri-advisory apps fail (trust, literacy, relevance), and what the successful ones did differently

### Takeaway
Most digital agri-services fail on relevance, trust and business model, not on phone access:
- Only a small minority of registered farmers stay active.
- Wrong or untimely advice drives drop-off.
- Women and low-literacy farmers are under-served.
- Venture-funded models like Wefarm collapsed.

The successes (8028, Ama Krushi, Kenya SMS lime, India monsoon forecasts) shared five traits: government ownership or distribution, voice/SMS on basic phones, trusted messengers, localized and timely content, and continuous A/B testing at near-zero marginal cost.

### Cited Findings
- [Sector data] GSMA's Digital Agriculture Maps (2020) tracked **700+** digital agriculture services in low- and middle-income countries. — [GSMA](https://www.gsma.com/solutions-and-impact/connectivity-for-good/mobile-for-development/uncategorized/introducing-digital-agriculture-maps-a-2020-state-of-the-sector-report/)
- [Sector analysis, via search summary; attribution between the two sources not verified]
  - About **13%** of farmers register and only about **5% actively use** digital services. Roughly **10% of about 500 million** low- and middle-income-country smallholders actively use at least one service.
  - Disengagement comes more from **inaccurate or poorly timed advice** and limited interactivity than from lack of phones.
  - Peer networks, WhatsApp and phone calls often beat dedicated platforms on relevance and trust.
  - Sources: [ICTworks "Error 404: Farmer Not Found" (2025)](https://www.ictworks.org/wp-content/uploads/2025/03/Error-404-farmer-not-found.pdf); [WUR "Not so digital platforms"](https://research.wur.nl/en/publications/not-so-digital-platforms-non-use-inappropriate-technologies-and-o/)
- [Documented failure] Wefarm, a peer-to-peer SMS farmer Q&A service in Kenya, Uganda and Peru:
  - Peaked at **1.1 million users** after crossing 1 million in Kenya.
  - Raised **>$10M** in venture capital.
  - **Went out of business in 2022.**
  - Sources: [Wikipedia](https://www.wikipedia.org/wiki/WeFarm); [Business Daily](https://www.businessdailyafrica.com/bd/corporate/companies/farmer-sms-platform-wefarm-hits-1m-users-2222636)
- [Documented failure] FAO's smartphone locust apps (eLocust3m/3w) had **unresolved data-quality problems**. Data gaps came from conflict (Yemen), internet shutdowns (Ethiopia) and weak national buy-in (Somalia). — [PreventionWeb / FAO RTE](https://recovery.preventionweb.net/publication/real-time-evaluation-faos-response-desert-locust-upsurge-2020-2021)
- [Documented failure] Plantix in Vietnam missed early symptoms on young rice and lost farmer trust, and pesticide over-use persisted. — [CGSpace](https://cgspace.cgiar.org/server/api/core/bitstreams/94ce0440-677e-4f88-9059-5dcb53d3c8bd/content)
- [Documented failure] Lab-to-field accuracy collapse: 99.35% → 31.4% (Mohanty 2016), and −32% F1 for the PlantVillage mobile model. — [arXiv 1604.03169](https://arxiv.org/pdf/1604.03169); [arXiv 1805.08692](https://arxiv.org/pdf/1805.08692)
- [RCT null] Avaaj Otalo changed input adoption but **not yields or profits**, and farmers' willingness to pay was below the cost of running the pilot. — [Cole and Fernando 2021](https://www.atai-research.org/wp-content/uploads/2020/06/ueaa084.pdf)
- [Meta-analysis] About half of digital-advisory studies show **no significant yield (7/13) or income (5/9)** effect. — [Beach et al. 2025](https://pmc.ncbi.nlm.nih.gov/articles/PMC12167173/)
- [Impl audit] Farmer.Chat could not answer **25% of queries**, mostly because of content gaps (66%). Usage was concentrated: **35% of users made 80% of queries**. — [arXiv 2409.08916](https://arxiv.org/html/2409.08916v2)
- [Eval] Gender gaps and trade-offs in voice design:
  - Women are **<25%** of Ethiopia 8028 users.
  - Narrator choice trades off pick-up against listening.
  - Female Farmer.Chat users benefit from **peer-supported onboarding**.
  - Sources: [PxD registry](https://registry.precisiondev.org/registry_entry/using-male-female-and-agronomist-narrators-for-push-call-service/); [Rural21](https://www.rural21.com/english/news/detail/article/testing-ai-advisory-services-insights-from-farmerchat-in-india-and-kenya.html)
- [Success factor: trusted messenger] The Telangana forecast RCT deliberately used **ICRISAT** so farmers would find the forecast credible. — [Burlig et al. 2025](https://www.atai-research.org/wp-content/uploads/2025/08/20250528_BJKLS_forecasts.pdf)
- [Success factor: government channel and scale] Examples:
  - Ama Krushi runs with Odisha's Department of Agriculture, reaching about 7M farmers on about $1M a year. — [PxD](https://precisiondev.org/2024-annual-report/customized-digital-advice-can-help-farmers-manage-crop-loss-and-weather-shocks-evidence-from-pxds-work-in-odisha/)
  - 8028 is run by ATA with Ethio Telecom. — [The New Humanitarian](https://www.thenewhumanitarian.org/node/255002)
  - The 2025 monsoon forecasts went out through the government's M-Kisan SMS. — [Global Agriculture](https://www.global-agriculture.com/india-region/ai-monsoon-alerts-influence-sowing-decisions-of-up-to-52-farmers-government-survey-finds/)
- [Success factor: local relevance and voice] Farmers engaged most when advice was "locally relevant and specific", and voice was critical for less-educated users. — [Rural21](https://www.rural21.com/english/news/detail/article/testing-ai-advisory-services-insights-from-farmerchat-in-india-and-kenya.html)
- [Success factor: probabilistic, tailored forecasts] Probabilistic forecasts benchmarked against farmers' existing beliefs. — [Aitken et al. 2026](https://arxiv.org/pdf/2603.07893v2)
- [Success factor: continuous experimentation] PxD runs routine A/B tests on the 8028 menus and narrators. — [PxD registry: experiment 104](https://registry.precisiondev.org/registry_entry/experiment-104-do-not-ask-to-add-crop-soil-altitude-in-profie/)

### Inferences
- What predicts failure: app-only (smartphone) delivery, a venture model that depends on farmer payment, generic content, no government or extension partner, and unvalidated crowd data.
- What predicts success: basic-phone voice/SMS, a government or extension partner, push alerts timed to crop stage and weather, narrow and well-curated content, and continuous A/B testing.
- For a Kurdistan hackathon project, the evidence-backed design is:
  - A narrow use case, such as yellow rust/Sunn pest alerts or rain-onset sowing advice for wheat and barley.
  - A government channel (KRG Ministry of Agriculture extension).
  - Sorani/Kurmanji push voice or SMS.
  - Photo-AI used as triage with human escalation, not as the headline claim.

### Gaps
- The exact GSMA registered vs active user figures could not be confirmed from the primary report (HTTP 403). The 13% and 5% figures are from search summaries.
- No systematic failure-rate statistic (share of services shut down) was found.
- No evidence on agri-advisory adoption or trust specific to Iraqi or Kurdish farmers.
