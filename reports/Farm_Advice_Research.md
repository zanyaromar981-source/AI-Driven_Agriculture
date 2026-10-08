# Can software really help a rain-fed wheat farmer? (research check, 2026-10-08)

Question from the user: "I have 2 km² of wheat. The app says it is not healthy. What can I do?" Three research agents read the studies.

## 1. What the satellite can and cannot tell (Sentinel-2, 10 m, every ~5 days)
- **It can tell where, when and how much.**
  - It catches sudden events: hail damage matched ground surveys at r = −0.86 ([doi](https://doi.org/10.3390/rs14040951)). An insurance zoning method was 87% right on 91 plots ([doi](https://doi.org/10.3390/agronomy11102078)).
  - It finds zones that are weak every year: 26% of subfields were stable-low over 8 years ([doi](https://doi.org/10.1038/s41598-019-42271-1)).
- **It cannot reliably tell why.**
  - No Sentinel-2 study cleanly separates nitrogen shortage from drought in rain-fed wheat. That needs a thermal camera, and Sentinel-2 has none ([doi](https://doi.org/10.1016/j.fcr.2007.03.023)).
  - Yellow rust: 85% on a regional map ([doi](https://doi.org/10.3390/s18030868)), but only once the damage is visible.
  - Weeds and pests cannot be seen at 10 m.
  - A 2022 review says that telling the causes apart "is still vague in many cases" ([doi](https://doi.org/10.1016/j.rse.2022.113198)).
- → Use it to **send the farmer to scout**: "check these 3 spots, look for X". Don't call it a diagnosis.

## 2. Does advice by phone actually help farmers?
- **On average, a little.**
  - A meta-analysis of 20 studies found yield **+6%** (CI +2 to +9%), income **+6%** and fertilizer adoption **+23%** ([Beach 2025](https://pmc.ncbi.nlm.nih.gov/articles/PMC12167173/)).
  - Science 2019 found +4%.
  - It is worth doing because it is very cheap: **$0.37 per farmer per year** in Odisha.
- **Biggest gains in bad years.** In Odisha's flood areas, harvest rose 9% and severe loss fell 21% ([PxD](https://precisiondev.org/wp-content/uploads/2025/02/Odisha_RCT_02052025.pdf)).
- **What works:** one concrete action at a time, tied to the weather, by voice, in the local language, repeated, with an easy first use.
  - Removing one registration step raised first-call access by 41% (Ethiopia 8028).
  - Adding more detail did not help (Kenya/Rwanda, 128k farmers).
- **What does not work:** generic advice, hard setup, and farmers over-reacting. In Bangladesh, a 4% loss came from farmers dropping phosphate.
- **Nobody has tested a satellite "stress map" for farmers in a trial.** Claims of scouting time saved come from vendors only.
- **LLM chatbots:**
  - No yield proof yet; the Kenya trial ends in 2027.
  - GPT-3.5 got 49% of 160 farm questions wrong ([arXiv](https://arxiv.org/abs/2309.09401)).
  - ChatGPT scored about 70% on a Sorani test.
  - → Use only checked content, and test the Kurdish quality.

## 3. The advice rules, checked (rain-fed wheat, Mediterranean and West Asia)

| Decision | Verdict | Rule with numbers | Safe app wording |
|---|---|---|---|
| Sowing after the first rain (Pelê) | Partly correct | 10 mm sprouts the seed but nothing emerges; ~15 mm gives 50% emergence; ~25 mm gives full emergence. 10–20 mm followed by dry weather = "false break". Best window in Kurdistan: 15 Nov–5 Dec | "20–25 mm or more coming while it is still sowing time: good time to sow. Only 10–20 mm then dry: risk of sprouting and failing." |
| Urea before rain | Partly (the rain must be big enough) | Urea on damp soil that then dries loses up to 22% in a week, >30% in total; it needs **≥12 mm** in one rain. Light showers make losses worse. Put ≥80% on before stem elongation | "Spread urea on dry soil just before a forecast rain of 12 mm or more. Not on wet soil, not before light showers." |
| Less nitrogen in drought | Correct in principle | Under late drought, high-N wheat gave 24% less yield (haying-off); with enough water, +31%. No tested skip rule for Iraq | "Dry season and dry forecast: hold off on top-dressing; add only if good rain comes before stem elongation." (a warning, not an order) |
| Cut a failed crop for forage | Partly | In Syria, grazing barley raised net income, more so at the drier site. Cut at boot stage. Up to 25% of failed crops had risky nitrate levels | "Grain prospects very low: compare forage value with grain; test nitrate; check spray waiting periods." |
| Rust | Correct | Yellow rust needs <18 °C (best 6–12) and ≥3 h of wet leaves; symptoms show 14–28 days later. On susceptible varieties it costs 24–39%; one spray at the first signs works as well as two. Iraq resistant varieties: Al-Wand, Kalar 1, Rezan, Sarahat | "Cool, wet nights: rust risk. Check the leaves; if you see yellow stripes, spray once to protect the flag leaf." |
| Sunn pest | Thresholds correct; calendar spraying wrong | Spray only above **8 nymphs/m²** (Syria and Turkey). First nymphs at **84 degree-days** above 13.3 °C counted from 1 Jan; spray between nymph stages 1 and 4. No aerial spraying | "Nymphs likely now. Count them with a 0.5 × 0.5 m frame and spray only above 8 per m²." |
| Herbicide weather | Depends on the product | Rain-free hours: pinoxaden 0.5 h, 2,4-D ester 1 h, sulfonylureas 4 h, 2,4-D amine 6 h; best at 15–24 °C | "6 dry hours, 15–24 °C, light wind: good spray day. Check your label." |
| Heat and frost at flowering | Correct; nothing to spray | Grain number falls above ~31 °C around flowering; frost damage starts below −2 °C | "Heat or frost alert: check the heads in 7–10 days." |

Sources for this table:
- Hartfield 2025 trials, DPIRD WA, Montana State urea volatilization studies, van Herwaarden et al. 1998, FAO Syria N trials.
- ICARDA: Sharma et al. 2016 on rust; the Sunn pest brochure.
- TUBITAK 2016 on Sunn pest degree-days; Oklahoma State L-468 on rainfastness.
- Full links are in the agent report below.

## Bottom line
1. **Software can't make it rain, but it can stop farmers wasting money.** That means urea before the right rain, spraying only above thresholds or in the right weather, and sowing on a real rain rather than a false break.
2. **The satellite says where to look; the farmer's eyes say why.**
3. Expect small average yield gains (2–6%). The money saved and the help in bad years are the honest selling points.
4. Every spray or fertilizer message must say "check your field and your label". There are no doses from the AI.

## Links (third agent)
[1] https://www.hartfieldsite.org.au/media/2025%20Trial%20Results/HTR25_TOS_depth_and_rainfall_interactions_on_wheat_and_barley_establishment_and_yield.pdf · [2] https://library.dpird.wa.gov.au/fc_researchart/24 · [5] https://sjpas.univsul.edu.iq/article?id=803 · [6] https://landresources.montana.edu/ureavolatilization/learned.html · [7] https://www.farmtrials.com.au/trial/13971 · [8] https://www.hartfieldsite.org.au/media/2022%20EVENTS/Winter_Walk_2022_Yield_prophet_and_nitrogen_applications_REBEKAH_ALLEN.pdf · [9] https://www.publish.csiro.au/cp/A97039 · [10] https://www.fao.org/4/Y4732E/y4732e0b.htm · [11] https://www.cambridge.org/core/journals/experimental-agriculture/article/abs/effects-of-greenstage-grazing-on-rainfed-barley-in-northern-syria-ii-yield-and-economic-returns/74A1D55F4374BD59EFEA29DD54932C6B · [15] https://extensionaus.com.au/FieldCropDiseasesVic/?p=2964 · [16] https://mel.cgiar.org/reporting/download/hash/A8NCG6 · [17] https://www.agriculturejournals.cz/pdfs/cjg/2024/04/04.pdf · [19] https://mel.cgiar.org/reporting/downloadmelspace/hash/BziBmcpd/v/e908b8785cbb674ea30a16d669ebc0aa · [20] https://avesis.erciyes.edu.tr/yayin/3241398f-41cc-41e8-b5fe-ed4a3044b0a8/economic-threshold-for-the-sunn-pest-eurygaster-integriceps-put-hemiptera-scutelleridae-on-wheat-in-southeastern-turkey · [21] https://journals.tubitak.gov.tr/agriculture/vol40/iss4/11 · [23] https://pods.okstate.edu/fact-sheets/L-468-Rainfastness.pdf · [25] https://pmc.ncbi.nlm.nih.gov/articles/PMC6754161
