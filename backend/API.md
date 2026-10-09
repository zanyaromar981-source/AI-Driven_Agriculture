# API reference

Written by `tools/api_reference.py` from the server's own description of itself. Do not edit by hand; run the tool again after a route changes. `FRONTEND.md` at the repo root explains how to use all this; the live, clickable version is at `/api-docs` on any running server.

127 operations. A `?` after a field name means it may be left out. Query parameters are listed with the route in `/api-docs`.

Access: **none** = no login; **farmer token** = `Authorization: Bearer <token>` from `POST /v1/auth/otp/verify`; **staff** = a token from `POST /v1/dashboard/auth/login` whose roles hold the named permission; **service key** = the `X-Service-Key` header, for our data jobs only.

## Sign in and profile (farmer app)

### `POST /v1/auth/otp/send`

Send a sign-in code to a phone.

- Access: none
- Body: `lang?`: `ku` \| `kmr` \| `ar` \| `en`, `phone`: text
- Answers: **200** `retry_after_s`: number, `sent`: true/false
- Can fail with: 400, 422, 429, 500

### `POST /v1/auth/otp/verify`

Exchange a sign-in code for a token.

- Access: none
- Body: `code`: text, `phone`: text
- Answers: **200** `farms_count`: number, `token`: text
- Can fail with: 400, 401, 422, 500

### `GET /v1/me`

Get the authenticated farmer's profile.

- Access: farmer token
- Answers: **200** `created_at`: timestamp, `lang`: `ku` \| `kmr` \| `ar` \| `en`, `name?`: text or null, `phone`: text
- Can fail with: 401, 404, 500

### `PUT /v1/me`

Update the authenticated farmer's profile.

- Access: farmer token
- Body: `lang`: `ku` \| `kmr` \| `ar` \| `en`, `name?`: text or null
- Answers: **200** `created_at`: timestamp, `lang`: `ku` \| `kmr` \| `ar` \| `en`, `name?`: text or null, `phone`: text
- Can fail with: 400, 401, 404, 422, 500

## Farms (farmer app)

### `GET /v1/farms`

List all farms for the authenticated user.

- Access: farmer token
- Answers: **200** `farms`: list of FarmSummaryResponse
- Can fail with: 401, 500

### `POST /v1/farms`

Create a new farm from a walked outline and its painted cells.

- Access: farmer token
- Body: `cells?`: list of CellParams, `created_offline_at?`: timestamp or null, `name`: text, `points`: list of PointParams
- Answers: **201** `dropped_cells`: list of GridCellResponse, `farm`: FarmResponse
- Can fail with: 400, 401, 422, 500

### `GET /v1/farms/{id}`

Get one farm with its outline and cells.

- Access: farmer token
- Answers: **200** `farm`: FarmResponse
- Can fail with: 401, 404, 500

### `PUT /v1/farms/{id}`

Edit a farm: replace its outline, its cells and its name.

- Access: farmer token
- Body: `cells?`: list of CellParams, `created_offline_at?`: timestamp or null, `name`: text, `points`: list of PointParams
- Answers: **200** `dropped_cells`: list of GridCellResponse, `farm`: FarmResponse
- Can fail with: 400, 401, 404, 422, 500

### `DELETE /v1/farms/{id}`

Delete a specific farm.

- Access: farmer token
- Answers: **204**
- Can fail with: 401, 404, 500

### `POST /v1/farms/{id}/ask`

Ask the Doctor about one of the farmer's farms.

- Access: farmer token
- Body: (multipart/form-data) `cell?`: text or null, `lang?`: text or null, `photos?`: list of text, `question?`: text or null
- Answers: **200** `actions_this_week`: list of text, `cannot_tell`: list of text, `confidence`: `sure` \| `likely` \| `unsure`, `en`: text, `inputs_used`: list of text, `ku`: text, `likely`: text, `refer_to_officer`: true/false, `why`: list of text
- Can fail with: 400, 401, 404, 422, 500, 502, 503

### `GET /v1/farms/{id}/brief`

Get the brief for one of the farmer's own farms.

- Access: farmer token
- Answers: **200** `brief?`: BriefResponse or null, `farm_id`: text, `zone_slug?`: text or null
- Can fail with: 401, 404, 500

### `PUT /v1/farms/{id}/cells`

Repaint the crops on a farm's cells.

- Access: farmer token
- Body: `cells`: list of CellParams
- Answers: **200** `dropped_cells`: list of GridCellResponse, `farm`: FarmResponse
- Can fail with: 400, 401, 404, 422, 500

### `GET /v1/farms/{id}/insights`

Get the current readings of one of the farmer's own farms.

- Access: farmer token
- Answers: **200** `farm_id`: text, `topics`: list of TopicInsightResponse
- Can fail with: 401, 404, 500

### `GET /v1/farms/{id}/status`

Get a farm's status from space.

- Access: farmer token
- Answers: **200** `cells`: list of CellStatusResponse, `crops`: list of CropStatusResponse, `greenness_pct_of_normal?`: number or null, `next_picture_expected?`: day or null, `picture_date?`: day or null, `weak_where?`: text or null
- Can fail with: 401, 404, 500

## Alwa market: farmers and buyers

### `GET /v1/alwa/deals`

List the deals made on one day.

- Access: none
- Query: `market?`, `day?`
- Answers: **200** `day`: day, `deals`: list of AlwaDealResponse, `summary`: AlwaDealsSummaryResponse
- Can fail with: 404, 422, 500

### `GET /v1/alwa/listings`

Browse the listings on sale.

- Access: none
- Query: `market?`, `crop?`, `status?`, `page?`, `rows_per_page?`
- Answers: **200** `count`: number, `listings`: list of AlwaListingSummaryResponse, `page`: number, `rows_per_page`: number
- Can fail with: 404, 422, 500

### `POST /v1/alwa/listings`

Put a crop on sale.

- Access: farmer token
- Body: `asking_price_iqd_per_kg`: number, `closes_at`: timestamp, `crop`: `wheat` \| `barley` \| `tomato` \| `cucumber` \| `potato` \| `onion` \| `watermelon` \| `grape` \| `olive` \| `sunflower` \| `chickpea` \| `pomegranate` \| `okra` \| `eggplant` \| `pepper` \| `apple`, `grade?`: `a` \| `b` \| `c` or null, `market`: text, `note?`: text or null, `pickup`: `farm` \| `alwa`, `quantity_kg`: number, `seller_name?`: text or null, `zone_slug?`: text or null
- Answers: **201** `listing`: AlwaListingResponse
- Can fail with: 400, 401, 404, 422, 500

### `GET /v1/alwa/listings/mine`

List the authenticated user's own listings with their offers.

- Access: farmer token
- Answers: **200** `listings`: list of AlwaListingResponse
- Can fail with: 401, 500

### `GET /v1/alwa/listings/{id}`

Get one listing with its offers.

- Access: none
- Answers: **200** `listing`: AlwaListingResponse
- Can fail with: 404, 500

### `DELETE /v1/alwa/listings/{id}`

Cancel an open listing of the authenticated user.

- Access: farmer token
- Answers: **204**
- Can fail with: 401, 404, 409, 500

### `POST /v1/alwa/listings/{id}/offers`

Make an offer on a listing.

- Access: farmer token
- Body: `buyer_kind`: `shop` \| `restaurant` \| `trader` \| `other`, `buyer_name`: text, `price_iqd_per_kg`: number, `quantity_kg`: number
- Answers: **201** `offer`: AlwaOfferResponse
- Can fail with: 400, 401, 404, 409, 422, 500

### `POST /v1/alwa/listings/{id}/offers/{offer_id}/accept`

Accept an offer on a listing of the authenticated user.

- Access: farmer token
- Answers: **200** `listing`: AlwaListingResponse
- Can fail with: 401, 404, 409, 500

### `GET /v1/alwa/markets`

List the alwa markets.

- Access: none
- Answers: **200** `markets`: list of AlwaMarketResponse
- Can fail with: 500

### `GET /v1/alwa/markets/{slug}/prices`

Get the prices of every crop at one market on one day.

- Access: none
- Query: `day?`
- Answers: **200** `day?`: day or null, `market`: text, `prices`: list of AlwaPriceResponse
- Can fail with: 404, 422, 500

### `GET /v1/alwa/markets/{slug}/prices/{crop}/history`

Get the price history of one crop at one market.

- Access: none
- Query: `days?`
- Answers: **200** `crop`: `wheat` \| `barley` \| `tomato` \| `cucumber` \| `potato` \| `onion` \| `watermelon` \| `grape` \| `olive` \| `sunflower` \| `chickpea` \| `pomegranate` \| `okra` \| `eggplant` \| `pepper` \| `apple`, `history`: list of AlwaPricePointResponse, `market`: text
- Can fail with: 404, 422, 500

### `GET /v1/alwa/offers/mine`

List the authenticated user's own offers.

- Access: farmer token
- Answers: **200** `offers`: list of AlwaMyOfferResponse
- Can fail with: 401, 500

## Region data, no login

### `GET /v1/briefs`

List the briefs of the region or of one zone, newest day first.

- Access: none
- Query: `scope?`, `from?`, `to?`
- Answers: **200** `briefs`: list of BriefResponse
- Can fail with: 422, 500

### `GET /v1/briefs/latest`

Get the newest brief of the region or of one zone.

- Access: none
- Query: `scope?`
- Answers: **200** `brief`: BriefResponse
- Can fail with: 404, 422, 500

### `GET /v1/dams`

List the dams with their latest reading and the one from a year before.

- Access: none
- Answers: **200** `dams`: list of DamResponse
- Can fail with: 500

### `GET /v1/dams/{slug}/history`

Get one dam's readings over time, oldest first.

- Access: none
- Query: `from?`, `to?`
- Answers: **200** `readings`: list of HistoryReadingResponse, `slug`: text
- Can fail with: 404, 422, 500

### `GET /v1/fires`

List the fires detected in the last hours, with their totals.

- Access: none
- Query: `hours?`
- Answers: **200** `fires`: list of FireResponse, `hours`: number, `summary`: FireSummaryResponse
- Can fail with: 400, 422, 500

### `GET /v1/outlooks`

Get the outlook for the next growing season, zone by zone.

- Access: none
- Query: `season?`, `issued?`
- Answers: **200** `counts`: OutlookCountsResponse, `issued`: text, `issues`: list of text, `season`: text, `track_record?`: TrackRecordResponse or null, `zones`: list of ZoneOutlookResponse
- Can fail with: 404, 422, 500

### `GET /v1/outlooks/zones/{zone_slug}`

Get one zone's outlook at every issue of a season, oldest first.

- Access: none
- Query: `season?`
- Answers: **200** `issues`: list of ZoneIssueResponse, `season`: text, `zone_slug`: text
- Can fail with: 404, 422, 500

### `GET /v1/region/compare`

One calendar month compared between two years.

- Access: none
- Query: `year`, `with`, `month`
- Answers: **200** `month`: number, `region`: list of YearAverageResponse, `with`: number, `year`: number, `zones`: list of ZoneComparisonResponse
- Can fail with: 422, 500

### `GET /v1/region/overview`

The whole region for one month: every zone and a summary.

- Access: none
- Query: `month?`
- Answers: **200** `month`: text, `summary`: RegionSummaryResponse, `zones`: list of ZoneOverviewResponse
- Can fail with: 422, 500

### `GET /v1/water/plan`

Get the plan for which zones should receive water first.

- Access: none
- Query: `season?`
- Answers: **200** `entries`: list of RankedEntryResponse, `season`: text, `totals`: PlanTotalsResponse
- Can fail with: 404, 422, 500

### `GET /v1/zones/{slug}`

One zone for one month, with its sub-zones and its history.

- Access: none
- Query: `month?`
- Answers: **200** `governorate`: text, `history`: list of YearDrynessResponse, `month`: text, `name_en`: text, `name_ku`: text, `reading?`: ZoneDetailReadingResponse or null, `slug`: text, `sub_zones`: list of SubZoneDrynessResponse
- Can fail with: 404, 422, 500

## Dashboard: sign in, staff and roles

### `POST /v1/dashboard/auth/login`

Sign in to the dashboard with an email and a password.

- Access: none
- Body: `email`: text, `password`: text
- Answers: **200** `permissions`: list of StaffPermission, `staff`: StaffResponse, `token`: text
- Can fail with: 400, 401, 500

### `GET /v1/dashboard/me`

Get the signed-in staff member and what they may do.

- Access: any staff
- Answers: **200** `permissions`: list of StaffPermission, `staff`: StaffResponse
- Can fail with: 401, 500

### `GET /v1/dashboard/permissions`

List the resources and actions a role can be built from.

- Access: any staff
- Answers: **200** `actions`: list of `create` \| `read` \| `update` \| `delete`, `resources`: list of `zones` \| `dams` \| `outlooks` \| `water` \| `fires` \| `alwa` \| `farmers` \| `farms` \| `insights` \| `staff` \| `roles` \| `briefs`
- Can fail with: 401

### `GET /v1/dashboard/roles`

List all roles.

- Access: staff `roles:read`
- Answers: **200** `roles`: list of StaffRoleResponse
- Can fail with: 401, 403, 500

### `POST /v1/dashboard/roles`

Create a custom role.

- Access: staff `roles:create`
- Body: `description?`: text or null, `name`: text, `permissions`: list of StaffPermission
- Answers: **201** `role`: StaffRoleResponse
- Can fail with: 400, 401, 403, 409, 422, 500

### `GET /v1/dashboard/roles/{id}`

Get one role.

- Access: staff `roles:read`
- Answers: **200** `role`: StaffRoleResponse
- Can fail with: 401, 403, 404, 500

### `PUT /v1/dashboard/roles/{id}`

Replace a role's name, description and whole permission set.

- Access: staff `roles:update`
- Body: `description?`: text or null, `name`: text, `permissions`: list of StaffPermission
- Answers: **200** `role`: StaffRoleResponse
- Can fail with: 400, 401, 403, 404, 409, 422, 500

### `DELETE /v1/dashboard/roles/{id}`

Delete a role nobody holds.

- Access: staff `roles:delete`
- Answers: **204**
- Can fail with: 401, 403, 404, 409, 500

### `GET /v1/dashboard/staff`

List all staff members.

- Access: staff `staff:read`
- Answers: **200** `staff`: list of StaffResponse
- Can fail with: 401, 403, 500

### `POST /v1/dashboard/staff`

Add a staff member.

- Access: staff `staff:create`
- Body: `email`: text, `name`: text, `password`: text, `role_ids?`: list of text
- Answers: **201** `staff`: StaffResponse
- Can fail with: 400, 401, 403, 409, 422, 500

### `GET /v1/dashboard/staff/{id}`

Get one staff member.

- Access: staff `staff:read`
- Answers: **200** `staff`: StaffResponse
- Can fail with: 401, 403, 404, 500

### `PUT /v1/dashboard/staff/{id}`

Change a staff member's name, state, roles and, optionally, password.

- Access: staff `staff:update`
- Body: `active`: true/false, `name`: text, `password?`: text or null, `role_ids`: list of text
- Answers: **200** `staff`: StaffResponse
- Can fail with: 400, 401, 403, 404, 409, 422, 500

### `DELETE /v1/dashboard/staff/{id}`

Delete a staff member.

- Access: staff `staff:delete`
- Answers: **204**
- Can fail with: 401, 403, 404, 409, 500

## Dashboard: data

### `GET /v1/dashboard/alwa/listings`

List every seller's listings, of every status, for moderation.

- Access: staff `alwa:read`
- Query: `market?`, `crop?`, `status?`, `seller_phone?`, `page?`, `rows_per_page?`
- Answers: **200** `count`: number, `listings`: list of AlwaModeratedListingResponse, `page`: number, `rows_per_page`: number
- Can fail with: 401, 403, 404, 422, 500

### `GET /v1/dashboard/alwa/listings/{id}`

Get one listing with all its offers and both sides' phones.

- Access: staff `alwa:read`
- Answers: **200** `listing`: AlwaModeratedListingDetailResponse
- Can fail with: 401, 403, 404, 500

### `PUT /v1/dashboard/alwa/listings/{id}`

Close an open listing and decline its open offers.

- Access: staff `alwa:update`
- Body: `note?`: text or null, `status`: `open` \| `sold` \| `closed` \| `cancelled`
- Answers: **200** `listing`: AlwaModeratedListingDetailResponse
- Can fail with: 400, 401, 403, 404, 409, 422, 500

### `DELETE /v1/dashboard/alwa/listings/{id}`

Remove a listing and its offers, unless it was sold.

- Access: staff `alwa:delete`
- Answers: **204**
- Can fail with: 401, 403, 409, 500

### `GET /v1/dashboard/alwa/markets`

List every alwa market.

- Access: staff `alwa:read`
- Answers: **200** `markets`: list of AlwaMarketResponse
- Can fail with: 401, 403, 500

### `POST /v1/dashboard/alwa/markets`

Add an alwa market.

- Access: staff `alwa:create`
- Body: `name_en`: text, `name_ku`: text, `slug`: text
- Answers: **201** `market`: AlwaMarketResponse
- Can fail with: 400, 401, 403, 409, 422, 500

### `PUT /v1/dashboard/alwa/markets/{slug}`

Rename an alwa market.

- Access: staff `alwa:update`
- Body: `name_en`: text, `name_ku`: text
- Answers: **200** `market`: AlwaMarketResponse
- Can fail with: 400, 401, 403, 404, 422, 500

### `DELETE /v1/dashboard/alwa/markets/{slug}`

Remove an alwa market that has no prices and no listings.

- Access: staff `alwa:delete`
- Answers: **204**
- Can fail with: 401, 403, 409, 422, 500

### `GET /v1/dashboard/alwa/markets/{slug}/prices`

List the prices stored for one market.

- Access: staff `alwa:read`
- Query: `crop?`, `from?`, `to?`, `page?`, `rows_per_page?`
- Answers: **200** `count`: number, `page`: number, `prices`: list of AlwaRecordedPriceResponse, `rows_per_page`: number
- Can fail with: 401, 403, 404, 422, 500

### `POST /v1/dashboard/alwa/markets/{slug}/prices`

Enter the price of one crop at one market on one day.

- Access: staff `alwa:create`
- Body: `crop`: text, `day`: text, `fixed?`: true/false, `price_iqd_per_kg`: number, `source`: text
- Answers: **201** `price`: AlwaRecordedPriceResponse
- Can fail with: 400, 401, 403, 404, 409, 422, 500

### `PUT /v1/dashboard/alwa/markets/{slug}/prices/{crop}/{day}`

Correct the price of one crop at one market on one day.

- Access: staff `alwa:update`
- Body: `fixed?`: true/false, `price_iqd_per_kg`: number, `source`: text
- Answers: **200** `price`: AlwaRecordedPriceResponse
- Can fail with: 400, 401, 403, 404, 422, 500

### `DELETE /v1/dashboard/alwa/markets/{slug}/prices/{crop}/{day}`

Remove the price of one crop at one market on one day.

- Access: staff `alwa:delete`
- Answers: **204**
- Can fail with: 401, 403, 422, 500

### `GET /v1/dashboard/briefs`

List the stored briefs of every scope, newest day first.

- Access: staff `briefs:read`
- Query: `scope?`, `from?`, `to?`, `page?`, `rows_per_page?`
- Answers: **200** `briefs`: list of BriefResponse, `count`: number, `page`: number, `rows_per_page`: number
- Can fail with: 401, 403, 422, 500

### `PUT /v1/dashboard/briefs/{day}/{scope}`

Replace the brief stored for one day and scope.

- Access: staff `briefs:update`
- Body: `author`: text, `generated_at`: timestamp, `headline_en`: text, `headline_ku`: text, `points`: list of BriefPointParams, `sources`: list of BriefSourceParams, `summary_en`: text, `summary_ku`: text
- Answers: **200** `brief`: BriefResponse
- Can fail with: 400, 401, 403, 404, 422, 500

### `DELETE /v1/dashboard/briefs/{day}/{scope}`

Remove the brief stored for one day and scope.

- Access: staff `briefs:delete`
- Answers: **204**
- Can fail with: 401, 403, 422, 500

### `GET /v1/dashboard/dams`

List the dams as reference data, for the dashboard's editing screens.

- Access: staff `dams:read`
- Answers: **200** `dams`: list of DamReferenceResponse
- Can fail with: 401, 403, 500

### `GET /v1/dashboard/dams/{slug}/readings`

List one dam's stored readings, newest day first.

- Access: staff `dams:read`
- Query: `from?`, `to?`, `page?`, `rows_per_page?`
- Answers: **200** `count`: number, `page`: number, `readings`: list of DamReadingResponse, `rows_per_page`: number
- Can fail with: 401, 403, 404, 422, 500

### `POST /v1/dashboard/dams/{slug}/readings`

Add a reading for a day the dam has none for.

- Access: staff `dams:create`
- Body: `day`: text, `farm_supply_bn_m3?`: number or null, `lake_area_km2?`: number or null, `pct_full`: number, `source`: text, `volume_bn_m3?`: number or null
- Answers: **201** `reading`: DamReadingResponse, `slug`: text
- Can fail with: 400, 401, 403, 404, 409, 422, 500

### `PUT /v1/dashboard/dams/{slug}/readings/{day}`

Replace the stored reading of one dam and day.

- Access: staff `dams:update`
- Body: `farm_supply_bn_m3?`: number or null, `lake_area_km2?`: number or null, `pct_full`: number, `source`: text, `volume_bn_m3?`: number or null
- Answers: **200** `reading`: DamReadingResponse, `slug`: text
- Can fail with: 400, 401, 403, 404, 422, 500

### `DELETE /v1/dashboard/dams/{slug}/readings/{day}`

Remove the stored reading of one dam and day.

- Access: staff `dams:delete`
- Answers: **204**
- Can fail with: 401, 403, 404, 422, 500

### `GET /v1/dashboard/farmers`

List farmers, newest first.

- Access: staff `farmers:read`
- Query: `phone?`, `page?`, `rows_per_page?`
- Answers: **200** `count`: number, `farmers`: list of DashboardFarmerResponse, `page`: number, `rows_per_page`: number
- Can fail with: 401, 403, 422, 500

### `POST /v1/dashboard/farmers`

Register a farmer without a sign-in code.

- Access: staff `farmers:create`
- Body: `lang`: `ku` \| `kmr` \| `ar` \| `en`, `name?`: text or null, `phone`: text
- Answers: **201** `farmer`: DashboardFarmerResponse
- Can fail with: 400, 401, 403, 409, 422, 500

### `GET /v1/dashboard/farmers/{id}`

Get one farmer.

- Access: staff `farmers:read`
- Answers: **200** `farmer`: DashboardFarmerResponse
- Can fail with: 401, 403, 404, 500

### `PUT /v1/dashboard/farmers/{id}`

Change a farmer's name and language.

- Access: staff `farmers:update`
- Body: `lang`: `ku` \| `kmr` \| `ar` \| `en`, `name?`: text or null
- Answers: **200** `farmer`: DashboardFarmerResponse
- Can fail with: 400, 401, 403, 404, 422, 500

### `DELETE /v1/dashboard/farmers/{id}`

Delete a farmer with their open sign-in code and all their farms.

- Access: staff `farmers:delete`
- Answers: **204**
- Can fail with: 401, 403, 500

### `GET /v1/dashboard/farms`

List every farmer's farms, newest first.

- Access: staff `farms:read`
- Query: `owner_phone?`, `page?`, `rows_per_page?`
- Answers: **200** `count`: number, `farms`: list of DashboardFarmSummaryResponse, `page`: number, `rows_per_page`: number
- Can fail with: 401, 403, 422, 500

### `POST /v1/dashboard/farms`

Register a farm on behalf of a farmer.

- Access: staff `farms:create`
- Body: CreateFarmParams or object
- Answers: **201** `dropped_cells`: list of GridCellResponse, `farm`: DashboardFarmResponse
- Can fail with: 400, 401, 403, 404, 422, 500

### `GET /v1/dashboard/farms/{id}`

Get one farm with its outline, cells and owner.

- Access: staff `farms:read`
- Answers: **200** `farm`: DashboardFarmResponse
- Can fail with: 401, 403, 404, 500

### `PUT /v1/dashboard/farms/{id}`

Rename a farm.

- Access: staff `farms:update`
- Body: `name`: text
- Answers: **200** `farm`: DashboardFarmResponse
- Can fail with: 400, 401, 403, 404, 422, 500

### `DELETE /v1/dashboard/farms/{id}`

Delete any farmer's farm.

- Access: staff `farms:delete`
- Answers: **204**
- Can fail with: 401, 403, 500

### `GET /v1/dashboard/farms/{id}/insights`

List every stored reading of any farmer's farm, in the fixed topic order.

- Access: staff `insights:read`
- Answers: **200** `farm_id`: text, `insights`: list of InsightDashboardResponse
- Can fail with: 401, 403, 404, 500

### `POST /v1/dashboard/farms/{id}/insights`

Enter a reading by hand for a topic the farm has none for.

- Access: staff `insights:create`
- Body: RecordFarmInsightParams or object
- Answers: **201** `farm_id`: text, `insight`: InsightDashboardResponse
- Can fail with: 400, 401, 403, 404, 409, 422, 500

### `PUT /v1/dashboard/farms/{id}/insights/{topic}`

Replace the reading a farm has for one topic.

- Access: staff `insights:update`
- Body: `as_of`: day, `confidence`: `sure` \| `likely` \| `unsure`, `measures`: list of MeasureParams, `source`: text, `summary_en?`: text or null, `summary_ku?`: text or null
- Answers: **200** `farm_id`: text, `insight`: InsightDashboardResponse
- Can fail with: 400, 401, 403, 404, 422, 500

### `DELETE /v1/dashboard/farms/{id}/insights/{topic}`

Remove the reading a farm has for one topic.

- Access: staff `insights:delete`
- Answers: **204**
- Can fail with: 401, 403, 404, 422, 500

### `GET /v1/dashboard/fires`

List the stored fires for the editing screen, newest first.

- Access: staff `fires:read`
- Query: `from?`, `to?`, `status?`, `zone_slug?`, `page?`, `rows_per_page?`
- Answers: **200** `count`: number, `fires`: list of FireDashboardResponse, `page`: number, `rows_per_page`: number
- Can fail with: 400, 401, 403, 422, 500

### `POST /v1/dashboard/fires`

Enter a fire by hand under a new external id.

- Access: staff `fires:create`
- Body: RecordFireParams or object
- Answers: **201** `fire`: FireDashboardResponse
- Can fail with: 400, 401, 403, 409, 422, 500

### `GET /v1/dashboard/fires/{id}`

Get one stored fire.

- Access: staff `fires:read`
- Answers: **200** `fire`: FireDashboardResponse
- Can fail with: 401, 403, 404, 500

### `PUT /v1/dashboard/fires/{id}`

Replace every field of a stored fire except its external id.

- Access: staff `fires:update`
- Body: `area_ha?`: number or null, `detected_at`: timestamp, `farmers_alerted?`: number or null, `farms_within_5km?`: number or null, `lat`: number, `lon`: number, `place_en?`: text or null, `place_ku?`: text or null, `source`: text, `status`: `active` \| `spreading` \| `under_control` \| `out`, `wind_direction?`: `n` \| `ne` \| `e` \| `se` \| `s` \| `sw` \| `w` \| `nw` or null, `wind_kmh?`: number or null, `zone_slug?`: text or null
- Answers: **200** `fire`: FireDashboardResponse
- Can fail with: 400, 401, 403, 404, 422, 500

### `DELETE /v1/dashboard/fires/{id}`

Remove a stored fire.

- Access: staff `fires:delete`
- Answers: **204**
- Can fail with: 401, 403, 404, 500

### `GET /v1/dashboard/outlook-runs`

List every stored track record, newest issue first.

- Access: staff `outlooks:read`
- Answers: **200** `runs`: list of OutlookRunResponse
- Can fail with: 401, 403, 500

### `POST /v1/dashboard/outlook-runs`

Add a track record for a season and issue that has none.

- Access: staff `outlooks:create`
- Body: `issued`: text, `method`: text, `season`: text, `seasons_right`: number, `seasons_tested`: number
- Answers: **201** `run`: OutlookRunResponse
- Can fail with: 400, 401, 403, 409, 422, 500

### `PUT /v1/dashboard/outlook-runs/{season}/{issued}`

Replace the stored track record of one season and issue.

- Access: staff `outlooks:update`
- Body: `method`: text, `seasons_right`: number, `seasons_tested`: number
- Answers: **200** `run`: OutlookRunResponse
- Can fail with: 400, 401, 403, 404, 422, 500

### `DELETE /v1/dashboard/outlook-runs/{season}/{issued}`

Remove the stored track record of one season and issue.

- Access: staff `outlooks:delete`
- Answers: **204**
- Can fail with: 401, 403, 422, 500

### `GET /v1/dashboard/outlooks`

List the stored zone outlooks, newest issue first.

- Access: staff `outlooks:read`
- Query: `season?`, `issued?`, `page?`, `rows_per_page?`
- Answers: **200** `count`: number, `outlooks`: list of StoredZoneOutlookResponse, `page`: number, `rows_per_page`: number
- Can fail with: 401, 403, 422, 500

### `POST /v1/dashboard/outlooks`

Add an outlook for a zone, season and issue that has none.

- Access: staff `outlooks:create`
- Body: `confidence_pct`: number, `issued`: text, `outlook`: `good` \| `normal` \| `bad`, `reason_en?`: text or null, `reason_ku?`: text or null, `season`: text, `zone_slug`: text
- Answers: **201** `outlook`: StoredZoneOutlookResponse
- Can fail with: 400, 401, 403, 409, 422, 500

### `PUT /v1/dashboard/outlooks/{season}/{issued}/zones/{zone_slug}`

Replace the stored outlook of one zone, season and issue.

- Access: staff `outlooks:update`
- Body: `confidence_pct`: number, `outlook`: `good` \| `normal` \| `bad`, `reason_en?`: text or null, `reason_ku?`: text or null
- Answers: **200** `outlook`: StoredZoneOutlookResponse
- Can fail with: 400, 401, 403, 404, 422, 500

### `DELETE /v1/dashboard/outlooks/{season}/{issued}/zones/{zone_slug}`

Remove the stored outlook of one zone, season and issue.

- Access: staff `outlooks:delete`
- Answers: **204**
- Can fail with: 401, 403, 422, 500

### `GET /v1/dashboard/water/plan/{season}/entries`

List one season's stored water plan entries, highest need first.

- Access: staff `water:read`
- Answers: **200** `entries`: list of StoredWaterPlanEntryResponse
- Can fail with: 401, 403, 422, 500

### `POST /v1/dashboard/water/plan/{season}/entries`

Add a zone to a season's water plan.

- Access: staff `water:create`
- Body: `dam_slug?`: text or null, `need`: number, `note_en?`: text or null, `note_ku?`: text or null, `send_million_m3?`: number or null, `urgent?`: true/false, `zone_slug`: text
- Answers: **201** `entry`: StoredWaterPlanEntryResponse
- Can fail with: 400, 401, 403, 409, 422, 500

### `PUT /v1/dashboard/water/plan/{season}/entries/{zone_slug}`

Replace one zone's entry in a season's water plan.

- Access: staff `water:update`
- Body: `dam_slug?`: text or null, `need`: number, `note_en?`: text or null, `note_ku?`: text or null, `send_million_m3?`: number or null, `urgent?`: true/false
- Answers: **200** `entry`: StoredWaterPlanEntryResponse
- Can fail with: 400, 401, 403, 404, 422, 500

### `DELETE /v1/dashboard/water/plan/{season}/entries/{zone_slug}`

Remove one zone's entry from a season's water plan.

- Access: staff `water:delete`
- Answers: **204**
- Can fail with: 401, 403, 422, 500

### `GET /v1/dashboard/water/seasons`

List the seasons that have a water plan entry, newest first.

- Access: staff `water:read`
- Answers: **200** `seasons`: list of text
- Can fail with: 401, 403, 500

### `GET /v1/dashboard/zones`

List every zone with its sub-zones, for the dashboard's pickers.

- Access: staff `zones:read`
- Answers: **200** `zones`: list of ZoneDashboardZoneResponse
- Can fail with: 401, 403, 500

### `GET /v1/dashboard/zones/{slug}/readings`

List a zone's stored readings, newest month first.

- Access: staff `zones:read`
- Query: `from?`, `to?`, `page?`, `rows_per_page?`
- Answers: **200** `count`: number, `page`: number, `readings`: list of ZoneReadingResponse, `rows_per_page`: number
- Can fail with: 401, 403, 404, 422, 500

### `POST /v1/dashboard/zones/{slug}/readings`

Create a zone's reading for a month that has none.

- Access: staff `zones:create`
- Body: ZoneReadingParams or object
- Answers: **201** `band`: `much_greener` \| `greener` \| `normal` \| `dry` \| `very_dry`, `best_crops`: list of `wheat` \| `barley` \| `tomato` \| `cucumber` \| `potato` \| `onion` \| `watermelon` \| `grape` \| `olive` \| `sunflower` \| `chickpea`, `dryness`: number, `greenness_pct_vs_normal?`: number or null, `month`: text, `nitrogen_hold`: true/false, `rain_pct_of_normal?`: number or null, `source`: text, `updated_at`: timestamp, `water_need?`: number or null, `zone_slug`: text
- Can fail with: 400, 401, 403, 404, 409, 422, 500

### `PUT /v1/dashboard/zones/{slug}/readings/{month}`

Replace a zone's stored reading for a month.

- Access: staff `zones:update`
- Body: `best_crops?`: list of `wheat` \| `barley` \| `tomato` \| `cucumber` \| `potato` \| `onion` \| `watermelon` \| `grape` \| `olive` \| `sunflower` \| `chickpea`, `dryness`: number, `greenness_pct_vs_normal?`: number or null, `nitrogen_hold`: true/false, `rain_pct_of_normal?`: number or null, `source`: text, `water_need?`: number or null
- Answers: **200** `band`: `much_greener` \| `greener` \| `normal` \| `dry` \| `very_dry`, `best_crops`: list of `wheat` \| `barley` \| `tomato` \| `cucumber` \| `potato` \| `onion` \| `watermelon` \| `grape` \| `olive` \| `sunflower` \| `chickpea`, `dryness`: number, `greenness_pct_vs_normal?`: number or null, `month`: text, `nitrogen_hold`: true/false, `rain_pct_of_normal?`: number or null, `source`: text, `updated_at`: timestamp, `water_need?`: number or null, `zone_slug`: text
- Can fail with: 400, 401, 403, 404, 422, 500

### `DELETE /v1/dashboard/zones/{slug}/readings/{month}`

Delete a zone's reading for a month.

- Access: staff `zones:delete`
- Answers: **204**
- Can fail with: 401, 403, 404, 422, 500

### `GET /v1/dashboard/zones/{slug}/sub-zones/{sub_slug}/readings`

List a sub-zone's stored readings, newest month first.

- Access: staff `zones:read`
- Query: `from?`, `to?`, `page?`, `rows_per_page?`
- Answers: **200** `count`: number, `page`: number, `readings`: list of SubZoneReadingResponse, `rows_per_page`: number
- Can fail with: 401, 403, 404, 422, 500

### `POST /v1/dashboard/zones/{slug}/sub-zones/{sub_slug}/readings`

Create a sub-zone's reading for a month that has none.

- Access: staff `zones:create`
- Body: `dryness`: number, `month`: text
- Answers: **201** `band`: `much_greener` \| `greener` \| `normal` \| `dry` \| `very_dry`, `dryness`: number, `month`: text, `sub_zone_slug`: text, `updated_at`: timestamp, `zone_slug`: text
- Can fail with: 400, 401, 403, 404, 409, 422, 500

### `PUT /v1/dashboard/zones/{slug}/sub-zones/{sub_slug}/readings/{month}`

Replace a sub-zone's stored dryness for a month.

- Access: staff `zones:update`
- Body: `dryness`: number
- Answers: **200** `band`: `much_greener` \| `greener` \| `normal` \| `dry` \| `very_dry`, `dryness`: number, `month`: text, `sub_zone_slug`: text, `updated_at`: timestamp, `zone_slug`: text
- Can fail with: 400, 401, 403, 404, 422, 500

### `DELETE /v1/dashboard/zones/{slug}/sub-zones/{sub_slug}/readings/{month}`

Delete a sub-zone's reading for a month.

- Access: staff `zones:delete`
- Answers: **204**
- Can fail with: 401, 403, 404, 422, 500

## Data jobs (service key)

### `PUT /v1/ingest/alwa/markets/{slug}/prices/{crop}/{day}`

Store the price of one crop at one market on one day.

- Access: service key
- Body: `fixed?`: true/false, `price_iqd_per_kg`: number, `source`: text
- Answers: **200** `price`: AlwaRecordedPriceResponse
- Can fail with: 400, 401, 404, 422, 500

### `PUT /v1/ingest/briefs/farm-zones`

Record which zone each farm lies in.

- Access: service key
- Body: `farms`: list of BriefFarmZoneParams
- Answers: **200** `recorded`: number
- Can fail with: 400, 401, 422, 500

### `PUT /v1/ingest/briefs/{day}/{scope}`

Store the brief of one day for one scope, replacing the one it had.

- Access: service key
- Body: `author`: text, `generated_at`: timestamp, `headline_en`: text, `headline_ku`: text, `points`: list of BriefPointParams, `sources`: list of BriefSourceParams, `summary_en`: text, `summary_ku`: text
- Answers: **200** `brief`: BriefResponse
- Can fail with: 400, 401, 422, 500

### `DELETE /v1/ingest/briefs/{day}/{scope}`

Remove the brief of one day for one scope.

- Access: service key
- Answers: **204**
- Can fail with: 401, 422, 500

### `PUT /v1/ingest/dams/{slug}/readings/{day}`

Store one dam's reading for one day, replacing an earlier one for that day.

- Access: service key
- Body: `farm_supply_bn_m3?`: number or null, `lake_area_km2?`: number or null, `pct_full`: number, `source`: text, `volume_bn_m3?`: number or null
- Answers: **200** `reading`: DamReadingResponse, `slug`: text
- Can fail with: 400, 401, 404, 422, 500

### `GET /v1/ingest/farms`

List every farm with the readings it already has, for the data jobs.

- Access: service key
- Answers: **200** `farms`: list of FarmCoverageResponse
- Can fail with: 401, 500

### `PUT /v1/ingest/farms/{id}/insights/{topic}`

Store a farm's reading for one topic, replacing the one it had.

- Access: service key
- Body: `as_of`: day, `confidence`: `sure` \| `likely` \| `unsure`, `measures`: list of MeasureParams, `source`: text, `summary_en?`: text or null, `summary_ku?`: text or null
- Answers: **200** `as_of`: day, `confidence`: `sure` \| `likely` \| `unsure`, `measures`: list of MeasureResponse, `source`: text, `summary_en?`: text or null, `summary_ku?`: text or null, `topic`: `surface_water` \| `groundwater` \| `soil` \| `rain` \| `dryness` \| `greenness` \| `weather`
- Can fail with: 400, 401, 404, 422, 500

### `PUT /v1/ingest/fires/{external_id}`

Store one fire under the job's own id, replacing an earlier push of it.

- Access: service key
- Body: `area_ha?`: number or null, `detected_at`: timestamp, `farmers_alerted?`: number or null, `farms_within_5km?`: number or null, `lat`: number, `lon`: number, `place_en?`: text or null, `place_ku?`: text or null, `source`: text, `status`: `active` \| `spreading` \| `under_control` \| `out`, `wind_direction?`: `n` \| `ne` \| `e` \| `se` \| `s` \| `sw` \| `w` \| `nw` or null, `wind_kmh?`: number or null, `zone_slug?`: text or null
- Answers: **200** `area_ha?`: number or null, `detected_at`: timestamp, `farmers_alerted?`: number or null, `farms_within_5km?`: number or null, `id`: text, `lat`: number, `lon`: number, `place_en?`: text or null, `place_ku?`: text or null, `source`: text, `status`: `active` \| `spreading` \| `under_control` \| `out`, `wind_direction?`: `n` \| `ne` \| `e` \| `se` \| `s` \| `sw` \| `w` \| `nw` or null, `wind_kmh?`: number or null, `zone_slug?`: text or null
- Can fail with: 400, 401, 422, 500

### `PUT /v1/ingest/outlooks/{season}/{issued}/run`

Store the track record of the method behind one issue.

- Access: service key
- Body: `method`: text, `seasons_right`: number, `seasons_tested`: number
- Answers: **200** `run`: OutlookRunResponse
- Can fail with: 400, 401, 422, 500

### `PUT /v1/ingest/outlooks/{season}/{issued}/zones/{zone_slug}`

Store one zone's outlook for one issue, replacing an earlier one.

- Access: service key
- Body: `confidence_pct`: number, `outlook`: `good` \| `normal` \| `bad`, `reason_en?`: text or null, `reason_ku?`: text or null
- Answers: **200** `outlook`: StoredZoneOutlookResponse
- Can fail with: 400, 401, 422, 500

### `PUT /v1/ingest/water/plan/{season}/zones/{zone_slug}`

Store one zone's entry in a season's water plan, replacing an earlier one.

- Access: service key
- Body: `dam_slug?`: text or null, `need`: number, `note_en?`: text or null, `note_ku?`: text or null, `send_million_m3?`: number or null, `urgent?`: true/false
- Answers: **200** `entry`: StoredWaterPlanEntryResponse
- Can fail with: 400, 401, 422, 500

### `DELETE /v1/ingest/water/plan/{season}/zones/{zone_slug}`

Remove one zone's entry from a season's water plan.

- Access: service key
- Answers: **204**
- Can fail with: 401, 404, 422, 500

### `PUT /v1/ingest/zones/{slug}/readings/{month}`

Store a zone's reading for a month, replacing the one already there.

- Access: service key
- Body: `best_crops?`: list of `wheat` \| `barley` \| `tomato` \| `cucumber` \| `potato` \| `onion` \| `watermelon` \| `grape` \| `olive` \| `sunflower` \| `chickpea`, `dryness`: number, `greenness_pct_vs_normal?`: number or null, `nitrogen_hold`: true/false, `rain_pct_of_normal?`: number or null, `source`: text, `water_need?`: number or null
- Answers: **200** `band`: `much_greener` \| `greener` \| `normal` \| `dry` \| `very_dry`, `best_crops`: list of `wheat` \| `barley` \| `tomato` \| `cucumber` \| `potato` \| `onion` \| `watermelon` \| `grape` \| `olive` \| `sunflower` \| `chickpea`, `dryness`: number, `greenness_pct_vs_normal?`: number or null, `month`: text, `nitrogen_hold`: true/false, `rain_pct_of_normal?`: number or null, `source`: text, `updated_at`: timestamp, `water_need?`: number or null, `zone_slug`: text
- Can fail with: 400, 401, 404, 422, 500

### `PUT /v1/ingest/zones/{slug}/sub-zones/{sub_slug}/readings/{month}`

Store a sub-zone's dryness for a month, replacing the one already there.

- Access: service key
- Body: `dryness`: number
- Answers: **200** `band`: `much_greener` \| `greener` \| `normal` \| `dry` \| `very_dry`, `dryness`: number, `month`: text, `sub_zone_slug`: text, `updated_at`: timestamp, `zone_slug`: text
- Can fail with: 400, 401, 404, 422, 500

## Server

### `GET /health`

- Access: none
- Answers: **200**
- Can fail with: 503

### `GET /status`

- Access: none
- Answers: **200**

## Shapes

Objects that the routes above refer to by name.

- **AlwaDealResponse**: `accepted_at`: timestamp, `asking_price_iqd_per_kg`: number, `buyer_name`: text, `crop`: `wheat` \| `barley` \| `tomato` \| `cucumber` \| `potato` \| `onion` \| `watermelon` \| `grape` \| `olive` \| `sunflower` \| `chickpea` \| `pomegranate` \| `okra` \| `eggplant` \| `pepper` \| `apple`, `listing_id`: text, `quantity_kg`: number, `seller_name?`: text or null, `sold_price_iqd_per_kg`: number, `vs_asking_pct`: number, `zone_slug?`: text or null
- **AlwaDealsResponse**: `day`: day, `deals`: list of AlwaDealResponse, `summary`: AlwaDealsSummaryResponse
- **AlwaDealsSummaryResponse**: `deals`: number, `tonnes`: number
- **AlwaListingResponse**: `asking_price_iqd_per_kg`: number, `best_offer_iqd_per_kg?`: number or null, `closes_at`: timestamp, `created_at`: timestamp, `crop`: `wheat` \| `barley` \| `tomato` \| `cucumber` \| `potato` \| `onion` \| `watermelon` \| `grape` \| `olive` \| `sunflower` \| `chickpea` \| `pomegranate` \| `okra` \| `eggplant` \| `pepper` \| `apple`, `fair_price`: `fair` \| `high` \| `low` \| `unknown`, `grade?`: `a` \| `b` \| `c` or null, `id`: text, `market`: text, `note?`: text or null, `offers`: list of AlwaOfferResponse, `pickup`: `farm` \| `alwa`, `quantity_kg`: number, `seller_name?`: text or null, `status`: `open` \| `sold` \| `closed` \| `cancelled`, `zone_slug?`: text or null
- **AlwaListingSummaryResponse**: `asking_price_iqd_per_kg`: number, `best_offer_iqd_per_kg?`: number or null, `closes_at`: timestamp, `crop`: `wheat` \| `barley` \| `tomato` \| `cucumber` \| `potato` \| `onion` \| `watermelon` \| `grape` \| `olive` \| `sunflower` \| `chickpea` \| `pomegranate` \| `okra` \| `eggplant` \| `pepper` \| `apple`, `fair_price`: `fair` \| `high` \| `low` \| `unknown`, `grade?`: `a` \| `b` \| `c` or null, `id`: text, `market`: text, `offers`: number, `pickup`: `farm` \| `alwa`, `quantity_kg`: number, `seller_name?`: text or null, `status`: `open` \| `sold` \| `closed` \| `cancelled`, `zone_slug?`: text or null
- **AlwaListingsResponse**: `count`: number, `listings`: list of AlwaListingSummaryResponse, `page`: number, `rows_per_page`: number
- **AlwaMarketPricesResponse**: `day?`: day or null, `market`: text, `prices`: list of AlwaPriceResponse
- **AlwaMarketResponse**: `name_en`: text, `name_ku`: text, `slug`: text
- **AlwaMarketsResponse**: `markets`: list of AlwaMarketResponse
- **AlwaModeratedListingDetailResponse**: AlwaModeratedListingResponse or object
- **AlwaModeratedListingResponse**: `asking_price_iqd_per_kg`: number, `closed_by_staff_id?`: text or null, `closes_at`: timestamp, `created_at`: timestamp, `crop`: `wheat` \| `barley` \| `tomato` \| `cucumber` \| `potato` \| `onion` \| `watermelon` \| `grape` \| `olive` \| `sunflower` \| `chickpea` \| `pomegranate` \| `okra` \| `eggplant` \| `pepper` \| `apple`, `fair_price`: `fair` \| `high` \| `low` \| `unknown`, `grade?`: `a` \| `b` \| `c` or null, `id`: text, `market`: text, `moderation_note?`: text or null, `note?`: text or null, `open_offers`: number, `pickup`: `farm` \| `alwa`, `quantity_kg`: number, `seller_name?`: text or null, `seller_phone`: text, `status`: `open` \| `sold` \| `closed` \| `cancelled`, `updated_at`: timestamp, `zone_slug?`: text or null
- **AlwaModeratedListingsResponse**: `count`: number, `listings`: list of AlwaModeratedListingResponse, `page`: number, `rows_per_page`: number
- **AlwaModeratedOfferResponse**: `buyer_kind`: `shop` \| `restaurant` \| `trader` \| `other`, `buyer_name`: text, `buyer_phone`: text, `created_at`: timestamp, `id`: text, `price_iqd_per_kg`: number, `quantity_kg`: number, `status`: `open` \| `accepted` \| `declined` \| `withdrawn`, `updated_at`: timestamp
- **AlwaMyListingsResponse**: `listings`: list of AlwaListingResponse
- **AlwaMyOfferResponse**: `buyer_kind`: `shop` \| `restaurant` \| `trader` \| `other`, `buyer_name`: text, `created_at`: timestamp, `id`: text, `listing`: AlwaOfferListingResponse, `price_iqd_per_kg`: number, `quantity_kg`: number, `seller_phone?`: text or null, `status`: `open` \| `accepted` \| `declined` \| `withdrawn`
- **AlwaMyOffersResponse**: `offers`: list of AlwaMyOfferResponse
- **AlwaOfferListingResponse**: `asking_price_iqd_per_kg`: number, `closes_at`: timestamp, `crop`: `wheat` \| `barley` \| `tomato` \| `cucumber` \| `potato` \| `onion` \| `watermelon` \| `grape` \| `olive` \| `sunflower` \| `chickpea` \| `pomegranate` \| `okra` \| `eggplant` \| `pepper` \| `apple`, `id`: text, `market`: text, `quantity_kg`: number, `seller_name?`: text or null, `status`: `open` \| `sold` \| `closed` \| `cancelled`
- **AlwaOfferResponse**: `buyer_kind`: `shop` \| `restaurant` \| `trader` \| `other`, `buyer_name`: text, `buyer_phone?`: text or null, `created_at`: timestamp, `id`: text, `price_iqd_per_kg`: number, `quantity_kg`: number, `status`: `open` \| `accepted` \| `declined` \| `withdrawn`
- **AlwaOneListingResponse**: `listing`: AlwaListingResponse
- **AlwaOneMarketResponse**: `market`: AlwaMarketResponse
- **AlwaOneModeratedListingResponse**: `listing`: AlwaModeratedListingDetailResponse
- **AlwaOneOfferResponse**: `offer`: AlwaOfferResponse
- **AlwaOnePriceResponse**: `price`: AlwaRecordedPriceResponse
- **AlwaPriceHistoryResponse**: `crop`: `wheat` \| `barley` \| `tomato` \| `cucumber` \| `potato` \| `onion` \| `watermelon` \| `grape` \| `olive` \| `sunflower` \| `chickpea` \| `pomegranate` \| `okra` \| `eggplant` \| `pepper` \| `apple`, `history`: list of AlwaPricePointResponse, `market`: text
- **AlwaPricePointResponse**: `day`: day, `price_iqd_per_kg`: number
- **AlwaPriceResponse**: `change_pct_7d?`: number or null, `crop`: `wheat` \| `barley` \| `tomato` \| `cucumber` \| `potato` \| `onion` \| `watermelon` \| `grape` \| `olive` \| `sunflower` \| `chickpea` \| `pomegranate` \| `okra` \| `eggplant` \| `pepper` \| `apple`, `fixed`: true/false, `price_iqd_per_kg`: number
- **AlwaRecordedPriceResponse**: `crop`: `wheat` \| `barley` \| `tomato` \| `cucumber` \| `potato` \| `onion` \| `watermelon` \| `grape` \| `olive` \| `sunflower` \| `chickpea` \| `pomegranate` \| `okra` \| `eggplant` \| `pepper` \| `apple`, `day`: day, `fixed`: true/false, `market`: text, `price_iqd_per_kg`: number, `source`: text, `updated_at`: timestamp
- **AlwaStoredPricesResponse**: `count`: number, `page`: number, `prices`: list of AlwaRecordedPriceResponse, `rows_per_page`: number
- **BriefFarmZoneParams**: `farm_id`: text, `zone_slug`: text
- **BriefFarmZonesRecordedResponse**: `recorded`: number
- **BriefPointParams**: `level`: `info` \| `watch` \| `alarm`, `text_en`: text, `text_ku`: text
- **BriefPointResponse**: `level`: `info` \| `watch` \| `alarm`, `text_en`: text, `text_ku`: text
- **BriefResponse**: `author`: text, `day`: day, `generated_at`: timestamp, `headline_en`: text, `headline_ku`: text, `points`: list of BriefPointResponse, `scope`: text, `sources`: list of BriefSourceResponse, `summary_en`: text, `summary_ku`: text, `updated_at`: timestamp
- **BriefSourceParams**: `title`: text, `url`: text
- **BriefSourceResponse**: `title`: text, `url`: text
- **BriefsPageResponse**: `briefs`: list of BriefResponse, `count`: number, `page`: number, `rows_per_page`: number
- **BriefsResponse**: `briefs`: list of BriefResponse
- **CellParams**: `crop`: `wheat` \| `barley` \| `tomato` \| `cucumber` \| `potato` \| `onion` \| `watermelon` \| `grape` \| `olive` \| `sunflower` \| `chickpea` \| `empty`, `e`: number, `n`: number
- **CellResponse**: `crop`: `wheat` \| `barley` \| `tomato` \| `cucumber` \| `potato` \| `onion` \| `watermelon` \| `grape` \| `olive` \| `sunflower` \| `chickpea` \| `empty`, `e`: number, `inside_pct`: number, `n`: number
- **CellStatusResponse**: `e`: number, `greenness_pct?`: number or null, `level`: `normal` \| `watch` \| `alarm` \| `none`, `n`: number, `since?`: day or null
- **CentroidResponse**: `lat`: number, `lon`: number
- **CreateAlwaMarketParams**: `name_en`: text, `name_ku`: text, `slug`: text
- **CreateAlwaPriceParams**: `crop`: text, `day`: text, `fixed?`: true/false, `price_iqd_per_kg`: number, `source`: text
- **CreateDamDashboardReadingParams**: `day`: text, `farm_supply_bn_m3?`: number or null, `lake_area_km2?`: number or null, `pct_full`: number, `source`: text, `volume_bn_m3?`: number or null
- **CreateFarmParams**: `cells?`: list of CellParams, `created_offline_at?`: timestamp or null, `name`: text, `points`: list of PointParams
- **CreateOutlookDashboardParams**: `confidence_pct`: number, `issued`: text, `outlook`: `good` \| `normal` \| `bad`, `reason_en?`: text or null, `reason_ku?`: text or null, `season`: text, `zone_slug`: text
- **CreateOutlookRunDashboardParams**: `issued`: text, `method`: text, `season`: text, `seasons_right`: number, `seasons_tested`: number
- **CreateStaffParams**: `email`: text, `name`: text, `password`: text, `role_ids?`: list of text
- **CreateWaterPlanEntryDashboardParams**: `dam_slug?`: text or null, `need`: number, `note_en?`: text or null, `note_ku?`: text or null, `send_million_m3?`: number or null, `urgent?`: true/false, `zone_slug`: text
- **CreateZoneDashboardReadingParams**: ZoneReadingParams or object
- **CreateZoneDashboardSubZoneReadingParams**: `dryness`: number, `month`: text
- **CropAreaResponse**: `crop`: `wheat` \| `barley` \| `tomato` \| `cucumber` \| `potato` \| `onion` \| `watermelon` \| `grape` \| `olive` \| `sunflower` \| `chickpea` \| `empty`, `dunam`: number
- **CropStatusResponse**: `crop`: `wheat` \| `barley` \| `tomato` \| `cucumber` \| `potato` \| `onion` \| `watermelon` \| `grape` \| `olive` \| `sunflower` \| `chickpea` \| `empty`, `dunam`: number, `greenness_pct_of_normal?`: number or null, `level`: `normal` \| `watch` \| `alarm` \| `none`
- **DamAllocationResponse**: `dam_slug`: text, `planned_million_m3`: number, `zones`: number
- **DamHistoryResponse**: `readings`: list of HistoryReadingResponse, `slug`: text
- **DamReadingResponse**: `day`: day, `farm_supply_bn_m3?`: number or null, `lake_area_km2?`: number or null, `pct_full`: number, `source`: text, `updated_at`: timestamp, `volume_bn_m3?`: number or null
- **DamReadingsPageResponse**: `count`: number, `page`: number, `readings`: list of DamReadingResponse, `rows_per_page`: number
- **DamReferenceResponse**: `capacity_bn_m3`: number, `name_en`: text, `name_ku`: text, `slug`: text
- **DamReferencesResponse**: `dams`: list of DamReferenceResponse
- **DamResponse**: `capacity_bn_m3`: number, `latest?`: LatestReadingResponse or null, `name_en`: text, `name_ku`: text, `slug`: text, `year_ago?`: YearAgoReadingResponse or null
- **DamsResponse**: `dams`: list of DamResponse
- **DashboardCreateFarmParams**: CreateFarmParams or object
- **DashboardCreateFarmerParams**: `lang`: `ku` \| `kmr` \| `ar` \| `en`, `name?`: text or null, `phone`: text
- **DashboardFarmResponse**: FarmResponse or object
- **DashboardFarmSummaryResponse**: FarmSummaryResponse or object
- **DashboardFarmerResponse**: `created_at`: timestamp, `farms_count`: number, `id`: text, `lang`: `ku` \| `kmr` \| `ar` \| `en`, `name?`: text or null, `phone`: text, `updated_at`: timestamp
- **DashboardFarmersResponse**: `count`: number, `farmers`: list of DashboardFarmerResponse, `page`: number, `rows_per_page`: number
- **DashboardFarmsResponse**: `count`: number, `farms`: list of DashboardFarmSummaryResponse, `page`: number, `rows_per_page`: number
- **DashboardOneFarmResponse**: `farm`: DashboardFarmResponse
- **DashboardOneFarmerResponse**: `farmer`: DashboardFarmerResponse
- **DashboardRenameFarmParams**: `name`: text
- **DashboardSavedFarmResponse**: `dropped_cells`: list of GridCellResponse, `farm`: DashboardFarmResponse
- **DashboardUpdateFarmerParams**: `lang`: `ku` \| `kmr` \| `ar` \| `en`, `name?`: text or null
- **DoctorAnswerResponse**: `actions_this_week`: list of text, `cannot_tell`: list of text, `confidence`: `sure` \| `likely` \| `unsure`, `en`: text, `inputs_used`: list of text, `ku`: text, `likely`: text, `refer_to_officer`: true/false, `why`: list of text
- **DoctorAskForm**: `cell?`: text or null, `lang?`: text or null, `photos?`: list of text, `question?`: text or null
- **EditProfileParams**: `lang`: `ku` \| `kmr` \| `ar` \| `en`, `name?`: text or null
- **ErrorBody**: `detail`: text, `error`: text, `field?`: text or null, `retry_after_s?`: number or null
- **FarmBriefResponse**: `brief?`: BriefResponse or null, `farm_id`: text, `zone_slug?`: text or null
- **FarmCoverageResponse**: `area_dunam`: number, `id`: text, `lat`: number, `lon`: number, `topics`: list of TopicStampResponse
- **FarmInsightsResponse**: `farm_id`: text, `topics`: list of TopicInsightResponse
- **FarmResponse**: `area_dunam`: number, `cells`: list of CellResponse, `centroid`: CentroidResponse, `created_at`: timestamp, `created_offline_at?`: timestamp or null, `crops`: list of CropAreaResponse, `id`: text, `name`: text, `outline`: list of OutlinePointResponse, `updated_at`: timestamp
- **FarmStatusResponse**: `cells`: list of CellStatusResponse, `crops`: list of CropStatusResponse, `greenness_pct_of_normal?`: number or null, `next_picture_expected?`: day or null, `picture_date?`: day or null, `weak_where?`: text or null
- **FarmSummaryResponse**: `area_dunam`: number, `centroid`: CentroidResponse, `created_at`: timestamp, `crops`: list of CropAreaResponse, `id`: text, `name`: text
- **FarmsCoverageResponse**: `farms`: list of FarmCoverageResponse
- **FarmsResponse**: `farms`: list of FarmSummaryResponse
- **FireDashboardCreateParams**: RecordFireParams or object
- **FireDashboardListResponse**: `count`: number, `fires`: list of FireDashboardResponse, `page`: number, `rows_per_page`: number
- **FireDashboardOneResponse**: `fire`: FireDashboardResponse
- **FireDashboardResponse**: `area_ha?`: number or null, `detected_at`: timestamp, `external_id`: text, `farmers_alerted?`: number or null, `farms_within_5km?`: number or null, `id`: text, `lat`: number, `lon`: number, `place_en?`: text or null, `place_ku?`: text or null, `source`: text, `status`: `active` \| `spreading` \| `under_control` \| `out`, `updated_at`: timestamp, `wind_direction?`: `n` \| `ne` \| `e` \| `se` \| `s` \| `sw` \| `w` \| `nw` or null, `wind_kmh?`: number or null, `zone_slug?`: text or null
- **FireResponse**: `area_ha?`: number or null, `detected_at`: timestamp, `farmers_alerted?`: number or null, `farms_within_5km?`: number or null, `id`: text, `lat`: number, `lon`: number, `place_en?`: text or null, `place_ku?`: text or null, `source`: text, `status`: `active` \| `spreading` \| `under_control` \| `out`, `wind_direction?`: `n` \| `ne` \| `e` \| `se` \| `s` \| `sw` \| `w` \| `nw` or null, `wind_kmh?`: number or null, `zone_slug?`: text or null
- **FireSummaryResponse**: `active`: number, `area_ha`: number, `farmers_alerted`: number, `farms_within_5km`: number, `under_control`: number, `zones`: list of text
- **FiresResponse**: `fires`: list of FireResponse, `hours`: number, `summary`: FireSummaryResponse
- **GridCellResponse**: `e`: number, `n`: number
- **HistoryReadingResponse**: `day`: day, `pct_full`: number, `volume_bn_m3?`: number or null
- **InsightDashboardCreateParams**: RecordFarmInsightParams or object
- **InsightDashboardListResponse**: `farm_id`: text, `insights`: list of InsightDashboardResponse
- **InsightDashboardOneResponse**: `farm_id`: text, `insight`: InsightDashboardResponse
- **InsightDashboardResponse**: `as_of`: day, `confidence`: `sure` \| `likely` \| `unsure`, `measures`: list of MeasureResponse, `source`: text, `summary_en?`: text or null, `summary_ku?`: text or null, `topic`: `surface_water` \| `groundwater` \| `soil` \| `rain` \| `dryness` \| `greenness` \| `weather`, `updated_at`: timestamp
- **LatestReadingResponse**: `day`: day, `farm_supply_bn_m3?`: number or null, `lake_area_km2?`: number or null, `pct_full`: number, `source`: text, `volume_bn_m3?`: number or null
- **MakeAlwaOfferParams**: `buyer_kind`: `shop` \| `restaurant` \| `trader` \| `other`, `buyer_name`: text, `price_iqd_per_kg`: number, `quantity_kg`: number
- **MeasureParams**: `code`: text, `label_en`: text, `label_ku?`: text or null, `unit?`: text, `value`: number
- **MeasureResponse**: `code`: text, `label_en`: text, `label_ku?`: text or null, `unit`: text, `value`: number
- **ModerateAlwaListingParams**: `note?`: text or null, `status`: `open` \| `sold` \| `closed` \| `cancelled`
- **OneBriefResponse**: `brief`: BriefResponse
- **OneFarmResponse**: `farm`: FarmResponse
- **OutlinePointResponse**: `lat`: number, `lon`: number
- **OutlookCountsResponse**: `bad`: number, `good`: number, `normal`: number
- **OutlookRunResponse**: `issued`: text, `method`: text, `season`: text, `seasons_right`: number, `seasons_tested`: number, `updated_at`: timestamp
- **OutlookRunsResponse**: `runs`: list of OutlookRunResponse
- **OutlooksPageResponse**: `count`: number, `outlooks`: list of StoredZoneOutlookResponse, `page`: number, `rows_per_page`: number
- **PlanTotalsResponse**: `by_dam`: list of DamAllocationResponse, `planned_million_m3`: number, `urgent_zones`: number
- **PointParams**: `acc_m?`: number or null, `lat`: number, `lon`: number, `t?`: timestamp or null
- **PostAlwaListingParams**: `asking_price_iqd_per_kg`: number, `closes_at`: timestamp, `crop`: `wheat` \| `barley` \| `tomato` \| `cucumber` \| `potato` \| `onion` \| `watermelon` \| `grape` \| `olive` \| `sunflower` \| `chickpea` \| `pomegranate` \| `okra` \| `eggplant` \| `pepper` \| `apple`, `grade?`: `a` \| `b` \| `c` or null, `market`: text, `note?`: text or null, `pickup`: `farm` \| `alwa`, `quantity_kg`: number, `seller_name?`: text or null, `zone_slug?`: text or null
- **ProfileResponse**: `created_at`: timestamp, `lang`: `ku` \| `kmr` \| `ar` \| `en`, `name?`: text or null, `phone`: text
- **RankedEntryResponse**: `dam_slug?`: text or null, `need`: number, `note_en?`: text or null, `note_ku?`: text or null, `rank`: number, `send_million_m3?`: number or null, `urgent`: true/false, `zone_slug`: text
- **RecordAlwaPriceParams**: `fixed?`: true/false, `price_iqd_per_kg`: number, `source`: text
- **RecordBriefFarmZonesParams**: `farms`: list of BriefFarmZoneParams
- **RecordBriefParams**: `author`: text, `generated_at`: timestamp, `headline_en`: text, `headline_ku`: text, `points`: list of BriefPointParams, `sources`: list of BriefSourceParams, `summary_en`: text, `summary_ku`: text
- **RecordDamReadingParams**: `farm_supply_bn_m3?`: number or null, `lake_area_km2?`: number or null, `pct_full`: number, `source`: text, `volume_bn_m3?`: number or null
- **RecordFarmInsightParams**: `as_of`: day, `confidence`: `sure` \| `likely` \| `unsure`, `measures`: list of MeasureParams, `source`: text, `summary_en?`: text or null, `summary_ku?`: text or null
- **RecordFireParams**: `area_ha?`: number or null, `detected_at`: timestamp, `farmers_alerted?`: number or null, `farms_within_5km?`: number or null, `lat`: number, `lon`: number, `place_en?`: text or null, `place_ku?`: text or null, `source`: text, `status`: `active` \| `spreading` \| `under_control` \| `out`, `wind_direction?`: `n` \| `ne` \| `e` \| `se` \| `s` \| `sw` \| `w` \| `nw` or null, `wind_kmh?`: number or null, `zone_slug?`: text or null
- **RecordOutlookRunParams**: `method`: text, `seasons_right`: number, `seasons_tested`: number
- **RecordZoneOutlookParams**: `confidence_pct`: number, `outlook`: `good` \| `normal` \| `bad`, `reason_en?`: text or null, `reason_ku?`: text or null
- **RegionComparisonResponse**: `month`: number, `region`: list of YearAverageResponse, `with`: number, `year`: number, `zones`: list of ZoneComparisonResponse
- **RegionOverviewResponse**: `month`: text, `summary`: RegionSummaryResponse, `zones`: list of ZoneOverviewResponse
- **RegionSummaryResponse**: `average_dryness?`: number or null, `change_vs_last_year?`: number or null, `driest`: list of text, `nitrogen_hold`: list of text, `zones_with_data`: number
- **RepaintFarmCellsParams**: `cells`: list of CellParams
- **SaveStaffRoleParams**: `description?`: text or null, `name`: text, `permissions`: list of StaffPermission
- **SavedDamReadingResponse**: `reading`: DamReadingResponse, `slug`: text
- **SavedFarmResponse**: `dropped_cells`: list of GridCellResponse, `farm`: FarmResponse
- **SavedOutlookRunResponse**: `run`: OutlookRunResponse
- **SavedWaterPlanEntryResponse**: `entry`: StoredWaterPlanEntryResponse
- **SavedZoneOutlookResponse**: `outlook`: StoredZoneOutlookResponse
- **SeasonOutlookResponse**: `counts`: OutlookCountsResponse, `issued`: text, `issues`: list of text, `season`: text, `track_record?`: TrackRecordResponse or null, `zones`: list of ZoneOutlookResponse
- **SendSignInCodeParams**: `lang?`: `ku` \| `kmr` \| `ar` \| `en`, `phone`: text
- **SetWaterPlanEntryParams**: `dam_slug?`: text or null, `need`: number, `note_en?`: text or null, `note_ku?`: text or null, `send_million_m3?`: number or null, `urgent?`: true/false
- **SignInCodeSentResponse**: `retry_after_s`: number, `sent`: true/false
- **SignedInResponse**: `farms_count`: number, `token`: text
- **StaffListResponse**: `staff`: list of StaffResponse
- **StaffLoginParams**: `email`: text, `password`: text
- **StaffMeResponse**: `permissions`: list of StaffPermission, `staff`: StaffResponse
- **StaffOneResponse**: `staff`: StaffResponse
- **StaffOneRoleResponse**: `role`: StaffRoleResponse
- **StaffPermission**: `action`: `create` \| `read` \| `update` \| `delete`, `resource`: `zones` \| `dams` \| `outlooks` \| `water` \| `fires` \| `alwa` \| `farmers` \| `farms` \| `insights` \| `staff` \| `roles` \| `briefs`
- **StaffPermissionCatalogueResponse**: `actions`: list of `create` \| `read` \| `update` \| `delete`, `resources`: list of `zones` \| `dams` \| `outlooks` \| `water` \| `fires` \| `alwa` \| `farmers` \| `farms` \| `insights` \| `staff` \| `roles` \| `briefs`
- **StaffResponse**: `active`: true/false, `created_at`: timestamp, `email`: text, `id`: text, `name`: text, `roles`: list of StaffRoleRefResponse, `updated_at`: timestamp
- **StaffRoleRefResponse**: `id`: text, `name`: text
- **StaffRoleResponse**: `created_at`: timestamp, `description?`: text or null, `id`: text, `name`: text, `permissions`: list of StaffPermission, `staff_count`: number, `system`: true/false, `updated_at`: timestamp
- **StaffRolesResponse**: `roles`: list of StaffRoleResponse
- **StaffSignedInResponse**: `permissions`: list of StaffPermission, `staff`: StaffResponse, `token`: text
- **StoredWaterPlanEntryResponse**: `dam_slug?`: text or null, `need`: number, `note_en?`: text or null, `note_ku?`: text or null, `season`: text, `send_million_m3?`: number or null, `updated_at`: timestamp, `urgent`: true/false, `zone_slug`: text
- **StoredZoneOutlookResponse**: `confidence_pct`: number, `issued`: text, `outlook`: `good` \| `normal` \| `bad`, `reason_en?`: text or null, `reason_ku?`: text or null, `season`: text, `updated_at`: timestamp, `zone_slug`: text
- **SubZoneDrynessResponse**: `band?`: `much_greener` \| `greener` \| `normal` \| `dry` \| `very_dry` or null, `dryness?`: number or null, `name_en`: text, `name_ku`: text, `slug`: text
- **SubZoneReadingParams**: `dryness`: number
- **SubZoneReadingResponse**: `band`: `much_greener` \| `greener` \| `normal` \| `dry` \| `very_dry`, `dryness`: number, `month`: text, `sub_zone_slug`: text, `updated_at`: timestamp, `zone_slug`: text
- **TopicInsightResponse**: `as_of`: day, `confidence`: `sure` \| `likely` \| `unsure`, `measures`: list of MeasureResponse, `source`: text, `summary_en?`: text or null, `summary_ku?`: text or null, `topic`: `surface_water` \| `groundwater` \| `soil` \| `rain` \| `dryness` \| `greenness` \| `weather`
- **TopicStampResponse**: `as_of`: day, `topic`: `surface_water` \| `groundwater` \| `soil` \| `rain` \| `dryness` \| `greenness` \| `weather`
- **TrackRecordResponse**: `method`: text, `seasons_right`: number, `seasons_tested`: number
- **UpdateAlwaMarketParams**: `name_en`: text, `name_ku`: text
- **UpdateStaffParams**: `active`: true/false, `name`: text, `password?`: text or null, `role_ids`: list of text
- **VerifySignInCodeParams**: `code`: text, `phone`: text
- **WaterPlanEntriesResponse**: `entries`: list of StoredWaterPlanEntryResponse
- **WaterPlanResponse**: `entries`: list of RankedEntryResponse, `season`: text, `totals`: PlanTotalsResponse
- **WaterSeasonsResponse**: `seasons`: list of text
- **YearAgoReadingResponse**: `day`: day, `pct_full`: number
- **YearAverageResponse**: `average_dryness`: number, `year`: number
- **YearDrynessResponse**: `dryness`: number, `year`: number
- **ZoneComparisonResponse**: `change?`: number or null, `dryness?`: number or null, `dryness_with?`: number or null, `name_en`: text, `name_ku`: text, `slug`: text
- **ZoneDashboardReadingsResponse**: `count`: number, `page`: number, `readings`: list of ZoneReadingResponse, `rows_per_page`: number
- **ZoneDashboardSubZoneReadingsResponse**: `count`: number, `page`: number, `readings`: list of SubZoneReadingResponse, `rows_per_page`: number
- **ZoneDashboardSubZoneResponse**: `name_en`: text, `name_ku`: text, `slug`: text
- **ZoneDashboardZoneResponse**: `governorate`: text, `name_en`: text, `name_ku`: text, `slug`: text, `sub_zones`: list of ZoneDashboardSubZoneResponse
- **ZoneDashboardZonesResponse**: `zones`: list of ZoneDashboardZoneResponse
- **ZoneDetailReadingResponse**: `band`: `much_greener` \| `greener` \| `normal` \| `dry` \| `very_dry`, `best_crops`: list of `wheat` \| `barley` \| `tomato` \| `cucumber` \| `potato` \| `onion` \| `watermelon` \| `grape` \| `olive` \| `sunflower` \| `chickpea`, `dryness`: number, `greenness_pct_vs_normal?`: number or null, `nitrogen_hold`: true/false, `rain_pct_of_normal?`: number or null, `rank`: number, `rank_of`: number, `source`: text, `updated_at`: timestamp, `water_need?`: number or null
- **ZoneDetailResponse**: `governorate`: text, `history`: list of YearDrynessResponse, `month`: text, `name_en`: text, `name_ku`: text, `reading?`: ZoneDetailReadingResponse or null, `slug`: text, `sub_zones`: list of SubZoneDrynessResponse
- **ZoneIssueResponse**: `confidence_pct`: number, `issued`: text, `outlook`: `good` \| `normal` \| `bad`, `reason_en?`: text or null, `reason_ku?`: text or null
- **ZoneOutlookHistoryResponse**: `issues`: list of ZoneIssueResponse, `season`: text, `zone_slug`: text
- **ZoneOutlookResponse**: `confidence_pct`: number, `outlook`: `good` \| `normal` \| `bad`, `reason_en?`: text or null, `reason_ku?`: text or null, `zone_slug`: text
- **ZoneOverviewResponse**: `band?`: `much_greener` \| `greener` \| `normal` \| `dry` \| `very_dry` or null, `change_vs_last_year?`: number or null, `dryness?`: number or null, `governorate`: text, `name_en`: text, `name_ku`: text, `nitrogen_hold?`: true/false or null, `rank?`: number or null, `slug`: text, `water_need?`: number or null
- **ZoneReadingParams**: `best_crops?`: list of `wheat` \| `barley` \| `tomato` \| `cucumber` \| `potato` \| `onion` \| `watermelon` \| `grape` \| `olive` \| `sunflower` \| `chickpea`, `dryness`: number, `greenness_pct_vs_normal?`: number or null, `nitrogen_hold`: true/false, `rain_pct_of_normal?`: number or null, `source`: text, `water_need?`: number or null
- **ZoneReadingResponse**: `band`: `much_greener` \| `greener` \| `normal` \| `dry` \| `very_dry`, `best_crops`: list of `wheat` \| `barley` \| `tomato` \| `cucumber` \| `potato` \| `onion` \| `watermelon` \| `grape` \| `olive` \| `sunflower` \| `chickpea`, `dryness`: number, `greenness_pct_vs_normal?`: number or null, `month`: text, `nitrogen_hold`: true/false, `rain_pct_of_normal?`: number or null, `source`: text, `updated_at`: timestamp, `water_need?`: number or null, `zone_slug`: text
