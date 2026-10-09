# Backend (farm-doctor-api)

Rust + axum, clean architecture combined with vertical slices. `BACKEND.md` and `FRONTEND.md` at the repo root are the contract with the app.

## Layout

```
src/
  app/        shared application types: errors, pagination, the signed-in user
  infra/      config, Postgres, HTTP plumbing (auth, errors, responses, health, OpenAPI), telemetry
  shared/     value objects and helpers every slice uses (Phone, JWT, AppState)
  features/
    farmers/  sign in with a code, the farmer's profile
    farms/    a farmer's farms
    insights/ zones/ dams/ outlooks/ water/ fires/ alwa/ doctor/
      domain/   (every slice has these four) entities, value objects, rules. No database, no HTTP
      app/      use cases and the repository port
      infra/    the Postgres repository behind that port
      web/      DTOs, handlers, routes
migration/    database migrations (SeaORM)
```

A new feature is a new folder under `src/features/` with the same four layers, wired in `src/infra/initializations/di.rs`, `src/shared/state.rs`, `src/main.rs` and `src/infra/http/openapi.rs`.

## Run it

Needs Rust (built and tested with 1.98) and Docker.

```sh
cd backend
cp .env.example .env          # then set AUTH__JWT_SECRET to any long random text
docker compose up -d          # Postgres on port 5432
cargo run -- migrate          # create the tables
cargo run                     # API on http://localhost:3000
```

- API docs: http://localhost:3000/api-docs
- Health: `GET /status` (process is up), `GET /health` (database answers)

## Signing in

`POST /v1/auth/otp/send` then `POST /v1/auth/otp/verify`, as the app does. With `OTPIQ__API_KEY` set, the code is delivered through OTPIQ (SMS, WhatsApp or Telegram; see the `OTPIQ__*` names in `.env.example`). Without a key the code is written to the server log. For a demo, set `AUTH__FIXED_SIGN_IN_CODE=123456` in `.env` and every phone signs in with that code. Do not use the log or a fixed code with real farmers; a fixed code together with an OTPIQ key stops the server at start-up.

A farmer's token works only while the farmer exists: once staff delete the farmer, every farmer route answers `401`.

For quick work with curl, a token can also be printed directly. The command needs the database, because it also creates the farmer if the phone has none:

```sh
cargo run -- token +9647501234567
```

## Dashboard staff

Ministry staff sign in with `POST /v1/dashboard/auth/login` (email and password). Every other route under `/v1/dashboard` needs the staff token and one permission (an action on a resource), which staff hold through roles. The first account is made on the command line and holds the `Owner` role:

```sh
OWNER_PASSWORD='at least 10 characters' cargo run -- create-owner owner@example.org "Full Name"
```

Running it again for the same email changes nothing.

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## What is here today

76 paths in the API docs (`/status` and `/health` included), with 71 operations under `/v1/dashboard`; the full list with shapes is at `/api-docs`. By slice:

| Slice | Routes | For |
|---|---|---|
| `farmers` | `/v1/auth/otp/send`, `/v1/auth/otp/verify`, `/v1/me` | sign in, profile |
| `farms` | `/v1/farms`, `/v1/farms/{id}`, `/{id}/cells`, `/{id}/status` | a farmer's farms |
| `insights` | `/v1/farms/{id}/insights` | water, groundwater, soil, rain per farm |
| `plans` | `/v1/farms/{id}/plan` | this week's plan: 10 days of weather turned into farm work, pushed by `jobs/farm_plan.py` |
| `doctor` | `/v1/farms/{id}/ask` | Ask the Doctor: passes the farmer's question to the local Doctor service |
| `zones` | `/v1/region/overview`, `/v1/zones/{slug}`, `/v1/region/compare` | dashboard: 33 districts |
| `dams` | `/v1/dams`, `/v1/dams/{slug}/history` | dashboard: dam levels |
| `outlooks` | `/v1/outlooks`, `/v1/outlooks/zones/{zone_slug}` | dashboard: next-season outlook |
| `water` | `/v1/water/plan` | dashboard: water plan |
| `fires` | `/v1/fires` | dashboard: fire detections |
| `alwa` | `/v1/alwa/...` | wholesale market: prices, listings, offers, deals |
| `staff` | `/v1/dashboard/auth/login`, `/me`, `/permissions`, `/roles`, `/staff` | dashboard: staff accounts, custom roles, sign-in |

Answers use the body shapes of `BACKEND.md`; errors are `{"error": "<code>", "detail": "..."}`.

## Pushing data in

The backend stores and serves numbers; it does not compute them. The data jobs write through `PUT /v1/ingest/...` with the header `X-Service-Key: <INGEST__SERVICE_KEY>`. Every `PUT` is an upsert on a natural key (district and month, dam and day, fire id, farm and topic), so a job can run again safely. The one `DELETE` removes a water plan entry. `GET /v1/ingest/farms` lists every farm's centre point and which topics it already has, without phone numbers. Example:

```sh
curl -X PUT localhost:3000/v1/ingest/dams/dukan/readings/2026-09-21 \
  -H "X-Service-Key: $INGEST__SERVICE_KEY" -H 'content-type: application/json' \
  -d '{"pct_full": 88, "volume_bn_m3": 6.14, "source": "Sentinel-2 lake area"}'
```

## Ask the Doctor

`POST /v1/farms/{id}/ask` (shapes in the root `FRONTEND.md` section 12) calls no AI itself. It passes the question to the Farm Doctor service at `DOCTOR_URL` (default `http://127.0.0.1:8090`) and waits up to 90 s:

```
POST {DOCTOR_URL}/ask
{"farm": {"id": "5", "name": "...", "lat": 36.01, "lon": 44.62, "area_m2": 19451.0, "crops": ["wheat"]},
 "history": {"topics": [...]} or null,
 "question": "..." or null, "cell": {"e": 1, "n": 2} or null, "lang": "ku",
 "photos": [{"mime": "image/jpeg", "data": "<base64>"}]}
```

`lat` and `lon` are the mean of the outline's corners, `area_m2` the exact area inside it, `crops` the painted crop codes (largest first). `history` is what `GET /v1/farms/{id}/insights` answers, without `farm_id`; `null` when nothing is known yet. The service answers `200` with `likely`, `confidence`, `why`, `actions_this_week`, `cannot_tell`, `refer_to_officer`, `ku`, `en`, `inputs_used`; `503 {"error": "doctor_not_ready"}` when it has no AI key yet; anything else counts as failed. The app then gets `503 doctor_not_ready` or `502 doctor_failed`, and the real cause is in the server log.

## Hosting it

`Dockerfile` builds the server; `deploy/docker-compose.yml` runs it with its own Postgres:

```sh
cd backend/deploy
cp .env.example .env     # fill in the three secrets; never commit .env
docker compose up -d --build
```

The API then answers on `PUBLIC_PORT` (default 8790). Migrations run on every start.

## Data jobs

`jobs/` holds the scripts that compute numbers and push them in (see `jobs/README.md`). `deploy/systemd/` has the timer that runs the region runner every 12 hours.

## Rules of the code

`CLAUDE.md` in this folder.

## Telling the frontend

- `FRONTEND.md` at the repo root is the guide for the app and dashboard teams: update it in the same commit as any route they use.
- `API.md` in this folder is the route-by-route reference. It is generated, never edited by hand. After changing a route, run the server and:

```sh
python3 tools/api_reference.py http://localhost:3000 > API.md
```
