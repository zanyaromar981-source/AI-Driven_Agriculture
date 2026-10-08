# FRONTEND.md: what the backend expects from the frontend

This is the backend's side of the contract. `BACKEND.md` says what the app needs; this file says what the backend in `backend/` really does today. The app follows this file. Disagreements are settled here and in `BACKEND.md`, not in code.

Status: v2, 2026-10-08. The backend now answers with the body shapes of `BACKEND.md` section 2, so the app's `HttpApi` works against it unchanged. (v1 of this file described `{data}` and `{errors}` wrappers; those are gone.)

## 1. How to point the app at it

Build with `--dart-define=API_URL=http://<host>:3000/v1`. Every route lives under `/v1`.

## 2. What is built

| BACKEND.md | Route | State |
|---|---|---|
| 2.1 | `POST /v1/auth/otp/send`, `POST /v1/auth/otp/verify` | built; see section 3 for how the code is delivered |
| 2.2 | `GET /v1/farms`, `POST /v1/farms`, `GET /v1/farms/{id}`, `PUT /v1/farms/{id}/cells`, `DELETE /v1/farms/{id}` | built |
| new | `GET /v1/me`, `PUT /v1/me` | built: the farmer's profile (`phone`, `name`, `lang`) |
| 2.3 to 2.8 | status, plan, ask, reports, alerts, devices, region | not built yet |

## 3. Sign in

- `send` answers `200 {"sent": true, "retry_after_s": 60}`. Asking again before that time answers `429 {"error": "rate_limited", "retry_after_s": <seconds left>}`.
- `verify` answers `200 {"token": "...", "farms_count": 2}` or `401 {"error": "bad_code"}`. A wrong code, an expired code, too many tries and a phone that never asked all give the same `bad_code`.
- Codes are 6 digits, live 10 minutes, allow 5 tries and work once.
- `lang` accepts `ku`, `kmr`, `ar`, `en`. It becomes the farmer's language on first sign-in.
- **No SMS provider is wired yet** (open point 2 in `BACKEND.md` section 8). The server writes the code to its own log. For a demo, the server can be started with one fixed code for every phone (`AUTH__FIXED_SIGN_IN_CODE` in `backend/.env.example`). Neither is safe with real farmers.

## 4. Farms

Shapes are those of `BACKEND.md` 2.2. Notes on what the backend does with them:

- `id` is a string holding a number, for example `"12"`. Treat it as opaque.
- `area_dunam` is the area inside the walked outline (not rounded). `crops[].dunam` counts painted cells, 25 cells to a dunam. `crops` never lists `empty`; largest crop first.
- The farm's cells are every 10 m cell whose centre is inside the outline. The backend works this list out itself; cells the app did not paint come back as `empty`.
- A painted cell that is not inside the outline is left out and listed in `dropped_cells` as `{"e", "n"}`. This is not an error.
- `Idempotency-Key` on `POST /v1/farms` is honoured: a repeat with the same key and phone returns the farm created the first time, with status `201` and an empty `dropped_cells`.
- `PUT /v1/farms/{id}/cells` changes only the cells listed. To clear a cell, send it with `"crop": "empty"`.
- A farm of another phone answers `404`, the same as a farm that does not exist.
- Not sent yet, because there are no satellite readings in the database: `status`, `last_picture`, `picture_date`, and on cells `greenness_pct`, `level`, `inside_pct`. The app already treats them as optional.
- The outline is returned as the farmer walked it. It is not snapped to the grid.
- Extra fields the app can ignore: `created_at`, `updated_at`, `created_offline_at`.

## 5. Errors

Always `{"error": "<code>", "detail": "<English text>"}`, plus `field` or `retry_after_s` when they apply.

| Status | Codes |
|---|---|
| 400 | `bad_request` (body or parameter cannot be read, including an unknown crop code) |
| 401 | `unauthorized`, `bad_code` |
| 404 | `not_found` |
| 422 | `invalid`, `bad_polygon` (outline crosses itself, has fewer than 3 or more than 50 corners, or holds no cell), `farm_too_large` (over 50,000 cells), `too_many_farms` (over 20 per phone) |
| 429 | `rate_limited` |
| 500 | `server_error` (the detail is always `An unexpected error occurred`) |

## 6. One number in BACKEND.md looks wrong

The `POST /farms` example has `{"e": 462337, "n": 398812}`. With the definition in section 1 (`e = floor(easting / 10)`, UTM zone 38N), a point at lat 36.0312, lon 44.6021 has easting 464,152 m and northing 3,987,482 m, so the cell is `{"e": 46415, "n": 398748}`. The example `e` is ten times too large. The backend uses the definition; the app's own test data (`e: 46415`) already agrees with it.

## 7. Open questions for the frontend

1. `PUT /farms/{id}/cells`: is "only the listed cells change" what the paint screen wants, or should the list replace the whole painting?
2. Is 50,000 cells (2,000 dunam) a safe upper limit for one farm?
3. `GET /v1/me` and `PUT /v1/me` are new. Does the settings screen want anything else on the profile?
