# 48-hour build template (works for any of the ideas)

## Team (4 people is the sweet spot)
| Role | Owns |
|---|---|
| AI lead | prompts, the model pipeline, eval set of 20 real samples, fallbacks |
| Backend | API, database (Supabase), WhatsApp/Telegram bot wiring |
| Frontend | the dashboard/app screens the judges see |
| Pitch + domain | user interviews, real data, deck, demo script, rehearsals. Should be the best speaker. For a health idea, this is ideally a medical student |

## Default stack (fast, known)
- Web: React (Vite) or Next.js + Tailwind. Flutter only if the demo must be a phone app
- Backend: Node/Express or Next API routes; Supabase for Postgres + auth + file storage
- AI: one frontier multimodal model via API (Claude / GPT / Gemini): vision + Kurdish text. Speech-to-text and text-to-speech only if the Kurdish quality check passes (see the tech-feasibility notes)
- Channel: Telegram bot (15 minutes to set up) as the safe choice; WhatsApp Cloud API test number as the "real" choice. Set it up BEFORE the event if the rules allow accounts and keys to exist beforehand

## Hour plan
| Hours | Goal |
|---|---|
| 0–2 | Lock scope to ONE demo path. Write the demo script first, then build only what the script shows |
| 2–12 | AI pipeline working end-to-end on 5 real samples (ugly UI is fine) |
| 12–24 | Wire the channel + dashboard; the full demo path runs once |
| 24–30 | Sleep in shifts. Seriously, a tired pitcher loses more points than a missing feature |
| 30–40 | Polish the demo path only. Record a backup video of the demo working |
| 40–46 | Deck (8–10 slides) + 3 full rehearsals with a timer + judge Q&A drill |
| 46–48 | Code freeze. Nothing new. Charge phones, test the venue Wi-Fi, have a hotspot ready |

## Demo rules
- Live beats video, and a judge taking part beats live. Keep the backup video ready anyway.
- Have the "bad input" case ready (blurry photo, mixed-language message) to show it's robust.
- Show one number: time saved, money recovered, people reached.

## Pitch skeleton (3–5 min)
1. A real person's story (10 sec)
2. The pain in numbers, with evidence you gathered yourselves (30 sec)
3. Live demo (90 sec)
4. Why now, and why AI (the Kurdish/local moat) (20 sec)
5. Who pays / how it grows across Iraq (30 sec)
6. Team + what you'd do with The Foundation's support (20 sec)

## Before the event (Sept 11 → Oct 7)
1. Apply before **Sept 28**
2. Pick the idea (this report)
3. 15–25 short interviews with real users in Slemani (photos and quotes allowed in the pitch)
4. Collect 20–50 real samples (lab reports / DMs / notebook pages / textbook pages), anonymized
5. Check the Kurdish speech and vision quality on those samples with 30 minutes of API testing
6. Check the event rules: what may exist before the start (API keys, accounts, datasets, design mockups)?
