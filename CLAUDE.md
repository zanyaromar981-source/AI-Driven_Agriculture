# Working rules for this repo

Read `README.md` first, then the END of `STATUS.md` (newest entries are at the bottom).

1. **Do not assume anything.** If you are unsure of anything (what the user means, which file, which number, which approach), ask a question before acting. One question at a time.
2. Say what you are about to do before creating, changing, running or publishing anything, and wait for a yes. A direct instruction ("do it", "go", "push it") is the yes.
3. Log every decision, test and result in `STATUS.md` with the date, newest at the end. One line per entry.
4. Never commit keys. `farm_doctor/.env` holds `ANTHROPIC_API_KEY` and is git-ignored. The 25 GB paper library and the 5.3 GB satellite data stay out of git (see README).
5. Scope is frozen in `Scope_and_Build_Plan_FINAL.md`. Anything on its OUT list is not built without a new decision in `STATUS.md`.
