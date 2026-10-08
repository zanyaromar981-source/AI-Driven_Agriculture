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
    insights/ zones/ dams/ outlooks/ water/ fires/ alwa/
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

`POST /v1/auth/otp/send` then `POST /v1/auth/otp/verify`, as the app does. No SMS provider is wired yet: the code is written to the server log. For a demo, set `AUTH__FIXED_SIGN_IN_CODE=123456` in `.env` and every phone signs in with that code. Do not use either with real farmers.

For quick work with curl, a token can also be printed directly:

```sh
cargo run -- token +9647501234567
```

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## What is here today

43 routes under `/v1` plus `/status` and `/health`; the full list with shapes is at `/api-docs`. By slice:

| Slice | Routes | For |
|---|---|---|
| `farmers` | `/v1/auth/otp/send`, `/v1/auth/otp/verify`, `/v1/me` | sign in, profile |
| `farms` | `/v1/farms`, `/v1/farms/{id}`, `/{id}/cells`, `/{id}/status` | a farmer's farms |
| `insights` | `/v1/farms/{id}/insights` | water, groundwater, soil, rain per farm |
| `zones` | `/v1/region/overview`, `/v1/zones/{slug}`, `/v1/region/compare` | dashboard: 33 districts |
| `dams` | `/v1/dams`, `/v1/dams/{slug}/history` | dashboard: dam levels |
| `outlooks` | `/v1/outlooks`, `/v1/outlooks/zones/{zone_slug}` | dashboard: next-season outlook |
| `water` | `/v1/water/plan` | dashboard: water plan |
| `fires` | `/v1/fires` | dashboard: fire detections |
| `alwa` | `/v1/alwa/...` | wholesale market: prices, listings, offers, deals |

Answers use the body shapes of `BACKEND.md`; errors are `{"error": "<code>", "detail": "..."}`.

## Pushing data in

The backend stores and serves numbers; it does not compute them. The data jobs write through `PUT /v1/ingest/...` with the header `X-Service-Key: <INGEST__SERVICE_KEY>`. Every `PUT` is an upsert on a natural key (district and month, dam and day, fire id, farm and topic), so a job can run again safely. The one `DELETE` removes a water plan entry. `GET /v1/ingest/farms` lists every farm's centre point and which topics it already has, without phone numbers. Example:

```sh
curl -X PUT localhost:3000/v1/ingest/dams/dukan/readings/2026-09-21 \
  -H "X-Service-Key: $INGEST__SERVICE_KEY" -H 'content-type: application/json' \
  -d '{"pct_full": 88, "volume_bn_m3": 6.14, "source": "Sentinel-2 lake area"}'
```

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
