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

| Route | What it does |
|---|---|
| `POST /v1/auth/otp/send`, `POST /v1/auth/otp/verify` | sign in with a phone and a code |
| `GET /v1/me`, `PUT /v1/me` | the farmer's profile |
| `GET /v1/farms` | the signed-in farmer's farms (summary of each) |
| `POST /v1/farms` | create a farm from walked corners and painted cells |
| `GET /v1/farms/{id}` | one farm with its outline and cells |
| `PUT /v1/farms/{id}/cells` | repaint crops on cells |
| `DELETE /v1/farms/{id}` | delete a farm and its cells |

Answers use the body shapes of `BACKEND.md`; errors are `{"error": "<code>", "detail": "..."}`.
