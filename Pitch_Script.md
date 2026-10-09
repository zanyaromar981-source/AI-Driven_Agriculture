# Pitch script: Khor (10 October 2026)

Speaker script for the 8 slides in `design/pitch_deck.pen`. Written to answer the four judging criteria in order: the problem in Sulaimani, how well the idea solves it, the use of AI, and whether it could be built in two days.

Length: about 4 minutes 30 seconds, of which 60 seconds is the live demo. Every number below is in the repo; the source is named in the last column of the checklist at the end.

## Round 1 at the table: use this one (10 October)

Rules from the organizers: 3 minutes to pitch, 2 minutes of questions, one judge group, ID asked on arrival. Time is recorded and breaks ties, so the target is **2 minutes 30 seconds**. One person speaks. Build and Demo and AI each count double, so they get most of the time.

### Before the judges arrive

From the organizers' message: doors open at 7:00, be at the table by 7:30, test between 7:30 and 9:00, judges arrive at 9:00 and may start early. The judges will not wait while anything loads, and that time comes out of the 3 minutes.

- Our SmartSuli ID printed or written large on the table, and memorised by the speaker.
- Laptops and the phone fully charged, chargers in the bag.
- A mobile hotspot switched on and tested, in case the hall's internet fails.
- Phone: app open, already signed in, one farm saved with crops painted, the Alwa listing visible.
- Ask the Doctor: one answer already open from a question asked a few minutes earlier. A live answer took 39 seconds on the server, which is too long to wait for inside 3 minutes.
- Laptop next to the phone with `design/Jutyar_Pitch_Round1.pptx` on slide 1, and the website open in a second window.
- The Mac that keeps the server awake stays on, lid open (`PROGRESS.md`).
- One teammate runs a stopwatch and shows a hand at 2:15.
- Everyone stays until the ceremony ends (1:00 to 1:30 p.m.).

### The script

213 words in total, about 1 minute 45 seconds spoken, which leaves about 45 seconds for handling the phone. The organizers say: start as soon as the judges arrive and show what you built straight away, so the demo starts after one sentence. Slide 1 (the title) is on the screen before they arrive.

| Time | Criterion | Show | Say | Words |
|---|---|---|---|---|
| 0:00 to 0:10 | Problem and local impact | Slide 2 | "Kurdistan has about 50,000 wheat farmers and only about 215 advisers. Jutyar puts an adviser in every farmer's pocket. Let me show you." | 23 |
| 0:10 to 1:15 | Build and Demo (double) | Slide 3, phone in your hand | "This is running now, on a real server. I sign in with my phone number and an SMS code. This is my farm. I walked its corners and marked my crops. Every square is 10 metres, checked from space. I asked a question with a photo, and this is the answer, in Sorani. And this is Alwa, the marketplace: my tomatoes are on sale, and other farmers see them on their phones." | 72 |
| 1:15 to 1:50 | AI Integration and Use (double) | Slide 4 | "AI is the translator. The satellite, the forecast and the field's history are only numbers. An AI model turns them, with the farmer's photo, into advice in Sorani, and it says when it is unsure. Every night another AI agent writes a brief on all 33 districts. And we built all of this with AI: Claude, Codex and Pencil." | 59 |
| 1:50 to 2:30 | Feasibility and scope, then close | Slide 5 | "Is it realistic? The data is free. The server is live, with about 2,000 tests. One AI answer costs half a cent. The Ministry gets its own website, with the farms, the alerts and the region on one map. We start with wheat farmers in Slemani. App, server and website, built in two days. This is Jutyar. Thank you." | 59 |

If the phone or the network fails: say "the network is against us" once, stay on slide 3 (the five screens), point at them, and keep going. Do not debug in front of the judges.

### Two minutes of questions

| Criterion | Likely question | Answer |
|---|---|---|
| Build and Demo | "Is that real data or a mock?" | The farm, the sign-in code, the alerts, the plan and the Marketplace listing come from our hosted server. The satellite and weather numbers are real feeds. The website design shows some sample numbers where the server has no data yet. |
| Build and Demo | "What did you have before the hackathon?" | Research and tests of the method on past seasons, done on 5 to 7 October. Every line of the app, the server and the website was written on 8 and 9 October. The repo history shows it. |
| AI | "Where exactly is the AI? Is the satellite part AI?" | No. The satellite and weather numbers are calculations, and we say so. AI is the translator: it turns those numbers and the farmer's photo into Sorani advice, and it writes the nightly brief. |
| AI | "How do you stop it giving wrong advice?" | It answers only from our rulebook and the measured data, never gives a chemical dose, shows how sure it is, and says "see an officer" when the inputs disagree. Photo diagnosis is not yet tested in Kurdistan: next step is 100 local photos with the plant-protection office. |
| Problem | "Was Slemani really hit?" | 57 percent of normal rain in 2024/25 on our 8 Slemani points. Erbil and Duhok were hit harder. Dukan fell to 24 percent of capacity in June 2025 (AFP). |
| Feasibility | "Who pays? Who runs it?" | Running cost is small: free data, a small server, about half a cent per AI answer. It needs a partner for trust and reach, and the Ministry's extension offices are the obvious one. We have not signed anyone yet. |
| Feasibility | "Is the scope too big?" | The core is small: one farm, one view, one message. The Marketplace and the website use the same server. We would launch the core first. |
| Pitch | "Why no forecast for next season?" | We tested it for three days. Long-range prediction had no skill, so we took it out. Jutyar tells you about now and the next 10 days. |

### Check on the morning of the pitch

- Ask the Doctor through the app: the log (9 October, 18:06) says the Doctor answered on the server in 39 seconds, and that the route through the backend was still to be checked after a rebuild. Ask one real question from the phone before the judges come. If it fails, show the saved answer and say it is from yesterday.
- "About 2,000 automated tests" comes from the log (1,974 unit tests on 9 October). Say "about".
- The name: the app says Jutyar, so say Jutyar.

## Before you start

- Phone with the app installed, signed in once already, screen mirrored or held up.
- One farm already saved on the phone, in case GPS is slow indoors.
- Slide 4 stays on screen during the demo: it shows the example message if the phone fails.
- One person speaks, one person holds the phone.

## The script

| # | Slide | Time | Say |
|---|---|---|---|
| 1 | Cover | 15 s | "Khor means sun in Kurdish. The sun sees every field every day. Our app does the same from space, and advises the farmer every week." |
| 2 | Problem | 35 s | "In the winter of 2024/25, Slemani's farmland got 57 percent of its normal rain. By the end of January the numbers already showed a bad season. Farmers found out months later, from their own fields. Why so late? Kurdistan has about 215 agricultural advisers for about 50,000 wheat farmers. That is one adviser for 230 farmers." |
| 3 | Slemani water | 25 s | "Water swings just as hard. We measured the lakes from satellite pictures. Last September, Dukan covered 31 percent of its full area. This September, 92. Darbandikhan went from 42 to 63. No farmer can plan around that without information." |
| 4 | Solution: the message | 25 s | "This is what the farmer receives. One message, five parts. What is most likely wrong. How sure Khor is. Why, with the source of each reason: the satellite, the weather, his own photo, his neighbours. What to do this week, three steps at most. And what Khor cannot tell: it never gives a chemical dose." |
| 4 | Live demo | 60 s | See the demo steps below. |
| 5 | How well | 30 s | "How do we know it is right? For every field we compare how green it is today with its own normal from 25 years of satellite history. Our numbers match NASA's own service on 1,089 of 1,089 pixels. And over 26 past seasons, the method agreed with what really happened 88 percent of the time." |
| 6 | Use of AI | 35 s | "Where is the AI? In three places. First, a language model reads the satellite, weather and report numbers and writes one plain answer in Sorani. Second, the same model looks at photos of a sick leaf and names the likely causes. Third, a vision model draws field borders on satellite pictures. And we built all of this with AI: Claude wrote the code and Pencil drew the screens. The satellite numbers themselves are plain calculations, and we say so." |
| 7 | Two days | 25 s | "What did we build in these two days? An app that runs on a phone. A server with 138 tests, hosted and answering for 33 districts. 29 designed screens. One map with real borders. Before the hackathon we prepared: we tested the method on 26 past seasons and checked 17 free data sources." |
| 8 | Summary | 15 s | "Four answers. The problem: Slemani's farmers learn of a bad season too late. The solution: Khor watches each field and advises every week. The AI: language and vision models, where judgement is needed. Two days: built and running. Thank you." |

## Live demo steps (60 seconds)

1. Open the app. Show My farms. (5 s)
2. Tap "add farm". Tap three or four corners on the map, or walk if there is room. (15 s)
3. Paint two crops on the grid with a finger. Save. (15 s)
4. Open the farm. Point at the coloured squares: green is normal, yellow is watch, red is alarm. Tap one square to show its card. (15 s)
5. Show "This week": the advice line that comes from the live weather forecast. (10 s)

If the phone fails: stay on slide 4 and walk through the example message shown there.

Not in the demo unless it is tested on the morning of the pitch: Ask the Doctor. The screens are built, but on 9 October the server route and the Gemini key were still in progress (`PROGRESS.md`).

## Round 2 on stage: if we reach the final

Six finalists pitch on stage: 5 minutes, then questions. Round 2 is scored fresh on five criteria of equal weight: Public Value and Impact, Innovation and Differentiation, Scalability and Adoption, Business Model and Investment Readiness, Trust and Responsible Use. Use `design/Jutyar_Pitch_Round2.pptx` (10 slides; the design is `design/pitch_deck_round2.pen`). The lines below are 441 words, about 3 minutes 45 seconds spoken, with a target finish of 4:50.

| Time | Criterion | Slide | Say |
|---|---|---|---|
| 0:00 to 0:10 |  | 1 | "We are AI-Driven Agriculture, and this is Jutyar." |
| 0:10 to 0:45 | Public Value and Impact | 2 | "Kurdistan has about 50,000 wheat farmers and about 215 advisers: one for every 230 farmers. In the 2025 drought the region lost about 800,000 tonnes of wheat. That winter Slemani got 57 percent of its normal rain, and most farmers learned it from their own fields. Jutyar gives every farmer advice for his own field, every week, in his own language, and gives the Ministry the same picture for the whole region." |
| 0:45 to 1:30 | (demo) | 3 | "This is running now, on a real server. I sign in with my phone number and an SMS code. This is my farm. I walked its corners and marked my crops. Every square is 10 metres, checked from space. I asked a question with a photo, and this is the answer, in Sorani. And this is Alwa, the marketplace: my tomatoes are on sale, and other farmers see them on their phones." |
| 1:30 to 2:05 | Innovation and Differentiation | 4 | "What is new? Our research found no farm AI in Kurdish anywhere. Jutyar checks every 10 metre square of a field from space and answers in Sorani, in one message of five parts, including what it cannot tell. And the farmer and the Ministry work on the same data." |
| 2:05 to 2:35 | Innovation (AI) | 5 | "AI is the translator. The satellite, the forecast and the field's history are only numbers. An AI model turns them, with the farmer's photo, into advice in Sorani, and it says when it is unsure. Every night another AI agent writes a brief on all 33 districts. And we built all of this with AI: Claude, Codex and Pencil." |
| 2:35 to 3:05 | Scalability and Adoption | 6 | "Can it grow? It already covers all 33 districts in four governorates, because the satellite and weather data are free everywhere. A government office adopts it through this website: the farms, the alerts, the reports and the region on one map." |
| 3:05 to 3:25 | Scalability and Adoption | 7 | "And farmers have a reason to open it every day: Alwa, the marketplace, where they sell directly to buyers." |
| 3:25 to 4:05 | Business Model and Investment Readiness | 8 | "Who pays? Not the farmer. Jutyar is free for farmers. Our plan is a yearly licence for the Ministry's website. Running costs are small: the data is free and one AI answer costs half a cent. What we need now is one pilot district with an extension office." |
| 4:05 to 4:40 | Trust and Responsible Use | 9 | "Is it safe? We keep very little: a phone number, no name, no password. A farmer can delete his account and his farms from Settings. Staff see only what their role allows. And the advice is careful: it never gives a chemical dose, it says when it is unsure, and it sends the farmer to an officer." |
| 4:40 to 4:50 | Close | 10 | "Built in two days, running today. Jutyar: from one farm to a smarter Kurdistan. Thank you." |

### Round 2 questions

| Criterion | Likely question | Answer |
|---|---|---|
| Public value | "How many people does this reach?" | About 50,000 wheat farmers in the Kurdistan Region and the Ministry offices that serve them. The 2025 drought cost about 800,000 tonnes of wheat (Peregraf), so even a small share saved is large. |
| Innovation | "FAO and others already do satellite monitoring. What is different?" | Those tools are for experts and are not in Kurdish. Our research found no farm AI in Kurdish at all. Jutyar goes down to one farmer's field and answers in Sorani. |
| Scalability | "Can it work outside Slemani?" | It already holds all 33 districts in four governorates. The satellite and weather data cover the whole of Iraq, so a new area needs district borders and local crop rules, not new sensors. |
| Business model | "Who pays, and can it keep running?" | The plan is a yearly licence for the Ministry's website, free for farmers. Running cost is small: free data, a small server, about half a cent per AI answer. No contract exists yet. We need one pilot district. |
| Trust | "What personal data do you hold?" | A phone number and the farm outline. No name and no password. The farmer can delete the account and the farms from Settings. Staff see farms through roles and permissions. |
| Trust | "What if the AI is wrong?" | It answers only from the measured data and our rulebook, never gives a chemical dose, shows how sure it is and sends the farmer to an officer when unsure. Photo diagnosis is not yet tested in Kurdistan, and we say so. |

To settle before the final:

- The business model on slide 8 (free for farmers, yearly Ministry licence, one pilot district) is a proposal written for the deck. The team must agree on it, or change the slide.
- Masked phone numbers for staff ("protected mode") are designed but not built, so do not claim them. The slide says only that staff see what their role allows.
- The Sorani message on the AI slide still needs a native speaker's check.

## The Round 1 deck, slide by slide

`design/pitch_deck_3min.pen` has 5 slides in the main row, cut down so the pitch fits in 3 minutes, and 3 backup slides below them for the questions. The same slides are in `design/Jutyar_Pitch_Round1.pptx`, with these lines as speaker notes.

| # | Slide | Criterion | What it shows |
|---|---|---|---|
| 1 | AI-Driven Agriculture | | Title and sun logo. On screen while the judges arrive |
| 2 | 50,000 farmers. 215 advisers. | Problem and local impact | "Help comes too late." and Slemani at 57% of normal rain |
| 3 | Built in two days. | Build and Demo | Five app screens: sign in by SMS, map your field, see it from space, ask in Sorani, sell in the Marketplace |
| 4 | AI is the translator. | AI | The Sorani answer with its five parts, the nightly AI brief, built with Claude, Codex and Pencil AI |
| 5 | Ready to grow. | Feasibility and scope | Free data, live server, half a cent per AI answer, the Ministry through our website. The pitch ends here |
| Backup | Farmers and government, connected. | Questions | The website: Control Room and region dashboard |
| Backup | Sell at a fair price. | Questions | Alwa, the Marketplace, in three screens |
| Backup | From one farm to a smarter Kurdistan. | Questions | Vision picture |

Notes:

- Slide 3 shows design screens. The real phone is the demo; the slide is the backdrop and the fallback.
- The Sorani message on slide 4 was translated for the slide and still needs a native speaker's check.
- "Partner" on slide 5 is the partner we would approach. Nobody has signed.
- The pictures on slides 1 and 2 and on the vision backup slide are AI-made.

## Questions the judges may ask

| Question | Honest answer |
|---|---|
| Is the photo check accurate? | Not tested in Kurdistan yet. Research shows lab accuracy of 99 percent falls to about 72 percent in real fields. Our plan is 100 local photos scored with the plant-protection office. The app never gives chemical doses and says "unsure, see an officer" when the inputs disagree. |
| Was 2025 really bad in Slemani? | Rain was 57 percent of normal on our 8 Slemani points. The drought hit Erbil and Duhok harder. Slemani's lakes show the swing most clearly: Dukan fell to 24 percent of capacity in June 2025 (AFP). |
| Do you predict next season? | No. We tested it for three days and long-range prediction had no skill, so we took it out. Khor tells you about now and the next 10 days. |
| How much was built before the hackathon? | The research and the tests on past seasons were done on 5 to 7 October. The app, the server, the designs and the map were built on 8 and 9 October. Every commit in the repo is dated 8 October or later. |
| Does the field-border AI work everywhere? | It works well on the plains and misses about half the fields on hills. That is why the app asks the farmer to walk the corners. |
| What does it cost to run? | The satellite and weather data are free. One AI answer costs about half a cent with Gemini. |
| Who are the advisers numbers from? | About 50,000 wheat farmers and about 215 extension staff in the Kurdistan Region, from our research report. We found no reliable count for Sulaimani alone. |

## Number checklist

| Number | Where it is said | Source in the repo |
|---|---|---|
| 57% of normal rain, winter 2024/25 | Slide 2 | `evidence/past_seasons/FINDINGS.md` (381 mm, 8 Slemani points) |
| Clear by the end of January | Slide 2 | same file (50% of normal by end of January) |
| 215 advisers, 50,000 wheat farmers | Slide 2 | `reports/What_AI_Can_Do_For_KRI_Agriculture.md` |
| Dukan 31% to 92%, Darbandikhan 42% to 63% | Slide 3 | `evidence/past_seasons/BACKTEST_RESULTS.md` section 6d (83 and 249 of 270 km², 47 and 71 of 113 km²) |
| 1,089 of 1,089 pixels, 88% over 26 seasons | Slide 5 | `README.md`, "What we can honestly claim" |
| 138 tests, 33 districts | Slide 7 | `PROGRESS.md` |
| 29 designed screens | Slide 7 | 14 app screens and 15 dashboard frames as of 8 October; the dashboard has more now |

## Open points for the team

- The deck says Khor. The app, the server and the Control Room still say Jutyar. Pick one name before the pitch, or say one line about it.
- Slide 8 still shows `[team names]`.
- Slide 5 uses an AI-made satellite picture as its background. Swap it for a real one if there is time.
- A native Sorani speaker should read the script aloud once if the pitch is given in Sorani.
- The example message on the message slide is now in Sorani, translated for the slide. A native speaker must check it before the pitch, above all the farming words (yellow rust, flag leaves, plant-protection office, urea).
