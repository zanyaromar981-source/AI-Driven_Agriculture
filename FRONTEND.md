# FRONTEND.md: what the backend expects from the frontend

This is the backend's side of the contract. `BACKEND.md` says what the app needs; this file says what the backend in `backend/` really does today and where it differs from `BACKEND.md`. The app follows this file. Disagreements are settled here and in `BACKEND.md`, not in code.

Status: v1, written by the backend on 2026-10-08. It covers `BACKEND.md` section 1 and section 2.2 (My farms). Everything else in `BACKEND.md` section 2 is not built yet.

## 1. Differences from BACKEND.md (please read)

| Topic | BACKEND.md says | The backend does | Why |
|---|---|---|---|
| Base path | `/farms` | `/v1/farms` | room to change the API later without breaking installed apps |
| Success body | bare object, for example `{"farms": [...]}` | always `{"data": ..., "meta": ...}`; `meta` only on lists | one shape for every endpoint |
| Error body | `{"error": "bad_polygon", "field": ...}` | `{"errors": [{"field": "global", "title": "Validation Error", "detail": "The farm outline crosses itself"}]}` | one shape for every error. There are no machine codes such as `bad_polygon` yet; say if the app needs them to show Sorani messages |
| Farm id | string `"f_01HX..."` | integer, for example `12` | simple database key |
| Unknown crop code | `422` | `400` with title `Invalid Request Body` | the body fails to parse before any rule runs |
| Outline | backend snaps points to the grid | the outline is returned as the farmer walked it; only the cells are on the grid | snapping needs the reverse projection, not built yet |
| `status`, `last_picture`, `greenness_pct`, `level`, `picture_date` | on FarmSummary and Farm | not returned yet | there are no satellite readings in the database yet |
| `Idempotency-Key` | repeat keys must not create duplicates | the header is ignored for now | not built yet; a retried `POST /v1/farms` creates a second farm |
| Sign in | `POST /auth/otp/send`, `/auth/otp/verify` | not built yet | see section 4 |

## 2. One number in BACKEND.md looks wrong

The `POST /farms` example has `{"e": 462337, "n": 398812}`. With the definition in section 1 (`e = floor(easting / 10)`, UTM zone 38N), a point at lat 36.0312, lon 44.6021 has easting 464,152 m and northing 3,987,482 m, so the cell is `{"e": 46415, "n": 398748}`. The example `e` is ten times too large. The backend uses the definition, not the example. A cell sent with the larger number falls outside the outline and comes back in `dropped_cells`.

## 3. My farms (what the app sends and gets)

All requests: `Authorization: Bearer <token>`, JSON, UTF-8, snake_case field names.

### GET /v1/farms
Optional query: `page` (default 1), `rows_per_page` (default 100, at most 100).

```json
{"data": [
   {"id": 1, "name": "کێڵگەی سەرەوە", "area_dunam": 3.96,
    "crops": [{"crop": "wheat", "dunam": 0.04}, {"crop": "tomato", "dunam": 0.04}],
    "centroid": {"lat": 36.0305, "lon": 44.6005},
    "created_at": "2026-10-08T17:01:35.445318"}],
 "meta": {"count": 1, "rows_per_page": 100, "page": 1}}
```

- `crops` never lists `empty`. Largest crop first.
- `area_dunam` counts every cell of the farm, painted or not (25 cells = 1 dunam). It is not rounded; the app rounds.
- `meta.count` is only sent on page 1.
- Timestamps in responses are UTC without the trailing `Z`.

### POST /v1/farms
```json
{"name": "کێڵگەی سەرەوە",
 "points": [{"lat": 36.03, "lon": 44.60, "acc_m": 6, "t": "2026-10-08T14:03:11Z"}, ...],
 "cells": [{"e": 46397, "n": 398737, "crop": "wheat"}, ...],
 "created_offline_at": "2026-10-08T14:10:00Z"}
```
Answers `201 {"data": {"farm": Farm, "dropped_cells": [{"e", "n"}]}}`.

Farm = the summary fields plus `outline: [{"lat", "lon"}]`, `cells: [{"e", "n", "crop"}]`, `created_offline_at`, `updated_at`.

Rules the backend applies:
- `name`: 1 to 100 characters after trimming.
- `points`: 3 to 50 corners in walking order. `acc_m` and `t` are optional. A corner sent twice in a row, or the first corner repeated at the end, is counted once.
- The outline must not cross itself: `422`, detail `The farm outline crosses itself`.
- The farm's cells are every 10 m cell whose centre is inside the outline. The backend works this list out itself; cells the app did not paint are stored as `empty`.
- A painted cell that is not inside the outline is left out and listed in `dropped_cells`. This is not an error.
- If the same cell is sent twice, the later crop wins.
- `cells` may be missing or empty: the farm is then all `empty`.
- Limits: 20 farms per phone, 50,000 cells per farm (2,000 dunam). Over either: `422`.

### PUT /v1/farms/{id}/cells
```json
{"cells": [{"e": 46397, "n": 398737, "crop": "barley"}]}
```
Answers `200` with the same body shape as `POST`.

- Only the cells listed change. Cells not listed keep their crop. To clear a cell, send it with `"crop": "empty"`.
- Cells that are not part of the farm come back in `dropped_cells`.
- A farm that belongs to another phone answers `404`, the same as a farm that does not exist.

### DELETE /v1/farms/{id}
`204`, or `404` if the farm does not exist for this phone. The farm's cells are deleted with it.

### Status codes
`400` unreadable body or parameter, `401` missing or bad token, `404` not found, `422` a rule above was broken, `500` server fault (the detail is always `An unexpected error occurred`).

## 4. Sign in, for now

The token is a JWT whose subject is the phone number. Until the SMS code endpoints exist, a developer prints one with `cargo run -- token +9647501234567` (see `backend/README.md`). The app should keep the token opaque: store it, send it, nothing else.

## 5. Open questions for the frontend

1. Does the app need machine error codes (`bad_polygon`, `too_many_farms`) next to the English `detail`?
2. `PUT /farms/{id}/cells`: is "only the listed cells change" what the paint screen wants, or should the list replace the whole painting?
3. Is 50,000 cells (2,000 dunam) a safe upper limit for one farm?
