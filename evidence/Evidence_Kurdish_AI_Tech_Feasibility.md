# Evidence: Kurdish AI tech feasibility, Sept 2026 (agent report, condensed)

## Verdict
Voice INPUT in Sorani works when rehearsed. Voice OUTPUT in Sorani does not exist from any major vendor. Text in/out in Sorani via LLM is usable with native-speaker review. Printed OCR works; handwriting does not.

| Capability | Best option | Sorani quality | Notes |
|---|---|---|---|
| Speech-to-text, Sorani | ElevenLabs Scribe (Central Kurdish listed); open: Qwen3-ASR-1.7B-ckb, Whisper-ckb fine-tunes; MetKurd (local) | usable on clear, read-style speech; poor on fast dialect (Hewleri) | ElevenLabs contradicts itself (3.1% WER claim vs. the 25–50% tier in its docs) → run your own bake-off |
| STT from OpenAI / Google Chirp / Azure / Deepgram | — | none found (unverified) | don't count on gpt-4o-transcribe for Kurdish |
| Badini/Kurmanji STT | research only | poor/none | keep the demo to Sorani |
| Text-to-speech, Sorani | NOT ElevenLabs, NOT Google, NOT Gemini Live. Local: MetKurd TTS, kurdishtts.com, F5-TTS-Sorani (MOS 3.94) | none from big vendors; local unverified | text replies, or pre-recorded audio for fixed phrases |
| LLM, Sorani | GPT-5 / Claude / Gemini | usable; grammar errors, drifts into Arabic/Persian | short answers, low temperature, fixed templates for critical wording |
| OCR, printed Sorani | Tesseract ckb (95% on court docs); vision LLMs unverified | usable on clean prints | |
| OCR, handwriting | research datasets only | POOR → avoid | kills "photograph the old debt notebook" and "grade handwritten Kurdish answers" |
| Translation ckb↔ar/en | Google Translate (ckb since 2022) > NLLB | usable | |
| Iraqi Arabic STT | standard Arabic vendors | usable for gist (~36% WER across dialects in research) | |

## Channels for a 48h demo
- **Telegram bot:** minutes to set up; ANYONE can message it → the only channel where a judge can join in on the spot without setup.
- **WhatsApp Cloud API test number:** minutes to set up, but only 5 pre-registered recipients → judges can't message it unless added beforehand.
- **Instagram Messaging API:** dev mode serves only accounts with roles on the app; review takes 2–4 weeks → NOT for a live public demo.
- **Viber bots:** commercial contract + €115/month → no.

## Danger zones → mitigations
1. No Sorani TTS → reply in text; if voice is essential, test MetKurd/kurdishtts this week or pre-record key phrases.
2. Dialect / fast speech breaks STT → push-to-talk, close mic, a rehearsed speaker, show the transcript before acting, text fallback.
3. Letter mixing (ي/ی, ك/ک, ه/ە) breaks matching → normalize Unicode before any lookup.
4. LLM Sorani fluency → a native speaker writes the templates; the LLM fills slots.
5. Handwriting → avoid entirely.
6. Venue Wi-Fi → backup video + hotspot.

## Impact on the idea list
- Debt notebook (#9): OK if the shopkeeper's voice note comes from a rehearsed teammate and the reply to the customer is text. CUT the "photograph the old notebook" feature.
- Lab explainer (#1): STRONG. Lab reports are printed English, the easiest input there is; the output is Kurdish text.
- Medicine scanner (#2): OK (printed packaging).
- Exam coach (#6): WEAKENED. The handwritten-answer grading wow is unreliable; only printed questions are safe.
- DM sales agent (#10): WEAKENED. No real Instagram demo; Telegram/WhatsApp only.
- Interpreter (#18), Kurdish Be-My-Eyes (#19): CUT (both need Sorani TTS).
- Teacher grader (#8), office handwriting digitizer (#21): CUT (handwriting).
- Procedure navigator + document pre-check (#13): OK (printed IDs/forms).
