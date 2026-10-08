# Test plan: prove the forecast works and that farmers want it

Goal: move usefulness from ~60% to 75–80% before the pitch (Oct 10).
Two tests, both done Oct 6–7 (preparing in advance is allowed).

---

## Test 1: Would the AI have warned about 2025? (backtest)

**Owner:** data person (with Claude). **Time:** ~4–6 hours.

### Fix the "not enough history" problem first
- Sentinel-2 only goes back to 2017 (9 seasons). Too few.
- Use **MODIS NDVI (250 m, from 2000)** for the outcome, plus **CHIRPS rain (from 1981)** and **ERA5-Land soil moisture (from 1950)** for the inputs. All of them are in Google Earth Engine.
- Result: ~25 seasons × ~8 zones ≈ **200 examples** instead of ~70.

### What the AI predicts
- **Question:** "Made in December, will next spring be BAD, NORMAL or GOOD for this zone?"
- **Answer key ("bad spring"):** the zone's peak greenness in Mar–Apr is in the bottom 25% of all years for that zone.

### What the AI gets to see (only things known by Dec 31)
1. Oct–Dec rain vs. that zone's normal
2. Soil moisture at the end of December
3. Temperature Oct–Dec
4. Last spring's greenness
5. Dam level in December (Dukan / Darbandikhan)

### Model
- Simple and explainable: **logistic regression or random forest**. Nothing fancy.
- It must beat 2 dumb guesses:
  - "always predict normal"
  - "same as last year"

### The test that matters
1. **Leave one year out.** Train on every year except one, then predict that year. Repeat for all years.
2. **The 2025 test:** train on 2000–2024 only. Give it data up to Dec 2024. Did it say "BAD spring"?
3. Check the other known dry years (2008, 2021–2022), and that wet years (2019, 2026) came out as "not bad."
4. **Monthly update:** repeat with data up to January, February and March. Expect it to get more accurate each month.

### Pass bar (for the pitch)
- ✅ Warns about 2025 **and** at least 2 of the other dry years
- ✅ False alarms in at most 1 of every 4 normal years
- ✅ Beats both dumb guesses

### If it fails (be ready)
Spring rain decides a lot, and nobody can forecast it well in December. If the December forecast is weak but the Feb/Mar one works, pitch it as: **"Early warning that gets sharper every month,"** with the farmer report updating monthly. That's still honest and useful.

### Output for the pitch
One chart showing each year 2000–2025 with the AI's warning vs. what really happened, and 2025 circled.

---

## Test 2: Would a real farmer use it?

**Owner:** presentation person + 1 more. **Time:** ~1 hour per farmer, including travel or a call.

### Who (5 farmers)
- 2–3 **rainfed** (Sharazur / Bazian / Chamchamal area)
- 2 **irrigated** (Garmiyan / Kalar)
- Use family and friends from the team. A phone call is fine.

### What to show
A phone mockup of the one-tap report, replaying **December 2024**: "This is what the app would have told you before the 2025 drought."

### Ask these 5 questions (in Kurdish)
1. What did you plant for the 2025 season, and what happened?
2. If you had seen this report in December 2024, would you have done anything differently?
3. Do you use a smartphone? Which apps?
4. Who do you trust today when deciding what to plant?
5. Would you use this? Should it come from the ministry, or would you pay for it?

### Bring back
- One sentence from each farmer (written down)
- **One short video quote** (with permission) for the pitch, e.g. "If I had known, I would have planted barley."
- An honest count: how many of the 5 said yes

---

## Done when
- [ ] Backtest chart ready, 2025 result known (pass or fail)
- [ ] 5 farmers asked, answers written down
- [ ] At least 1 video quote
- [ ] Results added to STATUS.md and the pitch
