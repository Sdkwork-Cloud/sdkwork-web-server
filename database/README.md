# WEB Database Module

Canonical lifecycle assets for `sdkwork-web-server` per `DATABASE_FRAMEWORK_SPEC.md`.

- moduleId: `web`
- serviceCode: `WEB`
- tablePrefix: `web_`

## Initialization state

This module is in **initialization state** for greenfield deployments:

1. **Baseline** — `database/ddl/baseline/{engine}/0001_web_baseline.sql` contains the full DDL snapshot.
2. **Migrations** — `database/migrations/{engine}/` is reserved for post-GA incremental schema changes only. It is intentionally empty at initialization.
3. **Drift** — run `pnpm db:drift:check` before release.

## Commands

```bash
pnpm run db:validate
pnpm run db:materialize:contract
pnpm run db:plan
pnpm run db:init
pnpm run db:migrate
pnpm run db:seed
pnpm run db:status
pnpm run db:drift:check
pnpm run db:test:sqlite
SDKWORK_WEB_POSTGRES_TEST_DATABASE_URL=<disposable-url> pnpm run db:test:postgres
pnpm run test:database:recovery
pnpm run test:postgres:ha
```

`db:test:postgres` is intentionally ignored by the default Cargo test run and requires an explicitly configured disposable, empty PostgreSQL database. The test refuses to continue when the target schema already contains `web_*` tables. SQLite is an explicit single-node profile only; PostgreSQL remains the default for standalone shared and cloud control-plane deployments.

`test:database:recovery` is a destructive recovery drill that owns only its temporary SQLite directory and one disposable PostgreSQL container. It proves a consistent SQLite snapshot and PostgreSQL custom-format dump can restore schema integrity and a tenant-scoped canary. It is not a production backup command and does not establish encrypted off-host retention, PostgreSQL PITR, managed-provider recovery, or the product RPO/RTO.

`test:postgres:ha` owns two disposable PostgreSQL containers and one internal Docker network. It proves physical base backup, asynchronous WAL streaming, replay to a recorded flush LSN, primary shutdown, standby promotion, and post-promotion tenant writes. It does not establish automatic leader election, client connection rerouting, synchronous-replication RPO, split-brain fencing, managed-provider behavior, multi-zone capacity, or production RTO.
