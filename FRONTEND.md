# FRONTEND.md: everything the backend has, and how to use it

For everyone building the farmer app and the website (public View page and staff Admin). The backend in `backend/` is the one source of data: the app and the website read and write everything through it and keep no numbers of their own.

Status: v5, 2026-10-09, API version 1.6.0. Three places describe the API, from short to complete:

1. **This file**: how things work, which screen calls what, the rules, what is empty today.
2. **`backend/API.md`**: every route (155 operations) with its body, its answer and who may call it. It is generated from the server, so it is always what the code does.
3. **`/api-docs` on the server**: the same, clickable, with a "try it" button.

`BACKEND.md` is where the frontend writes what it needs. Where the two files disagree, say so in the files and we fix one of them.

Contents: 1 where it is, 2 who calls what, 3 rules for every route, 4 the farmer app, 5 Alwa market, 6 Ask the Doctor, 7 public data, 8 website sign-in and roles, 9 website data routes, 10 caching, 11 what has real data today, 12 things that trip you up, 13 error codes, 14 not built yet, 15 open questions.

## 1. Where it is

- **Test server:** `http://95.217.14.92:8790`. Everything is under `/v1`, so the app is built with `--dart-define=API_URL=http://95.217.14.92:8790/v1`. It is a small shared server for the competition, plain `http`, not for real farmers' data.
- **Clickable docs:** `http://95.217.14.92:8790/api-docs`.
- **Is it up?** `GET /status` (the process) and `GET /health` (the database).
- **Which version is it?** Every answer under `/v1` carries the header `X-Api-Version`. This file describes 1.6.0.
- **A staff account for testing the website:** there is one with the `Owner` role (every permission) on the test server. Ask Arya for its email and password; they are not written in this public repo.
- **A farmer account:** sign in with any real Iraqi mobile number; the code is really sent.
- **Run your own:** `backend/README.md` (Rust, Docker, three commands).

## 2. Who calls what

There are four kinds of callers. Each has its own way in, and none works on another's routes.

| Caller | Gets in with | Routes |
|---|---|---|
| The farmer app | a farmer token: `POST /v1/auth/otp/send`, then `/verify` | `/v1/me`, `/v1/farms...`, `/v1/messages...`, `/v1/alwa...` |
| The website's Admin part | a staff token: `POST /v1/dashboard/auth/login` | everything under `/v1/dashboard` |
| Anyone, no login (the View page, the app before sign-in) | nothing | read-only: `/v1/region...`, `/v1/zones...`, `/v1/dams...`, `/v1/fires`, `/v1/outlooks...`, `/v1/water/plan`, `/v1/briefs...`, `/v1/stats/farms`, `/v1/app/config`, `/v1/versions`, and the Alwa market's public pages |
| Our data jobs | a service key | `/v1/ingest...`. Not for the app or the website. |

Send a token as `Authorization: Bearer <token>`. JSON in and out, UTF-8, field names in snake_case.

## 3. Rules that hold for every route

- **No wrapper.** A success answer is the object itself. A list sits under a named key: `{"farms": [...]}`.
- **Errors** are always `{"error": "<code>", "detail": "<English text>"}`, sometimes with `field` or `retry_after_s`. Act on `error`; `detail` is for logs. The codes are in section 13.
- **Ids are text.** `"id": "12"`. Do not do sums with them.
- **Times** are UTC with a `Z`. **Days** are `YYYY-MM-DD`, **months** `YYYY-MM`.
- **Two languages.** Text for people comes as `..._ku` (Sorani) and `..._en`. A `..._ku` may be `null` where no Sorani has been written yet: show the English.
- **Missing data is `null` or an empty list, never a made-up number.** Show "no data yet".
- **Retries are safe.** Sending the same call again after a lost answer does not make a duplicate or an error: a second delete answers `204`, a second cancel `204`, a second accept `200` with the same thing. For `POST /v1/farms`, `POST /v1/alwa/listings` and `POST /v1/messages` send an `Idempotency-Key` header (any unique text per thing you create) and repeat it on the retry.
- **`401`** means the token is missing, wrong, expired, or its owner was deleted: go back to sign-in. **`403 forbidden`** (website only) means signed in but not allowed. **`403 blocked`** (app only) means staff blocked this farmer. **`426 update_required`** (app only) means the app is too old.
- **The app sends `X-App-Version: 1.0.3`** (its own version) on every call. See "App settings and the update gate" in section 4.
- **Lists that can grow** take `page` and `rows_per_page` (at most 100) and answer with `count`, `page`, `rows_per_page`.
- **Places.** Districts are named by a slug: the English name in lower case with hyphens (`chamchamal`, `sulaymaniyah`). There are 4 governorates, 33 districts and 72 sub-districts, the same as `web/map_demo/kri_map_data.js`. `GET /v1/dashboard/zones` lists them with Sorani names.
- **Browsers (CORS).** The server allows only the origins in its `HTTP__CORS_ORIGINS` setting. **Tell Arya the website's address and the address you develop on** (for example `http://localhost:5173`); until they are added, a browser will refuse the calls even though curl works. Allowed methods `GET, POST, PUT, DELETE, OPTIONS`; request headers `Authorization, Content-Type, If-None-Match, Idempotency-Key, X-App-Version`; exposed headers `ETag, X-Api-Version`.

## 4. The farmer app

### Sign in (real codes)

- `POST /v1/auth/otp/send` with `{"phone": "+9647501234567", "lang": "ku"}` answers `{"sent": true, "retry_after_s": 60}`. **The code is really sent**, by SMS, WhatsApp or Telegram, through OTPIQ. There is no demo code on the test server: you need a real Iraqi mobile number.
- Asking again within 60 seconds answers `429` with `retry_after_s`: show a countdown.
- If the message cannot be sent the answer is `503 {"error": "upstream_down"}` and the farmer can ask again at once.
- `POST /v1/auth/otp/verify` with `{"phone", "code"}` answers `{"token": "...", "farms_count": 2}` or `401 {"error": "bad_code"}` (wrong, expired, too many tries, or never asked: always the same answer).
- A code is 6 digits, lives 10 minutes and allows 5 tries. If the answer to `verify` is lost, sending the same `verify` again within 2 minutes works.
- `lang` is `ku`, `kmr`, `ar` or `en`. It becomes the farmer's language on first sign-in.
- The token lasts 30 days. Keep it; do not look inside it.
- `GET /v1/me` and `PUT /v1/me` (`{"name": text or null, "lang"}`): the farmer's profile. The details staff keep about a farmer (section 9) are never sent here.
- **A blocked farmer** gets `403 {"error": "blocked"}` on every farmer route from the next request, and on `verify` even with the right code. `send` answers as for any phone but sends no message. Show "this account is blocked, call the office".

### App settings and the update gate

- `GET /v1/app/config` (no login; read it when the app starts) answers:
  ```json
  {"latest_version": "1.0.0", "min_version": "1.0.0",
   "update_message_ku": null, "update_message_en": null,
   "maintenance": false, "maintenance_message_ku": null, "maintenance_message_en": null,
   "maintenance_from": null, "maintenance_until": null,
   "announcement_on": false, "announcement_ku": null, "announcement_en": null,
   "features": {"add_farm": true, "walk_mode": true, "satellite": true, "doctor": true,
                "reports": false, "alwa": true, "plan": false, "push": false},
   "limits": {"farms_per_phone": 20, "max_farm_dunam": 2000, "min_corners": 3,
              "max_corners": 50, "gps_meters": 10},
   "help_phone": null, "public_farm_totals": true, "updated_at": "..."}
  ```
- Hide a part of the app whose switch in `features` is `false`. `plan`, `reports` and `push` are off because the server does not have them yet.
- Show the maintenance screen while `maintenance` is `true`, and the announcement while `announcement_on` is `true`.
- **Send `X-App-Version` on every call.** When it is below `min_version`, every farmer route and both sign-in routes answer `426 {"error": "update_required"}`: show the update screen with `update_message_*`. `GET /v1/app/config` is never refused, so an old app can still learn it must update. A call without the header is let through.
- Honest limit: the server does not yet enforce `limits` or `features` from these settings; they are what the app reads. The server's own limits are the same numbers today.

### Farms

| Call | What it does |
|---|---|
| `GET /v1/farms` | the farmer's farms: `id`, `name`, `area_dunam`, `crops`, `centroid`, and the place (`governorate`, `zone_slug`, `sub_zone_slug`) |
| `POST /v1/farms` | make a farm from walked corners and painted cells; answers the farm and `dropped_cells` |
| `GET /v1/farms/{id}` | one farm with `outline` and `cells` |
| `PUT /v1/farms/{id}` | **edit**: same body as create; replaces name, outline and cells; the farm keeps its id |
| `PUT /v1/farms/{id}/cells` | repaint some cells only |
| `DELETE /v1/farms/{id}` | delete it (`204`, also if it was already gone) |

What to know:

- **Use `PUT /v1/farms/{id}` for an edit.** The farm keeps its id, so its history and analysis stay attached.
- **Cells:** every 10 m cell the outline touches, each with `inside_pct` (the share of the cell inside the outline, not rounded). It is computed the same way as `cellsTouching` in `app/lib/geo.dart`. A cell needs more than 0.01 m2 inside to count.
- **Areas:** `area_dunam` is the exact area inside the outline. `crops[].dunam` is the sum of that crop's cells' inside areas, so the crops plus the `empty` cells add up to the farm. `crops` does not list `empty`.
- **Place:** `governorate` (written as in the zones list, for example `"Sulaymaniyah"`), `zone_slug` and `sub_zone_slug` come from the centre of the outline and the 72 sub-district shapes. All three are `null` for a farm outside every shape. They are set on create and on edit.
- **Painted cells outside the outline** come back in `dropped_cells`. That is not an error.
- **Limits:** 3 to 50 corners; the outline must not cross or touch itself (`bad_polygon`); 50,000 cells (`farm_too_large`); 20 farms per phone (`too_many_farms`); a name of 1 to 100 characters. Corners must lie in or near the region.
- **Another farmer's farm** answers `404`, exactly like one that does not exist.
- **Crop codes come from `GET /v1/crops`** (no login): `{"crops": [{"code", "name_en", "name_ku", "color", "category", "season", "yield_kg_per_dunam", "active", "sort_order"}]}`, active crops only, in display order. Read names and colours there instead of keeping a list in the app. `empty` (an unplanted cell) is not a crop and is not in the list. A code that is unknown or switched off is refused for new data with `422 unknown_crop`; data saved earlier keeps reading back.

### What the farm screens can show

| Call | What comes back | State today |
|---|---|---|
| `GET /v1/farms/{id}/status` | the farm from space: picture date, greenness, per-cell levels | a placeholder: every measured field is `null`, `cells` is empty, `crops` lists the real crop plots with `level: "none"`. There is no store for satellite readings yet. |
| `GET /v1/farms/{id}/insights` | what is known about the farm, topic by topic | groundwater is filled daily; other topics wait for the per-farm analysis job |
| `GET /v1/farms/{id}/brief` | the nightly brief for the farm's district | filled each night; `brief` is `null` for a district with none yet |
| `POST /v1/farms/{id}/ask` | Ask the Doctor | section 6 |

**Insights.** `{"farm_id", "topics": [...]}`. A topic is one of `surface_water`, `groundwater`, `soil`, `rain`, `dryness`, `greenness`, `weather`, with `as_of`, `source`, `confidence` (`sure`, `likely`, `unsure`), `summary_en`, `summary_ku` and `measures: [{"code", "value", "unit", "label_en", "label_ku"}]`. Only topics that have data are listed; an empty list means "nothing yet". Always show `source` and `as_of` next to a number.

The `groundwater` topic is **not** a well depth. It is a percentile (50 is normal for the time of year; 10 means only 10% of past years were this dry) from a NASA model for a square of about 25 km, so every farm in that square gets the same number, and it cannot see local pumping. It comes with `confidence: "unsure"`. Say "the wider area", never "your well". Its measure codes are `groundwater_percentile`, `root_zone_moisture_percentile` and `surface_moisture_percentile`.

**Brief.** `{"farm_id", "zone_slug", "brief"}`. A brief is `{"day", "scope", "headline_en", "headline_ku", "summary_en", "summary_ku", "points": [{"level": "info|watch|alarm", "text_en", "text_ku"}], "sources": [{"title", "url"}], "author", "generated_at"}`. It is written by an AI agent from our stored numbers and a web search. Show the `sources`, and treat it as a draft, not as checked advice.

### Messages to the Ministry (inbox)

- `POST /v1/messages`, `multipart/form-data`: `kind` (`question`, `report`, `complaint`, `request`, `other`), `text` (1 to 2,000 characters), optional `farm_id` (one of the farmer's own farms, else `404`), and 0 to 4 `photos` (JPEG or PNG, at most 4 MB each; set each part's Content-Type, and the bytes must really be that type). Send `Idempotency-Key`. Answers `201`:
  ```json
  {"message": {"id": "1", "kind": "question", "text": "...", "farm_id": "1", "state": "new",
               "photos": [{"id": "1", "url": "/v1/messages/1/photos/1", "content_type": "image/png", "size": 70}],
               "reply": null, "created_at": "..."}}
  ```
- At most 20 messages per farmer in 24 hours: `429 rate_limited` with `retry_after_s`.
- `GET /v1/messages/mine` answers `{"messages", "count", "page", "rows_per_page"}`, newest first. `state` is `new`, `read`, `replied` or `closed`. `reply` is `{"text_ku", "text_en", "replied_at"}` or null; `text_en` may be null.
- `GET /v1/messages/{id}/photos/{photo_id}` returns the picture to its sender only. Photo `url`s are paths: put the server address in front and send the token.

## 5. The Alwa market

Sellers set their own price on each listing. There is no automatic price feed: the "price at the alwa today" board only shows what Ministry staff have typed in.

- **No login:** `GET /v1/alwa/markets`, `/v1/alwa/markets/{slug}/prices`, `/prices/{crop}/history`, `GET /v1/alwa/listings` (filters `market`, `crop`, `status`; `page`, `rows_per_page`), `GET /v1/alwa/listings/{id}` (with its offers), `GET /v1/alwa/deals`.
- **With the farmer token:** `POST /v1/alwa/listings`, `GET /v1/alwa/listings/mine`, `DELETE /v1/alwa/listings/{id}` (cancel), `POST /v1/alwa/listings/{id}/offers`, `POST /v1/alwa/listings/{id}/offers/{offer_id}/accept`, `GET /v1/alwa/offers/mine`.
- **Phones are hidden** until a deal. After the seller accepts, the seller's answer shows the buyer's phone and the buyer's answer shows the seller's.
- **Rules:** at most 20 open listings per phone; a listing closes at `closes_at` (at most 14 days ahead); you cannot offer on your own listing; a buyer has one open offer per listing (a new one replaces it); accepting any offer sells the whole listing and declines the others.
- **`fair_price`** on a listing is `fair`, `high`, `low` or `unknown`. It is `unknown` unless staff have entered a price for that crop at that market in the last 7 days.
- **Codes:** `too_many_listings`, `own_listing`, `listing_not_open`, `offer_not_open`, `offer_too_large`, `bad_closes_at`.
- Crop codes: the same table, `GET /v1/crops` (16 crops seeded: wheat, barley, tomato, cucumber, potato, onion, watermelon, grape, olive, sunflower, chickpea, pomegranate, okra, eggplant, pepper, apple).
- **The simpler Alwa the app asked for in `BACKEND.md` 2.14** (a GPS point on each listing, the seller's phone shown from the start, nearest first, mark as sold, markets with a point) is **not built yet**. It is next after crops; see section 14. Until then the routes behave as written here.

## 6. Ask the Doctor

`POST /v1/farms/{id}/ask` with the farmer's token. The body is `multipart/form-data`:

| Field | Required | What the backend does with it |
|---|---|---|
| `question` | no | Text, up to 1000 characters (counted as characters, so Sorani is fine). Spaces around it are trimmed; a blank question counts as none. |
| `photos` (or `photos[]`) | no | One part per photo, 0 to 6, each JPEG or PNG and at most 4 MB. Set each part's Content-Type to `image/jpeg` or `image/png`: Flutter's `MultipartFile` sends `application/octet-stream` unless `contentType` is given, and that is refused. The bytes must really be that type. |
| `cell` | no | The cell the farmer tapped, as JSON text: `{"e": 46415, "n": 398748}`. |
| `lang` | no | `ku` (the default) or `en`. |

At least a question or one photo. `voice` and any other field are not read. The whole form may be up to about 26 MB on this route; every other route keeps 2 MB (messages with photos have their own higher limit).

Answer `200`, no outer wrapper:

```json
{"likely": "...", "confidence": "sure|likely|unsure", "why": ["weather -> ..."],
 "actions_this_week": ["..."], "cannot_tell": ["..."], "refer_to_officer": true,
 "ku": "...", "en": "...", "inputs_used": ["weather", "field_eye"]}
```

- The backend calls no AI. It checks the farm is the farmer's, sends the farm (centre of the outline, exact area, crops), what `GET /v1/farms/{id}/insights` knows about it, the question, photos, cell and `lang` to the local Doctor service, and passes its answer on after these checks: `confidence` is one of the three; `actions_this_week` holds 3 at most (extra ones are cut); a list the Doctor left out comes back empty. An answer without `likely`, `confidence`, `refer_to_officer`, `ku` or `en` is refused as `502 doctor_failed`, never filled in.
- `inputs_used` is extra to `BACKEND.md` 2.5: which sources the Doctor read.
- Not in this version: `case_id` (nothing is stored yet) and `transcript` (voice is deferred).
- The Doctor takes 10 to 25 s and the backend waits up to 90 s for it. Give this call its own answer timeout of at least 90 s.
- **On the test server the Doctor service is not running yet** (checked 2026-10-09: nothing answers on its port and the server has no address for it), so every question there answers `502 doctor_failed`. It needs the Doctor code and its AI key put on that server.

Errors, in the order they are checked:

| Status | Code | When |
|---|---|---|
| 404 | `not_found` | The id is not a number. |
| 400 | `bad_request` | The body is not a multipart form, `cell` is not the JSON above, or a text field is not UTF-8. |
| 422 | `bad_photo` | A seventh photo, a photo over 4 MB, a type other than JPEG or PNG, or bytes that are not the declared type. |
| 422 | `empty_question` | No question and no photo. |
| 422 | `invalid` | A question over 1000 characters, or `lang` other than `ku` or `en`. |
| 404 | `not_found` | The farm is another phone's or does not exist (the same answer; the Doctor is not asked). |
| 502 | `doctor_failed` | The Doctor service is down, took over 90 s, failed, or answered something that cannot be used. |
| 503 | `doctor_not_ready` | The Doctor service is up but cannot answer yet (it has no AI key). Try later. |

## 7. Data anyone can read (the View page)

No login. These are what the public View page and the app's region screens use; the Admin part may use them too.

| Call | What it answers |
|---|---|
| `GET /v1/region/overview?month=` | all 33 districts with `dryness`, `band`, `rank`, change against last year; a region summary |
| `GET /v1/zones/{slug}?month=` | one district: its reading, its sub-districts, the same month in earlier years |
| `GET /v1/region/compare?year=&with=&month=` | two years side by side per district |
| `GET /v1/dams`, `GET /v1/dams/{slug}/history` | Dukan and Darbandikhan: latest lake area and its share of the full area, the same a year ago, history since 2008 (see section 11 for what `pct_full` means) |
| `GET /v1/fires?hours=24` | fire detections and a summary |
| `GET /v1/outlooks`, `/v1/outlooks/zones/{zone_slug}` | next-season outlook per district, with the method's track record (`404` while none has been issued) |
| `GET /v1/water/plan` | districts ranked by water need (`404` while no plan has been made) |
| `GET /v1/briefs/latest?scope=`, `GET /v1/briefs` | the nightly brief (`scope` is `region` or a district slug) |
| `GET /v1/stats/farms` | farm totals: `{"as_of", "totals": {"farmers", "farms", "dunam"}, "by_governorate", "by_zone", "by_crop"}`. Counts and areas only: no names, no phones, nothing per sub-district. `404` when staff have switched `public_farm_totals` off. |
| `GET /v1/app/config` | the app settings (section 4) |
| `GET /v1/versions` | cache versions (section 10) |
| Alwa public pages | section 5 |

## 8. The website's Admin part: sign in, roles, staff

Everything is under `/v1/dashboard`.

- `POST /v1/dashboard/auth/login` with `{"email", "password"}` answers `{"token", "staff", "permissions": [{"resource", "action"}]}`. The token lasts 12 hours. A wrong email or password is `401 {"error": "bad_credentials"}`.
- `GET /v1/dashboard/me` answers the signed-in staff member and their permissions. Use it to decide which pages and buttons to show. The server checks every call anyway.
- `PUT /v1/dashboard/me` (any signed-in staff): `{"name", "phone", "current_password", "new_password"}`. The two passwords are optional and go together; a wrong current password is `403 {"error": "wrong_password"}`. It never changes roles, email or the active state.
- `GET /v1/dashboard/permissions` lists the resources and actions a role can hold.
- **A permission is `<resource>:<action>`.** The 17 resources: `zones`, `dams`, `outlooks`, `water`, `fires`, `alwa`, `farmers`, `farms`, `insights`, `briefs`, `staff`, `roles`, `crops`, `rules`, `messages`, `app`, `jobs`. Actions: `create`, `read`, `update`, `delete`. So `GET` needs `read`, `POST` needs `create`, `PUT` needs `update`, `DELETE` needs `delete`, each on its own.
- **Roles are made by staff.** `/v1/dashboard/roles` and `/v1/dashboard/staff`: list, create, read, update, delete. A role is a name and a set of permissions; a staff member holds one or more roles, and has an optional `phone` and `job_title`.
- The `Owner` role holds everything and cannot be changed or deleted.
- A change to a role, or deactivating someone, takes effect on that person's next request.
- Passwords are at least 10 characters. A staff member's email cannot be changed through `PUT /v1/dashboard/staff/{id}`: create a new account and delete the old one.
- **Codes:** `bad_credentials`, `forbidden` (403), `wrong_password` (403), `system_role`, `role_in_use`, `role_name_taken`, `email_taken`, `unknown_role`, `own_account`, `last_owner`, and `cannot_grant` (403: you tried to give a permission, a role or a password reset beyond what you hold yourself).
- **Getting an account:** there is no sign-up. Staff with `staff:create` make accounts; the first one is made on the server.

## 9. The website's Admin part: data

Every list, create, change and delete goes through these. Each method needs its own permission.

The same rules everywhere:

- `POST` creates. If the thing already exists: `409 {"error": "already_exists"}`, nothing changes.
- `PUT` changes an existing thing and is a full replace unless said otherwise. If there is none: `404`.
- `DELETE` answers `204`, also when it was already gone.
- A number changed by hand looks like any other, and the next data-job run for the same district and month replaces it.
- **Phone numbers:** the `farmers`, `farms`, `messages` and `alwa` listing routes show full phone numbers to staff who hold the permission. The "protected mode" of `BACKEND.md` 2.11 (masked phones, farms shown at 1 km) is not built; give those permissions only to people who should see phones.

### Region data

| Resource | Routes under `/v1/dashboard` |
|---|---|
| `zones` | `/zones` (districts with sub-districts), `/zones/{slug}/readings[/{month}]`, `/zones/{slug}/sub-zones/{sub_slug}/readings[/{month}]` |
| `dams` | `/dams`, `/dams/{slug}/readings[/{day}]` |
| `outlooks` | `/outlooks`, `/outlooks/{season}/{issued}/zones/{zone_slug}`, `/outlook-runs[/{season}/{issued}]` |
| `water` | `/water/seasons`, `/water/plan/{season}/entries[/{zone_slug}]` |
| `fires` | `/fires[/{id}]` |
| `briefs` | `/briefs[/{day}/{scope}]` (read, correct, delete; only the nightly job creates) |
| `alwa` | `/alwa/markets[/{slug}]`, `/alwa/markets/{slug}/prices[/{crop}/{day}]`, `/alwa/listings[/{id}]` (close or delete a listing) |

### Farmers (`farmers`)

- `GET /v1/dashboard/farmers`: filters `q` (name or phone), `phone`, `governorate`, `zone`, `blocked`; `sort=created_at|name`, `order=asc|desc`; paged.
- `POST /v1/dashboard/farmers` (register a farmer by phone), `GET /v1/dashboard/farmers/{id}`, `DELETE /v1/dashboard/farmers/{id}` (deletes their farms and signs them out at once).
- `PUT /v1/dashboard/farmers/{id}`: `name`, `lang`, `gender` (`male`, `female` or null), `birth_year`, `village`, `governorate`, `zone_slug`, `sub_zone_slug` (the farmer's home place), `notes` (staff only, up to 1,000 characters, never sent to the farmer app), `blocked`. A field you leave out is cleared, except `blocked`, which stays as it was.
- **Blocking:** `blocked: true` signs the farmer out at once and stops sign-in (section 4). Unblocking restores access.
- **Support letter:** `POST /v1/dashboard/farmers/{id}/letters` (`farmers:read`) with `{"purpose": 3 to 300 characters, "lang": "ku" | "en"}` answers `201` with everything the printed letter needs:
  ```json
  {"letter": {"number": "JTY-202610-1-1", "purpose": "...", "lang": "en", "created_at": "...",
              "issued_by": {"id": "3", "name": "..."},
              "farmer": {"name", "phone", "gender", "birth_year", "village", "governorate", "zone_slug", "sub_zone_slug"},
              "farms": [{"id", "name", "governorate", "zone_slug", "sub_zone_slug", "area_dunam", "crops": [{"crop", "dunam"}]}],
              "totals": {"farms": 1, "dunam": 12.06, "crops": [{"crop", "dunam"}]}}}
  ```
  The number is `JTY-<yyyymm>-<farmer id>-<n>`. **Every call issues a new number**, so call it once per printed letter. `GET /v1/dashboard/letters/{number}` (`farmers:read`) returns the stored record, to check a letter later. Letters cannot be changed or deleted.

### Farms (`farms`) and per-farm readings (`insights`)

- `GET /v1/dashboard/farms`: filters `owner_phone`, `governorate`, `zone`, `sub_zone` (the value `unknown` finds farms with no place), `crop`, `q` (farm name or owner phone); `sort=created_at|area_dunam|name`, `order=asc|desc`; paged. Rows carry the owner's phone and the place.
- `GET`, `PUT` (rename), `DELETE /v1/dashboard/farms/{id}`; `POST /v1/dashboard/farms` (a farm for a farmer).
- `GET /v1/dashboard/stats/farms?governorate=&zone=&crop=` (`farms:read`), the totals for reports and charts:
  ```json
  {"as_of": "...", "totals": {"farmers": 1, "farms": 1, "dunam": 12.06},
   "by_governorate": [{"slug", "name_en", "name_ku", "farmers", "farms", "dunam", "crops": [{"crop", "dunam", "farms"}]}],
   "by_zone": [...], "by_sub_zone": [...],
   "by_crop": [{"crop", "dunam", "farms", "farmers"}]}
  ```
  Only areas that have farms are listed. Farms with no place are in the totals and in a row with slug `unknown`. With `crop`, only farms growing it are counted and `dunam` stays the whole area of those farms. A farmer is counted once per area and once in the totals.
- `/v1/dashboard/farms/{id}/insights[/{topic}]` (`insights`): see and correct the stored readings of any farm.
- **Check the map:** the shape named `Qaradagh` in `web/map_demo/kri_map_data.js` reaches north to latitude 35.57 and contains the centre of Sulaymaniyah city, and the Sulaymaniyah district has no centre sub-district of its own. So a farm in the city is reported as `qaradagh`. The backend follows the map file exactly; if the shape or its name is wrong, fix it there and we reseed.

### Crops (`crops`)

- `GET /v1/dashboard/crops` (also the switched-off ones), `POST /v1/dashboard/crops`, `PUT` and `DELETE /v1/dashboard/crops/{code}`.
- Body: `{"code", "name_en", "name_ku", "color": "#rrggbb", "category": "cereal|vegetable|fruit|legume|oil|fodder|other", "season": "winter|summer|perennial", "yield_kg_per_dunam", "active", "sort_order"}`. `code` matches `^[a-z_]{2,24}$` and never changes; `empty` is refused (`422 reserved_code`).
- Deleting a crop that a farm cell, an Alwa listing or an Alwa price uses answers `409 {"error": "crop_in_use"}`: offer to switch it off (`active: false`) instead.
- Five seeded crops (pomegranate, okra, eggplant, pepper, apple) have no Sorani name yet, and no crop has a yield: the repo holds none, and we do not invent them. Staff can fill both in here.

### Inbox (`messages`)

- `GET /v1/dashboard/messages?state=&kind=&governorate=&zone=&q=&page=&rows_per_page=`, newest first. A row has the message, the farmer's name and phone, and the farm's name, governorate and district. `q` searches the text, the farmer's name and phone. The place filters match messages that name a farm.
- `GET /v1/dashboard/messages/counts` answers `{"new", "read", "replied", "closed"}` (for the badge).
- `GET /v1/dashboard/messages/{id}`; `GET /v1/dashboard/messages/{id}/photos/{photo_id}` returns the picture (send the token; it cannot be used as a plain `<img src>` without it).
- `PUT /v1/dashboard/messages/{id}` with `{"state": "new" | "read" | "replied" | "closed"}`. Setting `replied` by hand without a reply is `422 no_reply`.
- `POST /v1/dashboard/messages/{id}/reply` with `{"text_ku", "text_en"}` (`text_en` optional): the state becomes `replied`. A second reply replaces the first; a closed message can still be replied to.
- `DELETE /v1/dashboard/messages/{id}` (`messages:delete`) removes the message and its photos.

### Rules (`rules`)

- `GET /v1/dashboard/rules` answers `{"rules": [{"code", "grp", "name_en", "name_ku", "meaning_en", "meaning_ku", "value", "unit", "min_value", "max_value", "default_value", "used_by", "updated_by", "updated_at"}]}`.
- `PUT /v1/dashboard/rules/{code}` with `{"value", "reason"}` answers `{"rule", "changed"}`. Outside min and max: `422 bad_range`. A reason outside 3 to 500 characters: `422 bad_reason`.
- `GET /v1/dashboard/rules/{code}/history`, paged, newest first: `{"changes": [{"id", "code", "old_value", "new_value", "reason", "staff_id", "at"}], "count", "page", "rows_per_page"}`. The history cannot be edited or deleted.
- `POST /v1/dashboard/rules/{code}/reset` with `{"reason"}`: back to `default_value`, also logged.
- There is no create and no delete: a rule exists because code reads it.
- 15 rules are seeded with the numbers in the code today: nine weather planner rules (frost 0, hard frost -2, heat 31, heavy rain 12 mm, sowing rain 20 mm, rust weather 24 h, spray window 6 h, sunn pest 84 degree-days, dust PM10 150), four dryness band edges (25, 45, 60, 80), and two Field Eye rules.
- **Field Eye differs from the list in `BACKEND.md`.** `farm_doctor/field_eye.py` has no "watch below 85%" or "alarm below 70% of normal", and its cloud limit is 60%, not 30%. What it has was seeded: `field_eye_max_cloud_pct` 60 and `field_eye_weak_pixel_pct` 70. Tell us if the 85 / 70 / 30 numbers live somewhere else.
- Sorani names and meanings of the rules are null: they need a native speaker. Show the English until then.
- **Honest limit: changing a rule changes nothing yet.** The weather planner, the dryness bands and Field Eye still use their built-in numbers. Say so on the Rules page, or hide the save button, until this line is removed.

### App control (`app`)

- `GET` and `PUT /v1/dashboard/app/config` (`app:read`, `app:update`). `PUT` replaces everything; send back the whole object of section 4 without `updated_at` and `updated_by`. `min_version` may not be above `latest_version`. Maintenance and the announcement need their Sorani text while switched on (`422 missing_text`). Versions are `major.minor.patch`.
- `GET /v1/dashboard/app/versions` (`app:read`) answers `{"versions": [{"version", "farmers", "share"}]}` for the last 30 days, each farmer counted under the version last seen. Fetch it fresh when the page opens; it belongs to no cache topic.

### Data jobs (`jobs`, read only)

- `GET /v1/dashboard/jobs` answers `{"jobs": [{"job", "name_en", "name_ku", "every_hours", "last_run": {"started_at", "finished_at", "ok", "rows", "message"} or null, "last_ok", "next_due", "state", "last_14_days": [14 entries, oldest first, "ok" | "late" | "failed" | null], "message"}]}`.
- `state`: `never` (no run), `failed` (the run that finished last was not ok), `late` (nothing finished ok within 1.5 times `every_hours`), else `ok`.
- A job becomes `late` by time alone, which raises no cache topic: refetch this page on a timer.
- Jobs: `dryness` (12 h), `fires` (3 h), `groundwater` (24 h), `briefs` (24 h) report every run. `dams` (24 h) reports too. `farm_analysis` has no job on the server yet and shows `never`.

## 10. Caching (for the website)

- `GET /v1/versions` (no login) answers `{"api": "1.6.0", "versions": {"zones": 41, "dams": 7, ...}, "server_time"}` for the public topics: `zones, sub_zones, dams, fires, outlooks, water, alwa_prices, alwa_listings, crops, rules, briefs, app_config`.
- `GET /v1/dashboard/versions` (any signed-in staff) adds the private ones: `farmers, farms, messages, jobs, staff_roles`.
- A topic's number goes up whenever anything of that kind is written, by anyone (website, data job, farmer app). The database does it itself inside the write, so it cannot be forgotten. It may go up by more than one for a single action: only compare "is it higher than what I have".
- Topic to routes: `zones` district readings and `/v1/region...`; `sub_zones` sub-district readings; `dams`; `fires`; `outlooks` (outlooks and outlook runs); `water`; `alwa_prices` (markets and prices); `alwa_listings` (listings, offers, deals); `briefs`; `rules`; `app_config` (the settings, not the versions-in-use list); `farmers` (farmers and letters); `farms` (farms, cells, per-farm readings, and both totals routes); `messages`; `jobs`; `staff_roles` (staff, roles and their permissions). `crops` is the crops table.
- **`ETag` and `304` on every JSON `GET`.** Send the tag back in `If-None-Match`; if the answer would be the same you get `304` with no body. Treat the tag as opaque. `Cache-Control` is `no-cache` (`private, no-cache` when a token was sent).
- Wipe the cache when `X-Api-Version` changes.

## 11. What has real data on the test server today

Be honest on screen about this.

| Data | State | Where it comes from |
|---|---|---|
| District rain and `dryness` | **live**, all 33 districts, refreshed every 12 hours | rain of the last 365 days against the 10 years before (Open-Meteo, ERA5). `dryness` is only that rain figure on a 0 to 100 scale (50 = normal rain, lower = wetter). It is **not** soil moisture or crop condition: label it "rain against normal". `greenness`, `water_need`, `best_crops` are empty. |
| Fires | **live**, refreshed every 3 hours | NASA satellite detections inside the 33 districts, gas flares removed by a rule. About 3 hours behind the satellite. Nobody has checked the list by hand: call them "satellite fire detections", not confirmed fires. `area_ha`, wind and `farmers_alerted` are empty. |
| Nightly brief | **live** for the region and for districts that have farms | an AI agent reads our stored numbers and searches the web; show its sources |
| Groundwater per farm | **live**, daily | section 4; the wider area, not a well |
| Farmers and farms | the few test accounts people have made | the app |
| Dams | **live**: 116 readings from 2008 to now, refreshed daily | lake area measured from Sentinel-2 and Landsat with the team's tested method. **`pct_full` is the lake's AREA as a share of its full area (Dukan 270 km2, Darbandikhan 113 km2), not stored volume**: a lake loses volume faster than area, so label it "lake area, % of full". `volume_bn_m3` and `farm_supply_bn_m3` are null: no trustworthy area-to-volume curve exists for these dams. |
| Season outlook, water plan | **empty** (`404`) | no job; they can be typed in through the Admin part |
| Alwa prices | **empty** until staff type them in | by hand |
| Rules | 15 seeded rules | section 9; changing them has no effect yet |
| App settings | the starting values | section 4 |
| Messages | empty until a farmer sends one | the app |
| Farm status from space | placeholder | no store yet |
| Per-farm history (ten years of rain, heat, greenness) | not on the server yet | being built; section 14 |

## 12. Things that will trip you up

- **Unknown values in a body** (a wrong `status`, `category` or `kind`) answer `400 bad_request`, not `422`. An unknown crop code is `422 unknown_crop`.
- **An empty `page=`** in a query answers `400`. Leave the parameter out instead.
- **A `+` in a query string** must be sent as `%2B` (for example a phone filter).
- **`Ask the Doctor` needs a 90 second timeout** and each photo part's own content type; see section 6.
- **Photos behind a token** (inbox pictures) cannot be shown with a plain image address in a browser: fetch them with the token and show the bytes.
- **`BACKEND.md`'s create example has a wrong cell number** (`e` is ten times too large there). Use `e = floor(easting / 10)` in UTM zone 38N, as `geo.dart` already does.
- **An outline that only touches itself at a point** is refused by the backend (`bad_polygon`).
- **`created_offline_at` in an edit** is ignored: the farm keeps the one it was created with.
- **The backend does not round.** `inside_pct`, `area_dunam` and `dunam` come with all their decimals; round when you show them.
- **A support letter is numbered on every call.** Do not call it to preview.
- **The staff login path is `/v1/dashboard/auth/login`.** There is no `/v1/auth/login`.

## 13. Error codes

| Status | Codes |
|---|---|
| 400 | `bad_request` (the body or a parameter cannot be read) |
| 401 | `unauthorized`, `bad_code`, `bad_credentials` |
| 403 | `forbidden`, `cannot_grant`, `blocked` (a blocked farmer), `wrong_password` |
| 404 | `not_found` |
| 409 | `already_exists`, `crop_in_use`, `listing_not_open`, `offer_not_open`, `listing_has_deal`, `market_in_use`, `system_role`, `role_in_use`, `role_name_taken`, `email_taken`, `own_account`, `last_owner` |
| 422 | `invalid` (with `field` when one field is at fault), `bad_polygon`, `farm_too_large`, `too_many_farms`, `too_many_listings`, `own_listing`, `offer_too_large`, `bad_closes_at`, `bad_month`, `bad_range`, `bad_reason`, `empty_question`, `bad_photo`, `no_reply`, `missing_text`, `unknown_crop`, `reserved_code`, `unknown_role`, and other `bad_...` codes that name the field |
| 426 | `update_required` (the app is older than `min_version`) |
| 429 | `rate_limited` (with `retry_after_s`) |
| 500 | `server_error` (the detail is always "An unexpected error occurred") |
| 502 | `doctor_failed` |
| 503 | `upstream_down` (a sign-in code could not be sent), `doctor_not_ready` |

## 14. Not built yet

These answer `404` today. Build the screens so that a `404` or an empty answer shows a calm "coming soon", and tell us in `BACKEND.md` which you need first.

| Thing | State |
|---|---|
| Per-farm history: ten years of monthly rain, heat, evaporation, soil moisture and greenness (`GET /v1/farms/{id}/history`) | being built now |
| The simpler Alwa of `BACKEND.md` 2.14 (GPS point, phones shown, nearest first, mark as sold) | next |
| `GET /v1/farms/{id}/plan`, the 10-day weather plan | not started |
| Real data in `GET /v1/farms/{id}/status` | not started; needs a satellite job and a store |
| `DELETE /v1/account`, alerts, `POST /v1/devices` (push), reports | not started |
| Protected mode for phones and farm positions (`BACKEND.md` 2.11) | not started; the website team said it is not needed for now |
| Rules read by the jobs; limits and feature switches enforced by the server | not started |

## 15. Open questions for the frontend

1. **Farm status:** the app reads `/status`, but the per-farm analysis writes to `/insights`. Should `/status` be built from those, or should the app drop `/status`?
2. **The website's addresses** for CORS (section 3): which origins should the test server allow?
3. **The map:** is the `Qaradagh` shape meant to contain Sulaymaniyah city (section 9)?
4. **Field Eye numbers:** where do 85%, 70% and 30% cloud come from (section 9)?
5. **The per-farm analysis job** is not in the repo. It should be, so it can run on the test server.
6. Is anything you need missing from `backend/API.md`? Write it in `BACKEND.md`.
