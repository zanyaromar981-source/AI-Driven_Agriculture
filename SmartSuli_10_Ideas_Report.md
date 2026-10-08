# SmartSuli AI Challenge: 10 researched ideas

Researched 2026-09-11. Six research agents ran plus Claude's own analysis. The session's web-search limit (200) ran out mid-research, so a few facts are marked UNVERIFIED. Full sources are in `evidence/`.

## Bottom line
1. **Pick: Dafter**, a payday-aware Kurdish debt notebook for neighborhood shops. It has the best mix of a real pain, a live demo where a judge takes part, a clear business, and your own POS background.
2. **Backup: Dosya**, a patient-held health folder with a Kurdish lab-report explainer. It's the safest AI demo, and it answers SmartSuli's OWN 2024 health challenge.
3. **Apply before Sept 28** at foundation.krd/smartsuli (17 days left). Build days are Oct 8–9, pitches Oct 10.

## What matters about the judges
- SmartSuli is Deputy PM Qubad Talabani's smart-city program, run by The Foundation ("for private sector, rural area and economic development"). The partners are incubators in Basra (Tafa3ul), Mosul (QAF Lab) and Baghdad (Makers).
- So end the pitch with "how SmartSuli could pilot this in Slemani next month", plus one line on how it works across Iraq.
- The prizes at comparable Iraqi hackathons are small ($350–$6k). The real prize is incubation, so show a business path.
- Already seen or already built (avoid): a general Kurdish chatbot/TTS (KI, MetKurd exist), clinic booking (17 apps), Instagram DM bots (RABT Labs, Baaga), farming AI, Slemani traffic.

## Kurdish AI reality check (Sept 2026)
- Works: Sorani text via LLMs (with native review), Sorani speech-to-text when rehearsed (ElevenLabs Scribe, MetKurd), printed OCR, Telegram bots.
- Doesn't work: Sorani voice OUTPUT (no major vendor), handwriting OCR, Badini, a live Instagram API demo.
- Rule: Kurdish in, Kurdish TEXT out, printed documents only.

---

## The 10 (scored out of 100)

| # | Idea | Pain, with evidence | AI core | Biggest risk | Score |
|---|---|---|---|---|---|
| 1 | **Dafter**: payday-aware Kurdish debt notebook | ~70% of shop sales on credit, 3 notebooks per shop (Rudaw 2023); salaries late in 62 of 120 months, still late in 2026; cash salaries ENDED 31 Aug 2026 | Kurdish voice note → ledger; reads salary announcements per ministry → reminds exactly those customers with a pay link | Kurdish speech-to-text in a noisy venue | 80 |
| 2 | **Dosya**: patient-held family health folder | SmartSuli's own 2024 challenge ("no shared patient data → repeated tests"); paper records; the MoH's new system is public hospitals only; no Sorani lab explainer | any lab report photo → values, flags, Kurdish explanation, trends, doctor summary | medical wording/liability | 78 |
| 3 | **Rêber**: government-service guide + paper check | only 352 of 708 Ur services online; the KRG missed its 2025 digital target | Kurdish question → checklist for your case; photo of papers → missing/expired | data freshness; the government may build its own | 74 |
| 4 | **Derman**: medicine box check + antibiotic guard | antibiotics sold without prescription in >80% of 696 pharmacy visits (2026 study); 35% of drugs counterfeit/smuggled (syndicate) | box photo → molecule, duplicate-dose alert, Kurdish guidance | weak business; a photo can't prove a fake | 70 |
| 5 | **Scam checker** for the newly banked | ~1M moved to salary cards on 31 Aug 2026; fake ZainCash ads; 201 fraud arrests | forward a message/link → Kurdish verdict; scam-wave dashboard | a common idea; slow bank sales | 69 |
| 6 | **Used-car history check** (Kurdish/Arabic) | "Iraq a dumping ground for damaged cars" (Dec 2025); 170k+ imports; 2026 flood/burnt-car ban | VIN photo → auction history + damage explained | licensing auction data | 66 |
| 7 | **Digital-blackmail first aid** | 13,000+ harassment/blackmail complaints (2024); KRI law has no digital-crime cover | confidential Sorani guide + evidence pack | very sensitive; fake-image detection unreliable | 65 |
| 8 | **Grade-12 mock-exam engine** | 117k candidates, pass rate under 50%; 2 months lost to Slemani strikes (2025) | ministerial-format mocks, auto-marking, weak topics | Rahenan already does the AI-tutor part | 63 |
| 9 | **Kurdish legal-rights assistant** | laws online but unread; UNDP desks served ~7,300 | answers quoting the exact article | liability; weak business | 62 |
| 10 | **Kurdish Sign Language desk helper** | ~10k deaf, fewer than 5 interpreters in KRI | 30 clinic/police signs → text, and back | needs deaf signers to record data before Oct 8 | 60 |

**Checked and rejected:**
- Clinic booking: 17 competitors
- DM sales bot: already exists
- Mental-health bot: unsafe
- Farming: overdone
- Traffic: seen before and improving
- Air quality: weak
- Water early warning: dams refilled in 2026
- Kurdish chatbot/TTS: exists
- Voice interpreter or "Be My Eyes": no Sorani TTS
- Tourism: not a pain

---

# #1 Dafter (دەفتەر): the payday-aware debt notebook

**One line:** The shopkeeper speaks Kurdish, Dafter keeps the book, and it reminds each customer on the day THEIR salary lands.

**Why now:** cash salaries ended on 31 Aug 2026. About a million public employees are now paid on bank cards. For the first time, a shop can be paid back digitally the same day the salary lands.

**Evidence**
- ~70%+ of purchases on credit, 3 notebooks per shop, debts of IQD 10k–1M (Rudaw, Duhok, Sep 2023)
- Salaries late or partial in 62 of 120 months (2015–25); June/July 2026 were still weeks late
- 760k+ paid via MyAccount (May 2026); card payments +41% year on year
- No Sorani ledger exists, only generic Arabic debt apps with no payday logic and no pay links

**How it works**
1. The shopkeeper sends a voice note: "Kak Hama, two bread and oil, eight and a half"
2. Dafter shows "Hama Ali: 8,500 IQD. Confirm?" → one tap
3. The customer (opted in once via the shop's QR) gets "+8,500 IQD, total 43,500" in Kurdish
4. When the Ministry of Education announces salaries, every customer tagged "teacher" gets a polite reminder + a pay link (FIB/FastPay/Qi; mocked in the prototype)
5. Voice question: "Who owes more than 100 thousand?" → list; the dashboard shows the expected money this week by ministry payday

**AI pipeline:** Kurdish speech-to-text (bake-off: ElevenLabs Scribe vs. MetKurd vs. Qwen3-ASR-ckb) → LLM turns it into JSON {customer, items, amount} → Unicode-normalized fuzzy name matching → an LLM reads salary announcements into {ministry, date} → fixed Sorani reminder templates, with the LLM filling the slots.

**Live demo (90 sec):**
1. A judge scans a QR and becomes a "teacher" customer.
2. A teammate sends a Kurdish voice note and **the judge's phone buzzes** with "+5,000 IQD".
3. A teammate posts "Education salaries distributed today" and **the judge's phone buzzes again** with a reminder + a pay button.
4. The judge taps it and the dashboard shows "Paid ✓".

**48h scope**
- MUST: Telegram bot (shop + customer), voice/text → entry with confirm, receipts, salary trigger, dashboard, mock pay page
- NOT: handwritten notebook reading, real payments, Badini

**Business:**
- Free for shops.
- Money from payment partners (merchant acquisition), a premium tier for bigger shops, and later repayment history for supplier credit, with consent.
- India's Khatabook/OkCredit are the model (verify their numbers before quoting).

**Your founder story:** "I build point-of-sale systems for a restaurant in Switzerland. In Kurdistan, the point-of-sale for most neighborhood sales is a paper notebook."

**Hard judge questions**
- **"Arabic ledger apps exist."** They need typing and don't know paydays. Ours takes Kurdish voice, one tap, and knows each customer's payday.
- **"Old shopkeepers won't use apps."** There's no new app: it runs inside Telegram/WhatsApp, where they already send voice notes. Bring your interview numbers.
- **"Reminders shame people."** They're private and polite, the shop picks who gets them and the tone, only opted-in customers receive them, and only on their payday.
- **"The AI mishears the amount."** The shopkeeper confirms every entry and the customer gets an instant receipt, so both sides see it.
- **"Is this AI enough?"** Show messy REAL voice notes from your interviews being parsed, plus the per-ministry payday matching. No global app can do that.

**Risks:** venue noise (push-to-talk + text fallback), adoption (prove it with interviews), consent/privacy (QR opt-in), politics (stay neutral: "whatever the reason for delays, shops carry the cost").

---

# #2 Dosya (دۆسیە): the patient-held health folder

**One line:** Your family's medical papers, read by AI, explained in Kurdish, and carried to any doctor.

**Why it fits SmartSuli:** their 2024 challenge was "patients repeat tests because there's no shared patient data." Hospital-side systems need every clinic to integrate, and that's why they're slow. Dosya flips it: the patient holds the record, and AI builds it from the papers they already get. It works tomorrow with zero integration.

**Evidence:** the SmartSuli Hackathon'24 health challenge; Iraq's records are paper-based (2025 paper); the MoH's digital hospital system (~May 2026) covers public hospitals only; no Sorani lab explainer exists (Arabic ones do). The comprehension pain is UNVERIFIED, so interview patients.

**How it works:**
1. Photo of a lab report → values, units and reference ranges extracted.
2. Color-coded table + Kurdish explanation + questions to ask your doctor.
3. Saved per family member, with trend charts.
4. A "Share with doctor" QR opens a one-page English summary.
5. **Safety:** critical values trigger "go to the ER today" from FIXED RULES, not the LLM. Never diagnose.

**Live demo:**
1. Three anonymized reports from one person (2024, 2025, 2026).
2. Photograph the newest on stage → red flags + Kurdish explanation.
3. The HbA1c chart climbs over 3 years.
4. A judge scans the QR and reads the doctor summary on their own phone.

**48h scope:**
- MUST: 4 common panels (CBC, lipids, glucose/HbA1c, kidney/liver), ~40 explanation templates, profiles, timeline, share page.
- NOT: diagnosis, handwriting.

**Team need:** a medical student writes and checks the Sorani templates.

**Business:** labs pay per digital delivery (it replaces paper and keeps their brand in front of the patient); clinics; a family plan; later an MoH bridge for private labs.

**Before choosing it:** ask The Foundation what happened to the 2024 health teams. If one is still building this, pick Dafter.

**Update (2026-09-11, round 2: the user asked about a health card + a hospital web system):**
- **Federal Iraq's "Dhamani" smart health card already exists:** full record, targeting 3M people in 2026, starting in Baghdad; KRI coverage UNVERIFIED. https://www.iraqinews.com/health/iraq-launches-dhamani-digital-health-insurance/
- **The KRI MoH region-wide digital hospital system** already covers public hospitals. So "a card + a hospital system" competes with the government's own projects in front of government judges.
- **The real gap is PRIVATE clinics:**
  - In Erbil, 46.3% of patients use private clinics and 18.6% private hospitals (2024 study: https://pmc.ncbi.nlm.nih.gov/articles/PMC11334525/).
  - Patients see several doctors for one problem.
- **Legal risk:** Iraq/KRI have no data-protection law (Soran Univ., Nov 2025), so privacy is the judges' first question.
- **The design that works is exactly Dosya:**
  - The patient photographs their own papers, and AI builds the timeline.
  - The card is only a QR key; records sit in the cloud, opened with the patient's phone code, with emergency access logged.
  - The doctor views the record for 24h without an account.
  - Pitch it as covering private clinics and complementing Dhamani and the MoH system.
- **The agent's verdict:** keep Rêga as the main entry (no competing government project, lower legal risk). If the team prefers health, build only this patient-first version.

---

# #3 Rêber (ڕێبەر): the government-service guide

**One line:** Before you go to any office, Rêber tells you exactly what to bring, and checks your papers with your camera.

**How it works:**
1. Ask in Kurdish ("renew my passport").
2. Rêber finds the official services.gov.krd requirements (scraped) and makes a checklist for your case, with the office, fees, and whether it can be done on KRDPass.
3. Photograph your SPECIMEN papers → "residency card expired in March; renew that first."

**Weak spots:** the government pages may be outdated (verify 10 services by phone), B2C money is weak, and the KRG's IT department could see it as its own turf. Pitch it as "the pilot you can adopt."

---

## Next steps
1. **Apply at foundation.krd/smartsuli before Sept 28.** It takes about 20 minutes.
2. DM @thefoundationkrd with 4 questions: what are the scoring criteria, what team size is allowed, what may exist before Oct 8 (accounts, API keys, data), and what happened to the 2024 health teams?
3. Pick Dafter or Dosya with your team this week. Dosya needs a medical student.
4. Collect field evidence (1–2 days):
   - Dafter: 20 shop interviews in Slemani, with photos of notebooks (names blurred).
   - Dosya: 30 anonymized lab reports and 10 patient interviews.
5. If you pick Dafter: run a 1-hour speech-to-text bake-off on 50 recorded shop-style Kurdish voice notes.

The hour-by-hour build plan, team roles and pitch skeleton are in `Build_Plan_48h_Template.md`.

## Unverified: check before you pitch
- How common debt notebooks are in Slemani specifically (the only hard data is from Duhok, 2023). Your interviews fix this.
- Whether patients really can't read their lab reports. Your interviews fix this.
- Khatabook/OkCredit current numbers.
- Whether OpenAI or Google speech-to-text supports Kurdish now. Assume not.
