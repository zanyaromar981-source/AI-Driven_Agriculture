# FRONTEND.md: what the backend expects from the frontend

This is the backend's side of the contract. `BACKEND.md` says what the app needs; this file says what the backend in `backend/` really does today. The app follows this file. Disagreements are settled here and in `BACKEND.md`, not in code.

Status: v3, 2026-10-08. The backend now answers with the body shapes of `BACKEND.md` section 2, so the app's `HttpApi` works against it unchanged. (v1 of this file described `{data}` and `{errors}` wrappers; those are gone.) Added 2026-10-09 12:11: section 13, Ask the Doctor.

## 1. How to point the app at it

Build with `--dart-define=API_URL=http://<host>:3000/v1`. Every route lives under `/v1`.

## 2. What is built

Exact request and answer shapes for every route are in the live docs at `http://<host>:3000/api-docs`.

**Farmer app (token from sign-in)**

| BACKEND.md | Route | State |
|---|---|---|
| 2.1 | `POST /v1/auth/otp/send`, `POST /v1/auth/otp/verify` | built; see section 3 for how the code is delivered |
| 2.2 | `GET /v1/farms`, `POST /v1/farms`, `GET /v1/farms/{id}`, `PUT /v1/farms/{id}/cells`, `DELETE /v1/farms/{id}` | built |
| 2.3 | `GET /v1/farms/{id}/status` | built as a placeholder so Home opens. There is no table or write route for satellite readings yet, so it cannot show data even if a job runs: `picture_date`, `next_picture_expected`, `greenness_pct_of_normal`, `weak_where` are `null`, `cells` is empty, `crops` lists the farm's real crop plots with `level: "none"`. Home opens with "waiting for the first satellite picture" |
| new | `GET /v1/farms/{id}/insights` | built: what is known about the farm, topic by topic (section 8) |
| new | `GET /v1/me`, `PUT /v1/me` | built: the farmer's profile (`phone`, `name`, `lang`) |
| new | Alwa market, 6 routes under `/v1/alwa` | built (section 9) |
| 2.5 | `POST /v1/farms/{id}/ask` | built: Ask the Doctor (section 13). It needs the local Doctor service running; without it the answer is `502 doctor_failed` |
| 2.4, 2.6, 2.7 | plan, reports, alerts, devices, `DELETE /v1/account` | not built yet |

**Ministry dashboard (no login)**

| Route | What it answers |
|---|---|
| `GET /v1/region/overview?month=YYYY-MM` | all 33 districts with dryness, band, rank, change against last year; region summary |
| `GET /v1/zones/{slug}?month=` | one district: its reading, sub-districts, same month in earlier years |
| `GET /v1/region/compare?year=&with=&month=` | two years side by side per district, region average by year |
| `GET /v1/dams`, `GET /v1/dams/{slug}/history` | Dukan and Darbandikhan: latest level, a year ago, history |
| `GET /v1/outlooks`, `GET /v1/outlooks/zones/{zone_slug}` | next-season outlook per district with confidence and the method's track record |
| `GET /v1/water/plan` | districts ranked by water need, amounts per dam |
| `GET /v1/fires?hours=24` | satellite fire detections and a summary |
| `GET /v1/alwa/markets`, `/prices`, `/prices/{crop}/history`, `/listings`, `/listings/{id}`, `/deals` | the wholesale market: prices, crops on sale, offers, deals |

The districts are the 33 of `web/map_demo/kri_map_data.js` (4 governorates, 72 sub-districts). A district's slug is its English name in lower case with hyphens, for example `chamchamal`; a sub-district's is the same, for example `markaz-zakho`.

**Every number above is empty until a data job pushes it.** The backend stores and serves; it does not compute satellite, weather or forecast values. The jobs write through `PUT /v1/ingest/...` with the `X-Service-Key` header (`backend/README.md`).

## 3. Sign in

- `send` answers `200 {"sent": true, "retry_after_s": 60}`. Asking again before that time answers `429 {"error": "rate_limited", "retry_after_s": <seconds left>}`.
- `verify` answers `200 {"token": "...", "farms_count": 2}` or `401 {"error": "bad_code"}`. A wrong code, an expired code, too many tries and a phone that never asked all give the same `bad_code`.
- Codes are 6 digits, live 10 minutes and allow 5 tries. A code signs in once; if the answer is lost and the app sends the same `verify` again within 2 minutes, it succeeds again. After that the code is refused.
- `lang` accepts `ku`, `kmr`, `ar`, `en`. It becomes the farmer's language on first sign-in.
- **No SMS provider is wired yet** (open point 2 in `BACKEND.md` section 8). The server writes the code to its own log. For a demo, the server can be started with one fixed code for every phone (`AUTH__FIXED_SIGN_IN_CODE` in `backend/.env.example`). Neither is safe with real farmers.

## 4. Farms

Shapes are those of `BACKEND.md` 2.2. Notes on what the backend does with them:

- `id` is a string holding a number, for example `"12"`. Treat it as opaque.
- `area_dunam` is the area inside the walked outline (not rounded). `crops[].dunam` counts painted cells, 25 cells to a dunam. `crops` never lists `empty`; largest crop first.
- The farm's cells are every 10 m cell whose centre is inside the outline. The backend works this list out itself; cells the app did not paint come back as `empty`.
- A painted cell that is not inside the outline is left out and listed in `dropped_cells` as `{"e", "n"}`. This is not an error.
- `Idempotency-Key` on `POST /v1/farms` (and on `POST /v1/alwa/listings`) is honoured: a repeat with the same key and phone returns the farm created the first time, with status `201` and an empty `dropped_cells`.
- `PUT /v1/farms/{id}/cells` changes only the cells listed. To clear a cell, send it with `"crop": "empty"`.
- A farm of another phone answers `404`, the same as a farm that does not exist.
- Retries are safe: `DELETE /v1/farms/{id}` answers `204` whether or not the farm was still there; cancelling an Alwa listing twice answers `204` both times; accepting the same offer twice answers `200` with the same listing both times.
- Not sent yet, because there are no satellite readings in the database: `status`, `last_picture`, `picture_date`, and on cells `greenness_pct`, `level`, `inside_pct`. The app already treats them as optional.
- Cells are still "centre inside the outline", and `crops[].dunam` still counts whole cells. BACKEND.md 0.2 asks for every touched cell with `inside_pct`; that is not done yet.
- The outline is returned as the farmer walked it. It is not snapped to the grid.
- Extra fields the app can ignore: `created_at`, `updated_at`, `created_offline_at`.

## 5. Errors

Always `{"error": "<code>", "detail": "<English text>"}`, plus `field` or `retry_after_s` when they apply.

| Status | Codes |
|---|---|
| 400 | `bad_request` (body or parameter cannot be read, including an unknown crop code) |
| 401 | `unauthorized`, `bad_code` |
| 404 | `not_found` |
| 422 | `invalid`, `bad_polygon` (outline crosses itself, has fewer than 3 or more than 50 corners, or holds no cell), `farm_too_large` (over 50,000 cells), `too_many_farms` (over 20 per phone), `empty_question` and `bad_photo` (Ask the Doctor, section 13) |
| 429 | `rate_limited` |
| 500 | `server_error` (the detail is always `An unexpected error occurred`) |
| 502 | `doctor_failed` (Ask the Doctor: the Doctor service is down or failed; same detail) |
| 503 | `doctor_not_ready` (Ask the Doctor: the Doctor service cannot answer yet; same detail) |

## 6. One number in BACKEND.md looks wrong

The `POST /farms` example has `{"e": 462337, "n": 398812}`. With the definition in section 1 (`e = floor(easting / 10)`, UTM zone 38N), a point at lat 36.0312, lon 44.6021 has easting 464,152 m and northing 3,987,482 m, so the cell is `{"e": 46415, "n": 398748}`. The example `e` is ten times too large. The backend uses the definition; the app's own test data (`e: 46415`) already agrees with it.

## 7. Open questions for the frontend

Answered in BACKEND.md 0.3 (thank you). Still open:

1. Insights (section 8): which topics should the farm screen show first?
2. Alwa (section 9): who may make offers, and does accepting part of the quantity close the whole listing? Today any signed-in phone may offer, and accepting any offer marks the listing sold.

## 8. Farm insights

`GET /v1/farms/{id}/insights` answers `{"farm_id", "topics": [...]}`. Each topic is one of `surface_water`, `groundwater`, `soil`, `rain`, `dryness`, `greenness`, `weather` and carries `as_of`, `source`, `confidence` (`sure`, `likely`, `unsure`), `summary_en`, `summary_ku` and `measures: [{"code", "value", "unit", "label_en", "label_ku"}]`. Only topics that have data are listed; an empty list means nothing is known yet. Show `source` and `as_of` next to the numbers. No job fills these yet, and no source for groundwater depth at farm scale is known, so expect that topic to stay missing.

## 9. Alwa market

With the farmer's token: `POST /v1/alwa/listings` (put a crop on sale), `GET /v1/alwa/listings/mine`, `DELETE /v1/alwa/listings/{id}` (cancel), `POST /v1/alwa/listings/{id}/offers` (make an offer), `POST /v1/alwa/listings/{id}/offers/{offer_id}/accept` (seller only), `GET /v1/alwa/offers/mine`. Phone numbers stay hidden until a deal: then the seller sees the buyer's and the buyer sees the seller's. Error codes: `too_many_listings`, `own_listing`, `listing_not_open`, `offer_not_open`, `offer_too_large`, `bad_closes_at`.

## 10. Dashboard sign-in and roles

For the web dashboard, not the farmer app. Everything is under `/v1/dashboard`; shapes are in `/api-docs`.

- `POST /v1/dashboard/auth/login` with `{"email", "password"}` answers `{"token", "staff", "permissions": [{"resource", "action"}]}`. Send the token as `Authorization: Bearer`. It is a different kind of token from the farmer's: neither works on the other's routes.
- `GET /v1/dashboard/me` answers the signed-in staff member and their permissions. Use it to decide which buttons to show; the server still checks every call.
- `GET /v1/dashboard/permissions` lists the resources and actions a role can hold.
- `/v1/dashboard/roles` and `/v1/dashboard/staff`: list, create, read, update, delete. Each needs its own permission (`roles:read`, `roles:create`, `staff:update`, ...).
- A missing permission answers `403 {"error": "forbidden"}`; no token or a bad one answers `401`.
- Codes to handle: `bad_credentials`, `system_role` (the Owner role cannot be changed), `role_in_use`, `role_name_taken`, `email_taken`, `unknown_role`, `own_account`, `last_owner`, and `cannot_grant` (403: you tried to give a permission, a role or a password reset that goes beyond what you hold yourself).
- A change to a role, or deactivating a staff member, takes effect on that person's next request.

## 11. Dashboard data routes

For the web dashboard, with a staff token. All under `/v1/dashboard`; exact shapes are in `/api-docs`. Every method needs its own permission, named `<resource>:<action>`: `GET` needs `read`, `POST` needs `create`, `PUT` needs `update`, `DELETE` needs `delete`. Without it the answer is `403 {"error": "forbidden"}`.

| Resource | Routes |
|---|---|
| `zones` | `/zones` (districts with sub-districts), `/zones/{slug}/readings[/{month}]`, `/zones/{slug}/sub-zones/{sub_slug}/readings[/{month}]` |
| `dams` | `/dams`, `/dams/{slug}/readings[/{day}]` |
| `outlooks` | `/outlooks`, `/outlooks/{season}/{issued}/zones/{zone_slug}`, `/outlook-runs[/{season}/{issued}]` |
| `water` | `/water/seasons`, `/water/plan/{season}/entries[/{zone_slug}]` |
| `fires` | `/fires[/{id}]` |
| `insights` | `/farms/{id}/insights[/{topic}]` |
| `alwa` | `/alwa/markets[/{slug}]`, `/alwa/markets/{slug}/prices[/{crop}/{day}]`, `/alwa/listings[/{id}]` (moderation: close or delete; shows phone numbers) |
| `farmers` | `/farmers[/{id}]` (shows phone numbers) |
| `farms` | `/farms[/{id}]` (shows the owner's phone) |
| `roles`, `staff` | section 10 |

The same rules everywhere:

- `POST` creates. If the thing already exists the answer is `409 {"error": "already_exists"}` and nothing changes.
- `PUT` changes an existing thing. If there is none the answer is `404`.
- `DELETE` answers `204`, also when the thing was already gone.
- Lists that can grow take `page` and `rows_per_page` and answer with `count`, `page`, `rows_per_page`.
- A reading changed by hand looks like any other; the next data-job push for the same key replaces it.
- The public read routes of section 2 are unchanged and still need no login.

## 12. Daily briefs

A job writes a short brief each night (Sorani and English): one for the region and one per district that has farms. Until that job is running these routes answer with nothing.

- `GET /v1/farms/{id}/brief` (farmer token) answers `{"farm_id", "zone_slug", "brief"}`: the newest brief of the farm's district, or the region brief if the district has none. `brief` is `null` when nothing is stored yet: show "no brief yet", it is not an error.
- `GET /v1/briefs/latest?scope=` and `GET /v1/briefs?scope=&from=&to=` (no login): `scope` is `region` (default) or a district slug.
- A brief is `{"day", "scope", "headline_en", "headline_ku", "summary_en", "summary_ku", "points": [{"level": "info|watch|alarm", "text_en", "text_ku"}], "sources": [{"title", "url"}], "author", "generated_at", "updated_at"}`.
- The text is written by an AI agent from our stored numbers plus a web search. Show `sources` and `author`, and treat it as a draft that staff can correct (`/v1/dashboard/briefs`, permission `briefs:*`).

## 13. Ask the Doctor

`POST /v1/farms/{id}/ask` with the farmer's token. The body is `multipart/form-data`:

| Field | Required | What the backend does with it |
|---|---|---|
| `question` | no | Text, up to 1000 characters (counted as characters, so Sorani is fine). Spaces around it are trimmed; a blank question counts as none. |
| `photos` (or `photos[]`) | no | One part per photo, 0 to 6, each JPEG or PNG and at most 4 MB. Set each part's Content-Type to `image/jpeg` or `image/png`: Flutter's `MultipartFile` sends `application/octet-stream` unless `contentType` is given, and that is refused. The bytes must really be that type. |
| `cell` | no | The cell the farmer tapped, as JSON text: `{"e": 46415, "n": 398748}`. |
| `lang` | no | `ku` (the default) or `en`. |

At least a question or one photo. `voice` and any other field are not read. The whole form may be up to about 26 MB on this route; every other route keeps 2 MB.

Answer `200`, no outer wrapper:

```json
{"likely": "...", "confidence": "sure|likely|unsure", "why": ["weather -> ..."],
 "actions_this_week": ["..."], "cannot_tell": ["..."], "refer_to_officer": true,
 "ku": "...", "en": "...", "inputs_used": ["weather", "field_eye"]}
```

- The backend calls no AI. It checks the farm is the farmer's, sends the farm (centre of the outline, exact area, crops), what `GET /v1/farms/{id}/insights` knows about it, the question, photos, cell and `lang` to the local Doctor service, and passes its answer on after these checks: `confidence` is one of the three; `actions_this_week` holds 3 at most (extra ones are cut); a list the Doctor left out comes back empty. An answer without `likely`, `confidence`, `refer_to_officer`, `ku` or `en` is refused as `502 doctor_failed`, never filled in.
- `inputs_used` is extra to `BACKEND.md` 2.5: which sources the Doctor read.
- Not in this version: `case_id` (nothing is stored yet, there is no cases table; it is needed later for the Control Room inbox) and `transcript` (voice is deferred).
- The Doctor takes 10 to 25 s and the backend waits up to 90 s for it. Give this call its own answer timeout of at least 90 s: the app's usual 20 s would drop answers that are on their way.

Errors, in the order they are checked:

| Status | Code | When |
|---|---|---|
| 404 | `not_found` | The id is not a number. |
| 400 | `bad_request` | The body is not a multipart form, `cell` is not the JSON above, or a text field is not UTF-8. |
| 422 | `bad_photo` | A seventh photo, a photo over 4 MB, a type other than JPEG or PNG, or bytes that are not the declared type. The form is read part by part and refused at the first bad photo. |
| 422 | `empty_question` | No question and no photo. |
| 422 | `invalid` | A question over 1000 characters, or `lang` other than `ku` or `en`. |
| 404 | `not_found` | The farm is another phone's or does not exist (the same answer; the Doctor is not asked). |
| 502 | `doctor_failed` | The Doctor service is down, took over 90 s, failed, or answered something that cannot be used. |
| 503 | `doctor_not_ready` | The Doctor service is up but cannot answer yet (it has no AI key). Try later. |

