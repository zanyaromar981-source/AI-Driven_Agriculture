# Pitch script: Khor (10 October 2026)

Speaker script for the 8 slides in `design/pitch_deck.pen`. Written to answer the four judging criteria in order: the problem in Sulaimani, how well the idea solves it, the use of AI, and whether it could be built in two days.

Length: about 4 minutes 30 seconds, of which 60 seconds is the live demo. Every number below is in the repo; the source is named in the last column of the checklist at the end.

## Round 1 at the table: use this one (10 October)

Rules from the organizers: 3 minutes to pitch, 2 minutes of questions, one judge group, ID asked on arrival. Time is recorded and breaks ties, so the target is **2 minutes 45 seconds**. One person speaks. Build and Demo and AI each count double, so they get most of the time.

### Before the judges arrive

- IDs on the table.
- Phone charged, app open, already signed in, one farm saved with crops painted.
- Ask the Doctor: one answer already open from a question asked a few minutes earlier. A live answer took 39 seconds on the server, which is too long to wait for inside 3 minutes.
- Laptop next to the phone with `design/pitch_deck_3min.pen` on slide 1, and the website open in a second window.
- The Mac that keeps the server awake stays on, lid open (`PROGRESS.md`).
- One teammate runs a stopwatch and shows a hand at 2:15.

### The script

| Time | Criterion | Do | Say |
|---|---|---|---|
| 0:00 to 0:20 | Problem and local impact | Slide 1, then slide 2 | "Kurdistan has about 50,000 wheat farmers and about 215 agricultural advisers. In the winter of 2024/25 Slemani got 57 percent of its normal rain, and most farmers found out from their own fields. Jutyar gives every farmer an adviser in his pocket." |
| 0:20 to 1:20 | Build and Demo (double) | Slide 3 on the laptop, phone in the judges' hands or held up | "This is running now, on a real server. I sign in with my phone number and an SMS code. This is my farm: I walked its corners and marked my crops. Each square is 10 metres, checked from space. Here are my alerts and my 10-day plan from the live forecast. Here I asked a question with a photo, and this is the answer, in Sorani: the problem, how sure, why, what to do, and what it cannot tell. And here is the Marketplace, where I put my tomatoes on sale and another farmer's phone sees them." |
| 1:20 to 2:00 | AI Integration and Use (double) | Slide 4 (AI is the translator) | "AI is in the product in two places. First, the answer you just saw: the satellite, the weather forecast and the field's history are numbers. An AI model turns them and the farmer's photo into advice in Sorani, and it must say when it is unsure. Second, every night an AI agent reads all 33 districts, does its own research, and writes the daily brief for the Ministry. And we built it with AI: Claude and Codex wrote the app and the server with us, and Pencil's AI drew the screens." |
| 2:00 to 2:30 | Feasibility and scope | Slide 7 (Ready to grow), slide 5 for the website | "Is it realistic? The satellite and weather data are free. The server is already hosted, with about 2,000 automated tests. One AI answer costs about half a cent. The natural partner is the Ministry's extension offices: this website is their side, with the farms, the alerts and the region on one map. We start with wheat farmers in Slemani and grow from there." |
| 2:30 to 2:45 | Close | Slide 8 | "Built in two days: the app, the server, the website. Jutyar: from one farm to a smarter Kurdistan. Thank you." |

If the phone or the network fails: say "the network is against us" once, switch to slide 3 (the five screens) and slide 4, and keep going. Do not debug in front of the judges.

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

## The Round 1 deck, slide by slide

`design/pitch_deck_3min.pen` now has 8 slides that follow the five criteria. The speaker lines are in the Round 1 table above; this is only the map.

| # | Slide | Criterion | What it shows |
|---|---|---|---|
| 1 | AI-Driven Agriculture | | Title, sun logo, "He knows his field. Jutyar makes it easier." |
| 2 | 50,000 farmers. 215 advisers. | Problem and local impact | "Help comes too late." and Slemani at 57% of normal rain |
| 3 | Built in two days. | Build and Demo | Five app screens: sign in by SMS, walk the field, see it from space, alerts and plan, ask in Sorani |
| 4 | AI is the translator. | AI | The Sorani answer with its five parts, the nightly AI brief, built with Claude, Codex and Pencil AI |
| 5 | Farmers and government, connected. | Build, Feasibility | The website: Control Room and region dashboard |
| 6 | Sell at a fair price. | Build | The Marketplace: see what is on sale, call the seller, put yours on sale |
| 7 | Ready to grow. | Feasibility and scope | Free data, live server with about 2,000 tests, half a cent per AI answer, the Ministry as partner |
| 8 | From one farm to a smarter Kurdistan. | Close | Vision picture |

Notes:

- Slides 3 and 6 show the app design screens. Use the real phone for the demo and the slides as the backdrop.
- The Sorani message on slide 4 was translated for the slide and still needs a native speaker's check.
- The website pictures on slide 5 are the earlier designs with sample numbers.
- "Partner" on slide 7 is the partner we would approach. Nobody has signed.
- The pictures on slides 1, 2 and 8 are AI-made.

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
