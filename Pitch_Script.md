# Pitch script: Khor (10 October 2026)

Speaker script for the 8 slides in `design/pitch_deck.pen`. Written to answer the four judging criteria in order: the problem in Sulaimani, how well the idea solves it, the use of AI, and whether it could be built in two days.

Length: about 4 minutes 30 seconds, of which 60 seconds is the live demo. Every number below is in the repo; the source is named in the last column of the checklist at the end.

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

## 3-minute version: the story deck

For the 3-minute round use `design/pitch_deck_3min.pen` (7 slides, name Jutyar). The slides carry almost no text, so the numbers are spoken. About 3 minutes.

| # | Slide | Time | Say |
|---|---|---|---|
| 1 | AI-Driven Agriculture (title, sun logo) | 25 s | "We are AI-Driven Agriculture, and this is Jutyar. A farmer knows his field better than anyone. He walks it every day. And he can still miss the first signs that part of it is failing. Jutyar makes that easier." |
| 2 | Space sees more. | 25 s | "From space, weak areas appear early. A satellite passes every five days and sees every 10 metres of the field. Jutyar compares each square with its own normal and colours it: green, yellow, orange, red." |
| 3 | Simple to use. | 25 s | "For the farmer it is simple. He signs in with his phone number, walks the border of his field once, and marks his crops. After that: one farm, one view." |
| 4 | One message. | 35 s | "And he receives one message, in Sorani, with five parts. The problem. How confident we are. Why, with the source of each reason. What to do this week. And the limits: what we cannot tell. It never gives a chemical dose. This is where the AI works. AI is not the sensor. It is the translator." |
| 5 | Farmers and government, connected. | 25 s | "The farmer is one side. The government is the other. One website connects them. Farmers' fields and reports come in, alerts go out, and the Ministry sees the whole region on one dashboard: all 33 districts, the crops, the water in Dukan and Darbandikhan." |
| 6 | Sell at a fair price. | 20 s | "And when the crop is ready, Jutyar helps him sell it. This is Alwa, the farmers' marketplace. He sees today's price, puts his crop on sale, and accepts the best offer from the buyers." |
| 7 | From one farm to a smarter Kurdistan. | 25 s | "One farm is the start. The same view for every farm gives the region one picture. We checked the method on 26 past seasons: right 88 percent of the time. And we built the app, the server and the map in two days. Thank you." |

Notes for this version:

- If there is time for a demo, do it on slide 3: hold up the phone and tap one coloured square. 20 seconds at most.
- The coloured grid on slide 2 is an illustration drawn on a real Sentinel-2 picture, not a real result for that field. Say "this is how it looks", not "this is what we found".
- The picture on slide 7 is AI-made. The glowing network is a vision, not a map of real farms.
- The message on slide 4 is a sample in Sorani that a native speaker still has to check.
- The Alwa screens on slide 6 are designs with sample prices and buyers. The server routes for Alwa are built; the phone screens are not in the app yet.
- The website pictures on slide 5 are the team's Control Room and dashboard designs with sample data. The Control Room demo is not connected to the server yet, so say "this is the website we designed", not "this is live".

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
