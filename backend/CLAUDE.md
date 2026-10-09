# Rules for the backend

Read the repo's root `CLAUDE.md` first: its rules apply here too (ask when unsure, log decisions in `STATUS.md`, no em dashes, no AI attribution, push after every change, stop on merge conflicts). This file adds the rules of the code in `backend/`.

Read before changing anything: `README.md` (layout, how to run), then the `farms` and `farmers` slices end to end. They are the template; a new slice looks like them file for file.

## 1. Architecture: clean architecture in vertical slices

One folder per feature under `src/features/<slice>/`, always these four layers:

| Layer | Holds | May import |
|---|---|---|
| `domain/` | `entities.rs`, `enums.rs`, `errors.rs`, `value_objects/` | `chrono`, `getset`, `thiserror`, `crate::shared` only |
| `app/` | `errors.rs`, `ports/` (traits), `use_cases/` (one file each), `testing.rs` (fakes) | its own `domain`, `crate::app`, `crate::shared` |
| `infra/` | `persistence/postgres/{entities,mappings.rs,repo.rs}`, `services/` (adapters) | its own `domain` and `app`, `sea_orm`, other crates |
| `web/` | `dtos.rs`, `errors.rs`, `handlers.rs`, `routes.rs` | its own `domain` and `app`, `axum`, `serde`, `utoipa`, `crate::infra::http` |

Hard rules:

- `domain/` never imports `sea_orm`, `axum`, `serde`, `utoipa`, or anything from `infra` or `web`.
- Use cases never see HTTP types, SQL or SeaORM models. They talk to the outside only through ports (`Arc<dyn Trait>`).
- A slice never touches another slice's tables or `infra`. If it needs something from another slice, it defines its own port and adapts that slice's port in its own `infra/services/` (see `farmers/infra/services/farm_counter.rs`).
- Slices refer to districts by `zone_slug` strings. No foreign keys across slices.
- Cross-cutting code lives in `src/app` (errors, pagination, signed-in user), `src/infra` (config, HTTP plumbing, bootstrap) and `src/shared` (things every slice uses). Do not grow `shared` for one slice's convenience.
- Wiring a slice touches exactly: `src/features/mod.rs`, `src/shared/state.rs`, `src/infra/initializations/di.rs`, `src/main.rs`, `src/infra/http/openapi.rs`, `build.rs`, `migration/src/lib.rs`.

## 2. Domain

- Entities have private fields with `getset` getters. `new(...)` validates and returns `Result`; `rehydrate(...)` rebuilds from stored state. `id: Option<i32>` (`None` = not yet stored).
- A rule about the data lives in the domain, not in a handler, a DTO or SQL. Ranking, totals, bands and status changes are pure functions with unit tests.
- Value objects validate in `new`, return the slice's domain error, and count characters, not bytes, for length limits (Sorani text is multi-byte).
- Enums are stored as lower-case snake_case strings: `impl From<E> for String`, `impl TryFrom<&str> for E`, an `ALL` const and a round-trip test.
- Methods that depend on time take `now` as an argument so they can be tested.

## 3. Errors

- Every error reaches HTTP through `ToErrorInfo` (`src/app/errors.rs`). Never build an error body by hand in a handler.
- The body is always `{"error": "<code>", "detail": "<English text>"}`. Use `ErrorInfo::with_code` for a failure a client can act on (`bad_polygon`, `too_many_farms`, `listing_not_open`); otherwise the kind's default code.
- Server faults never show their cause: a 5xx detail is always "An unexpected error occurred". Log the real error with `tracing::error!` where it happens.
- Do not reveal whether something exists to someone who may not see it: another farmer's farm is `404`, exactly like a missing one; every failed sign-in code is `bad_code`.

## 4. HTTP

- JSON is snake_case. Success bodies are plain objects through `ApiResponse::ok` / `ApiResponse::created`, with no outer wrapper. A list is wrapped in a named key (`{"farms": [...]}`).
- The farmer app reads the shapes in the root `BACKEND.md`; what the backend really does is in the root `FRONTEND.md`. Change a route the app or the dashboard uses and you change `FRONTEND.md` and regenerate `API.md` (`python3 tools/api_reference.py <server> > API.md`) in the same commit.
- Ids travel as strings. A non-numeric id is `404`, not `400`.
- Timestamps are `DateTime<Utc>` (with the `Z`), days are `NaiveDate`. Database timestamps are naive UTC.
- Text a person reads comes in Sorani and English (`*_ku`, `*_en`).
- Every handler has `#[utoipa::path]` and is registered in `openapi.rs`. DTO enums are separate from domain enums, with `From` both ways. DTO type names must be unique across slices (prefix them, as `alwa` does).
- Three kinds of routes, each its own function in `web/routes.rs`:
  - `public_routes()`: reads with no login.
  - `routes()`: need the farmer's token (`Extension<AuthContext>`). Always scope the query by the signed-in phone; never trust an id alone.
  - `ingest_routes()`: writes by our data jobs, behind the `service_key` middleware, mounted under `/v1/ingest`.
- A public route never writes.

## 5. Writes: repeat-safe and race-safe

The app retries on timeouts and 5xx, and data jobs re-run and overlap. Every write must survive being sent twice and being sent twice at the same moment.

- Never check and then write in two steps. Put the condition in the statement: `INSERT ... ON CONFLICT`, `UPDATE ... WHERE status = 'open'` with a rows-affected check, `DELETE ... WHERE ...` with a rows-affected check, or `SELECT ... FOR UPDATE` inside a transaction. A read before the write may give a better error message, but it never decides.
- Every natural key has a unique index in the migration. The index, not the code, is what stops a duplicate.
- Ingest writes are `PUT` upserts on a natural key (district and month, dam and day, external id, farm and topic) in one statement, replacing every non-key column. Where a key has no date in it, guard against an older push replacing a newer one (`excluded.as_of >= table.as_of`).
- A `POST` that creates something the app may retry takes `Idempotency-Key` and returns the thing already created, also when two copies race (re-read by key after a unique violation). See `register_farm.rs` and `post_listing.rs`.
- A repeat of a call that already succeeded succeeds again with the same answer: deleting what is gone, cancelling what is cancelled, accepting the offer already accepted. The domain decides what "already done" means (`was_cancelled_by`, `was_sold_on`); a different request against the finished thing is still refused.
- Write only what changed. Rewriting a whole aggregate from memory undoes a parallel request (see the `repainted` flag on `Cell`).
- Several statements that belong together run in one transaction.
- Limits, counters and one-use tokens are enforced in one statement (`record_attempt`, `consume`, `save_if_due` in `farmers`).
- Bound the work a request can cause before doing it: check sizes and ranges first (see `Outline::cells`).

## 6. Data honesty

- The backend stores and serves numbers. It does not compute satellite, weather or forecast values, and it never invents data: no sample measurements, no seeded readings, no placeholder numbers that look real.
- Missing data is `null` or an empty list, never a guess.
- Only real reference data is seeded in migrations (districts, dams, markets).
- Data jobs live in `jobs/`, push through `/v1/ingest`, and say in their `source` text exactly what was measured and how any derived figure was made.

## 7. Security

- Never commit keys. `.env` files are ignored; `.env.example` holds names only.
- Secrets, sign-in codes and tokens never appear in logs or `Debug` output (the one deliberate exception is `LogSignInCodeSender`, used only when no OTPIQ key is configured).
- Compare secrets without leaking where they differ (`service_key.rs`).
- Phone numbers are private: do not return one user's phone to another unless a rule in the domain says so (an Alwa deal).

## 8. Database

- One migration per change, named `mYYYYMMDD_HHMMSS_what.rs`, registered in `migration/src/lib.rs` in time order. **Never edit a migration that has been pushed:** a server has already run it. Add a new one.
- **Every table that holds data a website shows belongs to a topic in `data_versions`.** A new table attaches the trigger in its own migration (`CREATE TRIGGER bump_data_version AFTER INSERT OR UPDATE OR DELETE ON <table> FOR EACH STATEMENT EXECUTE FUNCTION bump_data_version('<topic>')`), and a new topic is added to `data_versions`, to `Topic` in `src/features/versions/domain/entities.rs` and to `FRONTEND.md`. Without it the website never learns that the data changed.
- A new permission resource is added to `Resource` in `src/app/access.rs`, to `StaffResource` in the staff DTOs, and granted to the system role in a migration.
- Entities in `infra/persistence/postgres/entities/` are written in the exact style `sea-orm-codegen` produces.
- Repositories map `DbErr` through a `database_error` helper that logs and returns `GlobalAppError::DatabaseError`.
- Postgres takes 65,535 bind parameters per statement: chunk large inserts and `IN` lists.

## 9. Tests

- Domain: a test for every rule, value object and enum.
- Use cases: tests with the fake from `app/testing.rs`, asserting the result and the recorded calls (what was called, in which order, scoped to which owner).
- Test names are sentences that state the behaviour (`a_wrong_code_fails_and_is_counted`); add a reason string to an assert when the reason is not obvious.
- Unit tests need no database. Anything about SQL, transactions or races is proven by running the server against a throwaway Postgres and calling it, with parallel requests where the point is a race. Say in `STATUS.md` what was run and what it showed.

## 10. Before every commit

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

All three must pass. Then: no em dashes in anything you wrote, `FRONTEND.md` and `README.md` still tell the truth, `STATUS.md` and `PROGRESS.md` are updated, one commit per thing, pull, push, and stop to ask if the pull conflicts.

## 11. Style

- Match the surrounding code: comment density, naming, logging with structured fields (`tracing::info!(farm_id = id, "farm removed")`).
- Comments say why, not what. No comment restating the code.
- No new dependency without a reason written in the commit message. The commit scanner blocks crates with known advisories.
- Do not leave dead code, unused variants or stale comments behind a change.
