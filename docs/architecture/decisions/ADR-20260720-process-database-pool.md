# ADR-20260720 Process Database Pool Ownership

Status: Accepted

## Decision

Every Web Server process enables the `sdkwork-database` process-shared pool guard before database bootstrap. `sdkwork-webserver-database-host` creates and owns one typed `DatabasePool` per process and performs lifecycle initialization against that pool.

The repository consumes the exact installed pool returned by the database host. A single authored repository implementation is compiled for both concrete SQLx drivers:

- `PostgresWebRepository` consumes `sqlx::PgPool`.
- `SqliteWebRepository` consumes `sqlx::SqlitePool`.

Runtime selection matches the framework `DatabasePool` variant and injects the corresponding repository through `WebRepositoryPort`. No route, service, worker, or repository code constructs a secondary low-level pool.

## Consequences

- Lifecycle, readiness, repository work, and graceful shutdown share one bounded connection budget.
- PostgreSQL and SQLite keep one business implementation while retaining compile-time driver typing.
- SQL differences remain explicit through `DatabaseEngine` expression helpers and dual-engine parity tests.
- `sqlx::AnyPool`, temporary driver exceptions, and temporary pool-count environment variables are prohibited in production source and configuration.

## Verification

```powershell
node ../sdkwork-specs/tools/check-process-shared-database-pool.mjs --root .
pnpm db:validate
cargo test -p sdkwork-webserver-database-host
cargo test -p sdkwork-intelligence-webserver-repository-sqlx
```

The PostgreSQL parity test requires `SDKWORK_WEB_POSTGRES_TEST_DATABASE_URL` pointing to an explicitly disposable empty database and remains a required CI release gate.
