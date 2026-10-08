# Working rules for this repo

Read `README.md` first, then the END of `STATUS.md` (newest entries are at the bottom).

1. **Do not assume anything.** If you are unsure of anything (what the user means, which file, which number, which approach), ask a question before acting. One question at a time.
2. Say what you are about to do before creating, changing, running or publishing anything, and wait for a yes. A direct instruction ("do it", "go", "push it") is the yes.
3. Log every decision, test and result in `STATUS.md` with the date, newest at the end. One line per entry.
4. Never commit keys. `farm_doctor/.env` holds `ANTHROPIC_API_KEY` and is git-ignored. The 25 GB paper library and the 5.3 GB satellite data stay out of git (see README).
5. Scope is frozen in `Scope_and_Build_Plan_FINAL.md`. Anything on its OUT list is not built without a new decision in `STATUS.md`.

## Commits

6. Commit every change, one commit per thing you worked on. Do not pile unrelated changes into one commit.
7. Commit message = a short title line, then meaningful short bullet points saying what changed and why.
8. No em dashes anywhere (commit messages, docs, code comments). Use a comma, a colon or a new sentence.
9. No "Co-Authored-By: Claude" or any AI attribution line in commits or pull requests.

## Progress tracker

10. Keep `PROGRESS.md` (done, in progress, pending decisions, next) up to date with every change, and commit and push it together with that change.

## Frontend and backend contract

11. `BACKEND.md` (repo root) is what the frontend needs from the backend: payloads, endpoints, shared definitions. Keep it updated with every frontend change that touches data. After every `git pull`, check for `FRONTEND.md`: it is what the backend expects from the frontend, and the frontend follows it strictly. Disagreements are settled in the files, not in code.
