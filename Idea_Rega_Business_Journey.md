# Idea: Rêga (ڕێگا, "the way"): from "I want to open a café" to open, across every office

Status: the user's own idea, shaped with the justice research agent on 2026-09-11. It grows out of the earlier "Rêber" paperwork guide.

## Verdict: KEEP IT, with 4 upgrades
1. **Narrow and real:** only "open a small business in Slemani", with every office actually visited. One complete, true roadmap beats ten half-guessed ones, and judges can check it.
2. **An AI feature judges will notice: checking documents by camera.** Photograph your papers before going; the AI checks it's the right form, stamped, signed, not expired, and matches the office checklist. Plus Sorani voice input. (Repeat trips for a missing paper: common complaint, not measured.)
3. **A reason for the government to adopt it:** a bottleneck dashboard (median days per step, per office) built from users' reports. That's what makes it a smart-city system, not just a guide.
4. **Buttons first, chat second:** tap café / shop / office → roadmap instantly; voice and free text are the fallback.

**Pitch line:** "Starting a business in Iraq takes 8.5 procedures and 26.5 days (World Bank 2020). In Slemani, nobody tells you the order. Rêga does."
**Future-plans slide (not the demo):** the same engine for every government process + the Safe Door help channel.

## The problem
Opening a business in Slemani means many offices in an order nobody shows you:
- company registration (the business bureau alone takes **20–50 days**)
- a lawyer from the Lawyers Syndicate
- an accountant
- a tax ID
- social-security registration
- municipal, health and fire permits

No end-to-end Slemani guide exists online (only Saudi/UAE ones). Iraq overall: 8.5 procedures, 26.5 days, ranked 154th of 190 (World Bank Doing Business 2020, measured in Baghdad).

## What already exists (don't rebuild it)
- **services.gov.krd (Khizmet):** one info page per service, by ministry. https://services.gov.krd/en ; business-bureau page: https://service.gov.krd/en/service/moti-27-en
- **Online company registration** (June 2023, 80% cheaper), co-built with the **Deputy PM's office, which also runs SmartSuli**. https://www.iraq-businessnews.com/2023/06/05/krg-launches-business-registration-system/
- **KRD Pass:** digital ID + documents
- The model is proven elsewhere: Saudi Arabia's Balady portal. https://balady.gov.sa/en/services/issuing-commercial-license

## What Rêga adds
1. **The whole journey, not the pieces:** which step comes first and which step unlocks the next.
2. **Personal, in plain Sorani:** "I want to open a café in Slemani" → AI asks 3–4 questions (location, food or not, partners) → YOUR roadmap.
3. **What really happens at the office:** after each step, users report what was actually asked and how long it took ("Waze for paperwork").
4. **The city-system angle:** a governorate dashboard showing where people get stuck. Citizens get a map; the government sees its bottlenecks.
5. **The useful piece of Safe Door, without the risk:** an "asked for an unofficial payment?" button at each step sends an anonymous tip to the Integrity Commission.

## The hard part: data (the AI must never invent a step)
- Build a verified step list. Each step has: office, map pin, documents, fee, typical days, which step it depends on, and a link to the official Khizmet page.
- The AI only chooses and explains steps from this list.
- For 48h: map 2–3 journeys (e.g. café, small shop, online business) by interviewing people who did them recently (plus the user's own experience) + Khizmet pages as sources.
- Check the hackathon rules: may this data be collected before Oct 8?

## Data loop: PEOPLE collect the facts, not AI (the user's decision)
1. **Field agents build the map.** Start from Khizmet's list of services per office, then visit the office and record every service in the same format: office, who can apply, documents, fee, steps in order, typical days, depends-on, contact, date checked.
2. **Offices keep it current.** A citizen question goes to the office; staff reply by email; the answer is saved and becomes a public FAQ with office + date.
3. **Citizens confirm it.** After a step, users say what was actually asked and how long it took. If several reports differ, the step is flagged for an agent to re-check.

| Weak spot | Fix |
|---|---|
| Offices describe the official process, not the real one | agents also go through it once as an ordinary citizen |
| Staff don't reply | a named person per office + a public reply-time dashboard (needs governorate backing, which SmartSuli can give) |
| The same question 1,000 times | show existing answers first; only NEW questions reach the office |
| Two clerks, two answers | every answer shows office + date; users can flag "this was wrong when I went" |
| Staff may not use email | also a simple web inbox or WhatsApp |

## Where the AI is (needed: this is an AI challenge)
The AI never writes a step. It:
- matches a question in Sorani/Badini/Arabic to the right service or existing answer
- turns "I want to open a café" into a personal roadmap from the verified steps
- routes NEW questions to the right office
- spots citizen reports that contradict the map → flags them for re-check
- (from Rêber) checks photos of your papers before you go: missing / expired

## How the AI conversation works (agreed)
**Rule: the AI picks and explains steps, it never writes them.**
1. The citizen asks (typed or voice; Sorani/Badini/Arabic): "Show me the roadmap for starting a business"
2. The AI asks 3–4 short questions: what kind of business, which city, food or not, renting or own place, partners
3. The system builds the roadmap FROM THE DATABASE: steps in order, each office as a pin on a Slemani map, documents checklist, fees, expected days, and each step's source + date last checked
4. Not covered? "We don't have this yet, want to ask the office?" → the question goes to the office inbox → the answer is saved for everyone
5. It follows the person through: tick off steps, a reminder for the next one, and "what actually happened?" after each step (feeds the data loop)

**Example (steps illustrative; field agents provide the real list):**
> User: Show me the roadmap for starting a business.
> AI: What kind: shop, café/restaurant, office, or online?
> User: Café.
> AI: Which city? Will you serve food? Renting the place?
> User: Slemani, yes, renting.
> AI: Here's your roadmap, 9 steps, about X weeks: trade name → company registration (online) → register lease → municipality licence → health permit → fire safety approval → tax ID → social security for staff → signboard permit.

Steps WITH sources so far: trade name, company registration, lawyer, accountant, tax ID, social security. NOT verified: the municipality, health and fire-safety steps for a Slemani café (field agents confirm).

**Build parts:** a database (services, steps, depends-on) · a chat with an AI model that can only search services, fetch a roadmap, or send a question to an office · a Sorani right-to-left web app with roadmap + map · an office inbox (email in/out).

## 48h scope
- Map 1–2 real Slemani journeys (e.g. opening a small business), if the rules allow collecting data before the event
- Build: the verified service list + the roadmap view + the office inbox
- Demo: a teammate plays the office and answers by email
- Pitch: the field-agent method is the scale-up plan once SmartSuli gives the mandate

## Why it fits these judges
- SmartSuli sits under the Deputy PM's office, which co-built online company registration → a natural pilot partner.
- The Foundation's mission is "private sector and economic development", and its judges run incubators. They watch founders get stuck in paperwork every month.

## Before building
Open Khizmet and check whether it already has step-by-step guides for goals like "starting a business". If yes, the pitch narrows to personalisation + real-office reports + the bottleneck dashboard.
