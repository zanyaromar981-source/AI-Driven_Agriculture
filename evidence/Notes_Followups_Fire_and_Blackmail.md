# Follow-up notes (from the user's questions to the justice research agent, 2026-09-11)

## Safe Door: 3-sentence summary (draft, for the application)
Safe Door (working name) is an anonymous Kurdish and Arabic web app where anyone in the Kurdistan Region facing online blackmail, domestic violence, addiction, corruption or similar trouble can get help without revealing who they are, and can choose to file a confidential report that goes only to the right agency. It is needed because shame and fear of family finding out keep many people silent, and even so the region logged over 13,000 online harassment and blackmail complaints in 2024 and 12,456 domestic violence complaints in 2025, while help is split across separate hotlines and agencies. It works in two levels: getting help is fully anonymous, with no login, nothing stored and a quick-exit button, and AI understands the person's situation in Sorani or Arabic and walks them through the next steps, including blocking intimate images on participating platforms without the image ever leaving their phone; if they decide to accuse someone, they verify their identity and the AI fills in a case file that only the specialised unit can open and that they follow by a code number, so the police know who they are but their family, the public and the app team never do.

(One fix vs. the agent's text: the source counts "online harassment AND blackmail" complaints, not blackmail alone.)

## Fire detection: DROPPED by the user
- Satellites (NASA FIRMS) catch big remote fires but miss most small grass/crop fires, which are most KRI fires. Alerts arrive about 3 hours after a satellite pass.
- The working version would be hilltop cameras (Goyzha/Azmar) with AI smoke detection plus citizen photo reports.
- Slemani fires fell to 1,264 (Jan–Oct 2025) from 3,712.

## Online blackmail / sextortion help: the design that works
Scale: 13,000+ complaints in 2024 (confirmed, Shafaq "16-Day Action"). KRI law 8/2011 doesn't cover digital crimes.

**Level 1: fully anonymous (no name, phone or account)**
- Safety steps: don't pay, don't delete the chat, screenshot the threat with the account name
- Help reporting the blackmailer's account to the platform
- Block images from spreading via a fingerprint only (the photo never leaves the phone): StopNCII (adults), Take It Down / NCMEC (under-18). Limit: only on member platforms. Telegram/Snapchat/TikTok coverage is UNVERIFIED.

**Level 2: confidential, not anonymous (needed to prosecute)**
- Pre-fill the report and send it only to the specialised unit (Community Police / the anti-violence directorate), only when the victim chooses
- Only the Interior Ministry can promise the victim won't be exposed

**Rules for the prototype**
- A web page, NOT a Telegram/WhatsApp bot (the bot owner sees the account ID / phone number)
- No login, no stored chats, no IP logs
- Strip names/numbers before any text goes to a cloud AI, or run a local model
- A quick-exit button, and no history left on the phone

**Overlap: SafeYou app** (Interior Ministry + anti-violence directorate + UNFPA, launched 17 Dec 2021): a panic button sends location to police and trusted contacts. It's built for emergencies, not for evidence or image takedown. Pitch ours as filling that gap or as a plug-in to SafeYou. Is SafeYou still active in 2026? UNVERIFIED. https://iraq.un.org/en/165825-unfpa-moi-kri-launch-safeyou-app-helping-women-and-girls-kurdistan-be-protected-gender-based

**Can it be misused? One design rule:** the anonymous level only PROTECTS the user; only an identified person can ACCUSE someone. Nothing anonymous ever reaches another person.

| Misuse | Fix |
|---|---|
| A fake report to frame an ex or a rival | anonymous = advice + blocking your own images only; naming a suspect needs Level 2 (verified identity); police investigate first; the bot never contacts the accused or publishes names |
| Blocking non-intimate images to censor others | the fingerprint can only come from an image on your own phone; platforms check matches against their rules |
| Police flooded with junk | only Level 2 reaches police (verified phone or an in-person step) + rate limits + AI spam filter |
| Family finds out she used it | nothing kept on the phone/server + a quick-exit button |
| An insider leaks the report | only the specialised unit opens Level 2 reports, and every access is logged (an Interior Ministry promise) |
| Used as a general chatbot / harmful advice | narrow scope: fixed pre-written safety steps; the AI only sorts and fills forms |
| Faked evidence | the bot only saves original screenshots/chats, never creates or edits; police judge authenticity |

Pitch tip: put this table on a slide. Judges WILL ask.

**Who knows the victim's identity at each stage** (the word is "confidential", not "anonymous", once she accuses):

| Stage | Who knows |
|---|---|
| Getting help + blocking images | nobody (not the police, not the app team) |
| Filing an accusation | only the specialised police unit; hidden from family, public, the local station, the app team |
| Court | prosecutor, judge, usually the defence lawyer (closed hearings in KRI: UNVERIFIED) |

- The blackmailer usually already knows her. The goal is hiding her from family/community and stopping him from getting her address or contacts through the case.
- App support: she tracks the case with a code number (not her name); identity stored encrypted, opened only by the specialised unit; notifications say only "you have an update".

## One "safe door" for many problems (merges the blackmail bot + one reporting channel)
Same design everywhere: anonymous for help, confidential for accusations.

| Need | Anonymous help | Confidential report goes to |
|---|---|---|
| Online blackmail | evidence steps, image blocking | Community Police |
| Domestic violence (women AND men) | safety plan, shelters, legal rights | anti-violence directorate |
| Drug addiction | rehab options, what treatment involves | MoH rehab, NEVER the police |
| Corruption / bribe demands | how to document it | Integrity Commission (hotline 1015) |
| Child labour / abuse seen | anonymous tip | Slemani social monitoring directorate |
| Suicidal thoughts | straight to a trained human; the AI never handles it alone | health services (KRI crisis line: UNVERIFIED) |

Scale for the pitch: 13,000+ blackmail complaints (2024) · 12,456 domestic-violence complaints (2025) · 611 cases of violence against men (2025).

Cautions:
- In 48h, build the core once but DEMO ONLY 2 FLOWS (e.g. blackmail + domestic violence); mock the agency side.
- Tell the user where their info will go BEFORE they share anything. The drug and suicide routes are where a routing mistake causes real harm.
- vs. SafeYou: that's an emergency button; this is anonymous guidance + confidential case routing across agencies.

## Agent's updated top 3 (after fire was dropped)
1. Online blackmail/sextortion help (above)
2. Paperwork navigator (which documents, which office, which order + scan check; adds to KRDPass)
3. One reporting channel: a single Sorani/Arabic entry point for text/photo/voice; AI routes each report to the right agency; the citizen tracks status. Check with Slemani Governorate that it doesn't already exist.
