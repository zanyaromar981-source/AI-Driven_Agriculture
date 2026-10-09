# FRONTEND.md: how to use the backend

For everyone building the farmer app and the web dashboard. The backend in `backend/` is the one source of data: the app and the dashboard should read and write everything through it, and keep no numbers of their own.

Status: v4, 2026-10-09. Three places describe the API, from short to complete:

1. **This file**: how things work, which screen calls what, the rules, what is empty today.
2. **`backend/API.md`**: every route (127 operations) with its body, its answer and who may call it. It is generated from the server, so it is always what the code does.
3. **`/api-docs` on the server**: the same, clickable, with a "try it" button.

`BACKEND.md` is still where the frontend writes what it needs. Where the two files disagree, say so in the files and we fix one of them.

## 1. Where it is

- **Test server:** `http://95.217.14.92:8790`. Everything is under `/v1`, so the app is built with `--dart-define=API_URL=http://95.217.14.92:8790/v1`. It is a small shared server for the competition, plain `http`, not for real farmers' data.
- **Clickable docs:** `http://95.217.14.92:8790/api-docs`.
- **Is it up?** `GET /status` (the process) and `GET /health` (the database).
- **Run your own:** `backend/README.md` (Rust, Docker, three commands).

## 2. Who calls what

There are three kinds of callers. Each has its own way in, and none works on another's routes.

| Caller | Gets in with | Routes |
|---|---|---|
| The farmer app | a farmer token: `POST /v1/auth/otp/send`, then `/verify` | `/v1/me`, `/v1/farms...`, `/v1/alwa...` |
| The dashboard | a staff token: `POST /v1/dashboard/auth/login` | everything under `/v1/dashboard` |
| Anyone, no login | nothing | read-only region data: `/v1/region...`, `/v1/zones...`, `/v1/dams...`, `/v1/fires`, `/v1/outlooks...`, `/v1/water/plan`, `/v1/briefs...`, and the Alwa market's public pages |
| Our data jobs | a service key | `/v1/ingest...`. Not for the app or the dashboard. |

Send a token as `Authorization: Bearer <token>`. JSON in and out, UTF-8, field names in snake_case.

## 3. Rules that hold for every route

- **No wrapper.** A success answer is the object itself. A list sits under a named key: `{"farms": [...]}`.
- **Errors** are always `{"error": "<code>", "detail": "<English text>"}`, sometimes with `field` or `retry_after_s`. Act on `error`; `detail` is for logs. The codes are in section 12.
- **Ids are text.** `"id": "12"`. Do not do sums with them.
- **Times** are UTC with a `Z`. **Days** are `YYYY-MM-DD`, **months** `YYYY-MM`.
- **Two languages.** Text for people comes as `..._ku` (Sorani) and `..._en`. A `..._ku` may be `null` where no Sorani has been written yet: show the English.
- **Missing data is `null` or an empty list, never a made-up number.** Show "no data yet".
- **Retries are safe.** Sending the same call again after a lost answer does not make a duplicate or an error: a second delete answers `204`, a second cancel `204`, a second accept `200` with the same thing. For `POST /v1/farms` and `POST /v1/alwa/listings` send an `Idempotency-Key` header (any unique text per thing you create) and repeat it on the retry.
- **`401`** means the token is missing, wrong, expired, or its owner was deleted: go back to sign-in. **`403`** (dashboard only) means signed in but not allowed.
- **Districts** are named by a slug: the English name in lower case with hyphens (`chamchamal`, `sulaymaniyah`). There are 33, with 72 sub-districts, the same as `web/map_demo/kri_map_data.js`. `GET /v1/dashboard/zones` lists them with Sorani names.

## 4. The farmer app

### Sign in (real codes now)

- `POST /v1/auth/otp/send` with `{"phone": "+9647501234567", "lang": "ku"}` answers `{"sent": true, "retry_after_s": 60}`. **The code is really sent**, by SMS, WhatsApp or Telegram, through OTPIQ. The old demo code no longer works on the test server: you need a real Iraqi mobile number.
- Asking again within 60 seconds answers `429` with `retry_after_s`: show a countdown.
- If the message cannot be sent the answer is `503 {"error": "upstream_down"}` and the farmer can ask again at once.
- `POST /v1/auth/otp/verify` with `{"phone", "code"}` answers `{"token": "...", "farms_count": 2}` or `401 {"error": "bad_code"}` (wrong, expired, too many tries, or never asked: always the same answer).
- A code is 6 digits, lives 10 minutes and allows 5 tries. If the answer to `verify` is lost, sending the same `verify` again within 2 minutes works.
- `lang` is `ku`, `kmr`, `ar` or `en`. It becomes the farmer's language on first sign-in.
- The token lasts 30 days. Keep it; do not look inside it.
- `GET /v1/me` and `PUT /v1/me` (`{"name": text or null, "lang"}`): the farmer's profile.

### Farms

| Call | What it does |
|---|---|
| `GET /v1/farms` | the farmer's farms: `id`, `name`, `area_dunam`, `crops`, `centroid` |
| `POST /v1/farms` | make a farm from walked corners and painted cells; answers the farm and `dropped_cells` |
| `GET /v1/farms/{id}` | one farm with `outline` and `cells` |
| `PUT /v1/farms/{id}` | **edit**: same body as create; replaces name, outline and cells; the farm keeps its id |
| `PUT /v1/farms/{id}/cells` | repaint some cells only |
| `DELETE /v1/farms/{id}` | delete it (`204`, also if it was already gone) |

What to know:

- **Use `PUT /v1/farms/{id}` for an edit now.** The create-then-delete workaround is no longer needed, and with `PUT` the farm keeps its id, so its history and analysis stay attached.
- **Cells:** every 10 m cell the outline touches, each with `inside_pct` (the share of the cell inside the outline, not rounded). It is computed the same way as `cellsTouching` in `app/lib/geo.dart`; on three test outlines the two agreed to within 0.00000002 m2. A cell needs more than 0.01 m2 inside to count.
- **Areas:** `area_dunam` is the exact area inside the outline. `crops[].dunam` is the sum of that crop's cells' inside areas, so the crops plus the `empty` cells add up to the farm. `crops` does not list `empty`.
- **Painted cells outside the outline** come back in `dropped_cells`. That is not an error.
- **Limits:** 3 to 50 corners; the outline must not cross or touch itself (`bad_polygon`); 50,000 cells (`farm_too_large`); 20 farms per phone (`too_many_farms`); a name of 1 to 100 characters. Corners must lie in or near the region.
- **Another farmer's farm** answers `404`, exactly like one that does not exist.
- Farms saved before 9 October keep their old cells, all at `inside_pct` 100, until they are edited.

### What the farm screens can show

| Call | What comes back | State today |
|---|---|---|
| `GET /v1/farms/{id}/status` | the farm from space: picture date, greenness, per-cell levels | a placeholder: every measured field is `null`, `cells` is empty, `crops` lists the real crop plots with `level: "none"`. There is no store for satellite readings yet. |
| `GET /v1/farms/{id}/insights` | what is known about the farm, topic by topic | filled by the per-farm analysis job; see below |
| `GET /v1/farms/{id}/brief` | the nightly brief for the farm's district | `brief` is `null` until the nightly job has run |
| `POST /v1/farms/{id}/ask` | Ask the Doctor | section 6 |

**Insights.** `{"farm_id", "topics": [...]}`. A topic is one of `surface_water`, `groundwater`, `soil`, `rain`, `dryness`, `greenness`, `weather`, with `as_of`, `source`, `confidence` (`sure`, `likely`, `unsure`), `summary_en`, `summary_ku` and `measures: [{"code", "value", "unit", "label_en", "label_ku"}]`. Only topics that have data are listed; an empty list means "nothing yet". Always show `source` and `as_of` next to a number.

The `groundwater` topic is new. Read this before designing a screen for it: it is **not** a well depth. It is a percentile (50 is normal for the time of year; 10 means only 10% of past years were this dry) from a NASA model for a square of about 25 km, so every farm in that square gets the same number, and it cannot see local pumping. It comes with `confidence: "unsure"`. Say "the wider area", never "your well". Its measure codes are `groundwater_percentile`, `root_zone_moisture_percentile` and `surface_moisture_percentile`.

**Brief.** `{"farm_id", "zone_slug", "brief"}`. A brief is `{"day", "scope", "headline_en", "headline_ku", "summary_en", "summary_ku", "points": [{"level": "info|watch|alarm", "text_en", "text_ku"}], "sources": [{"title", "url"}], "author", "generated_at"}`. It is written by an AI agent from our stored numbers and a web search. Show the `sources`, and treat it as a draft, not as checked advice.

### Not built yet for the app

`GET /v1/farms/{id}/plan` (the 10-day weather plan), reports, alerts, push devices, `DELETE /v1/account`. They answer `404`.

## 5. The Alwa market

Sellers set their own price on each listing. There is no automatic price feed: the "price at the alwa today" board only shows what Ministry staff have typed in through the dashboard.

- **No login:** `GET /v1/alwa/markets`, `/v1/alwa/markets/{slug}/prices`, `/prices/{crop}/history`, `GET /v1/alwa/listings` (filters `market`, `crop`, `status`; `page`, `rows_per_page`), `GET /v1/alwa/listings/{id}` (with its offers), `GET /v1/alwa/deals`.
- **With the farmer token:** `POST /v1/alwa/listings`, `GET /v1/alwa/listings/mine`, `DELETE /v1/alwa/listings/{id}` (cancel), `POST /v1/alwa/listings/{id}/offers`, `POST /v1/alwa/listings/{id}/offers/{offer_id}/accept`, `GET /v1/alwa/offers/mine`.
- **Phones are hidden** until a deal. After the seller accepts, the seller's answer shows the buyer's phone and the buyer's answer shows the seller's.
- **Rules:** at most 20 open listings per phone; a listing closes at `closes_at` (at most 14 days ahead); you cannot offer on your own listing; a buyer has one open offer per listing (a new one replaces it); accepting any offer sells the whole listing and declines the others.
- **`fair_price`** on a listing is `fair`, `high`, `low` or `unknown`. It is `unknown` unless staff have entered a price for that crop at that market in the last 7 days.
- **Codes:** `too_many_listings`, `own_listing`, `listing_not_open`, `offer_not_open`, `offer_too_large`, `bad_closes_at`.
- Crop codes here: wheat, barley, tomato, cucumber, potato, onion, watermelon, grape, olive, sunflower, chickpea, pomegranate, okra, eggplant, pepper, apple.

## 6. Ask the Doctor

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

## 7. Region data anyone can read

No login. These are what a public page or the app's region screens use; the dashboard may use them too.

| Call | What it answers |
|---|---|
| `GET /v1/region/overview?month=` | all 33 districts with `dryness`, `band`, `rank`, change against last year; a region summary |
| `GET /v1/zones/{slug}?month=` | one district: its reading, its sub-districts, the same month in earlier years |
| `GET /v1/region/compare?year=&with=&month=` | two years side by side per district |
| `GET /v1/dams`, `GET /v1/dams/{slug}/history` | Dukan and Darbandikhan: latest level, a year ago, history |
| `GET /v1/fires?hours=24` | fire detections and a summary |
| `GET /v1/outlooks`, `/v1/outlooks/zones/{zone_slug}` | next-season outlook per district, with the method's track record |
| `GET /v1/water/plan` | districts ranked by water need |
| `GET /v1/briefs/latest?scope=`, `GET /v1/briefs` | the nightly brief (`scope` is `region` or a district slug) |

## 8. What has real data on the test server today

Be honest on screen about this: most of the dashboard has nothing to show yet.

| Data | State | Where it comes from |
|---|---|---|
| District rain and `dryness` | **live**, all 33 districts, refreshed every 12 hours | rain of the last 365 days against the 10 years before (Open-Meteo, ERA5). `dryness` is only that rain figure on a 0 to 100 scale (50 = normal rain, lower = wetter). It is **not** soil moisture or crop condition: label it "rain against normal". `greenness`, `water_need`, `best_crops` are empty. |
| Fires | **live**, refreshed every 3 hours | NASA satellite detections inside the 33 districts, gas flares removed by a rule. About 3 hours behind the satellite. Nobody has checked the list by hand: call them "satellite fire detections", not confirmed fires. `area_ha`, wind and `farmers_alerted` are empty. |
| Groundwater per farm | runs daily, but there are no farms on the server yet | see section 4 |
| Dams | **empty** | the satellite job is not on the server yet |
| Season outlook, water plan | **empty** | no job; they can be typed in through the dashboard |
| Alwa prices | **empty** until staff type them in | by hand |
| Nightly brief | **empty** | the job is ready but not switched on |
| Farm status from space | placeholder | no store yet |
| Per-farm history (soil, rain, frost, greenness) | filled only where the analysis job runs | that job is not in the repo yet |

## 9. The dashboard: sign in and roles

Everything is under `/v1/dashboard`.

- `POST /v1/dashboard/auth/login` with `{"email", "password"}` answers `{"token", "staff", "permissions": [{"resource", "action"}]}`. The token lasts 12 hours.
- `GET /v1/dashboard/me` answers the signed-in staff member and their permissions. Use it to decide which buttons to show. The server checks every call anyway.
- `GET /v1/dashboard/permissions` lists the resources and actions a role can hold.
- **A permission is `<resource>:<action>`.** Resources: `zones`, `dams`, `outlooks`, `water`, `fires`, `alwa`, `farmers`, `farms`, `insights`, `briefs`, `staff`, `roles`. Actions: `create`, `read`, `update`, `delete`. So `GET` needs `read`, `POST` needs `create`, `PUT` needs `update`, `DELETE` needs `delete`.
- **Roles are made by staff.** `/v1/dashboard/roles` and `/v1/dashboard/staff`: list, create, read, update, delete. A role is a name and a set of permissions; a staff member holds one or more roles.
- The `Owner` role holds everything and cannot be changed or deleted.
- A change to a role, or deactivating someone, takes effect on that person's next request.
- **Codes:** `bad_credentials`, `forbidden` (403), `system_role`, `role_in_use`, `role_name_taken`, `email_taken`, `unknown_role`, `own_account`, `last_owner`, and `cannot_grant` (403: you tried to give a permission, a role or a password reset beyond what you hold yourself).
- **Getting an account:** there is no sign-up. The first account is made on the server by Arya; ask him for one.

## 10. The dashboard: data

Every list, create, change and delete the dashboard does goes through these. Each method needs its own permission.

| Resource | Routes under `/v1/dashboard` |
|---|---|
| `zones` | `/zones` (districts with sub-districts), `/zones/{slug}/readings[/{month}]`, `/zones/{slug}/sub-zones/{sub_slug}/readings[/{month}]` |
| `dams` | `/dams`, `/dams/{slug}/readings[/{day}]` |
| `outlooks` | `/outlooks`, `/outlooks/{season}/{issued}/zones/{zone_slug}`, `/outlook-runs[/{season}/{issued}]` |
| `water` | `/water/seasons`, `/water/plan/{season}/entries[/{zone_slug}]` |
| `fires` | `/fires[/{id}]` |
| `insights` | `/farms/{id}/insights[/{topic}]` |
| `alwa` | `/alwa/markets[/{slug}]`, `/alwa/markets/{slug}/prices[/{crop}/{day}]`, `/alwa/listings[/{id}]` (close or delete a listing) |
| `farmers` | `/farmers[/{id}]` |
| `farms` | `/farms[/{id}]` |
| `briefs` | `/briefs[/{day}/{scope}]` (read, correct, delete; only the nightly job creates) |
| `roles`, `staff` | section 9 |

The same rules everywhere:

- `POST` creates. If the thing already exists: `409 {"error": "already_exists"}`, nothing changes.
- `PUT` changes an existing thing. If there is none: `404`.
- `DELETE` answers `204`, also when it was already gone.
- Lists that can grow take `page` and `rows_per_page` (at most 100) and answer with `count`, `page`, `rows_per_page`.
- A number changed by hand looks like any other, and the next data-job run for the same district and month replaces it.
- **Phone numbers:** the `farmers`, `farms` and `alwa` listing routes show full phone numbers to staff who hold the permission. `BACKEND.md` asks for a "protected mode" (masked phones, farms shown at 1 km); that is **not built**. Until it is, give those permissions only to people who should see phones.
- Deleting a farmer deletes their farms and signs them out at once.

## 11. Things that will trip you up

- **Unknown values in a body** (a crop code not on the list, a wrong `status`) answer `400 bad_request`, not `422`.
- **An empty `page=`** in a query answers `400`. Leave the parameter out instead.
- **A `+` in a query string** must be sent as `%2B` (for example a phone filter).
- **`Ask the Doctor` needs a 90 second timeout** and each photo part's own content type; see section 6.
- **`BACKEND.md`'s create example has a wrong cell number** (`e` is ten times too large there). Use `e = floor(easting / 10)` in UTM zone 38N, as `geo.dart` already does.
- **An outline that only touches itself at a point** is refused by the backend (`bad_polygon`), while the app's own check allows it.
- **`created_offline_at` in an edit** is ignored: the farm keeps the one it was created with.
- **The backend does not round.** `inside_pct`, `area_dunam` and `dunam` come with all their decimals; round when you show them.

## 12. Error codes

| Status | Codes |
|---|---|
| 400 | `bad_request` (the body or a parameter cannot be read) |
| 401 | `unauthorized`, `bad_code`, `bad_credentials` |
| 403 | `forbidden`, `cannot_grant` |
| 404 | `not_found` |
| 409 | `already_exists`, `listing_not_open`, `offer_not_open`, `listing_has_deal`, `market_in_use`, `system_role`, `role_in_use`, `role_name_taken`, `email_taken`, `own_account`, `last_owner` |
| 422 | `invalid` (with `field` when one field is at fault), `bad_polygon`, `farm_too_large`, `too_many_farms`, `too_many_listings`, `own_listing`, `offer_too_large`, `bad_closes_at`, `bad_month`, `bad_range`, `empty_question`, `bad_photo`, `unknown_role`, and other `bad_...` codes that name the field |
| 429 | `rate_limited` (with `retry_after_s`) |
| 500 | `server_error` (the detail is always "An unexpected error occurred") |
| 502 | `doctor_failed` |
| 503 | `upstream_down` (a sign-in code could not be sent), `doctor_not_ready` |

## 13. Open questions for the frontend

1. **Farm status:** the app reads `/status`, but the per-farm analysis writes to `/insights` (the "now" measures inside the `greenness` topic). Should `/status` be built from those, or should the app drop `/status`?
2. **Alwa:** may any signed-in phone make an offer, and should accepting part of the quantity close the whole listing? Today: yes and yes.
3. **Protected mode** for phones and farm positions on the dashboard (`BACKEND.md` 2.11): which screens need it first?
4. **The per-farm analysis job** is not in the repo. It should be, so it can run on the test server whenever a farm is created or edited.
5. Is anything you need missing from `backend/API.md`? Write it in `BACKEND.md`.

## 14. For the website: CORS, caching, and the 2.12 list

Answers to `BACKEND.md` 2.12 and 2.13, as built.

**CORS (A1): built.** The server allows the origins listed in its `HTTP__CORS_ORIGINS` setting (comma list, no `*`). Tell Arya the site's address and the address you develop on, to have them added on the test server. Allowed methods `GET, POST, PUT, DELETE, OPTIONS`; request headers `Authorization, Content-Type, If-None-Match, Idempotency-Key, X-App-Version`; exposed `ETag, X-Api-Version`; preflight is cached 600 s. The preflight answers `200`, not `204`; browsers treat both the same.

**Versions (2.13): built.**
- `GET /v1/versions` (no login) answers `{"api": "1.6.0", "versions": {"zones": 41, "dams": 7, ...}, "server_time"}` for the public topics: `zones, sub_zones, dams, fires, outlooks, water, alwa_prices, alwa_listings, crops, rules, briefs, app_config`.
- `GET /v1/dashboard/versions` (any signed-in staff) adds the private ones: `farmers, farms, messages, jobs, staff_roles`.
- A topic's number goes up whenever anything of that kind is written, by anyone (dashboard, data job, farmer app). The database does it itself inside the write's transaction, so it cannot be forgotten. It may go up by more than one for a single action: only compare "is it higher than what I have".
- Topic to routes: `zones` district readings and `/v1/region...`; `sub_zones` sub-district readings; `dams`; `fires`; `outlooks` (outlooks and outlook runs); `water`; `alwa_prices` (markets and prices); `alwa_listings` (listings, offers, deals); `briefs`; `farmers`; `farms` (farms, cells and per-farm insights); `staff_roles` (staff, roles and their permissions). `crops, rules, messages, app_config, jobs` exist as topics now and start counting when those parts are built.
- **`ETag` and `304`: built, on every JSON `GET`.** Send the tag back in `If-None-Match`; if the answer would be the same you get `304` with no body. The tag is made from the answer's bytes, not from the topic version, so treat it as opaque. `Cache-Control` is `no-cache` (`private, no-cache` when a token was sent).
- `X-Api-Version` is on every answer under `/v1`. Wipe the cache when it changes.

**Permissions (D): the five new resources exist** (`crops, rules, messages, app, jobs`), are listed by `GET /v1/dashboard/permissions`, can be put in roles, and the `Owner` role holds them. Their routes are still being built.

**Farmer details, blocking and letters (A5): built.**
- `PUT /v1/dashboard/farmers/{id}` (`farmers:update`) also takes `gender` (`male`, `female` or null), `birth_year`, `village`, `governorate`, `zone_slug`, `sub_zone_slug`, `notes` (staff only, never sent to the farmer app) and `blocked`. It is a full replace: a field you leave out is cleared, except `blocked`, which stays as it was.
- `GET /v1/dashboard/farmers` (`farmers:read`) also takes `q` (name or phone), `governorate`, `zone`, `blocked`, `sort=created_at|name` and `order=asc|desc`.
- A blocked farmer gets `403 {"error": "blocked"}` on every farmer route from the next request, and on `POST /v1/auth/otp/verify` even with the right code. `POST /v1/auth/otp/send` answers as for any phone but sends no message. The app should show "this account is blocked, call the office" on `blocked`.
- `POST /v1/dashboard/farmers/{id}/letters` (`farmers:read`) with `{"purpose", "lang": "ku" | "en"}` answers `201 {"letter": {"number", "purpose", "lang", "created_at", "issued_by": {"id", "name"}, "farmer": {...}, "farms": [...], "totals": {"farms", "dunam", "crops"}}}`. The number is `JTY-<yyyymm>-<farmer id>-<n>`. Every call issues a new number, so call it once per printed letter.
- `GET /v1/dashboard/letters/{number}` (`farmers:read`) returns the stored record, to check a letter later. Letters cannot be changed or deleted.

**Staff details (A6): built.** Staff have optional `phone` and `job_title` on `POST /v1/dashboard/staff` and `PUT /v1/dashboard/staff/{id}`. `PUT /v1/dashboard/me` (any signed-in staff) takes `{"name", "phone", "current_password", "new_password"}`; the two passwords are optional and go together; a wrong current password is `403 {"error": "wrong_password"}`. It answers like `GET /v1/dashboard/me`.

**Rules (B2): built, but nothing reads them yet.**
- `GET /v1/dashboard/rules` (`rules:read`) answers `{"rules": [...]}` with every column of the request. `PUT /v1/dashboard/rules/{code}` (`rules:update`) with `{"value", "reason"}` answers `{"rule", "changed"}`; outside min and max is `422 bad_range`, a reason outside 3 to 500 characters `422 bad_reason`. `GET /v1/dashboard/rules/{code}/history` is paged, newest first: `{"changes": [{"id", "code", "old_value", "new_value", "reason", "staff_id", "at"}], "count", "page", "rows_per_page"}`. `POST /v1/dashboard/rules/{code}/reset` with `{"reason"}`.
- 15 rules are seeded with the numbers in the code today: nine weather planner rules (as in your list), four dryness band edges (`dryness_greener_from` 25, `dryness_normal_from` 45, `dryness_dry_from` 60, `dryness_very_dry_from` 80), and two Field Eye rules.
- **Field Eye differs from your list.** `farm_doctor/field_eye.py` has no "watch below 85%" or "alarm below 70% of normal", and its cloud limit is 60%, not 30%. What it has was seeded: `field_eye_max_cloud_pct` 60 and `field_eye_weak_pixel_pct` 70 (a pixel below 70% of the field's own median). Tell us if the 85 / 70 / 30 numbers live somewhere else.
- Sorani names and meanings of the rules are null: they need a native speaker. Show the English until then.
- **Honest limit: changing a rule changes nothing yet.** The data jobs can read the values (`GET /v1/ingest/rules?used_by=`), but the weather planner, the dryness bands and Field Eye still use their built-in numbers. Say so on the Rules page, or hide the save button, until this line is removed.

**Job status (B5): built.** `GET /v1/dashboard/jobs` (`jobs:read`) answers `{"jobs": [...]}` in the shape of your request. States: `never` (no run), `failed` (the run that finished last was not ok), `late` (nothing finished ok within 1.5 times `every_hours`), else `ok`. A job becomes `late` by time alone, which does not raise the `jobs` topic: refetch this page on a timer. The rain, fires, groundwater and daily brief jobs report every run. `dams` and `farm_analysis` have no job yet and show `never`. The Sorani job names need a native check.

**A place on every farm (A3): built.**
- Every farm answer, on the app routes and the dashboard routes, carries `governorate` (as the zones list writes it, for example `"Sulaymaniyah"`), `zone_slug` and `sub_zone_slug`. All three come from the centre of the outline and the 72 sub-district shapes of `web/map_demo/kri_map_data.js`; all three are `null` for a farm outside every shape. They are set when a farm is created and when its outline is edited.
- `GET /v1/dashboard/farms` (`farms:read`) also takes `governorate`, `zone`, `sub_zone` (the value `unknown` finds farms with no place), `crop`, `q` (farm name or owner phone), `sort=created_at|area_dunam|name` and `order=asc|desc`.
- **Check the map:** the shape named `Qaradagh` in the map file reaches north to latitude 35.57 and contains the centre of Sulaymaniyah city, and the Sulaymaniyah district has no centre sub-district of its own. So a farm in the city is reported as `qaradagh`. The backend follows the map file exactly; if the shape or its name is wrong, fix it there and we reseed.
- Farms saved before this change get their place when `backfill-farm-places` is run on the server (done at each deploy).

**Totals (A4): built.**
- `GET /v1/dashboard/stats/farms?governorate=&zone=&crop=` (`farms:read`) answers `{"as_of", "totals": {"farmers", "farms", "dunam"}, "by_governorate", "by_zone", "by_sub_zone", "by_crop"}` as requested. Only areas that have farms are listed. Farms with no place are in the totals and in a row with slug `unknown`. With `crop`, only farms growing it are counted and `dunam` stays the whole area of those farms.
- `GET /v1/stats/farms` (no login) answers `totals`, `by_governorate`, `by_zone`, `by_crop` only: no names, no phones, nothing per sub-district. It answers `404` when the server setting `STATS__PUBLIC_FARM_TOTALS` is `false`.
- Both belong to the `farms` topic.

**Still being built from 2.12:** B1 (crops), B3 (inbox), B4 (app control). This section will say when each is in.
