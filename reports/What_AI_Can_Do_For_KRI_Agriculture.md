# What AI can really do for Kurdistan's agriculture (analysis of 96,941 papers, 2026-10-08)

Five analysts read the research library (`research_library/library.sqlite`, 96,941 papers 2000–2026 on AI + agriculture) by topic, against our rules: software only, free data that covers the Kurdistan Region (KRI), helpful for the public, honest evidence, no long-range forecasts (now + 10 days is fine).

## The big picture
- The field is exploding: 52% of all papers are from 2024–2026 (20,511 in 2025 alone). 92% of LLM/chatbot papers are from 2024 or later.
- Research is about the farm, not the public: disease/pest detection 27%, weather 20%, robots/hardware 19%, satellite 15%. Public-interest topics 5%. Middle East 2.4%. KRI: about 15 real papers. Kurdish language: 0.
- **Proof of real-world use is rare: only 70 papers (0.07%) use causal methods** (trials, impact evaluations), and almost none test an AI tool's effect on people.
- **Lab accuracy is not field accuracy.** 1,244 papers use PlantVillage (controlled leaf photos; the famous 99.35% result). On real field photos the best 2025 dataset tops out at 72%.
- Why tools fail (646 abstracts): cost 58%, connectivity 51%, institutions 49%, skills 45%, poor transfer to new places 36%, trust 16%, language 9%. A 2025 survey of 148 Iraqi extension workers found negative attitudes to AI: weak data, few specialists, low trust.
- What has worked: cheap information delivery (digital advice: +4% yield, +22% adoption of advice, about 10× return on cost), and open satellite maps with a public owner (Sen2-Agri in 3 countries; Togo's cropland map delivered in 10 days and used for COVID aid).

## What to INCLUDE (ranked by evidence × public benefit × 48 h feasibility)

| # | Feature | Who it helps | Evidence | 48 h? |
|---|---|---|---|---|
| 1 | **Crop condition now vs the field's own 25-year normal** (per field and district, from NASA/Sentinel) | farmers, ministry, journalists | **Strong.** FAO's ASIS runs on this since 1984 and backs insurance and social protection; Sen2-Agri cropland >90%. Limit: shows *that* a field is behind, not *why* | ✅ built |
| 2 | **Reservoir watch: Dukan, Darbandikhan (+ Duhok) area, in Sorani** | everyone who drinks or irrigates | **Medium-strong.** Simple radar+optical threshold 89% accurate | ✅ built |
| 3 | **10-day farm alerts from the free forecast**: frost, heat, heavy rain, spray/harvest windows, rust weather, sunn-pest window, "sow on ≥20 mm", "urea before ≥12 mm" | farmers | **Medium.** Free global forecast is the backbone; ML adds little. AI forecasts reached 38M Indian farmers in 2025; weather-linked advice gave the biggest gains in shock years. Italian public disease-risk system 88% | ✅ |
| 4 | **Sorani Q&A bot on Telegram** that answers only from a checked KRI knowledge base, with "ask an officer" for the rest | farmers (~50,000 wheat farmers, ~215 extension staff) | **Medium.** Phone advice +4% yield; ChatGPT preferred over agents 78% but failed on rates and dates; retrieval from documents +5 to +40 points accuracy | ✅ |
| 5 | **Photo triage ("plant doctor")**: Claude + 3–6 photos + short KRI disease notes → top-3 causes, confidence, non-chemical steps, referral | farmers, plant-protection office | **Medium.** On real farmer photos: GranoScan 77–95%, Nuru 65% (beats extension staff 40–58%); GPT-4o 46% alone → 89% with disease descriptions and multiple photos | ✅ (no doses) |
| 6 | **Outbreak map from geotagged farmer reports and questions** | ministry plant protection, vets, farmers | **Medium-weak.** Plantix: 78,000 geotagged images → pest maps; India's KisanQRS used 34M helpline logs | ✅ (web map exists) |
| 7 | **Farmland lost to cities, 1991 → today, per district** | public, planners, journalists | **Medium-strong** for the measurement: KRI Landsat study kappa 0.93–0.97; rangeland −11% 1991–2021 | ✅ (25-year MODIS on disk) |
| 8 | **Damage maps after flood, hail or fire for compensation claims** (before/after proof) | farmers, ministry | **Medium-weak.** Flood 97% (Turkey), hail zones 87% of plots, frost 90%; few operational programs | ✅ as a demo on a past event |
| 9 | **Sown area this season (sown vs fallow) per district** | ministry, months before statistics | **Strong for cropland (94%)**; weak for wheat vs barley without local labels | ✅ sown/fallow only |
| 10 | **7-day irrigation advice (FAO-56 water need × crop stage − forecast rain)** for Garmiyan and vegetable areas | irrigated farmers | **Strong on method, weak on proven savings** (IRRISAT Italy 25–30% claimed) | ✅ |
| 11 | **Price bulletin from WFP data** (this month vs last, 3 cities; no forecast) | shoppers, farmers | **Medium** for information; India: 33% acted and earned more | ✅ |
| 12 | **Livestock heat-stress alerts (THI) + pasture greenness vs normal** | herders | **Medium-weak**; thresholds vary by breed | ✅ index only |
| 13 | Groundwater trend from GRACE + map of summer-irrigated fields | water managers | **Medium**; GRACE sees ~300 km blocks; can't predict a farmer's well | ✅ trend + map |
| 14 | Lake water quality (algae, mud) relative to usual | utilities, fish farms, citizens | **Medium-weak** (relative only, no mg/L) | ✅ add-on |

## What to AVOID (and why)
- **Long-range forecasts** (season, climate 2030–2050, locust 35–79 days ahead, drought-index months ahead): out by rule and unverifiable.
- **Training our own weather/rain forecaster**: an Iraqi LSTM missed by 2.3–2.5 °C; free global forecasts are better.
- **Soil fertility / salinity / organic-carbon maps from satellite**: need 100–900 lab samples; KRI results (R² 0.92 from 96 samples) are over-optimistic; salinity is a southern-Iraq problem.
- **Training a disease model on PlantVillage and quoting 99%**: collapses in the field (detection scores 0.60 → 0.15 on new data).
- **Pesticide doses or fertilizer rates from an LLM**: exactly where ChatGPT failed (Nigeria); a chatbot spray plan destroyed a crop in China (2026).
- **"Is this food safe?" from a phone photo**: aflatoxin and residues need lab sensors; a false "safe" harms people.
- **Price or food-crisis forecasting**: models still miss 10–15%; prices here follow bans and smuggling.
- **Credit scoring and index insurance**: need banks/insurers and private data; payouts often miss real losses.
- **Field-level yield in tonnes**: R² ~0.45 per field; needs yield statistics KRI lacks.
- **Wheat-purchase fraud flags**: real risk of false accusations.
- **Anything with sensors or hardware** (animal sensors, hive microphones, fuel meters, drones).

## Addendum: "Is this a drought year?" is monitoring, not forecasting
- Method: compare this season's rain (SPI), greenness (VCI / NDVI z-score) and soil moisture with the 2001–2025 normal for the same place and date; label the season normal / dry / drought by standard thresholds. ~570 papers; FAO ASIS does this operationally.
- Reliability grows through the season: Chile, free data only, R² 0.95 one month before season end, 0.83 two months before, 0.37 six months out (2018, 98 cit). Iranian Kurdistan rain-fed wheat: R² 0.73–0.87 vs yield (2025). Australia: 30 satellite factors reproduced the ground drought index, checked on wheat yields 2001–2017 (2019, 215 cit).
- Limits to say plainly: greenness lags rain by weeks; irrigated fields look green in a drought (split rain-fed from irrigated); reliable from mid-season, not at sowing; says nothing about next season. Validate on the known bad years (2008, 2021, 2025) — our replays already do this.

## Addendum 2: "5 AIs + one AI doctor" — the literature supports it, with one condition
The doctor must reason over **structured evidence** (numbers and labels from the other AIs), never free text or raw guesses.
- Multi-agent beats one big model: a geospatial copilot with specialist agents + an orchestrator scored 17% higher than single-agent systems (2025, 30 cit). The exact pattern exists: soil + weather + disease-vision agents + a supervisor chatbot (2026, 16 cit), but the combined accuracy was never measured.
- An LLM judge over disagreeing vision models helped exactly where they disagreed: +7.6 points on the 42% of conflicting cases (2026). One model over-flagged "critical" by 3.5–14 points, so calibration matters.
- Reference text raises accuracy a lot: GPT-4o alone 45.9% → 88.9% with disease descriptions (ChatLeafDisease 2025, 19 cit). RAG +5 points, fine-tuning +6 (2024, 175 cit).
- LLMs are good on general advice, bad on local numbers (Nigeria: preferred 78% of the time, yet worse on planting time, seed rate, fertilizer rate), which matches our no-doses rule. GPT-4 passed agronomist exams at 93% (2023, 31 cit).
- A 2026 review: the defensible design is "multilingual, multimodal, human-supervised", grounded in curated regional knowledge, shows uncertainty and provenance, and escalates risky cases.

Suggested shape (48 h):
1. Photo triage AI → top-3 causes, confidence, photo-quality flag.
2. Weather AI (10-day) → infection-risk level per disease rule, heat/frost flag, spray-wind warning.
3. Satellite field AI → greenness anomaly vs the field's own history and vs neighbours.
4. Drought AI → season label (normal / dry / drought) from rain + greenness + soil moisture, tagged "reliable from mid-season".
5. Outbreak-map AI → similar reports nearby in the last 14 days.
6. **The doctor** (Claude, Sorani) gets all five as one JSON plus a short rulebook, and says: most likely cause, how sure, what to do now (non-chemical, timing, "call the officer"), what it cannot tell. Shows which input drove each conclusion. Refuses doses. Escalates when inputs conflict or confidence is low.

Rules: (a) each AI passes numbers and labels only; (b) log every case to measure the combined accuracy, which no paper has yet reported; (c) demo one case where the models disagree and the doctor says "unsure, see an officer".

## Design rules the literature agrees on (for the advisor)
1. Answer only from checked local sources and show the source (retrieval beats fine-tuning: +5 to +40 points).
2. Never let the LLM make up numbers: doses, rates and dates come from a fixed table or "ask an officer".
3. Human in the loop with a feedback cycle (Farmer.Chat improved with 25,000 expert-reviewed answers).
4. Reach people through channels they already trust: Telegram, voice, short Sorani messages, alongside extension staff. Chat is "chutney, not rice and dal".
5. Earn trust by being accurate and saying "I don't know"; test in Sorani with local agronomists before launch.

## Bottom line
1. The strongest, cheapest things are half built already: field and district condition vs its own normal, reservoir area, farmland-loss history, all measured, all free.
2. Add the farmer loop: 10-day alerts → Sorani Q&A → photo triage → outbreak map. It uses Claude's eyes, ears and language, needs no hardware, and the design rules above keep it safe.
3. Be honest on stage: mapping methods are proven; public impact of AI farm tools is unproven everywhere (0.07% of papers test it). Show a validation sample and name a public owner (ministry or KRSO).

Source lists per topic are in the five analyst reports (session transcript) and in the library itself: `sqlite3 research_library/library.sqlite`.
