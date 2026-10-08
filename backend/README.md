# Backend (farm-doctor-api)

Rust + axum, clean architecture combined with vertical slices. `BACKEND.md` and `FRONTEND.md` at the repo root are the contract with the app.

## Layout

```
src/
  app/        shared application types: errors, pagination, the signed-in user
  infra/      config, Postgres, HTTP plumbing (auth, errors, responses, health, OpenAPI), telemetry
  shared/     value objects and helpers every slice uses (Phone, JWT, AppState)
  features/
    farms/    one vertical slice
      domain/   entities, value objects, rules. No database, no HTTP
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

## Signing in while there is no SMS code yet

Every `/v1` route needs `Authorization: Bearer <token>`. Until the SMS sign-in is built, print a token for a phone number:

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

| Route | What it does |
|---|---|
| `GET /v1/farms` | the signed-in farmer's farms (summary of each) |
| `POST /v1/farms` | create a farm from walked corners and painted cells |
| `PUT /v1/farms/{id}/cells` | repaint crops on cells |
| `DELETE /v1/farms/{id}` | delete a farm and its cells |

Not built yet: SMS sign-in, satellite status, weather plan, the Doctor, reports, alerts, the region dashboard.
